pub mod bans;
pub mod create;
pub mod edit;
pub mod fetch;
pub mod search;
pub mod detect;

use crate::db::DbId;
use sqlx::error::DatabaseError;
