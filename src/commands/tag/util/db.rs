use crate::commands::tag::util::{CreateTagModel, TagKind};
use crate::commands::tag::util::{DbId, FetchTagModel};
use crate::commands::CommandContext;
use crate::BotState;
use log::error;
use serenity::all::{GuildId, User, UserId};
use sqlx::{Error, PgPool};
use std::sync::Arc;

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

pub enum CreateTagError {
    Serialize,
    Exists,
    DB(Error),
}

pub async fn create_tag(db: &PgPool, tag: CreateTagModel) -> Result<i32, CreateTagError> {
    let serialized_payload = match serde_json::to_value(tag.payload) {
        Ok(payload) => payload,
        Err(e) => {
            error!("Failed to serialize tag payload: {}", e);
            return Err(CreateTagError::Serialize);
        }
    };

    match sqlx::query_scalar!(
        r#"
        INSERT INTO tags (guild_id, owner_id, name, kind, payload)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id;
        "#,
        tag.guild_id,
        tag.owner_id,
        tag.name,
        tag.kind as TagKind,
        serialized_payload
    )
    .fetch_one(db)
    .await
    {
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err(CreateTagError::Exists);
                }
            }
            error!("Failed to create tag in db: {}", e);
            Err(CreateTagError::DB(e))
        }
        Ok(id) => Ok(id),
    }
}

async fn fetch_owners_tags(oid: i64, gid: i64, db: &PgPool) -> Result<Vec<FetchTagModel>, Error> {
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

pub async fn get_users_tags_msg(gid: GuildId, uid: UserId, db: &PgPool) -> String {
    let tags = match fetch_owners_tags(uid.db_id(), gid.db_id(), db).await {
        Ok(tags) => tags,
        Err(e) => {
            error!("Error fetching tags of user {uid}");
            return format!("Error fetching tags of user <@{uid}>!");
        }
    };

    if tags.is_empty() {
        return format!("User <@{uid}> does not own any tags!");
    }

    let tag_list = tags
        .iter()
        .map(|tag| tag.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    format!("<@{uid}>'s Tags:\n{tag_list}")
}
