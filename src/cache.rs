use crate::settings::GuildSettings;
use moka::future::Cache;
use serenity::all::GuildId;

pub struct GuildCache {
    pub settings: Cache<GuildId, GuildSettings>,
    pub detectable_tags: Cache<GuildId, Vec<String>>,
    pub enabled: Cache<GuildId, bool>,
}

impl GuildCache {
    pub fn new() -> GuildCache {
        GuildCache {
            settings: Cache::new(u64::MAX),
            detectable_tags: Cache::new(u64::MAX),
            enabled: Cache::new(u64::MAX),
        }
    }
}
