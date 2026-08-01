use crate::commands::{CommandContext, send_reply_ping_message, send_reply_ping_text};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{CreateEmbed, CreateMessage, Embed};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedTagContent {
    pub embed: Embed,
}

pub fn parse_embed_tag_content(raw: &str) -> Option<Result<EmbedTagContent, String>> {
    let embed_val = parse_embed_json(raw)?;
    let wrapper = serde_json::json!({ "embed": embed_val });
    let embed = serde_json::from_value::<EmbedTagContent>(wrapper).ok()?;
    match validate_embed(&embed.embed) {
        Ok(()) => Some(Ok(embed)),
        Err(msg) => Some(Err(msg)),
    }
}

pub fn parse_embed_json(raw: &str) -> Option<Value> {
    let s = raw.trim();
    let (json_str, is_explicit_tag) = if let Some(stripped) = s.strip_prefix("```") {
        let inner = stripped.strip_suffix("```").unwrap_or(stripped);
        let (lang, code) = inner.split_once('\n').unwrap_or(("", inner));
        let lang = lang.trim().to_lowercase();
        let is_tag = lang == "json" || lang == "embed";
        (code.trim(), is_tag)
    } else {
        (s, false)
    };

    let parsed: Value = serde_json::from_str(json_str).ok()?;
    if !parsed.is_object() {
        return None;
    }

    // Check Discord Webhook format: { "embeds": [ { ... } ] }
    if let Some(embeds) = parsed.get("embeds").and_then(|v| v.as_array()) {
        if let Some(first_embed) = embeds.first() {
            if first_embed.is_object() && is_embed_like(first_embed) {
                return Some(first_embed.clone());
            }
        }
    }

    // Check nested embed format: { "embed": { ... } }
    if let Some(embed) = parsed.get("embed") {
        if embed.is_object() && is_embed_like(embed) {
            return Some(embed.clone());
        }
    }

    // Check direct embed format: { "title": "...", "description": "...", ... }
    if is_embed_like(&parsed) || is_explicit_tag {
        return Some(parsed);
    }

    None
}

pub fn is_embed_like(val: &Value) -> bool {
    if let Value::Object(obj) = val {
        obj.contains_key("title")
            || obj.contains_key("description")
            || obj.contains_key("color")
            || obj.contains_key("fields")
            || obj.contains_key("author")
            || obj.contains_key("footer")
            || obj.contains_key("image")
            || obj.contains_key("thumbnail")
            || obj.contains_key("url")
    } else {
        false
    }
}

pub fn validate_embed(embed: &Embed) -> Result<(), String> {
    if embed.title.is_none()
        && embed.description.is_none()
        && embed.fields.is_empty()
        && embed.author.is_none()
        && embed.footer.is_none()
        && embed.image.is_none()
        && embed.thumbnail.is_none()
        && embed.url.is_none()
    {
        return Err(
            "Embed must contain at least one of: title, description, fields, author, footer, image, thumbnail, or url."
                .into(),
        );
    }

    if let Some(title) = &embed.title {
        if title.len() > 256 {
            return Err("Embed title cannot exceed 256 characters.".into());
        }
    }

    if let Some(description) = &embed.description {
        if description.len() > 4096 {
            return Err("Embed description cannot exceed 4096 characters.".into());
        }
    }

    if let Some(url) = &embed.url {
        if url.is_empty() {
            return Err("Embed URL cannot be empty.".into());
        }
    }

    if embed.fields.len() > 25 {
        return Err("Embed cannot contain more than 25 fields.".into());
    }

    let total_length = embed.title.as_ref().map(|x| x.len()).unwrap_or(0)
        + embed.description.as_ref().map(|x| x.len()).unwrap_or(0)
        + embed
            .fields
            .iter()
            .map(|f| f.name.len() + f.value.len())
            .sum::<usize>()
        + embed.footer.as_ref().map(|f| f.text.len()).unwrap_or(0);

    if total_length > 6000 {
        return Err("Embed total character count cannot exceed 6000 characters.".into());
    }

    Ok(())
}

pub async fn try_handle_embed_tag(ctx: &CommandContext, raw_code: &str) -> bool {
    let Some(embed_content) = parse_embed_tag_content(raw_code) else {
        return false;
    };

    match embed_content {
        Ok(embed_tag_content) => {
            if let Err(e) = send_reply_ping_message(
                ctx,
                CreateMessage::new().embed(CreateEmbed::from(embed_tag_content.embed)),
            )
            .await
            {
                send_reply_ping_text(ctx, format!("Embed invalid: {}", e).as_str()).await;
            }
        }
        Err(msg) => {
            send_reply_ping_text(ctx, &msg).await;
        }
    }

    true
}
