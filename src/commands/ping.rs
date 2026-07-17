use serenity::all::CreateMessage;

pub async fn ping(args: &str) -> CreateMessage {
    CreateMessage::new().content(format!("Pong! {} {}", if args.len() > 0 { "**Args:**" } else { "" }, args))
}

pub fn ping_help(prefix: &str) -> CreateMessage {
    CreateMessage::new().content(format!(r#"
        `{prefix}ping`
        Simply replies back to a ping, along with passing back the parsed arguments.
        "#
    ))
}
