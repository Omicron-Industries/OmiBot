mod tag;
mod user;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::util::permissions::{get_admin_action_msg, Permission};

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&tag::INFO, &user::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag bans",
    usage: Some("[tag | user]"),
    full_desc: "List banned tags or users in the server.",
    short_desc: Some("List tag/user bans."),
    aliases: &[],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageTags).await {
        return send_reply_ping_text(ctx, &msg).await;
    }
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("tag") | None => tag::dispatch(ctx).await,
        Some("user") => user::dispatch(ctx).await,
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        _ => command_usage(ctx, INFO).await,
    }
}

// pub async fn execute(ctx: &mut CommandContext) {}
