use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_reply_ping_text};
use crate::db::permissions::add_admin;
use crate::util::tag::get_uid_from_user_text;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "admin add",
    usage: Some("<user>"),
    full_desc: "Add a user as an admin.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(arg) = ctx.consume_arg() else {
        return command_usage(ctx, crate::commands::admins::INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Err(_) => send_reply_ping_text(ctx, "Expected a user!").await,
        Ok(uid) => match add_admin(uid, ctx.get_guild_id(), None, &ctx.state.db_pool).await {
            Err(e) => {
                send_reply_ping_text(ctx, format!("Failed to add admins: {e}").as_str()).await
            }
            Ok(true) => send_reply_ping_text(ctx, "Admin added successfully!").await,
            Ok(false) => send_reply_ping_text(ctx, "Admin already exists!").await,
        },
    }
}
