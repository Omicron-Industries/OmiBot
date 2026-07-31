Embeds allow you to send Discord embeds instead of plain text.

You can use embeds in the `eval` command, or as the content of a tag:

~~~
%t add ```json
<one_of_the_below_formats>
```
~~~

~~~
%eval ```json
<one_of_the_below_formats>
```
~~~

To create an embed tag, provide valid JSON as the tag content. The bot accepts three formats:

1. A direct embed object:

```json
{
  "title": "Welcome!",
  "description": "Enjoy your stay.",
  "color": 5763719
}
```

2. An object containing an `embed` field:

```json
{
  "embed": {
    "title": "Welcome!",
    "description": "Enjoy your stay.",
    "color": 5763719
  }
}
```

3. A Discord webhook payload. The first embed from the `embeds` array will be used:

```json
{
  "username": "Webhook",
  "embeds": [
    {
      "title": "Welcome!",
      "description": "Enjoy your stay."
    }
  ]
}
```

Example:

~~~
%tag add welcome ```json
{
  "title": "Welcome!",
  "description": "Please read the rules.",
  "color": 3447003,
  "fields": [
    {
      "name": "Rules",
      "value": "Be respectful."
    }
  ]
}
```
~~~

Supported embed properties include:
• title
• description
• color
• url
• author
• footer
• fields
• thumbnail
• image

The JSON must be a valid Discord embed. Invalid or unsupported fields will cause the tag to fail to create.

Further Reading

• Discord Embed Reference
https://docs.discord.com/developers/resources/message#embed-object

Lists every supported embed field, including authors, images, thumbnails,
timestamps, and field limits.

• Discohook
https://discohook.org/

An online embed editor. You can design an embed visually, then open the
JSON Data Editor and copy the generated JSON directly into a tag.

Since this command also accepts Discord webhook payloads, you can even paste
JSON exported from Discohook or other webhook tools without modifying it.