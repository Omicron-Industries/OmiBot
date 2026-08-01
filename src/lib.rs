use crate::cache::GuildCache;
use moka::future::Cache;
use serenity::all::{GuildId, MessageId, UserId};
use sqlx::{Pool, Postgres};

pub mod cache;
pub mod commands;
pub mod db;
pub mod settings;
pub mod util;

pub struct BotState {
    pub db_pool: Pool<Postgres>,
    pub guild_cache: GuildCache,
    pub pending_migrations: Cache<MessageId, MigrationInfo>,
}

#[derive(Clone, Debug)]
pub struct MigrationInfo {
    pub tag_name: String,
    pub guild_id: GuildId,
    pub migrator_id: UserId,
    pub content: bool,
}
