use crate::commands::help::command_help;
use crate::commands::{CommandContext, CommandInfo};

pub const INFO: CommandInfo = CommandInfo {
    command: "",
    usage: Some(""),
    full_desc: "",
    short_desc: Some(""),
    aliases: &[],
    further_help: None,
    subcommands: None,
};

// pub async fn dispatch(ctx: &mut CommandContext) {
//     let command = ctx.consume_arg();
//     match command.as_deref() {
//         Some("") => {}
//         _ => {}
//     }
// }

pub async fn execute(ctx: &CommandContext) {
    if ctx.help {
        command_help(ctx, INFO).await;
        return;
    }
}
