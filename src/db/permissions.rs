use crate::db::DbId;
use serenity::all::{GuildId, UserId};
use sqlx::{Error, PgPool};
// pub async fn has_edit_perms_on_tag(
//     name: &str,
//     uid: UserId,
//     gid: GuildId,
//     db: &PgPool,
// ) -> Result<bool, sqlx::Error> {
//     sqlx::query_scalar!(
//         r#"
//     SELECT EXISTS (
//         SELECT 1
//         FROM tags t
//         WHERE t.guild_id = $1
//           AND t.name = $2
//           AND (
//               t.owner_id = $3
//               OR EXISTS (
//                   SELECT 1
//                   FROM admins a
//                   WHERE a.guild_id = $1
//                     AND a.member_id = $3
//               )
//           )
//     ) as "exists!"
//     "#,
//         gid.db_id(),
//         name,
//         uid.db_id(),
//     )
//     .fetch_one(db)
//     .await
// }

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

pub struct TagEditableInfo {
    pub is_owner: bool,
    pub is_admin: bool,
    pub is_user_banned: bool,
    pub tag_enabled: bool,
}

pub async fn get_tag_editable_info(
    gid: GuildId,
    uid: UserId,
    tag_name: &str,
    db: &PgPool,
) -> Result<Option<TagEditableInfo>, Error> {
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

pub async fn add_admin(uid: UserId, gid: GuildId, db: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO admins (guild_id, member_id)
        VALUES ($1, $2)
        ON CONFLICT (guild_id, member_id) DO NOTHING
        "#,
        gid.db_id(),
        uid.db_id(),
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn remove_admin(uid: UserId, gid: GuildId, db: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM admins
        WHERE guild_id = $1
          AND member_id = $2
        "#,
        gid.db_id(),
        uid.db_id(),
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_admins(gid: GuildId, db: &PgPool) -> Result<Vec<UserId>, sqlx::Error> {
    let admins = sqlx::query!(
        r#"
        SELECT member_id
        FROM admins
        WHERE guild_id = $1
        ORDER BY t_created
        "#,
        gid.db_id(),
    )
    .fetch_all(db)
    .await?;

    Ok(admins
        .into_iter()
        .map(|admin| UserId::new(admin.member_id as u64))
        .collect())
}
