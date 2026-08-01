use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::tag_name_validator;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::edit::rename_tag;
use crate::util::tag::permissions::get_tag_edit_permissions_msg;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag rename",
    usage: Some("<old_name> <new_name>"),
    full_desc: "Rename an existing tag.",
    short_desc: Some("Rename a tag."),
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
    if let Some(msg) = get_tag_edit_permissions_msg(ctx, &name).await {
        send_reply_ping_text(ctx, &msg).await
    }
    match ctx.consume_arg() {
        None => command_usage(ctx, INFO).await,
        Some(new) => match tag_name_validator(&new) {
            Some(msg) => send_reply_ping_text(ctx, &msg).await,
            None => match rename_tag(ctx.get_guild_id(), &name, &new, &ctx.state.db_pool).await {
                Err(e) => {
                    if let Some(db_err) = e.as_database_error() {
                        if db_err.is_unique_violation() {
                            return send_reply_ping_text(
                                ctx,
                                format!("Tag with name **{new}** already exists!").as_str(),
                            )
                            .await;
                        }
                    }
                    send_reply_ping_text(ctx, format!("Error deleting tag: {:?}", e).as_str()).await
                }
                Ok(_) => {
                    send_reply_ping_text(
                        ctx,
                        format!("Successfully deleted tag **{}**.", name).as_str(),
                    )
                    .await
                }
            },
        },
    }
}
