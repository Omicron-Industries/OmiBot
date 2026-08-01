use crate::db::tags::fetch::fetch_tag_resolved;
use crate::util::tag::TagKind;
use rquickjs::{class::Trace, Context, Ctx, JsLifetime, Result, Runtime};
use serde::{Deserialize, Serialize};
use serenity::model::{
    channel::Message,
    id::{ChannelId, GuildId, UserId},
};
use serenity::prelude::Context as SerenityContext;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Debug, Clone)]
pub struct ReplyState {
    pub output: Option<ScriptOutput>,
    pub called: bool,
}

#[derive(Clone)]
pub struct ScriptContext {
    pub message: Message,
    pub args: Option<String>,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub serenity_ctx: Arc<SerenityContext>,
    pub db_pool: Arc<PgPool>,
    pub tag_name: Option<String>,
    pub tag_body: Option<String>,
    pub tag_owner_id: Option<UserId>,
    pub recursion_depth: usize,
    pub reply_state: Arc<std::sync::Mutex<ReplyState>>,
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
    id: String,

    #[qjs(skip_trace)]
    username: String,

    #[qjs(skip_trace)]
    avatar: Option<String>,
}

#[rquickjs::methods]
impl JsUser {
    #[qjs(get)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[qjs(get)]
    pub fn username(&self) -> String {
        self.username.clone()
    }

    #[qjs(get)]
    pub fn avatar(&self) -> Option<String> {
        self.avatar.clone()
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsMessage {
    #[qjs(skip_trace)]
    id: String,

    author: JsUser,

    #[qjs(skip_trace)]
    content: String,

    #[qjs(skip_trace)]
    script_ctx: ScriptContext,
}

#[rquickjs::methods]
impl JsMessage {
    #[qjs(get)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[qjs(get)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[qjs(get)]
    pub fn author(&self) -> JsUser {
        self.author.clone()
    }

    pub fn reply<'js>(
        &self,
        ctx: Ctx<'js>,
        arg1: rquickjs::Value<'js>,
        arg2: Option<rquickjs::Value<'js>>,
    ) -> Result<rquickjs::Value<'js>> {
        handle_reply(&ctx, &self.script_ctx, arg1, arg2)
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsChannel {
    #[qjs(skip_trace)]
    id: String,

    #[qjs(skip_trace)]
    name: String,

    #[qjs(skip_trace)]
    is_dm: bool,
}

#[rquickjs::methods]
impl JsChannel {
    #[qjs(get)]
    pub fn id(&self) -> String {
        self.id.clone()
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
    id: String,

    #[qjs(skip_trace)]
    name: String,

    #[qjs(skip_trace)]
    owner_id: String,
}

#[rquickjs::methods]
impl JsGuild {
    #[qjs(get)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[qjs(get)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[qjs(get)]
    pub fn owner_id(&self) -> String {
        self.owner_id.clone()
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

impl From<JsEmbed> for serenity::builder::CreateEmbed {
    fn from(e: JsEmbed) -> Self {
        let mut embed = serenity::builder::CreateEmbed::new();
        if let Some(title) = e.title {
            embed = embed.title(title);
        }
        if let Some(desc) = e.description {
            embed = embed.description(desc);
        }
        if let Some(url) = e.url {
            embed = embed.url(url);
        }
        if let Some(color) = e.color {
            embed = embed.color(color);
        }
        if let Some(author) = e.author {
            let mut a = serenity::builder::CreateEmbedAuthor::new(author.name);
            if let Some(url) = author.url {
                a = a.url(url);
            }
            if let Some(icon_url) = author.icon_url {
                a = a.icon_url(icon_url);
            }
            embed = embed.author(a);
        }
        if let Some(fields) = e.fields {
            for f in fields {
                embed = embed.field(f.name, f.value, f.inline.unwrap_or(false));
            }
        }
        if let Some(footer) = e.footer {
            let mut f = serenity::builder::CreateEmbedFooter::new(footer.text);
            if let Some(icon_url) = footer.icon_url {
                f = f.icon_url(icon_url);
            }
            embed = embed.footer(f);
        }
        if let Some(thumb) = e.thumbnail {
            embed = embed.thumbnail(thumb.url);
        }
        if let Some(img) = e.image {
            embed = embed.image(img.url);
        }
        embed
    }
}

impl From<&Message> for JsUser {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.author.id.get().to_string(),
            username: msg.author.name.clone(),
            avatar: msg.author.avatar_url(),
        }
    }
}

impl JsMessage {
    pub fn from_ctx(msg: &Message, script_ctx: &ScriptContext) -> Self {
        Self {
            id: msg.id.get().to_string(),
            author: JsUser::from(msg),
            content: msg.content.clone(),
            script_ctx: script_ctx.clone(),
        }
    }
}

impl JsChannel {
    pub fn from_ctx(script_ctx: &ScriptContext) -> Self {
        let channel_id = script_ctx.channel_id;
        let serenity_ctx = &script_ctx.serenity_ctx;

        let (name, is_dm) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(channel) = serenity_ctx.cache.channel(channel_id) {
                    return (channel.name.clone(), false);
                }
                if let Ok(channel) = serenity_ctx.http.get_channel(channel_id).await {
                    match channel {
                        serenity::all::Channel::Guild(g) => return (g.name, false),
                        serenity::all::Channel::Private(_) => return ("".to_string(), true),
                        _ => {}
                    }
                }
                ("".to_string(), script_ctx.message.guild_id.is_none())
            })
        });

