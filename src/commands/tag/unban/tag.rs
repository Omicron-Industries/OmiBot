use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::bans::unban_tag;

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
    let Some(name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    execute_unban_tag(&name, ctx).await;
}

pub async fn execute_unban_tag(name: &str, ctx: &mut CommandContext) {
    match unban_tag(ctx.get_guild_id(), &name, &ctx.state.db_pool).await {
        Ok(true) => send_reply_ping_text(ctx, format!("Unbanned tag **{name}**.").as_str()).await,
        Ok(false) => {
            send_reply_ping_text(
                ctx,
                format!("Tag **{name}** does not exist; cannot unban.").as_str(),
            )
            .await
        }
        Err(e) => send_reply_ping_text(ctx, format!("Error unbanning tag: {:?}", e).as_str()).await,
    }
}
