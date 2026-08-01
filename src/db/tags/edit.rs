use crate::db::DbId;
use crate::util::tag::{TagKind, TagPayload};
use log::error;
use serenity::all::{GuildId, UserId};
use sqlx::{Error, PgPool};

pub async fn rename_tag(
    gid: GuildId,
    old_name: &str,
    new_name: &str,
    db: &PgPool,
) -> Result<bool, Error> {
    let res = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET name = $3,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        "#,
        gid.db_id(),
        old_name,
        new_name,
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn change_tag_owner(
    gid: GuildId,
    tag_name: &str,
    new_owner: UserId,
    db: &PgPool,
) -> Result<bool, Error> {
    let res = sqlx::query_scalar!(
        r#"
        UPDATE tags
        SET owner_id = $3,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        "#,
        gid.db_id(),
        tag_name,
        new_owner.db_id(),
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() > 0)
}

pub enum EditTagError {
    Serialize,
    DB(Error),
}

pub async fn edit_tag_content(
    gid: GuildId,
    tag_name: &str,
    payload: TagPayload,
    db: &PgPool,
) -> Result<bool, EditTagError> {
    let serialized_payload = match serde_json::to_value(&payload) {
        Ok(payload) => payload,
        Err(e) => {
            error!("Failed to serialize tag payload: {}", e);
            return Err(EditTagError::Serialize);
        }
    };

    match sqlx::query!(
        r#"
        UPDATE tags
        SET payload = $3,
            kind = $4,
            t_updated = CURRENT_TIMESTAMP
        WHERE guild_id = $1
          AND name = $2
        "#,
        gid.db_id(),
        tag_name,
        serialized_payload,
        TagKind::from_payload(&payload) as TagKind,
    )
    .execute(db)
    .await
    {
        Err(e) => {
            error!("Failed to create tag in db: {}", e);
            Err(EditTagError::DB(e))
        }
        Ok(res) => Ok(res.rows_affected() > 0),
    }
}

pub async fn delete_tag(gid: GuildId, tag_name: &str, db: &PgPool) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
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

    Ok(res.rows_affected() > 0)
}
