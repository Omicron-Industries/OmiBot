use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::search::search_tags;

pub const INFO: CommandInfo = CommandInfo {
    command: "tag search",
    usage: Some("<query>"),
    full_desc: "Search for tags matching a query string.",
    short_desc: Some("Search for tags."),
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
    let Some(name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    match search_tags(ctx.get_guild_id(), &name, &ctx.state.db_pool).await {
        Ok(tags) => {
            send_reply_ping_text(ctx, format!("Found: **{}**", tags.join("**, **")).as_str()).await
        }
        Err(e) => {
            send_reply_ping_text(
                ctx,
                format!("An error occurred while searching for tags: {:?}", e).as_str(),
            )
            .await
        }
    }
}
