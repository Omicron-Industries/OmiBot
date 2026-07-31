use crate::commands::help::command_help;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::permissions::list_admins;

pub const INFO: CommandInfo = CommandInfo {
    command: "admin list",
    usage: None,
    full_desc: "List all admins.",
    short_desc: None,
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
    match list_admins(ctx.get_guild_id(), &ctx.state.db_pool).await {
        Err(e) => send_reply_ping_text(ctx, format!("Error fetching admins: {e}").as_str()).await,
        Ok(members) => {
            send_reply_ping_text(
                ctx,
                format!(
                    "Current admins: {}",
                    members
                        .iter()
                        .map(|usr| format!("<@{usr}>"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
                .as_str(),
            )
            .await
        }
    }
}