        Self {
            id: channel_id.get().to_string(),
            name,
            is_dm,
        }
    }
}

impl JsGuild {
    pub fn from_ctx(script_ctx: &ScriptContext) -> Self {
        let guild_id = script_ctx.guild_id;
        let serenity_ctx = &script_ctx.serenity_ctx;

        let (name, owner_id) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(guild) = serenity_ctx.cache.guild(guild_id) {
                    return (guild.name.clone(), guild.owner_id.get().to_string());
                }
                if let Ok(guild) = serenity_ctx.http.get_guild(guild_id).await {
                    return (guild.name.clone(), guild.owner_id.get().to_string());
                }
                ("".to_string(), "".to_string())
            })
        });

        Self {
            id: guild_id.get().to_string(),
            name,
            owner_id,
        }
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsTag {
    #[qjs(skip_trace)]
    name: String,

    #[qjs(skip_trace)]
    args: Option<String>,

    #[qjs(skip_trace)]
    body: String,

    #[qjs(skip_trace)]
    owner: String,
}

#[rquickjs::methods]
impl JsTag {
    #[qjs(get)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[qjs(get)]
    pub fn args(&self) -> Option<String> {
        self.args.clone()
    }

    #[qjs(get)]
    pub fn body(&self) -> String {
        self.body.clone()
    }

    #[qjs(get)]
    pub fn owner(&self) -> String {
        self.owner.clone()
    }
}

#[derive(Clone, Trace, JsLifetime)]
#[rquickjs::class]
pub struct JsUtil {
    #[qjs(skip_trace)]
    script_ctx: ScriptContext,
}

