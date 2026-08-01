use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_reply_ping_text};
use crate::db::tags::bans::unban_user;
use crate::util::tag::get_uid_from_user_text;
use serenity::all::UserId;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag unban user",
    usage: Some("<user>"),
    full_desc: "Unban a user so they can create and edit tags again.",
    short_desc: Some("Unban a user from tags."),
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
        return command_usage(ctx, INFO).await;
    };
    let Ok(uid) = get_uid_from_user_text(&arg) else {
        return command_usage(ctx, INFO).await;
    };

    execute_unban_user(uid, ctx).await;
}

pub async fn execute_unban_user(uid: UserId, ctx: &mut CommandContext) {
    match unban_user(uid, ctx.get_guild_id(), &ctx.state.db_pool).await {
        Ok(true) => send_reply_ping_text(ctx, format!("Unanned user <@{uid}>.").as_str()).await,
        Ok(false) => {
            send_reply_ping_text(
                ctx,
                format!("User <@{uid}> was not banned. Nothing was changed.").as_str(),
            )
            .await
        }
        Err(e) => {
            send_reply_ping_text(ctx, format!("Error unbanning user: {:?}", e).as_str()).await
        }
    }
}
