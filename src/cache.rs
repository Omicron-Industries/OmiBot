use moka::future::Cache;
use serenity::all::GuildId;
use crate::settings::GuildSettings;

pub struct GuildCache {
    pub settings: Cache<GuildId, GuildSettings>,
}

impl GuildCache {
    pub fn new() -> GuildCache {
        GuildCache {
            settings: Cache::new(u64::MAX),
        }
    }
}