#[rquickjs::methods]
impl JsUtil {
    pub fn fetch_tag<'js>(&self, ctx: Ctx<'js>, name: String) -> Result<rquickjs::Value<'js>> {
        fetch_tag_impl(&ctx, &self.script_ctx, &name)
    }

    #[qjs(rename = "fetchTag")]
    pub fn fetch_tag_camel<'js>(
        &self,
        ctx: Ctx<'js>,
        name: String,
    ) -> Result<rquickjs::Value<'js>> {
        fetch_tag_impl(&ctx, &self.script_ctx, &name)
    }

    pub fn get_tag<'js>(&self, ctx: Ctx<'js>, name: String) -> Result<rquickjs::Value<'js>> {
        fetch_tag_impl(&ctx, &self.script_ctx, &name)
    }

    #[qjs(rename = "getTag")]
    pub fn get_tag_camel<'js>(&self, ctx: Ctx<'js>, name: String) -> Result<rquickjs::Value<'js>> {
        fetch_tag_impl(&ctx, &self.script_ctx, &name)
    }

    pub fn exec_tag<'js>(
        &self,
        ctx: Ctx<'js>,
        name: String,
        args: Option<String>,
    ) -> Result<rquickjs::Value<'js>> {
        exec_tag_impl(&ctx, &self.script_ctx, &name, args)
    }

    #[qjs(rename = "execTag")]
    pub fn exec_tag_camel<'js>(
        &self,
        ctx: Ctx<'js>,
        name: String,
        args: Option<String>,
    ) -> Result<rquickjs::Value<'js>> {
        exec_tag_impl(&ctx, &self.script_ctx, &name, args)
    }

    #[qjs(rename = "executeTag")]
    pub fn execute_tag_camel<'js>(
        &self,
        ctx: Ctx<'js>,
        name: String,
        args: Option<String>,
    ) -> Result<rquickjs::Value<'js>> {
        exec_tag_impl(&ctx, &self.script_ctx, &name, args)
    }

    pub fn find_users<'js>(&self, ctx: Ctx<'js>, query: String) -> Result<rquickjs::Value<'js>> {
        find_users_impl(&ctx, &self.script_ctx, &query)
    }

    #[qjs(rename = "findUsers")]
    pub fn find_users_camel<'js>(
        &self,
        ctx: Ctx<'js>,
        query: String,
    ) -> Result<rquickjs::Value<'js>> {
        find_users_impl(&ctx, &self.script_ctx, &query)
    }

    pub fn reply<'js>(
        &self,
        ctx: Ctx<'js>,
        arg1: rquickjs::Value<'js>,
        arg2: Option<rquickjs::Value<'js>>,
    ) -> Result<rquickjs::Value<'js>> {
        handle_reply(&ctx, &self.script_ctx, arg1, arg2)
    }
}

fn fetch_tag_impl<'js>(
    ctx: &Ctx<'js>,
    script_ctx: &ScriptContext,
    name: &str,
) -> Result<rquickjs::Value<'js>> {
    let name_str = name.to_string();
    let guild_id = script_ctx.guild_id;
    let db_pool = script_ctx.db_pool.clone();

    let tag_opt = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { fetch_tag_resolved(&name_str, guild_id, &db_pool).await })
    });

    match tag_opt {
        Ok(Some(tag)) => {
            let obj = rquickjs::Object::new(ctx.clone())?;
            obj.set("name", tag.name.clone())?;

            let kind_str = match tag.kind {
                TagKind::Text => "text",
                TagKind::Script => "script",
                TagKind::Embed => "embed",
                TagKind::Alias => "alias",
            };
            obj.set("kind", kind_str)?;
            obj.set("owner_id", tag.owner_id.to_string())?;

            let content_str = match tag.kind {
                TagKind::Text => tag
                    .payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                TagKind::Script => tag
                    .payload
                    .get("script")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                TagKind::Embed | TagKind::Alias => {
                    serde_json::to_string(&tag.payload).unwrap_or_default()
                }
            };

            obj.set("content", content_str.clone())?;

            let to_string_fn = rquickjs::Function::new(ctx.clone(), {
                let c = content_str;
                move || c.clone()
            })?;
            obj.set("toString", to_string_fn)?;

            if let Ok(json_str) = serde_json::to_string(&tag.payload) {
                let code = format!("JSON.parse({})", serde_json::to_string(&json_str).unwrap());
                if let Ok(js_val) = ctx.eval::<rquickjs::Value, _>(code) {
                    obj.set("payload", js_val)?;
                }
            }

            Ok(obj.into_value())
        }
        _ => Ok(rquickjs::Value::new_null(ctx.clone())),
    }
}

