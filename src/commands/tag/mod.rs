use crate::commands::help::command_help;
use crate::commands::{CommandContext, CommandInfo};

mod add;
mod alias;
mod delete;
mod edit;
mod help;
mod info;
mod list;
mod raw;
mod search;

pub const INFO: CommandInfo = CommandInfo {
    command: "",
    usage: Some(""),
    full_desc: "",
    short_desc: Some(""),
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") => {}
        Some("edit") => {}
        Some("delete") => {}
        Some("alias") => {}
        Some("info") => {}
        Some("raw") => {}
        Some("list") => {}
        Some("search") => {}
        Some("help") => command_help(ctx, INFO).await,
        Some("chown") => {}
        Some("ban") => {}
        Some(args) => {}
        None => {}
    }
}

pub async fn execute(ctx: &CommandContext) {
    if ctx.help {
        command_help(ctx, INFO).await;
        return;
    }
}
