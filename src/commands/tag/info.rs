use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::fetch::{fetch_owners_tags, fetch_tag};
use crate::db::DbId;
use log::error;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;

pub const INFO: CommandInfo = CommandInfo {
    command: "tag info",
    usage: Some("<tag_name>"),
    full_desc: "Display information about a tag, including owner and timestamps.",
    short_desc: Some("Get tag information."),
    aliases: &["owner"],
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

pub async fn get_users_tags_msg(gid: GuildId, uid: UserId, db: &PgPool) -> String {
    let tags = match fetch_owners_tags(uid.db_id(), gid.db_id(), db).await {
        Ok(tags) => tags,
        Err(e) => {
            error!("Error fetching tags of user {uid}");
            return format!("Error fetching tags of user <@{uid}>!");
        }
    };

    if tags.is_empty() {
        return format!("User <@{uid}> does not own any tags!");
    }

    let tag_list = tags
        .iter()
        .map(|tag| tag.name.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    format!("<@{uid}>'s Tags:\n{tag_list}")
}
