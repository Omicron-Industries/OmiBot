use crate::commands::help::command_help;
use crate::commands::tag::info::get_users_tags_msg;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::util::tag::get_uid_from_user_text;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag list",
    usage: Some("[user]"),
    full_desc: "List all tags owned by yourself or a specified user.",
    short_desc: Some("List tags owned by a user."),
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
    match ctx.consume_arg() {
        None => {
            send_reply_ping_text(
                ctx,
                &get_users_tags_msg(ctx.get_guild_id(), ctx.get_author_id(), &ctx.state.db_pool)
                    .await,
            )
            .await;
        }
        Some(arg) => {
            let uid = match get_uid_from_user_text(&arg) {
                Ok(uid) => uid,
                Err(_) => {
                    return send_reply_ping_text(ctx, "Expected a user as an argument!").await;
                }
            };
            send_reply_ping_text(
                ctx,
                &get_users_tags_msg(ctx.get_guild_id(), uid, &ctx.state.db_pool).await,
            )
            .await;
        }
    }
}
