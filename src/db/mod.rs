use serenity::all::{GuildId, UserId};

pub mod permissions;
pub mod settings;
pub mod tags;

pub trait DbId {
    fn db_id(&self) -> i64;
}

impl DbId for GuildId {
    fn db_id(&self) -> i64 {
        self.get() as i64
    }
}

impl DbId for UserId {
    fn db_id(&self) -> i64 {
        self.get() as i64
    }
}
