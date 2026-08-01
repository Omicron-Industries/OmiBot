use crate::db::DbId;
use crate::BotState;
use serenity::all::GuildId;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn toggle_detectable(
    gid: GuildId,
    tag_name: &str,
    state: &BotState,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE tags
        SET
            t_updated = CURRENT_TIMESTAMP,
            detect = NOT detect
        WHERE guild_id = $1
          AND name = $2
        RETURNING detect
        "#,
        gid.db_id(),
        tag_name,
    )
    .fetch_one(&state.db_pool)
    .await?;

    if row.detect {
        add_detectable_to_cache(state, gid, tag_name);
    } else {
        if let Some(mut tags) = state.guild_cache.detectable_tags.get(&gid).await {
            tags.retain(|t| t != tag_name);
            state.guild_cache.detectable_tags.insert(gid, tags).await;
        }
    }

    Ok(row.detect)
}

pub async fn add_detectable_to_cache(state: &BotState, gid: GuildId, tag_name: &str) {
    if let Some(mut tags) = state.guild_cache.detectable_tags.get(&gid).await {
        tags.push(tag_name.to_string());
        state.guild_cache.detectable_tags.insert(gid, tags).await;
    }
}

pub async fn get_detectable_tags(
    db: &PgPool,
) -> Result<HashMap<GuildId, Vec<String>>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT guild_id, name
        FROM tags
          WHERE detect = true
        "#,
    )
    .fetch_all(db)
    .await?;

    let mut map = HashMap::new();

    for row in rows {
        map.entry(GuildId::new(row.guild_id as u64))
            .or_insert_with(Vec::new)
            .push(row.name);
    }

    Ok(map)
}
