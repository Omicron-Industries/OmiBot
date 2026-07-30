use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::util::embed::EmbedTagContent;
use crate::commands::tag::util::script::{ScriptContext, ScriptEngine, ScriptTagContent};
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::tag::util::TagKind;
use crate::commands::{
    get_prefix, send_reply_ping_message, send_reply_ping_text, CommandContext, CommandInfo,
};
use crate::db::tags::fetch::fetch_tag_resolved;
use crate::BotState;
use log::error;
use serenity::builder::{CreateEmbed, CreateMessage};
use std::sync::Arc;

mod add;
mod alias;
mod ban;
mod bans;
mod chown;
mod delete;
mod edit;
mod info;
mod list;
mod raw;
mod rename;
mod search;
mod unban;
pub(crate) mod util;

pub const INFO: CommandInfo = CommandInfo {
    command: "tag",
    usage: Some("(<tag_name> | <subcommand>)"),
    full_desc: "",
    short_desc: Some(""),
    aliases: &["t"],
    further_help: Some(
        "Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{PREFIX}help tag script` or `{PREFIX}help tag embed`",
    ),
    subcommands: None,
};

/// All names for the tag command reserved for subcommands, prohibited from being made into a tag.
pub const TAG_SUBCOMMANDS: &[&str] = &[
    "add", "create", "edit", "delete", "del", "alias", "info", "owner", "raw", "list", "search",
    "chown", "transfer", "ban", "unban", "bans", "help",
];

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") | Some("create") => add::dispatch(ctx).await,
        Some("edit") => edit::dispatch(ctx).await,
        Some("delete") | Some("del") => delete::dispatch(ctx).await,
        Some("alias") => alias::dispatch(ctx).await,
        Some("info") | Some("owner") => info::dispatch(ctx).await,
        Some("raw") => raw::dispatch(ctx).await,
        Some("list") => list::dispatch(ctx).await,
        Some("search") => search::dispatch(ctx).await,
        Some("chown") | Some("transfer") => chown::dispatch(ctx).await,
        Some("ban") => ban::dispatch(ctx).await,
        Some("unban") => unban::dispatch(ctx).await,
        Some("bans") => bans::dispatch(ctx).await,
        Some("help") => {
            let next = ctx.consume_arg();
            match next.as_deref() {
                Some("script") => help_script(ctx).await,
                Some("embed") => help_embed(ctx).await,
                _ => command_help(ctx, INFO).await,
            }
        }

        _ if ctx.help => command_help(ctx, INFO).await,
        Some(_) => execute(&mut orig_ctx).await,
        None => command_usage(ctx, INFO).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(tag_name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    match fetch_tag_resolved(&tag_name, ctx.msg.guild_id.unwrap(), &ctx.state.db_pool).await {
        Err(e) => {
            error!("Failed to get tag: {}", e);
            send_reply_ping_text(
                ctx,
                format!("Error when searching for tag: \"{}\"\n{}", tag_name, e).as_str(),
            )
            .await;
        }
        Ok(None) => {
            send_reply_ping_text(
                ctx,
                format!("No tag with name \"{}\" found!", tag_name).as_str(),
            )
            .await;
        }
        Ok(Some(tag)) => match tag.kind {
            TagKind::Text => {
                let payload: TextTagContent = match serde_json::from_value(tag.payload) {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("Failed to deserialize payload: {}", e);
                        return payload_mismatch_error(ctx, &tag_name).await;
                    }
                };

                send_reply_ping_text(ctx, &payload.content).await;
            }
            TagKind::Alias => {
                error!("Got an alias tag on a resolved fetch!");
                send_reply_ping_text(
                    ctx,
                    format!("There was an error resolving the alias tag {}", tag_name).as_str(),
                )
                .await;
            }
            TagKind::Embed => {
                let payload: EmbedTagContent = match serde_json::from_value(tag.payload) {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("Failed to deserialize payload: {}", e);
                        return payload_mismatch_error(ctx, &tag_name).await;
                    }
                };

                send_reply_ping_message(
                    ctx,
                    CreateMessage::new().embed(CreateEmbed::from(payload.embed)),
                )
                .await;
            }
            TagKind::Script => {
                let payload: ScriptTagContent = match serde_json::from_value(tag.payload) {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("Failed to deserialize payload: {}", e);
                        return payload_mismatch_error(ctx, &tag_name).await;
                    }
                };

                let result = (|| {
                    let engine = ScriptEngine::new()?;

                    let script_context = ScriptContext {
                        args: ctx.args.clone(),
                        guild_id: ctx.msg.guild_id.unwrap(),
                        channel_id: ctx.msg.channel_id,
                        author_id: ctx.msg.author.id,
                    };

                    engine.execute(&payload.script, script_context)
                })();

                match result {
                    Err(e) => {
                        send_reply_ping_text(
                            ctx,
                            format!("Failed to execute script: {:?}", e).as_str(),
                        )
                        .await;
                    }
                    Ok(output) => {
                        send_reply_ping_text(ctx, output.to_string().as_str()).await;
                    }
                }
            }
        },
    }
}

async fn help_script(ctx: &CommandContext) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"Script tags run sandboxed JS, and can take in args from the user.

To create a script tag, the content of the tag must be a multi-line JS code block:
```{prefix}tag add <name> `​`​`js
<script_content>
`​`​`
```

The String `args` is made available at the start of the script, containing all message content after the tag name.
For a full API reference, see [here](https://git.marinodev.com/drake/bunny_bot/api.md)."#
    );
    send_reply_ping_text(ctx, &content).await;
}

async fn help_embed(ctx: &CommandContext) {
    let prefix = get_prefix(ctx).await;
    let content = format!(r#"TODO"#);
    send_reply_ping_text(ctx, &content).await;
}

async fn payload_mismatch_error(ctx: &CommandContext, name: &str) {
    error!("Tag {} payload kind does not match tag kind!", name);
    send_reply_ping_text(
        ctx,
        format!(
            "Error when evaluating tag **{}**. Please report error to <@435572469496020992>",
            name
        )
        .as_str(),
    )
    .await;
}

pub fn tag_name_validator(name: &str) -> Option<String> {
    if TAG_SUBCOMMANDS.contains(&name) {
        Some(format!(
            "Tag name **{name}** is disallowed, as it is a subcommand!"
        ))
    } else if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Some(
            "Tag name must contain only letters, numbers, underscores (_), and hyphens (-)."
                .to_string(),
        )
    } else if name.len() > 32 {
        Some("Tag name must not exceed 32 characters.".to_string())
    } else {
        None
    }
}
