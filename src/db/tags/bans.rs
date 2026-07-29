use crate::db::DbId;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;

pub async fn ban_tag(gid: GuildId, tag_name: &str, db: &PgPool) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET enabled = false,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        RETURNING true AS "changed!"
        "#,
        gid.db_id(),
        tag_name,
    )
    .fetch_optional(db)
    .await?;

    Ok(changed.unwrap_or(false))
}

pub async fn unban_tag(gid: GuildId, tag_name: &str, db: &PgPool) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET enabled = true,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        RETURNING true AS "changed!"
        "#,
        gid.db_id(),
        tag_name,
    )
    .fetch_optional(db)
    .await?;

    Ok(changed.unwrap_or(false))
}

pub async fn ban_user(
    uid: UserId,
    banned_by: UserId,
    gid: GuildId,
    db: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO bans (guild_id, user_id, banned_by)
        VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, user_id) DO NOTHING
        "#,
        gid.db_id(),
        uid.db_id(),
        banned_by.db_id(),
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn unban_user(uid: UserId, gid: GuildId, db: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM bans
        WHERE guild_id = $1
          AND user_id = $2
        "#,
        gid.db_id(),
        uid.db_id(),
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_banned_users(gid: GuildId, db: &PgPool) -> Result<Vec<UserId>, sqlx::Error> {
    let ids = sqlx::query_scalar!(
        r#"
        SELECT user_id
        FROM bans
        WHERE guild_id = $1
        "#,
        gid.db_id(),
    )
    .fetch_all(db)
    .await?;

    Ok(ids.into_iter().map(|val| UserId::new(val as u64)).collect())
}

pub async fn list_banned_tags(gid: GuildId, db: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT name
        FROM tags
        WHERE guild_id = $1
          AND enabled = false
        "#,
        gid.db_id(),
    )
    .fetch_all(db)
    .await
}
