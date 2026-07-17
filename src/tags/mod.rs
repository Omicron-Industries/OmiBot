pub mod alias;
pub mod embed;
pub mod script;
pub mod text;

use std::any::Any;
use std::error::Error;
use std::sync::Arc;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{Context, CreateEmbed, CreateMessage, Embed, GuildId, Message, UserId};
use sqlx::Error::RowNotFound;
use sqlx::types::{chrono, Json};
use crate::BotState;
use crate::commands::get_prefix;
use crate::settings::DEFAULT_PREFIX;
use crate::tags::alias::AliasTagContent;
use crate::tags::embed::EmbedTagContent;
use crate::tags::script::{ScriptContext, ScriptEngine, ScriptTagContent};
use crate::tags::TagKind::Script;
use crate::tags::text::TextTagContent;

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
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
    kind: TagKind,
    payload: Value,
    t_created: chrono::DateTime<chrono::Utc>,
    t_updated: chrono::DateTime<chrono::Utc>,
    enabled: bool,
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
    pub fn with_msg (msg: &Message, name: String, kind: TagKind, payload: TagPayload) -> Self {
        CreateTagModel {
            guild_id: i64::from(msg.guild_id.unwrap()),
            owner_id: i64::from(msg.author.id),
            name,
            kind,
            payload,
        }
    }
}

pub async fn fetch_tag_resolved(name: &str, gid: i64, state: Arc<BotState>) -> Result<FetchTagModel, sqlx::Error> {
    // sqlx::query_as!(Tag, r#"SELECT id, guild_id, owner_id, name, kind as "kind: TagKind", payload as "payload: Json<TagPayload>", t_created, t_updated, enabled FROM tags WHERE name = $1"#, name).fetch_one(&state.db_pool).await
    sqlx::query_as!(FetchTagModel, r#"
    SELECT
        COALESCE(target.id, source.id) AS "id!",
        COALESCE(target.guild_id, source.guild_id) AS "guild_id!",
        COALESCE(target.owner_id, source.owner_id) AS "owner_id!",
        COALESCE(target.name, source.name) AS "name!",
        COALESCE(target.kind, source.kind) AS "kind!: TagKind",
        COALESCE(target.payload, source.payload) AS "payload!",
        COALESCE(target.t_created, source.t_created) AS "t_created!",
        COALESCE(target.t_updated, source.t_updated) AS "t_updated!",
        COALESCE(target.enabled, source.enabled) AS "enabled!"
    FROM tags AS source
    LEFT JOIN tags AS target
        ON source.kind = 'alias'
       AND target.id = (source.payload->>'target_id')::int
    WHERE source.guild_id = $1
      AND source.name = $2;"#, gid, name).fetch_one(&state.db_pool).await
}

fn payload_mismatch_error(name: &str) -> CreateMessage {
    error!("Tag {} payload kind does not match tag kind!", name);
    CreateMessage::new().content(format!("Error when evaluating tag **{}**. Please report error to <@435572469496020992>", name))
}

pub async fn tag(args: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
    if args.len() < 1 {
        return CreateMessage::new().content("Expected a tag name or command!");
    };
    if msg.guild_id.is_none() {
        return CreateMessage::new().content("Error when evaluating tag command. Message was either a DM or received outside of the gateway. Make sure you are requesting a tag in a server that has it. To see your own tags, run `%t list`".to_string())
    }

    match args.split_once(char::is_whitespace) {
        // With args
        Some(("help", new_args)) => tag_help(get_prefix(msg.guild_id, state.clone()).await.as_str(), Some(new_args)),
        Some(("add", new_args)) => add_tag(new_args, msg, state).await,
        Some((tag, new_args)) => resolve_tag(tag, Some(new_args), msg, state).await,

        // No args
        None => match args {
            "help" => tag_help(get_prefix(msg.guild_id, state.clone()).await.as_str(), None),
            _ => resolve_tag(args, None, msg, state).await
        },
    }
}

async fn add_tag(args: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
    match args.split_once(char::is_whitespace) {
        None => CreateMessage::new().content("Expected a tag name!"),
        Some((tag, content)) => {
            if !tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return CreateMessage::new().content("Tag name must contain only letters, numbers, underscores (_), and hyphens (-).");
            }
            if tag.len() > 32 {
                return CreateMessage::new().content("Tag name must not exceed 32 characters.");
            }
            if let Some(inner) = content.strip_prefix("```js").and_then(|args| args.strip_suffix("```")) {
                let payload = ScriptTagContent {
                    script: inner.to_string(),
                };
                return create_tag(CreateTagModel::with_msg(msg, tag.into(), Script, TagPayload::Script(payload)), state).await
            }

            let payload = TextTagContent {
                content: content.into()
            };
            create_tag(CreateTagModel::with_msg(msg, tag.into(), TagKind::Text, TagPayload::Text(payload)), state).await
        },
    }


}

