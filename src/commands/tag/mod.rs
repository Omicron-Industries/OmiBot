use crate::commands::help::{command_help, command_usage};
use crate::commands::{
    get_prefix, send_reply_ping_text, CommandCategory, CommandContext, CommandInfo,
};
use crate::util::tag::execute::execute_tag;
mod add;
mod alias;
mod ban;
mod bans;
mod chown;
mod delete;
mod detect;
mod edit;
mod info;
mod list;
pub mod migrate;
mod raw;
mod rename;
mod search;
mod unban;

const SUBCOMMANDS: &'static [&'static CommandCategory] = &[
    &CommandCategory {
        name: Some("Commands"),
        description: None,
        commands: &[
            &add::INFO,
            &edit::INFO,
            &delete::INFO,
            &rename::INFO,
            &alias::INFO,
            &info::INFO,
            &raw::INFO,
            &list::INFO,
            &search::INFO,
            &chown::INFO,
        ],
    },
    &CommandCategory {
        name: Some("Admin"),
        description: None,
        commands: &[&ban::INFO, &unban::INFO, &bans::INFO, &detect::INFO],
    },
];

pub const INFO: &'static CommandInfo = &CommandInfo {
    command: "tag",
    usage: Some("(<tag_name> | <subcommand>)"),
    full_desc: "Execute a tag or manage server tags.",
    short_desc: Some("Manage and execute tags."),
    aliases: &["t"],
    further_help: Some(
        "Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{PREFIX}help tag script` or `{PREFIX}help tag embed`",
    ),
    subcommands: Some(SUBCOMMANDS),
};

/// All names for the tag command reserved for subcommands, prohibited from being made into a tag.
pub const TAG_SUBCOMMANDS: &[&str] = &[
    "add",
    "create",
    "new",
    "edit",
    "delete",
    "del",
    "rm",
    "alias",
    "info",
    "owner",
    "raw",
    "list",
    "search",
    "chown",
    "transfer",
    "ban",
    "unban",
    "bans",
    "help",
    "script",
    "embed",
    "text",
    "detect",
    "detectable",
    "migrate",
];

pub async fn dispatch(ctx: &mut CommandContext) {
    let mut orig_ctx = ctx.clone();
    let command = ctx.consume_arg();
    match command.as_deref() {
        Some("add") | Some("create") | Some("new") => add::dispatch(ctx).await,
        Some("edit") => edit::dispatch(ctx).await,
        Some("rename") => rename::dispatch(ctx).await,
        Some("delete") | Some("del") | Some("rm") => delete::dispatch(ctx).await,
        Some("alias") => alias::dispatch(ctx).await,
        Some("info") | Some("owner") => info::dispatch(ctx).await,
        Some("raw") => raw::dispatch(ctx).await,
        Some("list") => list::dispatch(ctx).await,
        Some("search") => search::dispatch(ctx).await,
        Some("chown") | Some("transfer") => chown::dispatch(ctx).await,
        Some("detect") | Some("detectable") => detect::dispatch(ctx).await,
        Some("ban") => ban::dispatch(ctx).await,
        Some("unban") => unban::dispatch(ctx).await,
        Some("bans") => bans::dispatch(ctx).await,
        Some("migrate") => migrate::dispatch(ctx).await,
        Some("help") => {
            let next = ctx.consume_arg();
            match next.as_deref() {
                Some("script") => help_script(ctx).await,
                Some("embed") => help_embed(ctx).await,
                _ => command_help(ctx, INFO).await,
            }
        }
        Some("script") if ctx.help => help_script(ctx).await,
        Some("embed") if ctx.help => help_embed(ctx).await,

        _ if ctx.help => command_help(ctx, INFO).await,
        Some(_) => execute(&mut orig_ctx).await,
        None => command_usage(ctx, INFO).await,
    }
}

pub async fn execute(ctx: &mut CommandContext) {
    let Some(tag_name) = ctx.consume_arg() else {
        return command_usage(ctx, INFO).await;
    };

    execute_tag(ctx, &tag_name).await;
}

async fn help_script(ctx: &CommandContext) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"Script tags run sandboxed JS, and can take in args from the user.

To create a script tag, the content of the tag must be a multi-line JS code block:
```{prefix}tag add <name> `​`​`js
<script_content>
`​`​`
```

You can also upload a `.js` file.

The String `args` is made available at the start of the script, containing all message content after the tag name.
For a full API reference, see [here](https://github.com/Omicron-Industries/OmiBot/docs/api.md)."#
    );
    send_reply_ping_text(ctx, &content).await;
}

async fn help_embed(ctx: &CommandContext) {
    let prefix = get_prefix(ctx).await;
    let content = format!(
        r#"Embed tags accept several JSON formats to define embeds. Webhook embed tools like [Discohook](https://discohook.org) can be used to create the JSON.

To create an embed tag, the content of the tag must be a multi-line JSON code block:
```{prefix}tag add <name> `​`​`json
<embed_content>
`​`​`
```

See the full embed documentation [here](https://github.com/Omicron-Industries/OmiBot/docs/embed.md)."#
    );
    send_reply_ping_text(ctx, &content).await;
}

pub fn tag_name_validator(name: &str) -> Option<String> {
    if TAG_SUBCOMMANDS.contains(&name) {
        Some(format!(
            "Tag name **{name}** is disallowed, as it is a subcommand!"
        ))
    } else if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Some(
            "Tag name must contain only letters, numbers, underscores (_), and hyphens (-)."
                .to_string(),
        )
    } else if name.len() > 32 {
        Some("Tag name must not exceed 32 characters.".to_string())
    } else {
        None
    }
}
