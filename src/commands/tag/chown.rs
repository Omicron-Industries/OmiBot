use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_internal_error_msg, send_reply_ping_text};
use crate::db::tags::edit::change_tag_owner;
use crate::util::tag::get_uid_from_user_text;
use crate::util::tag::permissions::get_tag_edit_permissions_msg;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag chown",
    usage: Some("<tag_name> <user>"),
    full_desc: "Transfer ownership of a tag to another user.",
    short_desc: Some("Transfer tag ownership."),
    aliases: &["transfer"],
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
    if let Some(msg) = get_tag_edit_permissions_msg(ctx, &name).await {
        send_reply_ping_text(ctx, &msg).await
    }
    match ctx.consume_arg() {
        None => command_usage(ctx, INFO).await,
        Some(text) => match get_uid_from_user_text(&text) {
            Err(_) => command_usage(ctx, INFO).await,
            Ok(uid) => {
                match change_tag_owner(ctx.get_guild_id(), &name, uid, &ctx.state.db_pool).await {
                    Err(e) => {
                        send_internal_error_msg(
                            ctx,
                            format!("Error changing ownership: {:?}", e).as_str(),
                        )
                        .await
                    }

                    Ok(_) => {
                        send_reply_ping_text(
                            ctx,
                            format!(
                                "Successfully changed ownership of tag **{}** to <@{}>.",
                                name, uid
                            )
                            .as_str(),
                        )
                        .await
                    }
                }
            }
        },
    }
}
