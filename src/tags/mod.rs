// pub mod alias;
// pub mod embed;
// pub mod script;
// pub mod text;
//
// use crate::commands::get_prefix;
// use crate::settings::DEFAULT_PREFIX;
// use crate::tags::alias::AliasTagContent;
// use crate::tags::embed::EmbedTagContent;
// use crate::tags::script::{ScriptContext, ScriptEngine, ScriptTagContent};
// use crate::tags::text::TextTagContent;
// use crate::tags::TagKind::{Alias, Embed, Script, Text};
// use crate::BotState;
// use log::error;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use serenity::all::{
//     Context, CreateAttachment, CreateEmbed, CreateMessage, GuildId, Message, User, UserId,
// };
// use sqlx::error::DatabaseError;
// use sqlx::postgres::PgDatabaseError;
// use sqlx::types::{chrono, Json};
// use sqlx::Error::RowNotFound;
// use std::any::Any;
// use std::env::Args;
// use std::error::Error;
// use std::fmt::format;
// use std::num::ParseIntError;
// use std::sync::Arc;
//
// #[derive(sqlx::Type, Debug, Serialize, Deserialize, PartialEq)]
// #[sqlx(type_name = "tag_kind", rename_all = "lowercase")]
// pub enum TagKind {
//     Text,
//     Alias,
//     Embed,
//     Script,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum TagPayload {
//     Text(TextTagContent),
//     Alias(AliasTagContent),
//     Embed(EmbedTagContent),
//     Script(ScriptTagContent),
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct FetchTagModel {
//     id: i32,
//     guild_id: i64,
//     owner_id: i64,
//     name: String,
//     kind: TagKind,
//     payload: Value,
//     t_created: chrono::DateTime<chrono::Utc>,
//     t_updated: chrono::DateTime<chrono::Utc>,
//     enabled: bool,
//     alias_target_name: Option<String>,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct CreateTagModel {
//     guild_id: i64,
//     owner_id: i64,
//     name: String,
//     kind: TagKind,
//     payload: TagPayload,
// }
//
// impl CreateTagModel {
//     pub fn with_msg(msg: &Message, name: String, kind: TagKind, payload: TagPayload) -> Self {
//         CreateTagModel {
//             guild_id: i64::from(msg.guild_id.unwrap()),
//             owner_id: i64::from(msg.author.id),
//             name,
//             kind,
//             payload,
//         }
//     }
// }
//
// pub async fn fetch_tag(
//     name: &str,
//     gid: i64,
//     state: Arc<BotState>,
// ) -> Result<Option<FetchTagModel>, sqlx::Error> {
//     sqlx::query_as!(
//         FetchTagModel,
//         r#"
//         SELECT
//             t.id AS "id!",
//             t.guild_id AS "guild_id!",
//             t.owner_id AS "owner_id!",
//             t.name AS "name!",
//             t.kind AS "kind!: TagKind",
//             t.payload AS "payload!",
//             t.t_created AS "t_created!",
//             t.t_updated AS "t_updated!",
//             t.enabled AS "enabled!",
//             target.name AS "alias_target_name: Option<String>"
//         FROM tags t
//         LEFT JOIN tags target
//             ON target.id = t.target_id
//         WHERE t.guild_id = $1
//           AND t.name = $2;
//     "#,
//         gid,
//         name
//     )
//     .fetch_optional(&state.db_pool)
//     .await
// }
//
// pub async fn fetch_tag_resolved(
//     name: &str,
//     gid: i64,
//     state: Arc<BotState>,
// ) -> Result<Option<FetchTagModel>, sqlx::Error> {
//     sqlx::query_as!(
//         FetchTagModel,
//         r#"
//         SELECT
//             COALESCE(target.id, source.id) AS "id!",
//             COALESCE(target.guild_id, source.guild_id) AS "guild_id!",
//             COALESCE(target.owner_id, source.owner_id) AS "owner_id!",
//             COALESCE(target.name, source.name) AS "name!",
//             COALESCE(target.kind, source.kind) AS "kind!: TagKind",
//             COALESCE(target.payload, source.payload) AS "payload!",
//             COALESCE(target.t_created, source.t_created) AS "t_created!",
//             COALESCE(target.t_updated, source.t_updated) AS "t_updated!",
//             COALESCE(target.enabled, source.enabled) AS "enabled!",
//             null as alias_target_name
//         FROM tags AS source
//         LEFT JOIN tags AS target
//             ON source.kind = 'alias'
//            AND target.id = (source.payload->>'target_id')::int
//         WHERE source.guild_id = $1
//           AND source.name = $2;
//     "#,
//         gid,
//         name
//     )
//     .fetch_optional(&state.db_pool)
//     .await
// }
//
// pub fn payload_mismatch_error(name: &str) -> CreateMessage {
//     error!("Tag {} payload kind does not match tag kind!", name);
//     CreateMessage::new().content(format!(
//         "Error when evaluating tag **{}**. Please report error to <@435572469496020992>",
//         name
//     ))
// }
//
// pub async fn tag(args: Option<&str>, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     let Some(args) = args else {
//         return CreateMessage::new().content("Expected a tag name or command!");
//     };
//
//     let (cmd, new_args) = match args.split_once(char::is_whitespace) {
//         Some((cmd, rest)) => (cmd.to_lowercase(), Some(rest)),
//         None => (args.to_lowercase(), None),
//     };
//
//     match (cmd.as_str(), new_args) {
//         ("help", args) => tag_help(get_prefix(msg.guild_id, state.clone()).await.as_str(), args),
//         ("add", Some(args)) => add_tag(args, msg, state).await,
//         ("add", None) => tag_add_help_msg(get_prefix(msg.guild_id, state.clone()).await.as_str()),
//         ("alias", Some(args)) => alias_tag(args, msg, state).await,
//         ("alias", None) => {
//             tag_alias_help_msg(get_prefix(msg.guild_id, state.clone()).await.as_str())
//         }
//         ("raw", Some(args)) => raw_tag(args, msg, state).await,
//         ("raw", None) => tag_raw_help_msg(get_prefix(msg.guild_id, state.clone()).await.as_str()),
//         ("list", args) => tag_list(args, msg, state).await,
//         ("info" | "owner", args) => tag_info(args, msg, state).await,
//         (tag, args) => resolve_tag(tag, args, msg, state).await,
//     }
// }
//
// fn tag_name_subcommand_check(name: &str) -> Option<CreateMessage> {
//     if name == "add"
//         || name == "alias"
//         || name == "raw"
//         || name == "list"
//         || name == "edit"
//         || name == "delete"
//         || name == "info"
//         || name == "owner"
//         || name == "chown"
//     {
//         return Some(CreateMessage::new().content(format!(
//             "Tag name **{name}** is disallowed, as it is a subcommand!"
//         )));
//     }
//     None
// }
//
// async fn alias_tag(args: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     let (alias, tag) = match args.split_once(char::is_whitespace) {
//         Some((alias, rest)) => {
//             let tag = match rest.split_once(char::is_whitespace) {
//                 None => rest.to_lowercase(),
//                 Some((first, _)) => first.to_lowercase(),
//             };
//             (alias.to_lowercase(), tag)
//         }
//         None => {
//             let prefix = get_prefix(msg.guild_id, state.clone()).await;
//             return CreateMessage::new().content(format!(
//                 "Expected an alias name and tag name! See `{prefix}t alias help` for usage"
//             ));
//         }
//     };
//     if let Some(msg) = tag_name_subcommand_check(alias.as_str()) {
//         return msg;
//     }
//
//     let source_tag = match fetch_tag(&tag, i64::from(msg.guild_id.unwrap()), state.clone()).await {
//         Ok(fetched) => match fetched {
//             Some(src_tag) => src_tag,
//             None => {
//                 return CreateMessage::new().content(format!("Tag **{}** does not exist!", tag));
//             }
//         },
//         Err(e) => {
//             error!("Error when fetching source tag of alias: {}", e);
//             return CreateMessage::new().content("Error reading source tag!");
//         }
//     };
//
//     let payload = AliasTagContent {
//         target_id: source_tag.id,
//     };
//
//     create_tag(
//         CreateTagModel::with_msg(
//             msg,
//             alias.into(),
//             TagKind::Alias,
//             TagPayload::Alias(payload),
//         ),
//         state,
//     )
//     .await
// }
//
// async fn add_tag(args: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     match args.split_once(char::is_whitespace) {
//         None => {
//             let prefix = get_prefix(msg.guild_id, state.clone()).await;
//             CreateMessage::new().content(format!(
//                 "Expected a tag name and content! See `{prefix}t add help` for usage"
//             ))
//         }
//         Some((tag, content)) => {
//             if tag == "help" {
//                 let prefix = get_prefix(msg.guild_id, state.clone()).await;
//                 return tag_add_help_msg(&prefix);
//             }
//             if let Some(msg) = tag_name_subcommand_check(tag) {
//                 return msg;
//             }
//             if !tag
//                 .chars()
//                 .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
//             {
//                 return CreateMessage::new().content("Tag name must contain only letters, numbers, underscores (_), and hyphens (-).");
//             }
//             if tag.len() > 32 {
//                 return CreateMessage::new().content("Tag name must not exceed 32 characters.");
//             }
//             if let Some(inner) = content
//                 .strip_prefix("```js")
//                 .and_then(|args| args.strip_suffix("```"))
//             {
//                 let payload = ScriptTagContent {
//                     script: inner.to_string(),
//                 };
//                 return create_tag(
//                     CreateTagModel::with_msg(msg, tag.into(), Script, TagPayload::Script(payload)),
//                     state,
//                 )
//                 .await;
//             }
//             // TODO: Add embed support
//
//             let payload = TextTagContent {
//                 content: content.into(),
//             };
//             create_tag(
//                 CreateTagModel::with_msg(msg, tag.into(), TagKind::Text, TagPayload::Text(payload)),
//                 state,
//             )
//             .await
//         }
//     }
// }
//
// async fn create_tag(tag: CreateTagModel, state: Arc<BotState>) -> CreateMessage {
//     let serialized_payload = match serde_json::to_value(tag.payload) {
//         Ok(payload) => payload,
//         Err(e) => {
//             error!("Failed to serialize tag payload: {}", e);
//             return CreateMessage::new().content("Error serializing tag payload.");
//         }
//     };
//     match sqlx::query!(r#"INSERT INTO tags (guild_id, owner_id, name, kind, payload) VALUES ($1, $2, $3, $4, $5);"#, tag.guild_id, tag.owner_id, tag.name, tag.kind as TagKind, serialized_payload).execute(&state.db_pool).await {
//         Err(e) => {
//             if let Some(db_err) = e.as_database_error() {
//                 if db_err.is_unique_violation() {
//                     return CreateMessage::new().content(format!("Tag **{}** already exists!", tag.name))
//                 }
//             }
//             error!("Failed to create tag in db: {}", e);
//             CreateMessage::new().content("Error creating tag in db.")
//         },
//         Ok(_) => {
//             CreateMessage::new().content(format!("Created tag **{}**", tag.name))
//         }
//     }
// }
//
// async fn resolve_tag(
//     tag_name: &str,
//     args: Option<&str>,
//     msg: &Message,
//     state: Arc<BotState>,
// ) -> CreateMessage {
//     match fetch_tag_resolved(tag_name, msg.guild_id.unwrap().get() as i64, state).await {
//         Err(e) => {
//             error!("Failed to get tag: {}", e);
//             CreateMessage::new().content(format!(
//                 "Error when searching for tag: \"{}\"\n{}",
//                 tag_name, e
//             ))
//         }
//         Ok(None) => {
//             CreateMessage::new().content(format!("No tag with name \"{}\" found!", tag_name))
//         }
//         Ok(Some(tag)) => match tag.kind {
//             TagKind::Text => {
//                 let payload: TextTagContent = match serde_json::from_value(tag.payload) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         error!("Failed to deserialize payload: {}", e);
//                         return payload_mismatch_error(tag_name);
//                     }
//                 };
//
//                 CreateMessage::new().content(payload.content)
//             }
//             TagKind::Alias => {
//                 error!("Got an alias tag on a resolved fetch!");
//                 CreateMessage::new().content(format!(
//                     "There was an error resolving the alias tag {}",
//                     tag_name
//                 ))
//             }
//             TagKind::Embed => {
//                 let payload: EmbedTagContent = match serde_json::from_value(tag.payload) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         error!("Failed to deserialize payload: {}", e);
//                         return payload_mismatch_error(tag_name);
//                     }
//                 };
//
//                 CreateMessage::new().embed(CreateEmbed::from(payload.embed))
//             }
//             TagKind::Script => {
//                 let payload: ScriptTagContent = match serde_json::from_value(tag.payload) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         error!("Failed to deserialize payload: {}", e);
//                         return payload_mismatch_error(tag_name);
//                     }
//                 };
//
//                 match ScriptEngine::new() {
//                     Err(e) => {
//                         error!("Failed to initialize script engine: {}", e);
//                         CreateMessage::new().content(
//                             "Failed to initialize the script engine; cannot display tag"
//                                 .to_string(),
//                         )
//                     }
//                     Ok(engine) => {
//                         let script_context = ScriptContext {
//                             args: match args {
//                                 Some(args) => Some(args.to_string()),
//                                 None => None,
//                             },
//                             guild_id: msg.guild_id.unwrap(),
//                             channel_id: msg.channel_id,
//                             author_id: msg.author.id,
//                         };
//                         let result = engine.execute(&payload.script, script_context);
//                         match result {
//                             Err(e) => CreateMessage::new()
//                                 .content(format!("Failed to execute script: {:?}", e)),
//                             Ok(output) => CreateMessage::new().content(format!("{}", output)),
//                         }
//                     }
//                 }
//             }
//         },
//     }
// }
//
// pub async fn raw_tag(tag_name: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     match fetch_tag(tag_name, msg.guild_id.unwrap().get() as i64, state).await {
//         Err(e) => {
//             error!("Failed to get tag: {}", e);
//             CreateMessage::new().content(format!(
//                 "Error when searching for tag: \"{}\"\n{}",
//                 tag_name, e
//             ))
//         }
//         Ok(None) => {
//             CreateMessage::new().content(format!("No tag with name \"{}\" found!", tag_name))
//         }
//         Ok(Some(tag)) => {
//             if tag.kind == Alias {
//                 match tag.alias_target_name {
//                     Some(alias_target_name) => CreateMessage::new().content(format!(
//                         "**{}** is an alias of tag **{}**",
//                         tag.name, alias_target_name
//                     )),
//                     None => CreateMessage::new()
//                         .content(format!("Failed to resolve alias **{}**!", tag.name)),
//                 }
//             } else if tag.kind == Text {
//                 let payload: TextTagContent = match serde_json::from_value(tag.payload) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         error!("Failed to deserialize payload: {}", e);
//                         return payload_mismatch_error(&tag.name);
//                     }
//                 };
//
//                 let attachment = CreateAttachment::bytes(
//                     payload.content.as_bytes().to_vec(),
//                     format!("{}.txt", tag.name),
//                 );
//                 return CreateMessage::new().add_file(attachment);
//             } else if tag.kind == Script {
//                 let payload: ScriptTagContent = match serde_json::from_value(tag.payload) {
//                     Ok(payload) => payload,
//                     Err(e) => {
//                         error!("Failed to deserialize payload: {}", e);
//                         return payload_mismatch_error(&tag.name);
//                     }
//                 };
//
//                 let attachment = CreateAttachment::bytes(
//                     payload.script.as_bytes().to_vec(),
//                     format!("{}.js", tag.name),
//                 );
//                 return CreateMessage::new().add_file(attachment);
//             } else if tag.kind == Embed {
//                 return CreateMessage::new().content("TODO");
//             } else {
//                 CreateMessage::new().content("Unknown tag kind; cannot display.")
//             }
//         }
//     }
// }
//
// async fn fetch_owners_tags(
//     oid: i64,
//     gid: i64,
//     state: Arc<BotState>,
// ) -> Result<Vec<FetchTagModel>, sqlx::Error> {
//     sqlx::query_as!(
//         FetchTagModel,
//         r#"
//         SELECT
//             id,
//             guild_id,
//             owner_id as "owner_id!",
//             name,
//             kind AS "kind: TagKind",
//             payload,
//             t_created,
//             t_updated,
//             enabled,
//             null as alias_target_name
//         FROM tags
//         WHERE guild_id = $1
//           AND owner_id = $2;
//     "#,
//         gid,
//         oid
//     )
//     .fetch_all(&state.db_pool)
//     .await
// }
//
// pub async fn tag_info(args: Option<&str>, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     match args {
//         None => CreateMessage::new().content("Expected a tag argument."),
//         Some(args) => {
//             let tag_name = match args.split_once(char::is_whitespace) {
//                 Some((tag, _)) => tag,
//                 None => args,
//             };
//             match fetch_tag(tag_name, msg.guild_id.unwrap().get() as i64, state).await {
//                 Err(e) => {
//                     error!("Failed to get tag: {}", e);
//                     CreateMessage::new().content(format!(
//                         "Error when searching for tag: \"{}\"\n{}",
//                         tag_name, e
//                     ))
//                 }
//                 Ok(None) => CreateMessage::new()
//                     .content(format!("No tag with name \"{}\" found!", tag_name)),
//                 Ok(Some(tag)) => CreateMessage::new().content(format!(
//                     "Tag **{}**:\nOwner: <@{}>\nCreated <t:{}:R>\nLast updated <t:{}:R>",
//                     tag.name,
//                     tag.owner_id,
//                     tag.t_created.timestamp(),
//                     tag.t_updated.timestamp()
//                 )),
//             }
//         }
//     }
// }
//
// pub async fn tag_list(args: Option<&str>, msg: &Message, state: Arc<BotState>) -> CreateMessage {
//     let (gid, uid) = match args {
//         None => (
//             msg.guild_id.unwrap().get() as i64,
//             msg.author.id.get() as i64,
//         ),
//         Some(args) => {
//             let user = match args.split_once(char::is_whitespace) {
//                 Some((user, _)) => user,
//                 None => args,
//             };
//             let uid = match get_uid_from_user_text(user) {
//                 Ok(uid) => uid,
//                 Err(_) => return CreateMessage::new().content("Expected a user as an argument!"),
//             };
//             (msg.guild_id.unwrap().get() as i64, uid)
//         }
//     };
//     get_users_tags_msg(gid, uid, state).await
// }
//
// fn get_uid_from_user_text(text: &str) -> Result<i64, ParseIntError> {
//     text.trim_start_matches("<@")
//         .trim_start_matches('!')
//         .trim_end_matches('>')
//         .parse::<i64>()
// }
//
// async fn get_users_tags_msg(gid: i64, uid: i64, state: Arc<BotState>) -> CreateMessage {
//     let tags = match fetch_owners_tags(uid, gid, state).await {
//         Ok(tags) => tags,
//         Err(e) => {
//             error!("Error fetching tags of user {uid}");
//             return CreateMessage::new().content(format!("Error fetching tags of user <@{uid}>!"));
//         }
//     };
//
//     if tags.is_empty() {
//         return CreateMessage::new().content(format!("User <@{uid}> does not own any tags!"));
//     }
//
//     let tag_list = tags
//         .iter()
//         .map(|tag| tag.name.as_str())
//         .collect::<Vec<_>>()
//         .join("\n");
//
//     CreateMessage::new().content(format!("<@{uid}>'s Tags:\n{tag_list}"))
// }
//
// pub fn tag_help(prefix: &str, args: Option<&str>) -> CreateMessage {
//     match args.map(str::to_lowercase) {
//         Some(s) => match s.as_str() {
//             "script" => tag_script_help(prefix),
//             "embed" => tag_embed_help(prefix),
//             _ => tag_help_msg(prefix),
//         },
//         _ => tag_help_msg(prefix),
//     }
// }
//
// fn tag_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Save content under a name, and recall it later.
// **General Info:**
//  - Tags are server-specific.
//  - Tags can contain text, embeds, JS scripts, or alias other tags.
//  - `{prefix}t` is an alias for `{prefix}tag`, and can be used in its place anywhere.
//
// **View:**
// `{prefix}tag <name> [args]` - Show a saved tag.
//
// **Manage:**
// `{prefix}tag add <name> <content>` - Create a new tag.
// `{prefix}tag edit <name>` - Change the content of a tag you own.
// `{prefix}tag delete <name>` - Delete a tag you own.
// `{prefix}tag alias <new_name> <existing_tag>` - Alias an existing tag.
// `{prefix}tag info <name>` - Show information about a tag.
// `{prefix}tag raw <name>` - Show the raw content of a tag.
//
// **Extra:**
// `{prefix}tag list [user]` - List tags owned by you (or user provided).
// `{prefix}tag search <name>` - Fuzzy search for a tag.
// `{prefix}tag help`
//
// **Admin:**
// `{prefix}tag chown <tag_name> <new_owner>` - Change the owner of a tag.
// `{prefix}tag ban <name>` - Ban a tag (prevents deletion of tag, to stop the deletion and recreation of it).
//
// Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{prefix}tag help script` or `{prefix}tag help embed`
//     "#))
// }
//
// fn tag_script_help(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Script tags run sandboxed JS, and can take in args from the user.
//
// To create a script tag, the content of the tag must be a multi-line JS code block:
// ```{prefix}tag add <name> `​`​`js
// <script_content>
// `​`​`
// ```
//
// The String `args` is made available at the start of the script, containing all message content after the tag name.
// For a full API reference, see [here](https://git.marinodev.com/drake/bunny_bot/api.md).
//     "#
//     ))
// }
//
// fn tag_add_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Creates a new tag.
// Usage: `{prefix}t add <name> <content>`
// Tags can store text, JS scripts, or embeds. Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{prefix}tag help script` or `{prefix}tag help embed`
//     "#
//     ))
// }
//
// fn tag_alias_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
// Creates an alias for a tag.
// Usage: `{prefix}t alias <new_name> <existing_tag>`
//     "#
//     ))
// }
//
// fn tag_raw_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
// Shows the raw content of a tag.
// Usage: `{prefix}t raw <tag_name>`
//     "#
//     ))
// }
//
// fn tag_embed_help(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
//         TODO
//         "#
//     ))
// }
