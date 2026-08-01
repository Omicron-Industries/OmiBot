use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::permissions::remove_admin_permission;
use crate::util::permissions::Permission;
use crate::util::tag::get_uid_from_user_text;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "admin permissions remove",
    usage: Some("<user> <permission>"),
    full_desc: "Remove a permission from an admin.",
    short_desc: None,
    aliases: &["rm"],
    further_help: Some(
        "Available permissions:\n`manage_tags`: Manage all tags, including the ability to edit, delete, and ban them.\n`manage_detect`: The ability to mark tags as detectable, triggering if the tag name appears anywhere in the message content.\n`manage_settings`: Manage the settings of the bot for the server.\n`manage_admins`: Manage other admins, including their permissions.",
    ),
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
pub async fn execute(ctx: &mut CommandContext) {
    let Some(arg) = ctx.consume_arg() else {
        return command_usage(ctx, crate::commands::admins::permissions::add::INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Err(_) => send_reply_ping_text(ctx, "Expected a user!").await,
        Ok(uid) => {
            match ctx.consume_arg() {
                Some(perm_str) => {
                    if let Some(perm) = Permission::from_name(&perm_str) {
                        if let Err(e) = remove_admin_permission(
                            ctx.get_guild_id(),
                            uid,
                            perm.value(),
                            &ctx.state.db_pool,
                        )
                        .await
                        {
                            send_reply_ping_text(
                                ctx,
                                format!("Failed to remove admin permission: {}", e).as_str(),
                            )
                            .await;
                        } else {
                            send_reply_ping_text(ctx, "Admin permission removed successfully.")
                                .await;
                        }
                    } else {
                        send_reply_ping_text(ctx, "Please provide a valid permission! See the help page for all permissions.").await;
                    }
                }
                None => {
                    send_reply_ping_text(ctx, "Expected a permission!").await;
                }
            }
        }
    }
}
