pub mod add;
pub mod remove;
pub mod set;

use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandCategory, CommandContext, CommandInfo};
use crate::db::permissions::admin_permissions;
use crate::util::permissions::{get_admin_action_msg, Permission};
use crate::util::tag::get_uid_from_user_text;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[&CommandCategory {
    name: None,
    description: None,
    commands: &[&add::INFO, &remove::INFO, &set::INFO],
}];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "admin permissions",
    usage: Some("<subcommand>"),
    full_desc: "Manage admin permissions.",
    short_desc: None,
    aliases: &["perms"],
    further_help: None,
    subcommands: Some(SUBCOMMANDS),
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageAdmins).await {
        return send_reply_ping_text(ctx, &msg).await;
    }
    let mut orig_ctx = ctx.clone();

    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") => add::dispatch(ctx).await,
        Some("remove") | Some("rm") => remove::dispatch(ctx).await,
        Some("set") => set::dispatch(ctx).await,
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        Some(_) => execute(&mut orig_ctx).await,
        None => command_usage(ctx, INFO).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(arg) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Err(_) => send_reply_ping_text(ctx, "Expected a user!").await,
        Ok(uid) => match admin_permissions(ctx.get_guild_id(), uid, &ctx.state.db_pool).await {
            Err(e) => {
                send_reply_ping_text(ctx, format!("Failed to read permissions: {e}").as_str()).await
            }
            Ok(perms) => {
                send_reply_ping_text(
                    ctx,
                    format!(
                        "<@{uid}>'s permissions: {}",
                        perms
                            .iter()
                            .map(|e| e.name())
                            .collect::<Vec<&str>>()
                            .join(", ")
                    )
                    .as_str(),
                )
                .await
            }
        },
    }
}
