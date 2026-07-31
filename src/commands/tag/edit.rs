use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::tag_name_validator;
use crate::commands::tag::util::embed::parse_embed_tag_content;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::tag::util::TagPayload;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::edit::{edit_tag_content, EditTagError};

pub const INFO: CommandInfo = CommandInfo {
    command: "tag edit",
    usage: Some("<tag_name> <content>"),
    full_desc: "Edit the content of an existing tag.",
    short_desc: Some("Edit a tag's content."),
    aliases: &[],
    further_help: Some(
        "Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{PREFIX}help tag script` or `{PREFIX}help tag embed`",
    ),
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
    if ctx.args.is_none() {
        return command_usage(ctx, INFO).await;
    }
    match tag_name_validator(&name) {
        Some(err_msg) => send_reply_ping_text(ctx, err_msg.as_str()).await,
        None => {
            let arg = ctx.args.clone().unwrap_or_default();
            let trimmed = arg.trim();

            let payload: TagPayload = if let Some(embed_content) = parse_embed_tag_content(&arg) {
                match embed_content {
                    Ok(embed_tag_content) => TagPayload::Embed(embed_tag_content),
                    Err(msg) => return send_reply_ping_text(ctx, &msg).await,
                }
            } else if trimmed.starts_with("```json") || trimmed.starts_with("```embed") {
                return send_reply_ping_text(
                    ctx,
                    "Failed to parse embed JSON. Please verify that the JSON formatting is valid.",
                )
                .await;
            } else if let Some(inner) = arg.strip_prefix("```").and_then(|s| s.strip_suffix("```"))
            {
                let inner = inner
                    .strip_prefix("js")
                    .map(str::trim_start)
                    .unwrap_or(inner);

                TagPayload::Script(ScriptTagContent {
                    script: inner.to_string(),
                })
            } else {
                TagPayload::Text(TextTagContent { content: arg })
            };

            match edit_tag_content(ctx.get_guild_id(), &name, payload, &ctx.state.db_pool).await {
                Ok(_) => send_reply_ping_text(ctx, "Successfully edited tag content.").await,
                Err(EditTagError::Serialize) => {
                    send_reply_ping_text(ctx, "Failed to serialize tag.").await
                }
                Err(EditTagError::DB(e)) => {
                    send_reply_ping_text(
                        ctx,
                        format!("Error editing tag content: {:?}", e).as_str(),
                    )
                    .await
                }
            }
        }
    }
}
