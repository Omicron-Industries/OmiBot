use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_internal_error_msg, send_reply_ping_text};
use crate::db::tags::bans::ban_tag;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag ban tag",
    usage: Some("<tag_name>"),
    full_desc: "Ban a tag so it can no longer be executed, edited, or deleted.",
    short_desc: Some("Ban a tag."),
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
    let Some(name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    execute_ban_tag(&name, ctx).await;
}

pub async fn execute_ban_tag(name: &str, ctx: &mut CommandContext) {
    match ban_tag(ctx.get_guild_id(), &name, &ctx.state.db_pool).await {
        Ok(true) => send_reply_ping_text(ctx, format!("Banned tag **{name}**. It can no longer be executed, edited or deleted, except by admins.").as_str()).await,
        Ok(false) => send_reply_ping_text(ctx, format!("Tag **{name}** does not exist; cannot ban.").as_str()).await,
        Err(e) => send_internal_error_msg(ctx, format!("Error banning tag: {:?}", e).as_str()).await,
    }
}
