use rquickjs::{class::Trace, Context, Ctx, FromJs, JsLifetime, Result, Runtime};
use serde::{Deserialize, Serialize};
use serenity::model::{
    channel::Message,
    id::{ChannelId, GuildId, UserId},
};
use serenity::prelude::Context as SerenityContext;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ScriptContext {
    pub message: Message,
    pub args: Option<String>,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub serenity_ctx: Arc<SerenityContext>,
    pub db_pool: Arc<PgPool>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ScriptOutput {
    Text(String),
    Embed(serde_json::Value),
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsUser {
    #[qjs(skip_trace)]
    id: u64,

    #[qjs(skip_trace)]
    username: String,

    #[qjs(skip_trace)]
    avatar: Option<String>,

    #[qjs(skip_trace)]
    discriminator: String,
}

#[rquickjs::methods]
impl JsUser {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[qjs(get)]
    pub fn username(&self) -> String {
        self.username.clone()
    }

    #[qjs(get)]
    pub fn avatar(&self) -> Option<String> {
        self.avatar.clone()
    }

    #[qjs(get)]
    pub fn discriminator(&self) -> String {
        self.discriminator.clone()
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsMessage {
    #[qjs(skip_trace)]
    id: u64,

    author: JsUser,

    #[qjs(skip_trace)]
    content: String,
}

#[rquickjs::methods]
impl JsMessage {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[qjs(get)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[qjs(get)]
    pub fn author(&self) -> JsUser {
        self.author.clone()
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsChannel {
    #[qjs(skip_trace)]
    id: u64,

    #[qjs(skip_trace)]
    name: String,

    #[qjs(skip_trace)]
    is_dm: bool,
}

#[rquickjs::methods]
impl JsChannel {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[qjs(get)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[qjs(get)]
    pub fn is_dm(&self) -> bool {
        self.is_dm
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsGuild {
    #[qjs(skip_trace)]
    id: u64,

    #[qjs(skip_trace)]
    name: String,

    #[qjs(skip_trace)]
    owner_id: u64,
}

#[rquickjs::methods]
impl JsGuild {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[qjs(get)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[qjs(get)]
    pub fn owner_id(&self) -> u64 {
        self.owner_id
    }
}

#[derive(Debug, Deserialize)]
pub struct JsEmbed {
    pub title: Option<String>,

    pub description: Option<String>,

    pub url: Option<String>,

    pub color: Option<u32>,

    pub author: Option<JsEmbedAuthor>,

    pub fields: Option<Vec<JsEmbedField>>,

    pub footer: Option<JsEmbedFooter>,

    pub thumbnail: Option<JsEmbedImage>,

    pub image: Option<JsEmbedImage>,
}

#[derive(Debug, Deserialize)]
pub struct JsEmbedAuthor {
    pub name: String,

    pub url: Option<String>,

    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JsEmbedField {
    pub name: String,

    pub value: String,

    pub inline: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JsEmbedFooter {
    pub text: String,

    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JsEmbedImage {
    pub url: String,
}

impl From<&Message> for JsUser {
    fn from(msg: &Message) -> Self {
        let discriminator = msg.author.discriminator
            .map(|d| format!("{:04}", d))
            .unwrap_or_else(|| "0000".to_string());

        Self {
            id: msg.author.id.get(),
            username: msg.author.name.clone(),
            avatar: msg.author.avatar_url(),
            discriminator,
        }
    }
}

impl From<&Message> for JsMessage {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id.get(),
            author: JsUser::from(msg),
            content: msg.content.clone(),
        }
    }
}

impl From<&Message> for JsChannel {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.channel_id.get(),
            name: "".to_string(),
            is_dm: msg.is_private(),
        }
    }
}

impl From<&Message> for JsGuild {
    fn from(msg: &Message) -> Self {
        let guild_id = msg.guild_id.unwrap_or_default();
        Self {
            id: guild_id.get(),
            name: "".to_string(),
            owner_id: 0,
        }
    }
}

pub struct ScriptEngine {
    runtime: Runtime,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new()?;

        runtime.set_memory_limit(64 * 1024 * 1024);
        runtime.set_max_stack_size(1024 * 1024);

        Ok(Self { runtime })
    }

    pub fn execute(&self, script: &str, script_ctx: ScriptContext) -> Result<ScriptOutput> {
        let start = Instant::now();

        self.runtime.set_interrupt_handler(Some(Box::new(move || {
            start.elapsed() > Duration::from_secs(3)
        })));

        let ctx = Context::full(&self.runtime)?;

        ctx.with(|ctx| {
            self.register_globals(&ctx, &script_ctx)?;

            let result: rquickjs::Value = ctx.eval(script)?;

            parse_output(&ctx, result)
        })
    }

    fn register_globals<'js>(&self, ctx: &Ctx<'js>, script_ctx: &ScriptContext) -> Result<()> {
        let global = ctx.globals();

        let message = JsMessage::from(&script_ctx.message);
        let channel = JsChannel::from(&script_ctx.message);
        let guild = JsGuild::from(&script_ctx.message);

        global.set("msg", message)?;
        global.set("args", script_ctx.args.clone().unwrap_or_default())?;
        global.set("channel", channel)?;
        global.set("guild", guild)?;

        // TODO: Add util object with functions like findUsers, fetchTag, reply
        // let util_obj = create_util_object(ctx, script_ctx)?;
        // global.set("util", util_obj)?;

        Ok(())
    }
}

fn parse_output<'js>(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> Result<ScriptOutput> {
    if value.is_null() || value.is_undefined() {
        return Ok(ScriptOutput::Text(String::new()));
    }

    if let Some(s) = value.as_string() {
        let s_str = s.to_string()?;
        return Ok(ScriptOutput::Text(s_str));
    }

    if value.is_object() {
        // For now, just skip embed detection and convert to string
        // TODO: Properly deserialize objects as embeds
    }

    // Default: try to convert to string
    let str_val = rquickjs::String::from_value(value)
        .map(|s| s.to_string().unwrap_or_else(|_| "undefined".to_string()))
        .unwrap_or_else(|_| "undefined".to_string());
    
    Ok(ScriptOutput::Text(str_val))
}

fn is_embed_like(val: &serde_json::Value) -> bool {
    if let serde_json::Value::Object(obj) = val {
        obj.contains_key("title")
            || obj.contains_key("description")
            || obj.contains_key("color")
            || obj.contains_key("fields")
            || obj.contains_key("author")
            || obj.contains_key("footer")
            || obj.contains_key("image")
            || obj.contains_key("thumbnail")
    } else {
        false
    }
}

// TODO: Utility functions will be added once we understand the rquickjs API better
// Planned utilities: findUsers(query), fetchTag(name, options), reply(message, embed)
