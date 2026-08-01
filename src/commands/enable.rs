use crate::commands::help::command_help;
use crate::commands::{get_prefix, send_reply_ping_text, CommandContext, CommandInfo};

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "enable",
    usage: None,
    full_desc: "Enables or disables the bot entirely.",
    short_desc: None,
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

pub async fn execute(ctx: &CommandContext) {
    match ctx.state.guild_cache.enabled.get(&ctx.get_guild_id()).await {
        Some(true) => {
            ctx.state
                .guild_cache
                .enabled
                .insert(ctx.get_guild_id(), false)
                .await;
            send_reply_ping_text(ctx, format!("Disabled the bot entirely. Re-enable with `{}enable`; No other command will work until the bot is re-enabled.", get_prefix(ctx).await).as_str()).await;
        }
        Some(false) | None => {
            ctx.state
                .guild_cache
                .enabled
                .insert(ctx.get_guild_id(), true)
                .await;
            send_reply_ping_text(ctx, "Enabled the bot.").await
        }
    }
}
