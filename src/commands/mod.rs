use crate::settings::DEFAULT_PREFIX;
// use crate::commands::eval::dispatch as other_dispatch;
use crate::BotState;
use log::error;
use rquickjs::Ctx;
use serenity::all::{Context, CreateAllowedMentions, CreateMessage, GuildId, Message, UserId};
use std::sync::Arc;

mod admins;
mod eval;
mod help;
mod ping;
mod settings;
pub(crate) mod tag;

pub async fn get_prefix(ctx: &CommandContext) -> String {
    ctx.state
        .guild_cache
        .settings
        .get(&ctx.msg.guild_id.unwrap())
        .await
        .unwrap()
        .prefix
        .to_string()
}

#[derive(Clone)]
pub struct CommandContext {
    pub(crate) serenity_ctx: Context,
    pub(crate) msg: Message,
    args: Option<String>,
    pub(crate) state: Arc<BotState>,
    help: bool,
}

impl CommandContext {
    pub fn new(
        serenity_ctx: Context,
        msg: Message,
        args: Option<String>,
        state: Arc<BotState>,
        help: bool,
    ) -> Self {
        CommandContext {
            serenity_ctx,
            msg,
            args,
            state,
            help,
        }
    }

    pub fn with_new_args(&self, args: Option<String>) -> Self {
        CommandContext {
            serenity_ctx: self.serenity_ctx.clone(),
            msg: self.msg.clone(),
            args,
            state: self.state.clone(),
            help: self.help,
        }
    }

    pub fn get_guild_id(&self) -> GuildId {
        self.msg.guild_id.unwrap()
    }

    pub fn get_author_id(&self) -> UserId {
        self.msg.author.id
    }
}

impl CommandContext {
    pub fn consume_arg(&mut self) -> Option<String> {
        let (first, remaining) = self.parse_next_arg();
        self.args = remaining;
        first
    }

    pub fn peek_arg(&mut self) -> Option<String> {
        self.parse_next_arg().0
    }

    fn parse_next_arg(&mut self) -> (Option<String>, Option<String>) {
        if let Some(args) = &self.args {
            let (first, remaining) = match args.split_once(char::is_whitespace) {
                Some((next, args)) => (next.to_lowercase(), Some(args.to_string())),
                None => (args.to_lowercase(), None),
            };
            (Some(first), remaining)
        } else {
            (None, None)
        }
    }

    pub fn set_help(&mut self) {
        self.help = true;
    }
}

pub struct CommandInfo {
    pub command: &'static str,
    pub usage: Option<&'static str>,
    pub full_desc: &'static str,
    pub short_desc: Option<&'static str>,
    pub aliases: &'static [&'static str],
    pub further_help: Option<&'static str>,
    pub subcommands: Option<&'static [&'static CommandCategory]>,
}

pub struct CommandCategory {
    pub name: Option<&'static str>,
    pub description: Option<&'static str>,
    pub commands: &'static [&'static CommandInfo],
}

pub async fn send_reply_ping_text(ctx: &CommandContext, content: &str) {
    if let Err(e) = ctx
        .msg
        .reply_ping(ctx.serenity_ctx.http.clone(), content)
        .await
    {
        error!("Error sending message: {:?}", e);
    }
}

pub async fn send_reply_ping_message(
    ctx: &CommandContext,
    msg: CreateMessage,
) -> Result<(), String> {
    let message = msg
        .allowed_mentions(CreateAllowedMentions::new().replied_user(true))
        .reference_message(&ctx.msg);

    if let Err(e) = ctx
        .msg
        .channel_id
        .send_message(&ctx.serenity_ctx.http, message)
        .await
    {
        error!("Error sending message: {:?}", e);
        return Err(format!("Error sending message: {:?}", e));
    }
    Ok(())
}

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[
    &CommandCategory {
        name: Some("Commands"),
        description: None,
        commands: &[&ping::INFO, &tag::INFO, &eval::INFO, &help::INFO],
    },
    &CommandCategory {
        name: Some("Admin"),
        description: None,
        commands: &[&settings::INFO, &admins::INFO],
    },
];

pub const INFO: CommandInfo = CommandInfo {
    command: "<command>",
    usage: None,
    full_desc: "",
    short_desc: None,
    aliases: &[],
    further_help: Some(
        "`{PREFIX}<any_command> help` will give additional information about each command.\
    \n-# <angle_brackets> denote required arguments, while [square_brackets] denote optional arguments in all help commands.",
    ),
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if matches!(ctx.peek_arg().as_deref(), Some("help")) {
        ctx.consume_arg();
        ctx.set_help();
    }

    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("ping") => ping::dispatch(ctx).await,
        Some("tag") | Some("t") => tag::dispatch(ctx).await,
        Some("eval") => eval::dispatch(ctx).await,
        Some("settings") => settings::dispatch(ctx).await,
        Some("admins") | Some("admin") => admins::dispatch(ctx).await,
        None if ctx.help => help::execute(ctx).await,
        _ => help::execute(ctx).await,
    }
}
