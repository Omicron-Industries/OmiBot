mod tag;
mod user;

use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::unban::tag::execute_unban_tag;
use crate::commands::tag::unban::user::execute_unban_user;
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::util::permissions::{get_admin_action_msg, Permission};
use crate::util::tag::get_uid_from_user_text;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&tag::INFO, &user::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag unban",
    usage: Some("(<tag_name> | <user> | <subcommand>)"),
    full_desc: "Unban a tag or user.",
    short_desc: None,
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
        Ok(uid) => execute_unban_user(uid, ctx).await,
        Err(_) => execute_unban_tag(&arg, ctx).await,
    }
}
