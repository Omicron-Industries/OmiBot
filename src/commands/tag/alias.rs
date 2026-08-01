use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::tag_name_validator;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::fetch::fetch_tag;
use crate::util::tag::alias::AliasTagContent;
use crate::util::tag::script::ScriptTagContent;
use crate::util::tag::text::TextTagContent;
use crate::util::tag::{try_create_tag, CreateTagModel, TagPayload};
use log::error;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag alias",
    usage: Some("<new_alias> <existing_tag>"),
    full_desc: "Alias a tag to another tag.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") | _ if ctx.help => command_help(ctx, INFO).await,
        Some("detect") | Some("detectable") => execute(&mut orig_ctx, true).await,
        _ => execute(&mut orig_ctx, false).await,
    }
}

pub async fn execute(ctx: &mut CommandContext, detect: bool) {
    let Some(name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };
    if ctx.args.is_none() {
        return command_usage(ctx, INFO).await;
    }
    match tag_name_validator(&name) {
        Some(err_msg) => send_reply_ping_text(ctx, err_msg.as_str()).await,
        None => {
            let source_tag = match fetch_tag(&name, ctx.get_guild_id(), &ctx.state.db_pool).await {
                Ok(fetched) => match fetched {
                    Some(src_tag) => src_tag,
                    None => {
                        send_reply_ping_text(
                            ctx,
                            format!("Tag **{}** does not exist!", name).as_str(),
                        )
                        .await;
                        return;
                    }
                },
                Err(e) => {
                    error!("Error when fetching source tag of alias: {}", e);
                    send_reply_ping_text(ctx, "Error reading source tag!").await;
                    return;
                }
            };

            let payload = AliasTagContent {
                target_id: source_tag.id,
            };

            try_create_tag(
                ctx,
                CreateTagModel::with_ctx(&ctx, &name, TagPayload::Alias(payload), Some(detect)),
            )
            .await
        }
    }
}
