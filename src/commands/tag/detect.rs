use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::detect::toggle_detectable;
use crate::util::permissions::get_admin_action_msg;

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag detect",
    usage: Some("<tag_name>"),
    full_desc: "Toggle a tag's detectability. If enabled, the name of the tag appearing anywhere within a message will trigger its execution",
    short_desc: Some("Toggle a tag's detectability."),
    aliases: &["detectable"],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx).await {
        send_reply_ping_text(ctx, &msg).await
    }
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

    match toggle_detectable(ctx.get_guild_id(), &name, &ctx.state).await {
        Err(e) => {
            send_reply_ping_text(
                ctx,
                format!("Error toggling tag detectable: {:?}", e).as_str(),
            )
            .await
        }
        Ok(current) => {
            send_reply_ping_text(
                ctx,
                format!(
                    "Tag **{}** is {} detectable.",
                    name,
                    if current { "now" } else { "no longer" }
                )
                .as_str(),
            )
            .await
        }
    }
}
