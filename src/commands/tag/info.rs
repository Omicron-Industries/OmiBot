use crate::commands::help::{command_help, command_usage};
use crate::commands::tag::util::db::fetch_tag;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use log::error;

pub const INFO: CommandInfo = CommandInfo {
    command: "",
    usage: Some(""),
    full_desc: "",
    short_desc: Some(""),
    aliases: &[],
    further_help: None,
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

    match fetch_tag(&name, ctx.get_guild_id(), &ctx.state.db_pool).await {
        Err(e) => {
            error!("Failed to get tag: {}", e);
            send_reply_ping_text(
                ctx,
                format!("Error when searching for tag: \"{}\"\n{}", name, e).as_str(),
            )
            .await
        }
        Ok(None) => {
            send_reply_ping_text(
                ctx,
                format!("No tag with name \"{}\" found!", name).as_str(),
            )
            .await
        }
        Ok(Some(tag)) => {
            send_reply_ping_text(
                ctx,
                format!(
                    "Tag **{}**:\nOwner: <@{}>\nCreated <t:{}:R>\nLast updated <t:{}:R>",
                    tag.name,
                    tag.owner_id,
                    tag.t_created.timestamp(),
                    tag.t_updated.timestamp()
                )
                .as_str(),
            )
            .await
        }
    }
}