async fn create_tag(tag: CreateTagModel, state: Arc<BotState>) -> CreateMessage {
    let serialized_payload = match serde_json::to_value(tag.payload) {
        Ok(payload) => payload,
        Err(e) => {
            error!("Failed to serialize tag payload: {}", e);
            return CreateMessage::new().content("Error serializing tag payload.");
        }
    };
    match sqlx::query!(r#"INSERT INTO tags (guild_id, owner_id, name, kind, payload) VALUES ($1, $2, $3, $4, $5);"#, tag.guild_id, tag.owner_id, tag.name, tag.kind as TagKind, serialized_payload).execute(&state.db_pool).await {
        Err(e) => {
            error!("Failed to create tag in db: {}", e);
            CreateMessage::new().content("Error creating tag in db.")
        },
        Ok(_) => {
            CreateMessage::new().content(format!("Created tag **{}**", tag.name))
        }
    }
}

async fn resolve_tag(tag_name: &str, args: Option<&str>, msg: &Message, state: Arc<BotState>) -> CreateMessage {
    match fetch_tag_resolved(tag_name, msg.guild_id.unwrap().get() as i64, state).await {
        Err(RowNotFound) => {
            CreateMessage::new().content(format!("No tag with name \"{}\" found!", tag_name))
        },
        Err(e) => {
            error!("Failed to get tag: {}", e);
            CreateMessage::new().content(format!("Error when searching for tag: \"{}\"\n{}", tag_name, e))
        },
        Ok(tag) => {
            match tag.kind {
                TagKind::Text => {
                    let payload: TextTagContent = match serde_json::from_value(tag.payload) {
                        Ok(payload) => payload,
                        Err(e) => {
                            error!("Failed to deserialize payload: {}", e);
                            return payload_mismatch_error(tag_name)
                        }
                    };

                    CreateMessage::new().content(payload.content)
                },
                TagKind::Alias => {
                    error!("Got an alias tag on a resolved fetch!");
                    CreateMessage::new().content(format!("There was an error resolving the alias tag {}", tag_name))
                },
                TagKind::Embed => {
                    let payload: EmbedTagContent = match serde_json::from_value(tag.payload) {
                        Ok(payload) => payload,
                        Err(e) => {
                            error!("Failed to deserialize payload: {}", e);
                            return payload_mismatch_error(tag_name)
                        }
                    };

                    CreateMessage::new().embed(CreateEmbed::from(payload.embed))
                },
                TagKind::Script => {
                    let payload: ScriptTagContent = match serde_json::from_value(tag.payload) {
                        Ok(payload) => payload,
                        Err(e) => {
                            error!("Failed to deserialize payload: {}", e);
                            return payload_mismatch_error(tag_name)
                        }
                    };

                    match ScriptEngine::new() {
                        Err(e) => {
                            error!("Failed to initialize script engine: {}", e);
                            CreateMessage::new().content("Failed to initialize the script engine; cannot display tag".to_string())
                        }
                        Ok(engine) => {
                            let script_context = ScriptContext {
                                args: match args {
                                    Some(args) => Some(args.to_string()),
                                    None => None
                                },
                                guild_id: msg.guild_id.unwrap(),
                                channel_id: msg.channel_id,
                                author_id: msg.author.id,
                            };
                            let result = engine.execute(&payload.script, script_context);
                            match result {
                                Err(e) => {
                                    CreateMessage::new().content(format!("Failed to execute script: {:?}", e))
                                },
                                Ok(output) => {
                                    CreateMessage::new().content(format!("{}", output))
                                }
                            }

                        }
                    }
                }
            }
        },
    }
}



pub fn tag_help(prefix: &str, args: Option<&str>) -> CreateMessage {
    match args {
        Some("script") => { tag_script_help(prefix) }
        Some("embed") => { tag_embed_help(prefix) }
        _ => tag_help_msg(prefix)
    }
}

fn tag_help_msg(prefix: &str) -> CreateMessage {
    CreateMessage::new().content(format!(r#"
**General Info:**
Save content under a name, and recall it later.
Tags are server-specific.
Tags can contain text content, embed content, or JS content.
`{prefix}t` is an alias for {prefix}tag, and can be used in its place anywhere.

**View:**
`{prefix}tag <name> <optional_args>` - Show a saved tag.

**Manage:**
`{prefix}tag add <name> <content>` - Create a new tag.
`{prefix}tag edit <name>` - Change the content of a tag you own.
`{prefix}tag delete <name>` - Delete a tag you own.
`{prefix}tag alias <new_name> <existing_tag>` - Alias an existing tag.
`{prefix}tag info <name>` - Show information about a tag.
`{prefix}tag raw <name>` - Show the raw content of a tag.

**Extra:**
`{prefix}tag list <optional_user_id>` - List tags owned by you (or user provided).
`{prefix}tag search <name>` - Fuzzy search for a tag.
`{prefix}tag help`

Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{prefix}tag help script` or `{prefix}tag help embed`
    "#))
}

fn tag_script_help(prefix: &str) -> CreateMessage {
    CreateMessage::new().content(format!(r#"
Script tags run sandboxed JS, and can take in args from the user.

To create a script tag, the content of the tag must be a multi-line JS code block:
```{prefix}tag add <name> `​`​`js
<script_content>
`​`​`
```

The String `args` is made available at the start of the script, containing all message content after the tag name.
For a full API reference, see [here](https://git.marinodev.com/drake/bunny_bot/api.md).
    "#
    ))
}

fn tag_embed_help(prefix: &str) -> CreateMessage {
    CreateMessage::new().content(format!(r#"
        TODO
        "#
    ))
}