fn exec_tag_impl<'js>(
    ctx: &Ctx<'js>,
    script_ctx: &ScriptContext,
    name: &str,
    args: Option<String>,
) -> Result<rquickjs::Value<'js>> {
    if script_ctx.recursion_depth >= 5 {
        return Ok(rquickjs::Value::new_null(ctx.clone()));
    }

    let name_str = name.to_string();
    let guild_id = script_ctx.guild_id;
    let db_pool = script_ctx.db_pool.clone();

    let tag_opt = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { fetch_tag_resolved(&name_str, guild_id, &db_pool).await })
    });

    let tag = match tag_opt {
        Ok(Some(t)) => t,
        _ => return Ok(rquickjs::Value::new_null(ctx.clone())),
    };

    match tag.kind {
        TagKind::Text => {
            let content = tag
                .payload
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let js_str = rquickjs::String::from_str(ctx.clone(), content)?;
            Ok(js_str.into_value())
        }
        TagKind::Embed => {
            let embed_val = tag.payload.get("embed").cloned().unwrap_or(tag.payload);
            if let Ok(json_str) = serde_json::to_string(&embed_val) {
                let code = format!("JSON.parse({})", serde_json::to_string(&json_str).unwrap());
                if let Ok(js_val) = ctx.eval::<rquickjs::Value, _>(code) {
                    return Ok(js_val);
                }
            }
            Ok(rquickjs::Value::new_null(ctx.clone()))
        }
        TagKind::Script => {
            let script_body = tag
                .payload
                .get("script")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let child_args = args.or_else(|| script_ctx.args.clone());
            let child_ctx = ScriptContext {
                message: script_ctx.message.clone(),
                args: child_args,
                guild_id: script_ctx.guild_id,
                channel_id: script_ctx.channel_id,
                author_id: script_ctx.author_id,
                serenity_ctx: script_ctx.serenity_ctx.clone(),
                db_pool: script_ctx.db_pool.clone(),
                tag_name: Some(tag.name.clone()),
                tag_body: Some(script_body.to_string()),
                tag_owner_id: Some(UserId::new(tag.owner_id as u64)),
                recursion_depth: script_ctx.recursion_depth + 1,
                reply_state: Default::default(),
            };

            let engine = ScriptEngine::new()?;
            match engine.execute(script_body, child_ctx) {
                Ok(ScriptOutput::Text(t)) => {
                    let js_str = rquickjs::String::from_str(ctx.clone(), &t)?;
                    Ok(js_str.into_value())
                }
                Ok(ScriptOutput::Embed(embed_json)) => {
                    if let Ok(json_str) = serde_json::to_string(&embed_json) {
                        let code =
                            format!("JSON.parse({})", serde_json::to_string(&json_str).unwrap());
                        if let Ok(js_val) = ctx.eval::<rquickjs::Value, _>(code) {
                            return Ok(js_val);
                        }
                    }
                    Ok(rquickjs::Value::new_null(ctx.clone()))
                }
                Err(_) => Ok(rquickjs::Value::new_null(ctx.clone())),
            }
        }
        TagKind::Alias => Ok(rquickjs::Value::new_null(ctx.clone())),
    }
}

fn find_users_impl<'js>(
    ctx: &Ctx<'js>,
    script_ctx: &ScriptContext,
    query: &str,
) -> Result<rquickjs::Value<'js>> {
    let query_str = query.trim().to_string();
    let serenity_ctx = script_ctx.serenity_ctx.clone();
    let guild_id = script_ctx.guild_id;

    let users: Vec<JsUser> = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut results = Vec::new();

            let parsed_id = crate::util::tag::get_uid_from_user_text(&query_str).ok();

            if let Some(uid) = parsed_id {
                if let Ok(user) = serenity_ctx.http.get_user(uid).await {
                    let avatar = user.avatar_url();
                    results.push(JsUser {
                        id: user.id.get().to_string(),
                        username: user.name,
                        avatar,
                    });
                }
            } else {
                if let Ok(members) = serenity_ctx
                    .http
                    .search_guild_members(guild_id, &query_str, Some(10))
                    .await
                {
                    for member in members {
                        let avatar = member.user.avatar_url();
                        results.push(JsUser {
                            id: member.user.id.get().to_string(),
                            username: member.user.name,
                            avatar,
                        });
                    }
                }
            }
            results
        })
    });

    let js_array = rquickjs::Array::new(ctx.clone())?;
    for (idx, u) in users.into_iter().enumerate() {
        js_array.set(idx, u)?;
    }

    Ok(js_array.into_value())
}

