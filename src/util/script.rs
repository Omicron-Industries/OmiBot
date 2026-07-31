use rquickjs::{class::Trace, Context, Ctx, FromJs, JsLifetime, Result, Runtime};
use serde::Deserialize;
use serenity::model::{
    channel::Message,
    id::{ChannelId, GuildId},
};
use serenity::prelude::Context as SerenityContext;
use std::time::{Duration, Instant};

pub struct ScriptContext {
    pub serenity_ctx: SerenityContext,
    pub message: Message,
    pub args: Option<String>,
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsUser {
    #[qjs(skip_trace)]
    id: u64,

    #[qjs(skip_trace)]
    username: String,
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
}

#[rquickjs::methods]
impl JsChannel {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsGuild {
    #[qjs(skip_trace)]
    id: u64,
}

#[rquickjs::methods]
impl JsGuild {
    #[qjs(get)]
    pub fn id(&self) -> u64 {
        self.id
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

impl From<&Message> for JsMessage {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id.get(),

            author: JsUser {
                id: msg.author.id.get(),
                username: msg.author.name.clone(),
            },

            content: msg.content.clone(),
        }
    }
}

impl From<ChannelId> for JsChannel {
    fn from(id: ChannelId) -> Self {
        Self { id: id.get() }
    }
}

impl From<GuildId> for JsGuild {
    fn from(id: GuildId) -> Self {
        Self { id: id.get() }
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

    pub fn execute(&self, script: &str, script_ctx: ScriptContext) -> Result<String> {
        let start = Instant::now();

        self.runtime.set_interrupt_handler(Some(Box::new(move || {
            start.elapsed() > Duration::from_secs(3)
        })));

        let ctx = Context::full(&self.runtime)?;

        ctx.with(|ctx| {
            self.register_globals(&ctx, &script_ctx)?;

            ctx.eval(script)
        })
    }

    fn register_globals<'js>(&self, ctx: &Ctx<'js>, script_ctx: &ScriptContext) -> Result<()> {
        let global = ctx.globals();

        let message = JsMessage::from(&script_ctx.message);

        global.set("msg", message)?;
        global.set("args", script_ctx.args.clone().unwrap_or_default())?;
        global.set("channel", JsChannel::from(script_ctx.message.channel_id))?;
        global.set("guild", JsGuild::from(script_ctx.message.guild_id.unwrap()))?;

        Ok(())
    }
}
