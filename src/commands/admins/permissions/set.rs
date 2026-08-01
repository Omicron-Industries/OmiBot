use crate::commands::help::{command_help, command_usage};
use crate::commands::{CommandContext, CommandInfo, send_internal_error_msg, send_reply_ping_text};
use crate::db::permissions::set_admin_permissions;
use crate::util::tag::get_uid_from_user_text;
use std::str::FromStr;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "admin permissions set",
    usage: Some("<user> <permissions>"),
    full_desc: "Set the permissions of an admin.",
    short_desc: None,
    aliases: &[],
    further_help: Some(
        "Permissions are assigned based on the bits of a permissions integer. To calculate a permissions integer, add together the value of the permissions you want the admin to have:\n`manage_tags` = 1\n`manage_detect` = 2\n`manage_settings` = 4\n`manage_admins` = 8",
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
        return command_usage(ctx, INFO).await;
    };
    match get_uid_from_user_text(&arg) {
        Err(_) => send_reply_ping_text(ctx, "Expected a user!").await,
        Ok(uid) => match ctx.consume_arg() {
            Some(perm_str) => {
                if let Ok(perm) = i32::from_str(&perm_str) {
                    if let Err(e) =
                        set_admin_permissions(ctx.get_guild_id(), uid, perm, &ctx.state.db_pool)
                            .await
                    {
                        send_internal_error_msg(
                            ctx,
                            format!("Failed to set admin permission: {}", e).as_str(),
                        )
                        .await;
                    } else {
                        send_reply_ping_text(ctx, "Admin permissions set successfully!").await;
                    }
                } else {
                    send_reply_ping_text(ctx, "Please provide a valid permission integer! See the help page for all permissions.").await;
                }
            }
            None => {
                send_reply_ping_text(ctx, "Expected a permission!").await;
            }
        },
    }
}
