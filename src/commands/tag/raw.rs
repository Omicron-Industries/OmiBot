use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::payload_mismatch_error;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::tag::util::TagKind::{Alias, Embed, Script, Text};
use crate::commands::{send_reply_ping_message, send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::fetch::fetch_tag;
use log::error;
use serenity::all::{CreateAttachment, CreateMessage};

pub const INFO: CommandInfo = CommandInfo {
    command: "tag raw",
    usage: Some("<tag_name>"),
    full_desc: "Get the unformatted raw content of a tag sent as a file attachment.",
    short_desc: Some("Get raw tag content."),
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    match fetch_tag(&name, ctx.msg.guild_id.unwrap(), &ctx.state.db_pool).await {
        Err(e) => {
            error!("Failed to get tag: {}", e);
            send_reply_ping_text(
                ctx,
                format!("Error when searching for tag: \"{}\"\n{}", name, e).as_str(),
            )
            .await;
        }
        Ok(None) => {
            send_reply_ping_text(
                ctx,
                format!("No tag with name \"{}\" found!", name).as_str(),
            )
            .await;
        }
        Ok(Some(tag)) => match tag.kind {
            Alias => match tag.alias_target_name {
                Some(alias_target_name) => {
                    send_reply_ping_text(
                        ctx,
                        format!(
                            "**{}** is an alias of tag **{}**",
                            tag.name, alias_target_name
                        )
                        .as_str(),
                    )
                    .await
                }
                None => {
                    send_reply_ping_text(
                        ctx,
                        format!("Failed to resolve alias **{}**!", tag.name).as_str(),
                    )
                    .await
                }
            },
            Text => {
                let payload: TextTagContent = match serde_json::from_value(tag.payload) {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("Failed to deserialize payload: {}", e);
                        return payload_mismatch_error(ctx, &tag.name).await;
                    }
                };

                let attachment = CreateAttachment::bytes(
                    payload.content.as_bytes().to_vec(),
                    format!("{}.txt", tag.name),
                );
                send_reply_ping_message(ctx, CreateMessage::new().add_file(attachment)).await;
            }
            Script => {
                let payload: ScriptTagContent = match serde_json::from_value(tag.payload) {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("Failed to deserialize payload: {}", e);
                        return payload_mismatch_error(ctx, &tag.name).await;
                    }
                };

                let attachment = CreateAttachment::bytes(
                    payload.script.as_bytes().to_vec(),
                    format!("{}.js", tag.name),
                );
                send_reply_ping_message(ctx, CreateMessage::new().add_file(attachment)).await;
            }
            Embed => {}
        },
    }
}
