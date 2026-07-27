use crate::settings::DEFAULT_PREFIX;
use crate::{settings, BotState};
use log::error;
use rquickjs::Ctx;
use serenity::all::{Context, CreateAllowedMentions, CreateMessage, GuildId, Message};
use std::sync::Arc;

mod admin;
mod eval;
mod help;
mod ping;
mod tag;

//TODO make sure prefix is always in cache
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
    serenity_ctx: Context,
    msg: Message,
    args: Option<String>,
    state: Arc<BotState>,
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
}

// trait ExecutableBranch {
//     fn parse(ctx: &mut CommandContext) -> BranchParseResult<impl ExecutableBranch>;
//     async fn execute(self, ctx: &mut CommandContext);
//     fn resolve(self, ctx: &mut CommandContext);
// }
//
// enum BranchParseResult<T: ExecutableBranch> {
//     Subcommand(T),
//     Here,
//     InvalidArg(String),
// }

// impl<T: ExecutableBranch> BranchParseResult<T> {
//     pub async fn resolve(self, ctx: &mut CommandContext) {
//         match self {
//             BranchParseResult::Subcommand(s) => {
//                 BranchParseResult::resolve(T::parse(ctx).await, ctx).await
//             }
//             BranchParseResult::Here => T::execute(ctx).await,
//             BranchParseResult::InvalidArg(s) => send_reply_ping_text(ctx, s.as_str()).await,
//         }
//     }
// }

// enum RootParseResult {
//     Command(RootCommands),
//     NoArgs,
//     InvalidArg(String),
// }

impl CommandContext {
    pub fn consume_arg(&mut self) -> Option<String> {
        if let Some(args) = &self.args {
            let (first, remaining) = match args.split_once(char::is_whitespace) {
                Some((next, args)) => (next.to_lowercase(), Some(args.to_string())),
                None => (args.to_lowercase(), None),
            };
            self.args = remaining;
            Some(first)
        } else {
            None
        }
    }

    pub fn set_help(&mut self) {
        self.help = true;
    }
}

// enum RootCommands {
//     Ping,
//     Tag,
//     Eval,
//     Help,
// }
//
// impl RootCommands {
//     pub fn parse(ctx: &mut CommandContext) -> RootParseResult {
//         let (command, new_ctx) = ctx.consume_arg();
//         match command.as_deref() {
//             Some("ping") => RootParseResult::Command(RootCommands::Ping),
//             Some("tag") => RootParseResult::Command(RootCommands::Tag),
//             Some("eval") => RootParseResult::Command(RootCommands::Eval),
//             Some("help") => RootParseResult::Command(RootCommands::Help),
//             None => RootParseResult::NoArgs,
//             Some(arg) => RootParseResult::InvalidArg(arg.to_string()),
//         }
//     }
// }
//
// impl RootParseResult {
//     pub async fn resolve(self, ctx: &mut CommandContext) {
//         match self {
//             RootParseResult::Command(RootCommands::Ping) => {}
//             RootParseResult::Command(RootCommands::Tag) => {}
//             RootParseResult::Command(RootCommands::Eval) => {}
//             RootParseResult::Command(RootCommands::Help) => {}
//             RootParseResult::NoArgs => send_reply_ping_text(ctx, "Expected a command!").await,
//             RootParseResult::InvalidArg(arg) => {
//                 send_reply_ping_text(ctx, format!("`{}` is not a valid command!", arg).as_str())
//                     .await
//             }
//         }
//     }
// }

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

pub async fn send_reply_ping_message(ctx: &CommandContext, msg: CreateMessage) {
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
    }
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
        commands: &[&admin::ban::INFO, &admin::settings::INFO],
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
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("ping") => ping::dispatch(ctx).await,
        Some("tag") | Some("t") => tag::dispatch(ctx).await,
        Some("eval") => eval::dispatch(ctx).await,
        Some("help") => help::dispatch(ctx).await,
        Some("settings") => admin::settings::dispatch(ctx).await,
        Some("ban") => admin::ban::dispatch(ctx).await,
        _ => {}
    }
}
