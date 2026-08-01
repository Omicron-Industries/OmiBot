use crate::commands;
use crate::commands::{get_prefix, send_reply_ping_text, CommandContext, CommandInfo};

pub async fn command_help(ctx: &CommandContext, cmd_info: &CommandInfo) {
    let prefix = get_prefix(ctx).await;

    let sub_list = generate_subcommand_list(&cmd_info, &prefix);
    let content = format!(
        "{}{}{}{}{}",
        cmd_info.full_desc,
        cmd_info
            .usage
            .map(|s| format!("\nUsage: `{}{} {}`", prefix, cmd_info.command, s))
            .unwrap_or_default(),
        if !cmd_info.aliases.is_empty() {
            format!(
                "\nAliases: {}",
                cmd_info
                    .aliases
                    .iter()
                    .map(|a| format!("`{}`", a))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        },
        sub_list.map(|s| format!("\n\n{}", s)).unwrap_or_default(),
        cmd_info
            .further_help
            .map(|s| format!("\n\n{}", s).replace("{PREFIX}", &prefix))
            .unwrap_or_default(),
    );
    send_reply_ping_text(ctx, &content).await;
}

fn generate_subcommand_list(cmd_info: &CommandInfo, prefix: &str) -> Option<String> {
    let subcommands = cmd_info.subcommands?;

    let has_categories = subcommands.iter().any(|cat| cat.name.is_some());

    let commands = subcommands
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
        .join("\n\n");

    if has_categories {
        Some(commands)
    } else {
        Some(format!("**Subcommands:**\n{}", commands))
    }
}

pub async fn command_usage(ctx: &CommandContext, cmd_info: &CommandInfo) {
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

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "help",
    usage: Some("[command]"),
    full_desc: "Prints information about a command.",
    short_desc: None,
    aliases: &[],
    further_help: None,
    subcommands: None,
};

// pub async fn dispatch(ctx: &mut CommandContext) {}

pub async fn execute(ctx: &CommandContext) {
    command_help(ctx, commands::INFO).await;
}
