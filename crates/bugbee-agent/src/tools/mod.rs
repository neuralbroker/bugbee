//! OpenCode-inspired tool suite specialized for AppSec.

mod dispatch;
mod registry;
mod truncate;

pub use dispatch::{parallel_enrich, ToolContext, ToolExecutor};
pub use registry::{tool_specs, ToolName};
pub use truncate::truncate_output;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub ok: bool,
    pub title: String,
    pub output: String,
    pub metadata: Value,
}

impl ToolResult {
    pub fn ok(title: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            ok: true,
            title: title.into(),
            output: output.into(),
            metadata: Value::Null,
        }
    }

    pub fn err(title: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            ok: false,
            title: title.into(),
            output: output.into(),
            metadata: Value::Null,
        }
    }

    pub fn with_meta(mut self, meta: Value) -> Self {
        self.metadata = meta;
        self
    }
}
