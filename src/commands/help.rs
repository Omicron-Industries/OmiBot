use crate::commands::{get_prefix, send_reply_ping_text, CommandContext, CommandInfo};

// pub enum HelpCommands {
//     Ping,
//     Tag,
//     Eval,
//     Help,
// }
//
// impl ExecutableBranch for HelpCommands {
//     fn parse(ctx: &mut CommandContext) -> BranchParseResult<Self> {
//         let (command, new_ctx) = ctx.consume_arg();
//         match command.as_deref() {
//             Some("ping") => BranchParseResult::Subcommand(HelpCommands::Ping),
//             Some("tag") => BranchParseResult::Subcommand(HelpCommands::Tag),
//             Some("eval") => BranchParseResult::Subcommand(HelpCommands::Eval),
//             Some("help") => BranchParseResult::Subcommand(HelpCommands::Help),
//             Some(arg) => BranchParseResult::InvalidArg(arg.to_string()),
//             None => BranchParseResult::Here,
//         }
//     }
//
//     fn resolve(self, ctx: &mut CommandContext) {
//         match self {
//             HelpCommands::Ping => ping(ctx),
//             HelpCommands::Tag => TagCommands::parse(ctx).resolve(ctx),
//             HelpCommands::Eval => {}
//             HelpCommands::Help => {}
//         }
//     }
// }
//
// impl BranchParseResult<HelpCommands> {
//     async fn resolve(self, ctx: &mut CommandContext) {
//         match self {
//             BranchParseResult::Subcommand(HelpCommands::Ping) => {}
//             BranchParseResult::Subcommand(HelpCommands::Tag) => {
//                 TagCommands::parse(ctx).resolve(ctx).await;
//             }
//             BranchParseResult::Subcommand(HelpCommands::Eval) => {}
//             BranchParseResult::Subcommand(HelpCommands::Help) => {}
//             BranchParseResult::InvalidArg(arg) => {}
//             BranchParseResult::Here => {}
//         }
//     }
// }

// pub enum HelpCommandsParseResult<T> {
//     Child(T),
//     Here,
// }
// impl HelpCommands {
//     pub fn parse(ctx: &mut CommandContext) -> BranchParseResult<HelpCommands> {
//         match ctx.consume_word().as_deref() {
//             Some("ping") => BranchParseResult::Subcommand(HelpCommands::Ping),
//             Some("tag") => BranchParseResult::Subcommand(HelpCommands::Tag),
//             Some("eval") => BranchParseResult::Subcommand(HelpCommands::Eval),
//             Some("help") => BranchParseResult::Subcommand(HelpCommands::Help),
//             None => BranchParseResult::Here,
//             Some(cmd) => Err(CommandParseError::UnknownCommand(cmd.to_string())),
//         }
//     }
//
//     pub async fn execute(ctx: &mut CommandContext) {
//         let gid = ctx.msg.guild_id;
//
//         help_msg(&get_prefix(gid, ctx.state.clone()).await, &ctx.msg);
//     }
// }
// impl HelpCommandsParseResult<HelpCommands> {
//     pub fn execute(self, ctx: &CommandContext) {
//         match self {
//             HelpCommandsParseResult::Child(HelpCommands::Ping) => {}
//             HelpCommandsParseResult::Child(HelpCommands::Tag) => {}
//             HelpCommandsParseResult::Child(HelpCommands::Eval) => {}
//             HelpCommandsParseResult::Child(HelpCommands::Help) => {}
//             HelpCommandsParseResult::Here => {}
//         }
//     }
// }

pub async fn command_help(ctx: &CommandContext, cmd_info: CommandInfo) {
    let prefix = get_prefix(ctx).await;

    let sub_list = generate_subcommand_list(&cmd_info, &prefix);
    let content = format!(
        r#"
{}
{}
{}
See `{prefix}help {}` for more information.
    "#,
        cmd_info.full_desc,
        sub_list.map(|s| format!("\n{}\n", s)).unwrap_or_default(),
        cmd_info
            .further_help
            .map(|s| format!("\n{}\n", s))
            .unwrap_or_default(),
        cmd_info.command
    );
    send_reply_ping_text(ctx, &content).await;
}

fn generate_subcommand_list(cmd_info: &CommandInfo, prefix: &str) -> Option<String> {
    if cmd_info.subcommands.is_none() {
        return None;
    }
    Some(
        cmd_info
            .subcommands
            .unwrap()
            .iter()
            .map(|cat| {
                let commands = cat
                    .commands
                    .iter()
                    .map(|sub| {
                        format!(
                            "`{prefix}{}{}` - {}",
                            sub.command,
                            sub.usage.map(|s| format!(" {}", s)).unwrap_or_default(),
                            sub.short_desc.unwrap_or(sub.full_desc)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                match cat.name {
                    Some(name) => format!(
                        "**{}:**\n{}{}",
                        name,
                        cat.description
                            .map(|d| format!("{}\n", d))
                            .unwrap_or_default(),
                        commands
                    ),
                    None => commands,
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

pub async fn command_usage(ctx: &mut CommandContext, cmd_info: CommandInfo) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"Improper command usage!
Usage: `{}{}`
See `{prefix}help {}` for more information.
    "#,
        cmd_info.command,
        cmd_info
            .usage
            .map(|s| format!(" {}", s))
            .unwrap_or_default(),
        cmd_info.command
    );
    send_reply_ping_text(ctx, &content).await;
}

pub const INFO: CommandInfo = CommandInfo {
    command: "help",
    usage: Some("[command]"),
    full_desc: "Prints information about a command.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("ping") => {}
        Some("tag") | Some("t") => {}
        Some("eval") => {}
        Some("help") => command_help(ctx, INFO).await,
        Some("settings") => {}
        Some("ban") => {}
        _ => execute(ctx).await,
    }
}

// pub async fn help(args: Option<&str>, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     let prefix = get_prefix(msg.guild_id, state.clone()).await;
//
//     let Some(args) = args else {
//         return help_msg(&prefix, msg);
//     };
//
//     let (first_arg, rest) = match args.split_once(char::is_whitespace) {
//         Some((arg, rest)) => (arg, Some(rest)),
//         None => (args, None),
//     };
//
//     match first_arg {
//         "tag" => tag_help(get_prefix(msg.guild_id, state.clone()).await.as_str(), rest),
//         "ping" => ping_help(prefix.as_str()),
//
//         _ => help_msg(&prefix, msg),
//     }
// }

pub async fn execute(ctx: &CommandContext) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"**Commands:**
`{prefix}ping`
`{prefix}tag` (`{prefix}t`): Store and recall tags
`{prefix}eval`: Run sandboxed JS
`{prefix}help`: Print this help
**Admin Commands:**
`{prefix}settings`: Edit server settings
`{prefix}ban`: Ban a user from creating tags

`{prefix}<Any command> help` will give additional information about each command.
-# <angle_brackets> denote required arguments, while [square_brackets] denote optional arguments in all help commands.
    "#,
    );
    send_reply_ping_text(ctx, &content).await;
}
