mod add;
mod list;
pub mod permissions;
mod remove;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandCategory, CommandContext, CommandInfo, send_reply_ping_text};
use crate::util::permissions::{Permission, get_admin_action_msg};

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&add::INFO, &remove::INFO, &list::INFO, &permissions::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "admin",
    usage: Some("<subcommand>"),
    full_desc: "Manage admins of the bot.",
    short_desc: Some("Manage admins."),
    aliases: &["admin"],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageAdmins).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") => add::dispatch(ctx).await,
        Some("remove") | Some("rm") => remove::dispatch(ctx).await,
        Some("list") => list::dispatch(ctx).await,
        Some("permissions") | Some("perms") => permissions::dispatch(ctx).await,
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        _ => command_usage(ctx, INFO).await,
    }
}
