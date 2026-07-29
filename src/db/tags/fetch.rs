use crate::commands::tag::util::FetchTagModel;
use crate::commands::tag::util::TagKind;
use crate::db::DbId;
use serenity::all::GuildId;
use sqlx::{Error, PgPool};

pub async fn fetch_tag_resolved(
    name: &str,
    gid: GuildId,
    db: &PgPool,
) -> Result<Option<FetchTagModel>, Error> {
    sqlx::query_as!(
        FetchTagModel,
        r#"
        SELECT
            COALESCE(target.id, source.id) AS "id!",
            COALESCE(target.guild_id, source.guild_id) AS "guild_id!",
            COALESCE(target.owner_id, source.owner_id) AS "owner_id!",
            COALESCE(target.name, source.name) AS "name!",
            COALESCE(target.kind, source.kind) AS "kind!: TagKind",
            COALESCE(target.payload, source.payload) AS "payload!",
            COALESCE(target.t_created, source.t_created) AS "t_created!",
            COALESCE(target.t_updated, source.t_updated) AS "t_updated!",
            COALESCE(target.enabled, source.enabled) AS "enabled!",
            null as alias_target_name
        FROM tags AS source
        LEFT JOIN tags AS target
            ON source.kind = 'alias'
           AND target.id = (source.payload->>'target_id')::int
        WHERE source.guild_id = $1
          AND source.name = $2;
        "#,
        gid.db_id(),
        name
    )
    .fetch_optional(db)
    .await
}

pub async fn fetch_tag(
    name: &str,
    gid: GuildId,
    db: &PgPool,
) -> Result<Option<FetchTagModel>, sqlx::Error> {
    sqlx::query_as!(
        FetchTagModel,
        r#"
        SELECT
            t.id AS "id!",
            t.guild_id AS "guild_id!",
            t.owner_id AS "owner_id!",
            t.name AS "name!",
            t.kind AS "kind!: TagKind",
            t.payload AS "payload!",
            t.t_created AS "t_created!",
            t.t_updated AS "t_updated!",
            t.enabled AS "enabled!",
            target.name AS "alias_target_name: Option<String>"
        FROM tags t
        LEFT JOIN tags target
            ON target.id = t.target_id
        WHERE t.guild_id = $1
          AND t.name = $2;
    "#,
        gid.db_id(),
        name
    )
    .fetch_optional(db)
    .await
}

pub async fn fetch_owners_tags(
    oid: i64,
    gid: i64,
    db: &PgPool,
) -> Result<Vec<FetchTagModel>, Error> {
    sqlx::query_as!(
        FetchTagModel,
        r#"
        SELECT
            id,
            guild_id,
            owner_id as "owner_id!",
            name,
            kind AS "kind: TagKind",
            payload,
            t_created,
            t_updated,
            enabled,
            null as alias_target_name
        FROM tags
        WHERE guild_id = $1
          AND owner_id = $2;
    "#,
        gid,
        oid
    )
    .fetch_all(db)
    .await
}
