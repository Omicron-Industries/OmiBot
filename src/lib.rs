use crate::cache::GuildCache;
use sqlx::{Pool, Postgres};

pub mod cache;
pub mod commands;
mod db;
pub mod settings;
pub mod tags;


pub struct BotState {
    pub db_pool: Pool<Postgres>,
    pub guild_cache: GuildCache,
}
