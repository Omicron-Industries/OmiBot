use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasTagContent {
    pub target_id: i32,
}
