use crate::commands::tag::util::alias::AliasTagContent;
use crate::commands::tag::util::embed::EmbedTagContent;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::{send_reply_ping_text, CommandContext};
use crate::db::tags::create::{create_tag, CreateTagError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{Message, UserId};
use sqlx::types::chrono;
use std::num::ParseIntError;

pub mod alias;
pub mod embed;
pub mod permissions;
pub mod script;
pub mod text;

#[derive(sqlx::Type, Debug, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "tag_kind", rename_all = "lowercase")]
pub enum TagKind {
    Text,
    Alias,
    Embed,
    Script,
}
impl TagKind {
    pub fn from_payload(tag_payload: &TagPayload) -> Self {
        match tag_payload {
            TagPayload::Text(_) => TagKind::Text,
            TagPayload::Alias(_) => TagKind::Alias,
            TagPayload::Embed(_) => TagKind::Embed,
            TagPayload::Script(_) => TagKind::Script,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagPayload {
    Text(TextTagContent),
    Alias(AliasTagContent),
    Embed(EmbedTagContent),
    Script(ScriptTagContent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchTagModel {
    pub id: i32,
    pub guild_id: i64,
    pub owner_id: i64,
    pub name: String,
    pub kind: TagKind,
    pub payload: Value,
    pub t_created: chrono::DateTime<chrono::Utc>,
    pub t_updated: chrono::DateTime<chrono::Utc>,
    pub enabled: bool,
    pub alias_target_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTagModel {
    pub guild_id: i64,
    pub owner_id: i64,
    pub name: String,
    pub kind: TagKind,
    pub payload: TagPayload,
}

impl CreateTagModel {
    pub fn with_msg(msg: &Message, name: &str, payload: TagPayload) -> Self {
        CreateTagModel {
            guild_id: i64::from(msg.guild_id.unwrap()),
            owner_id: i64::from(msg.author.id),
            name: name.to_string(),
            kind: TagKind::from_payload(&payload),
            payload,
        }
    }
    pub fn with_ctx(ctx: &CommandContext, name: &str, payload: TagPayload) -> Self {
        Self::with_msg(&ctx.msg, name, payload)
    }
}

pub async fn try_create_tag(ctx: &CommandContext, tag: CreateTagModel) {
    let name = tag.name.clone();
    match create_tag(&ctx.state.db_pool, tag).await {
        Ok(_) => send_reply_ping_text(ctx, format!("Created tag **{name}**.").as_str()).await,
        Err(CreateTagError::Exists) => {
            send_reply_ping_text(ctx, format!("Tag **{name}** already exists.").as_str()).await
        }
        Err(CreateTagError::Serialize) => {
            send_reply_ping_text(ctx, "Failed to serialize tag.").await
        }
        Err(CreateTagError::DB(e)) => {
            send_reply_ping_text(
                ctx,
                format!(
                    "Error creating tag: {e}\n Please report this error to <@435572469496020992>"
                )
                .as_str(),
            )
            .await
        }
    };
}

pub fn get_uid_from_user_text(text: &str) -> Result<UserId, ParseIntError> {
    text.trim_start_matches("<@")
        .trim_start_matches('!')
        .trim_end_matches('>')
        .parse::<u64>()
        .map(UserId::new)
}
