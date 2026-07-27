use crate::commands::help::command_help;
use crate::commands::{send_reply_ping_text, CommandContext, CommandInfo};
use sqlx::types::chrono;

pub const INFO: CommandInfo = CommandInfo {
    command: "ping",
    usage: None,
    full_desc: "Simply replies back to a ping, and shows the full time to response.",
    short_desc: Some("Ping!"),
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn execute(ctx: &CommandContext) {
    if ctx.help {
        command_help(ctx, INFO).await;
        return;
    }
    let time = chrono::Utc::now().timestamp_millis() - ctx.msg.timestamp.timestamp_millis();
    send_reply_ping_text(ctx, format!("Pong! Took {}ms", time).as_str()).await
}

// pub const INFO: CommandInfo = CommandInfo {
//     command: "",
//     usage: "",
//     full_desc: "",
//     short_desc: Some(""),
//     aliases: &[],
//     further_help: None,
//     subcommands: None,
// };
//
// pub async fn dispatch(ctx: &mut CommandContext) {
//     let command = ctx.consume_arg();
//     match command.as_deref() {
//         Some("") => {}
//         _ => {}
//     }
// }
//
// pub async fn execute(ctx: &CommandContext) {
//
// }
