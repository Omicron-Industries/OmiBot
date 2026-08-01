use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_reply_ping_text};
use crate::db::tags::bans::unban_tag;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag unban tag",
    usage: Some("<tag_name>"),
    full_desc: "Unban a tag so it can be used, edited, and deleted again.",
    short_desc: Some("Unban a tag."),
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

    execute_unban_tag(&name, ctx).await;
}

pub async fn execute_unban_tag(name: &str, ctx: &mut CommandContext) {
    match unban_tag(ctx.get_guild_id(), &name, &ctx.state.db_pool).await {
        Ok(true) => send_reply_ping_text(ctx, format!("Unbanned tag **{name}**.").as_str()).await,
        Ok(false) => {
            send_reply_ping_text(
                ctx,
                format!("Tag **{name}** does not exist; cannot unban.").as_str(),
            )
            .await
        }
        Err(e) => send_reply_ping_text(ctx, format!("Error unbanning tag: {:?}", e).as_str()).await,
    }
}
