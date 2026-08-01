use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_reply_ping_text};
use crate::db::tags::edit::delete_tag;
use crate::util::tag::permissions::get_tag_edit_permissions_msg;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag delete",
    usage: Some("<tag_name>"),
    full_desc: "Delete a tag.",
    short_desc: None,
    aliases: &["del"],
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
        return command_usage(ctx, crate::commands::tag::chown::INFO).await;
    };
    if let Some(msg) = get_tag_edit_permissions_msg(ctx, &name).await {
        send_reply_ping_text(ctx, &msg).await
    }
    match delete_tag(ctx.get_guild_id(), &name, &ctx.state.db_pool).await {
        Err(e) => send_reply_ping_text(ctx, format!("Error deleting tag: {:?}", e).as_str()).await,
        Ok(_) => {
            send_reply_ping_text(
                ctx,
                format!("Successfully deleted tag **{}**.", name).as_str(),
            )
            .await
        }
    }
}
