use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Local MessageRole wrapper to enable comparison
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    System,
}

/// Message content type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    WithThinking { thinking: String, output: String },
}

/// UI Application state
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub datetime: DateTime<Local>,
}
