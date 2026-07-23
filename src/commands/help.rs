use std::sync::Arc;
use serenity::all::{CreateMessage, Message};
use crate::BotState;
use crate::commands::get_prefix;
use crate::commands::ping::ping_help;
use crate::tags::tag_help;

pub async fn help(args: &str, msg: &Message, state: Arc<BotState>) -> CreateMessage {
    let prefix = get_prefix(msg.guild_id, state.clone()).await;

    match args.split_once(char::is_whitespace) {
        Some(("tag", new_args)) => tag_help(get_prefix(msg.guild_id, state.clone()).await.as_str(), Some(new_args)),
        Some(("ping", _)) => ping_help(prefix.as_str()),

        _ => help_msg(&prefix, msg),
    }
}

pub fn help_msg(prefix: &str, msg: &Message) -> CreateMessage {
    CreateMessage::new().content(format!(r#"{}
**Commands:**
`{prefix}ping`
`{prefix}tag` (`{prefix}t`): Store and recall tags
`{prefix}eval`: Run sandboxed JS
`{prefix}help`: Print this help
**Admin Commands:**
`{prefix}settings`: Edit server settings
`{prefix}ban`: Ban a user from creating tags

`{prefix}[Any command] help` will give additional information about each command.
    "#, if msg.guild_id.is_some() { format!("Server prefix: {}", prefix) } else { "".to_string() }
    ))
}


