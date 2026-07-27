use crate::commands::tag::util::alias::AliasTagContent;
use crate::commands::tag::util::embed::EmbedTagContent;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::Message;
use sqlx::types::chrono;

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
    id: i32,
    guild_id: i64,
    owner_id: i64,
    name: String,
    pub(crate) kind: TagKind,
    pub(crate) payload: Value,
    t_created: chrono::DateTime<chrono::Utc>,
    t_updated: chrono::DateTime<chrono::Utc>,
    enabled: bool,
    alias_target_name: Option<String>,
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
    pub fn with_msg(msg: &Message, name: String, kind: TagKind, payload: TagPayload) -> Self {
        CreateTagModel {
            guild_id: i64::from(msg.guild_id.unwrap()),
            owner_id: i64::from(msg.author.id),
            name,
            kind,
            payload,
        }
    }
}
