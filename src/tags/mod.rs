//

//

//
// pub fn tag_help(prefix: &str, args: Option<&str>) -> CreateMessage {
//     match args.map(str::to_lowercase) {
//         Some(s) => match s.as_str() {
//             "script" => tag_script_help(prefix),
//             "embed" => tag_embed_help(prefix),
//             _ => tag_help_msg(prefix),
//         },
//         _ => tag_help_msg(prefix),
//     }
// }
//
// fn tag_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Save content under a name, and recall it later.
// **General Info:**
//  - Tags are server-specific.
//  - Tags can contain text, embeds, JS scripts, or alias other tags.
//  - `{prefix}t` is an alias for `{prefix}tag`, and can be used in its place anywhere.
//
// **View:**
// `{prefix}tag <name> [args]` - Show a saved tag.
//
// **Manage:**
// `{prefix}tag add <name> <content>` - Create a new tag.
// `{prefix}tag edit <name>` - Change the content of a tag you own.
// `{prefix}tag delete <name>` - Delete a tag you own.
// `{prefix}tag alias <new_name> <existing_tag>` - Alias an existing tag.
// `{prefix}tag info <name>` - Show information about a tag.
// `{prefix}tag raw <name>` - Show the raw content of a tag.
//
// **Extra:**
// `{prefix}tag list [user]` - List tags owned by you (or user provided).
// `{prefix}tag search <name>` - Fuzzy search for a tag.
// `{prefix}tag help`
//
// **Admin:**
// `{prefix}tag chown <tag_name> <new_owner>` - Change the owner of a tag.
// `{prefix}tag ban <name>` - Ban a tag (prevents deletion of tag, to stop the deletion and recreation of it).
//
// Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{prefix}tag help script` or `{prefix}tag help embed`
//     "#))
// }
//
// fn tag_script_help(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Script tags run sandboxed JS, and can take in args from the user.
//
// To create a script tag, the content of the tag must be a multi-line JS code block:
// ```{prefix}tag add <name> `​`​`js
// <script_content>
// `​`​`
// ```
//
// The String `args` is made available at the start of the script, containing all message content after the tag name.
// For a full API reference, see [here](https://git.marinodev.com/drake/bunny_bot/api.md).
//     "#
//     ))
// }
//
// fn tag_add_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(r#"
// Creates a new tag.
// Usage: `{prefix}t add <name> <content>`
// Tags can store text, JS scripts, or embeds. Creating embed and JS script tags are more in-depth than simple text. For information about how these tags work, use `{prefix}tag help script` or `{prefix}tag help embed`
//     "#
//     ))
// }
//
// fn tag_alias_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
// Creates an alias for a tag.
// Usage: `{prefix}t alias <new_name> <existing_tag>`
//     "#
//     ))
// }
//
// fn tag_raw_help_msg(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
// Shows the raw content of a tag.
// Usage: `{prefix}t raw <tag_name>`
//     "#
//     ))
// }
//
// fn tag_embed_help(prefix: &str) -> CreateMessage {
//     CreateMessage::new().content(format!(
//         r#"
//         TODO
//         "#
//     ))
// }
