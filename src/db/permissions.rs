use crate::db::DbId;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;

pub async fn has_edit_perms_on_tag(
    name: &str,
    uid: UserId,
    gid: GuildId,
    db: &PgPool,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
    SELECT EXISTS (
        SELECT 1
        FROM tags t
        WHERE t.guild_id = $1
          AND t.name = $2
          AND (
              t.owner_id = $3
              OR EXISTS (
                  SELECT 1
                  FROM admins a
                  WHERE a.guild_id = $1
                    AND a.member_id = $3
              )
          )
    ) as "exists!"
    "#,
        gid.db_id(),
        name,
        uid.db_id(),
    )
    .fetch_one(db)
    .await
}

pub async fn is_admin(uid: UserId, gid: GuildId, db: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
    SELECT EXISTS (
        SELECT 1
        FROM admins
        WHERE guild_id = $1
          AND member_id = $2
    ) AS "exists!"
    "#,
        gid.db_id(),
        uid.db_id(),
    )
    .fetch_one(db)
    .await
}

struct TagEditableInfo {
    is_owner: bool,
    is_admin: bool,
    is_user_banned: bool,
    tag_enabled: bool,
}

pub async fn get_tag_editable_info(
    gid: GuildId,
    uid: UserId,
    tag_name: &str,
    db: &PgPool,
) -> Result<Option<TagEditableInfo>, sqlx::Error> {
    sqlx::query_as!(
        TagEditableInfo,
        r#"
        SELECT
            (t.owner_id = $2) AS "is_owner!",
            EXISTS (
                SELECT 1
                FROM admins a
                WHERE a.guild_id = $1
                  AND a.member_id = $2
            ) AS "is_admin!",
            EXISTS (
                SELECT 1
                FROM bans b
                WHERE b.guild_id = $1
                  AND b.user_id = $2
            ) AS "is_user_banned!",
            t.enabled AS "tag_enabled!"
        FROM tags t
        WHERE t.guild_id = $1
          AND t.name = $3
        "#,
        gid.db_id(),
        uid.db_id(),
        tag_name,
    )
    .fetch_optional(db)
    .await
}
