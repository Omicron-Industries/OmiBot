use std::env;
use std::sync::Arc;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use log::{error, info};
use serenity::all::GuildId;
use sqlx::{query, Pool, Postgres, QueryBuilder};
use bunny_bot::cache::GuildCache;
use bunny_bot::settings::{GuildSettings, DEFAULT_PREFIX};


struct BotState {
    db_pool: Pool<Postgres>,
    guild_cache: GuildCache,
}

struct Bot {
    state: Arc<BotState>,
}


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let guild_id = msg.guild_id.unwrap().get();

        let mut prefix = DEFAULT_PREFIX;
        let settings = self.state.guild_cache.settings.get(&GuildId::from(guild_id)).await;

        if settings.is_none() {
            self.state.guild_cache.settings.insert(GuildId::from(guild_id), GuildSettings::default()).await;
            query!("INSERT INTO guilds_settings (guild_id) VALUES ($1)", guild_id as i64).execute(&self.state.db_pool).await.unwrap();
        } else {
            prefix = settings.unwrap().prefix;
        }
        if !msg.content.starts_with(prefix) { return }

        // matches prefix
        let mut parts = msg.content.strip_prefix(prefix).unwrap().split_whitespace();
        match parts.next() {
            None => return,
            Some("ping") => { if let Err(why) = msg.reply_ping(&ctx.http, "pong!").await { error!("Error sending ping: {:?}", why); } },
            _ => return
        }

    }
}

#[tokio::main]
async fn main() {
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

    let mut bot = Bot {
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
        bot_state.guild_cache.settings.insert(GuildId::from(row.guild_id as u64), GuildSettings::new(row.prefix.chars().next().unwrap())).await;
    }



    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}