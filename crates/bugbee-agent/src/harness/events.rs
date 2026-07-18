use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessEventKind {
    Phase,
    Step,
    ToolStart,
    ToolEnd,
    Model,
    Finding,
    Warn,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessEvent {
    pub at: DateTime<Utc>,
    pub kind: HarnessEventKind,
    pub message: String,
}

impl HarnessEvent {
    pub fn new(kind: HarnessEventKind, message: impl Into<String>) -> Self {
        Self {
            at: Utc::now(),
            kind,
            message: message.into(),
        }
    }
}
