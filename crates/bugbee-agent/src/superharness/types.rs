//! SuperHarness event model (Pi AgentEvent + OpenCode processor events).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperEventKind {
    AgentStart,
    AgentEnd,
    TurnStart,
    TurnEnd,
    Message,
    ToolStart,
    ToolEnd,
    ToolBlocked,
    Steering,
    FollowUp,
    Compaction,
    DoomLoop,
    Hook,
    Warn,
    Phase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperEvent {
    pub at: DateTime<Utc>,
    pub kind: SuperEventKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl SuperEvent {
    pub fn new(kind: SuperEventKind, message: impl Into<String>) -> Self {
        Self {
            at: Utc::now(),
            kind,
            message: message.into(),
            tool: None,
            meta: None,
        }
    }

    pub fn tool(kind: SuperEventKind, tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            at: Utc::now(),
            kind,
            message: message.into(),
            tool: Some(tool.into()),
            meta: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnOutcome {
    Continue,
    Stop,
    Compact,
}
