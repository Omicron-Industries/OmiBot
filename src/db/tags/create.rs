use crate::commands::tag::util::TagKind;
use crate::commands::tag::util::{CreateTagModel, FetchTagModel};
use crate::db::DbId;
use log::error;
use serenity::all::GuildId;
use sqlx::error::DatabaseError;
use sqlx::{Error, PgPool};

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
