use crate::commands::help::command_help;
use crate::commands::tag::util::db::get_users_tags_msg;
use crate::commands::tag::util::get_uid_from_user_text;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};

pub const INFO: CommandInfo = CommandInfo {
    command: "",
    usage: Some(""),
    full_desc: "",
    short_desc: Some(""),
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    match ctx.consume_arg() {
        None => {
            get_users_tags_msg(ctx.get_guild_id(), ctx.get_author_id(), &ctx.state.db_pool).await;
        }
        Some(arg) => {
            let uid = match get_uid_from_user_text(&arg) {
                Ok(uid) => uid,
                Err(_) => {
                    return send_reply_ping_text(ctx, "Expected a user as an argument!").await;
                }
            };
            get_users_tags_msg(ctx.get_guild_id(), uid, &ctx.state.db_pool).await;
        }
    }
}
