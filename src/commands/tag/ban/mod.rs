mod tag;
mod user;

use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::ban::tag::execute_ban_tag;
use crate::commands::tag::ban::user::execute_ban_user;
use crate::commands::tag::bans;
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::util::permissions::{get_admin_action_msg, Permission};
use crate::util::tag::get_uid_from_user_text;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&tag::INFO, &user::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag ban",
    usage: Some("(<tag_name> | <user> | <subcommand>)"),
    full_desc: "Ban a tag or user from using tags.",
    short_desc: Some("Ban a tag or user."),
    aliases: &[],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageTags).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("tag") => tag::dispatch(ctx).await,
        Some("user") => user::dispatch(ctx).await,
        Some("list") => bans::dispatch(ctx).await,
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(arg) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Ok(uid) => execute_ban_user(uid, ctx).await,
        Err(_) => execute_ban_tag(&arg, ctx).await,
    }
}
