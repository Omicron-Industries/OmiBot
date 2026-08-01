use crate::commands::{send_reply_ping_message, send_reply_ping_text, CommandContext};
use crate::db::tags::fetch::fetch_tag_resolved;
use crate::util::script::{ScriptContext, ScriptEngine, ScriptOutput};
use crate::util::tag::embed::EmbedTagContent;
use crate::util::tag::script::ScriptTagContent;
use crate::util::tag::text::TextTagContent;
use crate::util::tag::TagKind;
use log::error;
use serenity::all::{CreateEmbed, CreateMessage};

pub async fn execute_tag(ctx: &mut CommandContext, tag_name: &str) {
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

                let _ = send_reply_ping_message(
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
                        message: ctx.msg.clone(),
                        args: ctx.args.clone(),
                        guild_id: ctx.msg.guild_id.unwrap(),
                        channel_id: ctx.msg.channel_id,
                        author_id: ctx.msg.author.id,
                        serenity_ctx: std::sync::Arc::new(ctx.serenity_ctx.clone()),
                        db_pool: std::sync::Arc::new(ctx.state.db_pool.clone()),
                        tag_name: Some(tag_name.to_string()),
                        tag_body: Some(payload.script.clone()),
                        tag_owner_id: Some(serenity::all::UserId::new(tag.owner_id as u64)),
                        recursion_depth: 0,
                        reply_state: Default::default(),
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
                    Ok(output) => match output {
                        ScriptOutput::Text(text) => {
                            if !text.is_empty() {
                                send_reply_ping_text(ctx, &text).await;
                            }
                        }
                        ScriptOutput::Embed(embed_json) => {
                            let inner_json = embed_json.get("embed").unwrap_or(&embed_json);

                            if let Ok(js_embed) = serde_json::from_value::<
                                crate::util::script::JsEmbed,
                            >(inner_json.clone())
                            {
                                send_embed_reply(ctx, CreateEmbed::from(js_embed)).await;
                            } else if let Ok(embed_data) =
                                serde_json::from_value::<EmbedTagContent>(embed_json)
                            {
                                send_embed_reply(ctx, CreateEmbed::from(embed_data.embed)).await;
                            } else {
                                send_reply_ping_text(
                                    ctx,
                                    "Failed to parse embed from script output.",
                                )
                                .await;
                            }
                        }
                    },
                }
            }
        },
    }
}

async fn send_embed_reply(ctx: &CommandContext, embed: CreateEmbed) {
    if let Err(e) = send_reply_ping_message(ctx, CreateMessage::new().embed(embed)).await {
        send_reply_ping_text(ctx, format!("Embed invalid: {}", e).as_str()).await;
    }
}

pub async fn payload_mismatch_error(ctx: &CommandContext, name: &str) {
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
