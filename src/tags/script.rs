use serde::{Deserialize, Serialize};
use rquickjs::{Context, Runtime};
use serenity::all::{ChannelId, GuildId, UserId};

pub struct ScriptContext {
    pub args: Option<String>,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
}


pub struct ScriptEngine {
    runtime: Runtime,
}

impl ScriptEngine {
    pub fn new() -> rquickjs::Result<Self> {
        Ok(Self {
            runtime: Runtime::new()?,
        })
    }

    pub fn execute(
        &self,
        script: &str,
        script_ctx: ScriptContext,
    ) -> rquickjs::Result<String> {
        let ctx = Context::full(&self.runtime)?;

        ctx.with(|ctx| {
            // let globals = ctx.globals();

            // let js_ctx = rquickjs::serde::to_value(
            //     ctx,
            //     script_ctx,
            // )?;


            // let result: String = ctx.eval(script)?;

            let result = match ctx.eval::<String, _>(script) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Rust error: {:?}", e);

                    if let e = ctx.catch() {
                        eprintln!("JS exception: {:?}", e);
                    }

                    return Err(e.into());
                }
            };

            Ok(result)
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptTagContent {
    pub script: String,
}

