use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::settings::set_prefix;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "settings prefix",
    usage: Some("<prefix>"),
    full_desc: "Set the bot's command prefix for the server. Prefix must be a single character.",
    short_desc: Some("Set the server command prefix."),
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
    let Some(prefix) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };
    if prefix.len() != 1 {
        return send_reply_ping_text(ctx, "Expected a single character!").await;
    }

    match set_prefix(
        ctx.get_guild_id(),
        prefix.chars().next().unwrap(),
        &ctx.state,
    )
    .await
    {
        Ok(true) => send_reply_ping_text(ctx, format!("Set prefix to `{prefix}`.").as_str()).await,
        Ok(false) => {
            send_reply_ping_text(
                ctx,
                "Failed to update prefix. DB query succeeded, but 0 rows affected",
            )
            .await
        }
        Err(e) => {
            send_reply_ping_text(ctx, format!("Failed to update prefix: {:?}", e).as_str()).await
        }
    }
}
