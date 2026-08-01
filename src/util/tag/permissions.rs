use crate::commands::CommandContext;
use crate::db::permissions::get_tag_editable_info;
use crate::util::permissions::is_discord_admin;
use std::env;

pub async fn get_tag_edit_permissions_msg(ctx: &CommandContext, tag_name: &str) -> Option<String> {
    if ctx.get_author_id().to_string() == env::var("OWNER_ID").expect("OWNER_ID must be set") {
        return None;
    }

    if is_discord_admin(ctx).await.ok() == Some(true) {
        return None;
    }

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
                    "The tag **{tag_name}** is banned, meaning it cannot be edited (by a non-admins)"
                ));
            }
            None
        }
    }
}
