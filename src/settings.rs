
pub const DEFAULT_PREFIX: char = '%';

#[derive(Clone)]
pub struct GuildSettings {
    pub prefix: char,
}

impl GuildSettings {
    pub fn new(prefix: char) -> Self {
        GuildSettings {
            prefix,
        }
    }
    
    pub fn default() -> Self {
        GuildSettings::new(DEFAULT_PREFIX)
    }
}


