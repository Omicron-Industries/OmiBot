use crate::cache::GuildCache;
use sqlx::{Pool, Postgres};

pub mod tags;
pub mod settings;
pub mod admins;
pub mod cache;
pub mod commands;


pub struct BotState {
    pub db_pool: Pool<Postgres>,
    pub guild_cache: GuildCache,
}

