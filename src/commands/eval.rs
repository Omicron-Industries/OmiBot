use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_message, send_reply_ping_text, CommandContext, CommandInfo};
use crate::util::script::{JsEmbed, ScriptContext, ScriptEngine, ScriptOutput};
use crate::util::tag::embed::{try_handle_embed_tag, EmbedTagContent};
use serenity::builder::{CreateEmbed, CreateMessage};

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "eval",
    usage: Some("<script_content>"),
    full_desc: "Evaluates JavaScript code or renders embed JSON in the bot's script environment.",
    short_desc: Some("Evaluates JS code or renders embed JSON."),
    aliases: &[],
    further_help: Some(
        "Script content can be passed as raw text, inline backticks, a JS codeblock, or JSON/embed block:\n`{PREFIX}eval 6+4`\n`{PREFIX}eval `​`​`json\n{\"title\": \"My Embed\"}\n`​`​`",
    ),
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if ctx.help {
        return command_help(ctx, INFO).await;
    }
    if ctx.peek_arg() == Some("help".to_string()) {
        return command_help(ctx, INFO).await;
    }
    execute(ctx).await;
}

pub async fn execute(ctx: &CommandContext) {
    let Some(raw_code) = &ctx.args else {
        return command_usage(ctx, INFO).await;
    };

    if try_handle_embed_tag(ctx, raw_code).await {
        return;
    }

    let code = clean_code(raw_code);
    if code.is_empty() {
        return command_usage(ctx, INFO).await;
    }

    let script_context = ScriptContext {
        message: ctx.msg.clone(),
        args: None,
        guild_id: ctx.msg.guild_id.unwrap_or_default(),
        channel_id: ctx.msg.channel_id,
        author_id: ctx.msg.author.id,
        serenity_ctx: std::sync::Arc::new(ctx.serenity_ctx.clone()),
        db_pool: std::sync::Arc::new(ctx.state.db_pool.clone()),
        tag_name: None,
        tag_body: None,
        tag_owner_id: None,
        recursion_depth: 0,
        reply_state: Default::default(),
    };

    let result = (|| {
        let engine = ScriptEngine::new()?;
        engine.execute(code, script_context)
    })();

    match result {
        Err(e) => {
            send_reply_ping_text(ctx, format!("Execution error:\n```\n{:?}\n```", e).as_str())
                .await;
        }
        Ok(output) => match output {
            ScriptOutput::Text(text) => {
                if text.is_empty() {
                    send_reply_ping_text(ctx, "*No output*").await;
                } else {
                    send_reply_ping_text(ctx, &text).await;
                }
            }
            ScriptOutput::Embed(embed_json) => {
                let inner_json = embed_json.get("embed").unwrap_or(&embed_json);
                if let Ok(js_embed) = serde_json::from_value::<JsEmbed>(inner_json.clone()) {
                    send_reply_ping_message(
                        ctx,
                        CreateMessage::new().embed(CreateEmbed::from(js_embed)),
                    )
                    .await;
                } else if let Ok(embed_data) = serde_json::from_value::<EmbedTagContent>(embed_json)
                {
                    send_reply_ping_message(
                        ctx,
                        CreateMessage::new().embed(CreateEmbed::from(embed_data.embed)),
                    )
                    .await;
                } else {
                    send_reply_ping_text(ctx, "Failed to parse embed from script output.").await;
                }
            }
        },
    }
}

fn clean_code(raw: &str) -> &str {
    let mut s = raw.trim();
    if let Some(stripped) = s.strip_prefix("```js") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("```javascript") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("```") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix('`') {
        s = stripped;
    }

    if let Some(stripped) = s.strip_suffix("```") {
        s = stripped;
    } else if let Some(stripped) = s.strip_suffix('`') {
        s = stripped;
    }
    s.trim()
}
