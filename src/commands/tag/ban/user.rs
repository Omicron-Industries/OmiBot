use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_reply_ping_text};
use crate::db::tags::bans::ban_user;
use crate::util::tag::get_uid_from_user_text;
use serenity::all::UserId;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag ban user",
    usage: Some("<user>"),
    full_desc: "Ban a user from creating or editing tags.",
    short_desc: Some("Ban a user from tags."),
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

    execute_ban_user(uid, ctx).await;
}

pub async fn execute_ban_user(uid: UserId, ctx: &mut CommandContext) {
    match ban_user(
        uid,
        ctx.get_author_id(),
        ctx.get_guild_id(),
        &ctx.state.db_pool,
    )
    .await
    {
        Ok(true) => {
            send_reply_ping_text(
                ctx,
                format!("Banned user <@{uid}>. They can no longer create or edit commands.")
                    .as_str(),
            )
            .await
        }
        Ok(false) => {
            send_reply_ping_text(
                ctx,
                format!("User <@{uid}> does not exist; cannot ban.").as_str(),
            )
            .await
        }
        Err(e) => send_reply_ping_text(ctx, format!("Error banning user: {:?}", e).as_str()).await,
    }
}
