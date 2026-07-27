use serde::{Deserialize, Serialize};
use serenity::all::Embed;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedTagContent {
    pub embed: Embed,
}
