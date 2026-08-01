use crate::commands::help::{command_help, command_usage};
use crate::commands::{send_internal_error_msg, send_reply_ping_text, CommandContext, CommandInfo};
use crate::db::tags::create::create_tag;
use crate::db::tags::edit::change_tag_owner;
use crate::db::DbId;
use crate::util::permissions::{get_admin_action_msg, Permission};
use crate::util::tag::script::ScriptTagContent;
use crate::util::tag::text::TextTagContent;
use crate::util::tag::{get_uid_from_user_text, read_attachment, CreateTagModel, TagPayload};
use crate::{BotState, MigrationInfo};
use serenity::all::{Context, CreateAllowedMentions, CreateMessage, Message};
use std::sync::Arc;

// Not made public
pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag migrate",
    usage: Some("migrate <tag1> [tag2] ..."),
    full_desc: "Migrates a list of tags from leveret.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    if let Some(msg) = get_admin_action_msg(ctx, Permission::ManageTags).await {
        return send_reply_ping_text(ctx, &msg).await;
    }

    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") => command_help(ctx, INFO).await,
        _ if ctx.help => command_help(ctx, INFO).await,
        Some(_) => execute(&mut orig_ctx).await,
        None => command_usage(ctx, INFO).await,
    }
}

use tokio::time::{sleep, Duration};

pub async fn execute(ctx: &mut CommandContext) {
    let args = ctx.args.clone().unwrap();
    let tags = args.split_whitespace().collect::<Vec<_>>();

    send_reply_ping_text(
        ctx,
        format!("Starting migration of {} tags...", tags.len()).as_str(),
    )
    .await;

    for (i, tag) in tags.iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_millis(1000)).await; // dont spam too hard
        }

        match ctx
            .msg
            .channel_id
            .send_message(
                &ctx.serenity_ctx.http,
                CreateMessage::new().content(format!("%t raw {}", tag)),
            )
            .await
        {
            Err(e) => {
                send_internal_error_msg(ctx, &format!("Error sending migration message: {}", e))
                    .await;
            }
            Ok(new_msg) => {
                let info = MigrationInfo {
                    guild_id: ctx.get_guild_id(),
                    migrator_id: ctx.get_author_id(),
                    tag_name: (*tag).to_string(),
                    content: true,
                };
                ctx.state.pending_migrations.insert(new_msg.id, info).await;
                println!("{:?}", ctx.state.pending_migrations)
            }
        }
    }
    send_reply_ping_text(ctx, "Sent all initial raw requests.").await;
}

pub async fn receive_tag_content(
    msg: &Message,
    migration_info: MigrationInfo,
    state: Arc<BotState>,
    ctx: Context,
) {
    state.pending_migrations.remove(&msg.id).await;
    match migration_info.tag_name.split(":").last() {
        None => {
            let _ = msg.reply(&ctx.http, "Could not determine tag name!").await;
            state.pending_migrations.remove(&msg.id).await;
            return;
        }

        Some(name) => match migration_info.content {
            true => match msg.attachments.first() {
                None => {
                    let _ = msg.reply(&ctx.http, "Expected an attachment.").await;
                    return;
                }
                Some(attachment) => {
                    if msg.content.contains("alias") {
                        let _ = msg.reply(&ctx.http, "Migrating aliases is unsupported, as the alias may not exist in OmiBot's DB").await;
                        return;
                    }
                    let Ok(contents) = read_attachment(attachment).await else {
                        let _ = msg.reply(&ctx.http, "Error reading attachment.").await;
                        return;
                    };
                    let payload = match attachment.content_type.as_deref() {
                        Some("text/plain; charset=utf-8") => {
                            TagPayload::Text(TextTagContent { content: contents })
                        }
                        Some("text/javascript; charset=utf-8") => {
                            TagPayload::Script(ScriptTagContent { script: contents })
                        }
                        _ => {
                            let _ = msg.reply(&ctx.http, "Unsupported file type.").await;
                            return;
                        }
                    };
                    let model = CreateTagModel::new(
                        migration_info.guild_id.db_id(),
                        migration_info.migrator_id.db_id(),
                        name.to_string(),
                        payload,
                        Some(false),
                    );
                    match create_tag(&state.db_pool, model).await {
                        Ok(_) => {
                            let _ = msg.reply(&ctx.http, "Tag created successfully.").await;
                            match msg
                                .channel_id
                                .send_message(
                                    &ctx.http,
                                    CreateMessage::new()
                                        .content(format!("%t owner {}", migration_info.tag_name)),
                                )
                                .await
                            {
                                Err(_) => {}
                                Ok(new_msg) => {
                                    let mut info = migration_info.clone();
                                    info.content = false;
                                    state.pending_migrations.insert(new_msg.id, info).await;
                                }
                            }

                            return;
                        }
                        Err(e) => {
                            let _ = msg
                                .reply(&ctx.http, format!("Error creating tag: {:?}", e))
                                .await;
                            return;
                        }
                    }
                }
            },
            false => match msg.content.split_whitespace().last() {
                None => {
                    let _ = msg.reply(&ctx.http, "Could not find owner!").await;
                    return;
                }
                Some(owner) => match get_uid_from_user_text(owner.trim_end_matches(".")) {
                    Err(_) => {
                        let _ = msg.reply(&ctx.http, "Could not parse owner ID!").await;
                        return;
                    }
                    Ok(uid) => {
                        match change_tag_owner(migration_info.guild_id, name, uid, &state.db_pool)
                            .await
                        {
                            Ok(_) => {
                                let _ = msg
                                    .reply(&ctx.http, "Changed tag ownership successfully.")
                                    .await;
                            }
                            Err(e) => {
                                let _ = msg
                                    .reply(
                                        &ctx.http,
                                        format!("Error changing tag ownership: {:?}", e),
                                    )
                                    .await;
                            }
                        }
                        if state
                            .pending_migrations
                            .iter()
                            .all(|(_, info)| info.guild_id != migration_info.guild_id)
                        {
                            let _ = msg
                                .channel_id
                                .send_message(
                                    &ctx.http,
                                    CreateMessage::new()
                                        .content(format!(
                                            "<@{}> Finished migrating tags!",
                                            migration_info.migrator_id
                                        ))
                                        .allowed_mentions(
                                            CreateAllowedMentions::new().all_users(true),
                                        ),
                                )
                                .await;
                        }
                    }
                },
            },
        },
    }
}
