use crate::commands::help::command_help;
use crate::commands::{CommandContext, CommandInfo};

pub const INFO: CommandInfo = CommandInfo {
    command: "tag add",
    usage: Some("<tag_name> <content>"),
    full_desc: "Create a new tag.",
    short_desc: None,
    aliases: &[],
    further_help: Some(
        "Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{PREFIX}help tag script` or `{PREFIX}help tag embed`",
    ),
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("help") => command_help(ctx, INFO).await,
        _ => execute(&mut orig_ctx).await,
    }
}

pub async fn execute(ctx: &CommandContext) {}
