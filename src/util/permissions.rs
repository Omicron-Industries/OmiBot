use crate::commands::CommandContext;
use crate::db::permissions::is_admin;
use serenity::all::Permissions;

pub async fn get_admin_action_msg(ctx: &CommandContext) -> Option<String> {
    let guild = match ctx.get_guild_id().to_guild_cached(&ctx.serenity_ctx.cache) {
        Some(guild) => guild.clone(),
        None => return Some("Failed to get guild.".to_string()),
    };

    let member = match guild
        .member(&ctx.serenity_ctx.http, ctx.get_author_id())
        .await
    {
        Ok(member) => member,
        Err(e) => return Some(format!("Failed to get member: {:?}", e)),
    };

    let channel = match ctx.msg.channel(&ctx.serenity_ctx.http).await {
        Ok(channel) => match channel.guild() {
            Some(channel) => channel,
            None => return Some("Command was not run in a guild channel.".to_string()),
        },
        Err(e) => return Some(format!("Failed to get channel: {:?}", e)),
    };

    let permissions = guild.user_permissions_in(&channel, &member);

    if !permissions.contains(Permissions::ADMINISTRATOR) {
        return match is_admin(ctx.get_author_id(), ctx.get_guild_id(), &ctx.state.db_pool).await {
            Err(e) => Some(format!("Failed to retrieve admins status: {:?}", e)),
            Ok(false) => Some("This command required admins permissions to execute!".to_string()),
            _ => None,
        };
    }
    None
}
