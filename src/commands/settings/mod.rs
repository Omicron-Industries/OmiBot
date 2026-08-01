mod prefix;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandCategory, CommandContext, CommandInfo, send_reply_ping_text};
use crate::util::permissions::get_admin_action_msg;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&prefix::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "settings",
    usage: Some("<subcommand>"),
    full_desc: "Manage server settings for the bot.",
    short_desc: Some("Manage server settings."),
    aliases: &[],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        Some("prefix") => prefix::dispatch(ctx).await,
        _ => command_usage(ctx, INFO).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {}
