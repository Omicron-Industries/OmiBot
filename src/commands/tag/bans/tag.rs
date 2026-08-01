use crate::commands::help::command_help;
use crate::commands::{CommandContext, CommandInfo, send_internal_error_msg, send_reply_ping_text};
use crate::db::tags::bans::list_banned_tags;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag bans tag",
    usage: None,
    full_desc: "List all banned tags in the server.",
    short_desc: Some("List banned tags."),
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
    execute_bans_tag(ctx).await;
}

pub async fn execute_bans_tag(ctx: &mut CommandContext) {
    match list_banned_tags(ctx.get_guild_id(), &ctx.state.db_pool).await {
        Ok(list) => {
            send_reply_ping_text(
                ctx,
                format!("Banned tags:**{}**", list.join("**, **")).as_str(),
            )
            .await
        }
        Err(e) => {
            send_internal_error_msg(ctx, format!("Error listing banned tags: {:?}", e).as_str())
                .await
        }
    }
}
