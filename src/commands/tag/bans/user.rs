use crate::commands::help::command_help;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::bans::list_banned_users;

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
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    execute_bans_user(ctx).await;
}

pub async fn execute_bans_user(ctx: &mut CommandContext) {
    match list_banned_users(ctx.get_guild_id(), &ctx.state.db_pool).await {
        Ok(list) => {
            let mentions = list
                .iter()
                .map(|id| format!("<@{}>", id))
                .collect::<Vec<_>>()
                .join(", ");

            send_reply_ping_text(ctx, &format!("Banned users: {}", mentions)).await
        }
        Err(e) => {
            send_reply_ping_text(ctx, format!("Error listing banned users: {:?}", e).as_str()).await
        }
    }
}
