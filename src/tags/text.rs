use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use crate::tags::{payload_mismatch_error, FetchTagModel};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextTagContent {
    pub content: String,
}

