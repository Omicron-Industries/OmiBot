use crate::commands::CommandContext;
use crate::db::permissions::{get_tag_editable_info, is_admin};

pub async fn get_tag_edit_permissions_msg(ctx: &CommandContext, tag_name: &str) -> Option<String> {
    let info_query = get_tag_editable_info(
        ctx.get_guild_id(),
        ctx.get_author_id(),
        tag_name,
        &ctx.state.db_pool,
    )
    .await;

    match info_query {
        Err(e) => Some(format!("Failed to retrieve edit permissions: {:?}", e)),
        Ok(None) => Some(format!("Tag **{tag_name}** does not exist.")),
        Ok(Some(info)) => {
            if info.is_admin {
                return None;
            }
            if info.is_user_banned {
                return Some(
                    "You are banned from making/editing commands in this server.".to_string(),
                );
            }
            if !info.is_owner {
                return Some(format!("You are not the owner of the tag **{tag_name}**"));
            }
            if !info.tag_enabled {
                return Some(format!(
                    "The tag **{tag_name}** is banned, meaning it cannot be edited (by a non-admin)"
                ));
            }
            None
        }
    }
}

pub async fn get_admin_action_msg(ctx: &CommandContext) -> Option<String> {
    match is_admin(ctx.get_author_id(), ctx.get_guild_id(), &ctx.state.db_pool).await {
        Err(e) => Some(format!("Failed to retrieve admin status: {:?}", e)),
        Ok(false) => Some("This command required admin permissions to execute!".to_string()),
        _ => None,
    }
}
