use crate::db::DbId;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;

pub async fn rename_tag(
    gid: GuildId,
    old_name: &str,
    new_name: &str,
    db: &PgPool,
) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET name = $3,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        RETURNING true AS "changed!"
        "#,
        gid.db_id(),
        old_name,
        new_name,
    )
    .fetch_optional(db)
    .await?;

    Ok(changed.unwrap_or(false))
}

pub async fn change_tag_owner(
    gid: GuildId,
    tag_name: &str,
    new_owner: UserId,
    db: &PgPool,
) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET owner_id = $3,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        RETURNING true AS "changed!"
        "#,
        gid.db_id(),
        tag_name,
        new_owner.db_id(),
    )
    .fetch_optional(db)
    .await?;

    Ok(changed.unwrap_or(false))
}

pub async fn edit_tag_content(
    gid: GuildId,
    tag_name: &str,
    payload: serde_json::Value,
    db: &PgPool,
) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET payload = $3,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        RETURNING true AS "changed!"
        "#,
        gid.db_id(),
        tag_name,
        payload,
    )
    .fetch_optional(db)
    .await?;

    Ok(changed.unwrap_or(false))
}

pub async fn delete_tag(gid: GuildId, tag_name: &str, db: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM tags
        WHERE guild_id = $1
          AND name = $2
        "#,
        gid.db_id(),
        tag_name,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}