fn handle_reply<'js>(
    ctx: &Ctx<'js>,
    script_ctx: &ScriptContext,
    arg1: rquickjs::Value<'js>,
    arg2: Option<rquickjs::Value<'js>>,
) -> Result<rquickjs::Value<'js>> {
    let mut state = script_ctx.reply_state.lock().unwrap();
    if state.called {
        return Err(rquickjs::Exception::throw_message(
            ctx,
            "reply can only be used once per execution",
        ));
    }

    state.called = true;
    let output = parse_reply_output(ctx, arg1, arg2)?;
    state.output = Some(output);

    Ok(rquickjs::Value::new_undefined(ctx.clone()))
}

fn parse_reply_output<'js>(
    ctx: &Ctx<'js>,
    arg1: rquickjs::Value<'js>,
    arg2: Option<rquickjs::Value<'js>>,
) -> Result<ScriptOutput> {
    let mut embed_val: Option<serde_json::Value> = None;
    let mut text_val: Option<String> = None;

    if let Some(s) = arg1.as_string() {
        text_val = s.to_string().ok();
        if let Some(e) = arg2 {
            if e.is_object() {
                if let Some(val) = try_parse_embed(ctx, &e) {
                    embed_val = Some(val);
                }
            }
        }
    } else if arg1.is_object() {
        if let Some(val) = try_parse_embed(ctx, &arg1) {
            embed_val = Some(val);
        }
    }

    if let Some(embed) = embed_val {
        Ok(ScriptOutput::Embed(embed))
    } else if let Some(text) = text_val {
        Ok(ScriptOutput::Text(text))
    } else {
        Ok(ScriptOutput::Text(String::new()))
    }
}

fn try_parse_embed<'js>(ctx: &Ctx<'js>, arg: &rquickjs::Value<'js>) -> Option<serde_json::Value> {
    if let Ok(json_obj) = ctx.globals().get::<_, rquickjs::Object>("JSON") {
        if let Ok(stringify) = json_obj.get::<_, rquickjs::Function>("stringify") {
            if let Ok(json_js_str) = stringify.call::<_, rquickjs::String>((arg,)) {
                if let Ok(json_str) = json_js_str.to_string() {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        return Some(val);
                    }
                }
            }
        }
    }
    None
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

            let state = script_ctx.reply_state.lock().unwrap();
            if let Some(explicit_output) = state.output.clone() {
                Ok(explicit_output)
            } else {
                parse_output(&ctx, result)
            }
        })
    }

    fn register_globals<'js>(&self, ctx: &Ctx<'js>, script_ctx: &ScriptContext) -> Result<()> {
        let global = ctx.globals();

        let message = JsMessage::from_ctx(&script_ctx.message, script_ctx);
        let channel = JsChannel::from_ctx(script_ctx);
        let guild = JsGuild::from_ctx(script_ctx);

        global.set("msg", message)?;
        global.set("args", script_ctx.args.clone().unwrap_or_default())?;
        global.set("channel", channel)?;
        global.set("guild", guild)?;

        if let Some(name) = &script_ctx.tag_name {
            let js_tag = JsTag {
                name: name.clone(),
                args: script_ctx.args.clone(),
                body: script_ctx.tag_body.clone().unwrap_or_default(),
                owner: script_ctx
                    .tag_owner_id
                    .map(|id| id.get().to_string())
                    .unwrap_or_default(),
            };
            global.set("tag", js_tag)?;
        }

        let js_util = JsUtil {
            script_ctx: script_ctx.clone(),
        };
        global.set("util", js_util)?;

        Ok(())
    }
}

fn parse_output<'js>(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> Result<ScriptOutput> {
    if value.is_null() || value.is_undefined() {
        return Ok(ScriptOutput::Text(String::new()));
    }

    if let Some(s) = value.as_string() {
        let s_str = s.to_string()?;
        return Ok(ScriptOutput::Text(s_str));
    }

    if value.is_object() {
        if let Some(val) = try_parse_embed(ctx, &value) {
            if is_embed_like(&val) {
                return Ok(ScriptOutput::Embed(val));
            }
        }
    }

    let str_val = if let Ok(string_fn) = ctx.globals().get::<_, rquickjs::Function>("String") {
        if let Ok(res) = string_fn.call::<_, rquickjs::String>((value,)) {
            res.to_string().unwrap_or_else(|_| "undefined".to_string())
        } else {
            "undefined".to_string()
        }
    } else {
        "undefined".to_string()
    };

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
