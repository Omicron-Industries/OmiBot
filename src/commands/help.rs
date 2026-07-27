use crate::commands::admin::{ban, settings};
use crate::commands::{
    eval, get_prefix, ping, send_reply_ping_text, tag, CommandContext, CommandInfo,
};

pub async fn command_help(ctx: &CommandContext, cmd_info: CommandInfo) {
    let prefix = get_prefix(ctx).await;

    let sub_list = generate_subcommand_list(&cmd_info, &prefix);
    let content = format!(
        "{}{}{}",
        cmd_info.full_desc,
        sub_list.map(|s| format!("\n\n{}", s)).unwrap_or_default(),
        cmd_info
            .further_help
            .map(|s| format!("\n\n\n{}", s).replace("{PREFIX}", &prefix))
            .unwrap_or_default(),
    );
    send_reply_ping_text(ctx, &content).await;
}

fn generate_subcommand_list(cmd_info: &CommandInfo, prefix: &str) -> Option<String> {
    if cmd_info.subcommands.is_none() {
        return None;
    }
    Some(
        cmd_info
            .subcommands
            .unwrap()
            .iter()
            .map(|cat| {
                let commands = cat
                    .commands
                    .iter()
                    .map(|sub| {
                        format!(
                            "`{prefix}{}{}` - {}",
                            sub.command,
                            sub.usage.map(|s| format!(" {}", s)).unwrap_or_default(),
                            sub.short_desc.unwrap_or(sub.full_desc)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                match cat.name {
                    Some(name) => format!(
                        "**{}:**\n{}{}",
                        name,
                        cat.description
                            .map(|d| format!("{}\n", d))
                            .unwrap_or_default(),
                        commands
                    ),
                    None => commands,
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    )
}

pub async fn command_usage(ctx: &mut CommandContext, cmd_info: CommandInfo) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"Improper command usage!
Usage: `{}{}`
See `{prefix}help {}` for more information.
    "#,
        cmd_info.command,
        cmd_info
            .usage
            .map(|s| format!(" {}", s))
            .unwrap_or_default(),
        cmd_info.command
    );
    send_reply_ping_text(ctx, &content).await;
}

pub const INFO: CommandInfo = CommandInfo {
    command: "help",
    usage: Some("[command]"),
    full_desc: "Prints information about a command.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

pub async fn dispatch(ctx: &mut CommandContext) {
    let orig_ctx = &ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("ping") => command_help(ctx, ping::INFO).await,
        Some("tag") | Some("t") => command_help(ctx, tag::INFO).await,
        Some("eval") => command_help(ctx, eval::INFO).await,
        Some("help") => command_help(ctx, INFO).await,
        Some("settings") => command_help(ctx, settings::INFO).await,
        Some("ban") => command_help(ctx, ban::INFO).await,
        _ => execute(orig_ctx).await,
    }
}

pub async fn execute(ctx: &CommandContext) {
    command_help(ctx, crate::commands::INFO).await;
    //     let prefix = get_prefix(ctx).await;
    //     let content = format!(
    //         r#"**Commands:**
    // `{prefix}ping`
    // `{prefix}tag` (`{prefix}t`): Store and recall tags
    // `{prefix}eval`: Run sandboxed JS
    // `{prefix}help`: Print this help
    // **Admin Commands:**
    // `{prefix}settings`: Edit server settings
    // `{prefix}ban`: Ban a user from creating tags
    //
    // `{prefix}<Any command> help` will give additional information about each command.
    // -# <angle_brackets> denote required arguments, while [square_brackets] denote optional arguments in all help commands.
    //     "#,
    //     );
    //     send_reply_ping_text(ctx, &content).await;
}
