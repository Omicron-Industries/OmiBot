use std::sync::Arc;
use serenity::all::GuildId;
use crate::BotState;
use crate::settings::DEFAULT_PREFIX;

pub mod ping;
pub mod help;

pub async fn get_prefix(gid: Option<GuildId>, state: Arc<BotState>) -> String {
    if gid.is_some() { state.guild_cache.settings.get(&gid.unwrap()).await.unwrap().prefix.to_string() } else { DEFAULT_PREFIX.to_string() }
}