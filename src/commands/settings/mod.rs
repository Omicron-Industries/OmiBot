mod prefix;

use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::util::permissions::get_admin_action_msg;
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
    if let Some(msg) = get_admin_action_msg(ctx).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        Some("prefix") => prefix::dispatch(ctx).await,
        _ => command_usage(ctx, INFO).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {}
