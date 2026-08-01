use crate::util::tag::CreateTagModel;
use crate::util::tag::TagKind;
use log::error;
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
        INSERT INTO tags (guild_id, owner_id, name, kind, payload, detect)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id;
        "#,
        tag.guild_id,
        tag.owner_id,
        tag.name,
        tag.kind as TagKind,
        serialized_payload,
        tag.detect
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
