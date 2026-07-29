use crate::commands::tag::util::alias::AliasTagContent;
use crate::commands::tag::util::db::{create_tag, CreateTagError};
use crate::commands::tag::util::embed::EmbedTagContent;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::{send_reply_ping_text, CommandContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{GuildId, Message, UserId};
use sqlx::types::chrono;
use std::num::ParseIntError;

pub mod alias;
pub mod db;
pub mod embed;
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
            TagPayload::Text(tag_content) => TagKind::Text,
            TagPayload::Alias(tag_content) => TagKind::Alias,
            TagPayload::Embed(tag_content) => TagKind::Embed,
            TagPayload::Script(tag_content) => TagKind::Script,
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
    pub(crate) id: i32,
    guild_id: i64,
    pub(crate) owner_id: i64,
    pub(crate) name: String,
    pub(crate) kind: TagKind,
    pub(crate) payload: Value,
    pub(crate) t_created: chrono::DateTime<chrono::Utc>,
    pub(crate) t_updated: chrono::DateTime<chrono::Utc>,
    enabled: bool,
    pub(crate) alias_target_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTagModel {
    guild_id: i64,
    owner_id: i64,
    name: String,
    kind: TagKind,
    payload: TagPayload,
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

trait DbId {
    fn db_id(&self) -> i64;
}

impl DbId for GuildId {
    fn db_id(&self) -> i64 {
        self.get() as i64
    }
}

impl DbId for UserId {
    fn db_id(&self) -> i64 {
        self.get() as i64
    }
}

pub fn get_uid_from_user_text(text: &str) -> Result<UserId, ParseIntError> {
    text.trim_start_matches("<@")
        .trim_start_matches('!')
        .trim_end_matches('>')
        .parse::<u64>()
        .map(UserId::new)
}
