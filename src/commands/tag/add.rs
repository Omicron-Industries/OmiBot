use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::tag_name_validator;
use crate::commands::tag::util::script::ScriptTagContent;
use crate::commands::tag::util::text::TextTagContent;
use crate::commands::tag::util::{try_create_tag, CreateTagModel, TagPayload};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};

pub const INFO: CommandInfo = CommandInfo {
    command: "tag add",
    usage: Some("<tag_name> <content>"),
    full_desc: "Create a new tag.",
    short_desc: None,
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
        Some("help") => command_help(ctx, INFO).await,
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
            let payload: TagPayload = {
                if let Some(inner) = ctx
                    .args
                    .clone()
                    .unwrap()
                    .strip_prefix("```js")
                    .and_then(|args| args.strip_suffix("```"))
                {
                    TagPayload::Script(ScriptTagContent {
                        script: inner.to_string(),
                    })
                } else {
                    TagPayload::Text(TextTagContent {
                        content: ctx.args.clone().unwrap_or_default().to_string(),
                    })
                }
            };
            // TODO: Add embed support

            try_create_tag(ctx, CreateTagModel::with_ctx(&ctx, &name, payload)).await
        }
    }
}
