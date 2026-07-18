//! Hook bus — Claude Code hookify + Pi before/after tool semantics.

use serde_json::Value;
use std::sync::Arc;

use crate::superharness::types::{SuperEvent, SuperEventKind};

/// Action returned by a hook handler.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Continue normally.
    Continue,
    /// Block tool execution (Pi `beforeToolCall` block).
    Block { reason: String },
    /// Replace tool result content after execution (Pi `afterToolCall`).
    OverrideResult { content: String, is_error: bool },
    /// Request early terminate after this tool batch (Pi `terminate`).
    Terminate,
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub tool_name: String,
    pub tool_call_id: String,
    pub args: Value,
    pub result_preview: Option<String>,
    pub is_error: bool,
    pub turn: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    TurnStart,
    TurnEnd,
    AgentStart,
    AgentEnd,
}

pub type HookHandler = Arc<dyn Fn(HookEvent, &HookContext) -> HookAction + Send + Sync>;

/// Multiplexed hooks (Claude Code style named rules + Pi semantics).
#[derive(Default, Clone)]
pub struct HookBus {
    handlers: Vec<(String, HookHandler)>,
}

impl HookBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on(&mut self, name: impl Into<String>, handler: HookHandler) {
        self.handlers.push((name.into(), handler));
    }

    /// Built-in defense hooks for Bugbee.
    pub fn with_security_defaults(mut self) -> Self {
        self.on(
            "block-live-exploit",
            Arc::new(|ev, ctx| {
                if ev != HookEvent::PreToolUse {
                    return HookAction::Continue;
                }
                let s = format!("{} {}", ctx.tool_name, ctx.args).to_ascii_lowercase();
                for bad in [
                    "metasploit",
                    "msfvenom",
                    "reverse_shell",
                    "exploit-db",
                    "c2 callback",
                ] {
                    if s.contains(bad) {
                        return HookAction::Block {
                            reason: format!("defense-only: blocked pattern `{bad}`"),
                        };
                    }
                }
                HookAction::Continue
            }),
        );
        self.on(
            "block-rm-rf-root",
            Arc::new(|ev, ctx| {
                if ev != HookEvent::PreToolUse || ctx.tool_name != "shell" {
                    return HookAction::Continue;
                }
                let cmd = ctx
                    .args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if cmd.contains("rm -rf /") || cmd.contains("rm -rf ~") {
                    return HookAction::Block {
                        reason: "catastrophic rm blocked (Claude Code policy)".into(),
                    };
                }
                HookAction::Continue
            }),
        );
        self
    }

    pub fn emit_pre(&self, ctx: &HookContext, events: &mut Vec<SuperEvent>) -> HookAction {
        for (name, h) in &self.handlers {
            match h(HookEvent::PreToolUse, ctx) {
                HookAction::Block { reason } => {
                    events.push(SuperEvent::tool(
                        SuperEventKind::Hook,
                        &ctx.tool_name,
                        format!("hook `{name}` blocked: {reason}"),
                    ));
                    return HookAction::Block { reason };
                }
                HookAction::Terminate => {
                    events.push(SuperEvent::new(
                        SuperEventKind::Hook,
                        format!("hook `{name}` requested terminate"),
                    ));
                    return HookAction::Terminate;
                }
                other => {
                    if !matches!(other, HookAction::Continue) {
                        return other;
                    }
                }
            }
        }
        HookAction::Continue
    }

    pub fn emit_post(&self, ctx: &HookContext, events: &mut Vec<SuperEvent>) -> HookAction {
        let mut action = HookAction::Continue;
        for (name, h) in &self.handlers {
            match h(HookEvent::PostToolUse, ctx) {
                HookAction::OverrideResult { content, is_error } => {
                    events.push(SuperEvent::new(
                        SuperEventKind::Hook,
                        format!("hook `{name}` overrode result"),
                    ));
                    action = HookAction::OverrideResult { content, is_error };
                }
                HookAction::Terminate => {
                    events.push(SuperEvent::new(
                        SuperEventKind::Hook,
                        format!("hook `{name}` terminate after tool"),
                    ));
                    return HookAction::Terminate;
                }
                HookAction::Block { reason } => {
                    // post-block becomes error override
                    action = HookAction::OverrideResult {
                        content: reason,
                        is_error: true,
                    };
                }
                HookAction::Continue => {}
            }
        }
        action
    }
}
