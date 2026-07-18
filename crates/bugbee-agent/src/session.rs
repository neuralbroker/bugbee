use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::roles::{AgentRole, RoleKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub at: DateTime<Utc>,
    pub kind: String,
    pub detail: String,
}

/// In-memory agent session with event log (OpenCode session analogue).
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub role: AgentRole,
    pub events: Vec<SessionEvent>,
    pub todos: Vec<TodoItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub done: bool,
}

impl Session {
    pub fn new(role: AgentRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            events: Vec::new(),
            todos: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn switch_role(&mut self, kind: RoleKind) {
        self.role = AgentRole::builtin(kind);
        self.log("role_switch", kind.as_str());
    }

    pub fn log(&mut self, kind: impl Into<String>, detail: impl Into<String>) {
        self.events.push(SessionEvent {
            at: Utc::now(),
            kind: kind.into(),
            detail: detail.into(),
        });
    }

    pub fn set_todos(&mut self, items: Vec<TodoItem>) {
        self.todos = items;
        self.log("todo", format!("{} items", self.todos.len()));
    }
}
