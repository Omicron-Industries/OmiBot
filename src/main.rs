use std::env;
use std::sync::Arc;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use log::{error, info, logger};
use serenity::all::{CreateAllowedMentions, CreateEmbed, CreateMessage, GuildId};
use sqlx::{query, query_as, Pool, Postgres, QueryBuilder};
use bunny_bot::BotState;
use bunny_bot::cache::GuildCache;
use bunny_bot::commands::help::help;
use bunny_bot::commands::ping::ping;
use bunny_bot::settings::{GuildSettings, DEFAULT_PREFIX};
use bunny_bot::tags::tag;

pub struct Bot {
    state: Arc<BotState>,
}


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let guild_id = match msg.guild_id {
            Some(id) => id.get(),
            None => {
                let _ = msg.reply_ping(ctx.http, "Error when evaluating tag command. Message was either a DM or received outside of the gateway. Make sure you are requesting a tag in a server that has it. To see your own tags, run `%t list`".to_string()).await;
                return;
            }
        };
        
        let settings = match self.state.guild_cache.settings.get(&GuildId::from(guild_id)).await {
            Some(settings) => settings,
            None => {
                match query_as!(GuildSettings, "SELECT prefix FROM guilds_settings WHERE guild_id = $1", guild_id as i64).fetch_optional(&self.state.db_pool).await {
                    Ok(Some(new_settings)) => new_settings,
                    Ok(None) => {
                        match query!("INSERT INTO guilds_settings (guild_id) VALUES ($1)", guild_id as i64).execute(&self.state.db_pool).await {
                            Ok(_) => {
                                let settings = GuildSettings::default();
                                self.state.guild_cache.settings.insert(GuildId::from(guild_id), settings.clone()).await;
                                settings
                            },
                            Err(e) => {
                                error!("Failed to insert new server settings: {}", e);
                                let _ = msg.reply_ping(ctx.http, "Error when setting new server settings".to_string()).await;
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get server settings: {}", e);
                        let _ = msg.reply_ping(ctx.http, "Error when getting server settings".to_string()).await;
                        return;
                    }
                        
                }
            }
            
        };
        
        if !msg.content.starts_with(&settings.prefix) { return }

        // matches prefix
        let content = msg.content.to_lowercase();
        let content = content.trim_start_matches(&settings.prefix);
        let (command, args) = match content.split_once(char::is_whitespace) {
            Some((command, args)) => (command, args.trim()),
            None => (content, ""),
        };
        let message: CreateMessage = match command {
            "ping" => ping(args).await,
            "tag" => tag(args, &msg, self.state.clone()).await,
            "t" => tag(args, &msg, self.state.clone()).await,
            "help" => help(args, &msg, self.state.clone()).await,

            _ => return
        };

        let message = message.allowed_mentions(CreateAllowedMentions::new().replied_user(true)).reference_message(&msg);

        if let Err(e) = msg.channel_id.send_message(&ctx.http, message).await {
            error!("Error sending message: {:?}", e);
        }

    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().expect("Expected a .env file!");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env::var("POSTGRES_USER").expect("POSTGRES_USER must be set"),
        env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set"),
        env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set"),
        env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set"),
        env::var("POSTGRES_DB").expect("POSTGRES_DB must be set"),
    );

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .expect("could not connect to database_url");

    info!("Migrating the database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate the database");
    info!("Finished migrating the database");




    // Login with a bot token from the environment
    let token = env::var("APPLICATION_TOKEN").expect("Expected APPLICATION_TOKEN");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let mut guild_cache = GuildCache::new();
    let mut bot_state = Arc::new(BotState {
        db_pool: pool,
        guild_cache
    });

    let bot = Bot {
        state: bot_state.clone(),
    };

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(bot).await.expect("Err creating client");
    println!("{:?}", client.http.get_guilds(None, Some(10)).await.unwrap());

    // Build initial cache
    info!("Getting guilds");
    let guilds = client.http.get_guilds(None, Some(200)).await.unwrap();

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO guilds_settings (guild_id) "
    );
    query_builder.push_values(guilds.iter(), |mut row, guild| {
        row.push_bind(guild.id.get() as i64);
    });
    query_builder.push(" ON CONFLICT (guild_id) DO NOTHING");

    query_builder.build().execute(&bot_state.db_pool).await.expect("Couldn't add guilds to database");

    let server_settings_records = query!("SELECT guild_id, prefix FROM GUILDS_SETTINGS;").fetch_all(&bot_state.db_pool).await.expect("Couldn't fetch settings from database");
    for row in server_settings_records {
        bot_state.guild_cache.settings.insert(GuildId::from(row.guild_id as u64), GuildSettings::new(&row.prefix)).await;
    }
    

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}