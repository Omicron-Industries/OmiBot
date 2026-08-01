pub const DEFAULT_PREFIX: &str = "%";

#[derive(Clone)]
pub struct GuildSettings {
    pub prefix: String,
}

impl GuildSettings {
    pub fn new(prefix: &str) -> Self {
        GuildSettings {
            prefix: prefix.to_string(),
        }
    }

    pub fn default() -> Self {
        GuildSettings::new(DEFAULT_PREFIX)
    }
}
