mod prefix;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::util::permissions::{get_admin_action_msg, Permission};

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
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageSettings).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let command = ctx.consume_arg();
    println!("Command: {}", command.as_deref().unwrap_or("None"));
    match command.as_deref() {
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        Some("prefix") => prefix::dispatch(ctx).await,
        _ => command_usage(ctx, INFO).await,
    }
}
