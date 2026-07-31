use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::util::get_uid_from_user_text;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::permissions::remove_admin;

pub const INFO: CommandInfo = CommandInfo {
    command: "admin remove",
    usage: Some("<user>"),
    full_desc: "Remove an admin.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}
pub async fn execute(ctx: &mut CommandContext) {
    let Some(arg) = ctx.consume_arg() else {
        return command_usage(ctx, crate::commands::admins::INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Err(e) => send_reply_ping_text(ctx, "Expected a user!").await,
        Ok(uid) => match remove_admin(uid, ctx.get_guild_id(), &ctx.state.db_pool).await {
            Err(e) => {
                send_reply_ping_text(ctx, format!("Failed to remove admins: {e}").as_str()).await
            }
            Ok(true) => send_reply_ping_text(ctx, "Admin removed successfully!").await,
            Ok(false) => send_reply_ping_text(ctx, "User was not an admins!").await,
        },
    }
}
