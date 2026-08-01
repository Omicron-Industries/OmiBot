use crate::commands::{send_internal_error_msg, send_reply_ping_text, CommandContext};
use crate::db::tags::create::{create_tag, CreateTagError};
use crate::db::tags::detect::add_detectable_to_cache;
use crate::util::tag::alias::AliasTagContent;
use crate::util::tag::embed::EmbedTagContent;
use crate::util::tag::script::ScriptTagContent;
use crate::util::tag::text::TextTagContent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{Attachment, Message, UserId};
use sqlx::types::chrono;
use std::num::ParseIntError;

pub mod alias;
pub mod embed;
pub mod execute;
pub mod permissions;
pub mod script;
pub mod text;

#[derive(sqlx::Type, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TagPayload {
    Text(TextTagContent),
    Alias(AliasTagContent),
    Embed(EmbedTagContent),
    Script(ScriptTagContent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTagModel {
    pub guild_id: i64,
    pub owner_id: i64,
    pub name: String,
    pub kind: TagKind,
    pub payload: TagPayload,
    pub detect: Option<bool>,
}

impl CreateTagModel {
    pub fn new(
        guild_id: i64,
        owner_id: i64,
        name: String,
        payload: TagPayload,
        detect: Option<bool>,
    ) -> Self {
        CreateTagModel {
            guild_id,
            owner_id,
            name,
            kind: TagKind::from_payload(&payload),
            payload,
            detect,
        }
    }

    pub fn with_msg(msg: &Message, name: &str, payload: TagPayload, detect: Option<bool>) -> Self {
        Self::new(
            i64::from(msg.guild_id.unwrap()),
            i64::from(msg.author.id),
            name.to_string(),
            payload,
            detect,
        )
    }
    pub fn with_ctx(
        ctx: &CommandContext,
        name: &str,
        payload: TagPayload,
        detect: Option<bool>,
    ) -> Self {
        Self::with_msg(&ctx.msg, name, payload, detect)
    }
}

pub async fn try_create_tag(ctx: &CommandContext, tag: CreateTagModel) {
    let name = tag.name.clone();
    match create_tag(&ctx.state.db_pool, tag.clone()).await {
        Ok(_) => {
            if tag.detect.unwrap_or(false) {
                add_detectable_to_cache(&ctx.state, ctx.msg.guild_id.unwrap(), &name).await;
            }
            send_reply_ping_text(
                ctx,
                format!(
                    "Created tag **{name}**{}.",
                    if tag.detect.is_some() && tag.detect.unwrap() {
                        " (detectable)"
                    } else {
                        ""
                    }
                )
                .as_str(),
            )
            .await
        }
        Err(CreateTagError::Exists) => {
            send_reply_ping_text(ctx, format!("Tag **{name}** already exists.").as_str()).await
        }
        Err(CreateTagError::Serialize) => {
            send_reply_ping_text(ctx, "Failed to serialize tag.").await
        }
        Err(CreateTagError::DB(e)) => {
            send_internal_error_msg(ctx, format!("Error creating tag: {e}.").as_str()).await
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

pub async fn add_attachments_to_args(ctx: &mut CommandContext) -> Result<(), String> {
    let had_args = ctx.args.is_some();

    let mut txt_md = None;
    let mut js = None;
    let mut other = Vec::new();

    for attachment in &ctx.msg.attachments {
        match attachment.content_type.as_deref() {
            Some("text/plain; charset=utf-8") | Some("text/markdown; charset=utf-8") => {
                println!("found text/markdown");
                if txt_md.is_some() {
                    return Err("Only one text/markdown attachment is allowed.".into());
                }
                txt_md = Some(attachment);
            }
            Some("text/javascript; charset=utf-8") | Some("application/json; charset=utf-8") => {
                if js.is_some() {
                    return Err("Only one JavaScript/JSON attachment is allowed.".into());
                }
                js = Some(attachment);
            }
            _ => other.push(attachment),
        }
    }

    if let Some(js_attachment) = js {
        if txt_md.is_some() || !other.is_empty() {
            return Err(
                "A JavaScript/JSON attachment cannot be combined with any other attachments."
                    .into(),
            );
        }

        if had_args {
            return Err("A JavaScript/JSON attachment requires no command arguments.".into());
        }

        let contents = read_attachment(js_attachment).await?;
        ctx.args = if js_attachment.content_type == Some("text/javascript; charset=utf-8".into()) {
            Some(format!("```js\n{}\n```", contents))
        } else {
            Some(format!("```json\n{}\n```", contents))
        };
        return Ok(());
    }

    if txt_md.is_some() && had_args {
        return Err("A text/markdown attachment requires no command arguments.".into());
    }

    if let Some(text_attachment) = txt_md {
        let contents = read_attachment(text_attachment).await?;
        println!("{}", contents);
        ctx.args = Some(contents);
    }

    // Append other attachment URLs
    for attachment in other {
        if ctx.args.is_none() {
            ctx.args = Some(String::new());
        }

        let args = ctx.args.as_mut().unwrap();

        if !args.is_empty() {
            args.push('\n');
        }

        args.push_str(&attachment.url);
    }

    Ok(())
}

pub async fn read_attachment(attachment: &Attachment) -> Result<String, String> {
    let response = reqwest::get(&attachment.url)
        .await
        .map_err(|e| format!("Failed to fetch attachment: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read attachment bytes: {}", e))?;

    String::from_utf8(bytes.to_vec()).map_err(|e| format!("Attachment is not valid UTF-8: {}", e))
}
