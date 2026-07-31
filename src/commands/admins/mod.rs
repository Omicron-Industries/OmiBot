mod add;
mod list;
mod remove;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::util::permissions::get_admin_action_msg;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&add::INFO, &remove::INFO, &list::INFO],
}];

pub const INFO: CommandInfo = CommandInfo {
    command: "admin",
    usage: Some("<subcommand>"),
    full_desc: "Manage admins of the bot.",
    short_desc: Some("Manage admins."),
    aliases: &["admin"],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") => add::dispatch(ctx).await,
        Some("remove") | Some("rm") => remove::dispatch(ctx).await,
        Some("list") => list::dispatch(ctx).await,
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        _ => command_usage(ctx, INFO).await,
    }
}
