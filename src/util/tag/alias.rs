use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasTagContent {
    pub target_id: i32,
}
