//! Ralph outer loop — Claude Code ralph-wiggum pattern.
//!
//! Re-inject the same goal until a completion promise is met or max iterations.

use std::sync::Arc;

use bugbee_core::{Redactor, Result};
use bugbee_llm::LlmClient;
use parking_lot::Mutex;

use crate::roles::AgentRole;
use crate::session::Session;
use crate::superharness::loop_::{SuperHarness, SuperHarnessConfig, SuperRunResult};
use crate::superharness::types::{SuperEvent, SuperEventKind};
use crate::tools::ToolExecutor;

#[derive(Debug, Clone)]
pub struct RalphConfig {
    pub max_iterations: u32,
    /// If the model outputs this exact substring, the loop exits successfully.
    pub completion_promise: Option<String>,
    /// Stop when no tool calls for a full iteration (idle).
    pub stop_on_idle: bool,
}

impl Default for RalphConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            completion_promise: Some("BUGBEE_COMPLETE".into()),
            stop_on_idle: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RalphStatus {
    Completed,
    MaxIterations,
    Idle,
    Failed,
}

pub struct RalphRunner {
    pub inner: SuperHarness,
    pub ralph: RalphConfig,
}

impl RalphRunner {
    pub fn new(
        client: Arc<dyn LlmClient>,
        tools: ToolExecutor,
        role: AgentRole,
        redactor: Redactor,
        session: Arc<Mutex<Session>>,
        harness: SuperHarnessConfig,
        ralph: RalphConfig,
    ) -> Self {
        Self {
            inner: SuperHarness {
                client,
                tools,
                role,
                redactor,
                config: harness,
                session,
                steering: Arc::new(Mutex::new(std::collections::VecDeque::new())),
                follow_ups: Arc::new(Mutex::new(std::collections::VecDeque::new())),
            },
            ralph,
        }
    }

    pub async fn run(
        &self,
        user_goal: &str,
        extra_context: &str,
    ) -> Result<(SuperRunResult, RalphStatus)> {
        let promise = self
            .ralph
            .completion_promise
            .clone()
            .unwrap_or_else(|| "BUGBEE_COMPLETE".into());

        let goal = format!(
            "{user_goal}\n\n\
             ## Ralph loop\n\
             Iterate until the work is fully done. When and only when the goal is \
             completely achieved, include the exact token `{promise}` in your final message. \
             Do not emit a false promise."
        );

        let mut all_events = Vec::new();
        let mut last = SuperRunResult {
            final_text: String::new(),
            steps: 0,
            tool_calls: 0,
            events: Vec::new(),
            stopped_reason: "not started".into(),
            compacted: false,
        };
        let mut status = RalphStatus::MaxIterations;

        for i in 0..self.ralph.max_iterations {
            all_events.push(SuperEvent::new(
                SuperEventKind::Phase,
                format!("ralph iteration {}/{}", i + 1, self.ralph.max_iterations),
            ));

            let ctx = if i == 0 {
                extra_context.to_string()
            } else {
                format!(
                    "{extra_context}\n\n## Previous iteration summary\n{}\n\n\
                     Continue improving. Do not restart from zero. Emit `{promise}` only if fully done.",
                    last.final_text.chars().take(2000).collect::<String>()
                )
            };

            last = self.inner.run(&goal, &ctx).await?;
            all_events.extend(last.events.clone());

            if last.final_text.contains(&promise) {
                status = RalphStatus::Completed;
                all_events.push(SuperEvent::new(
                    SuperEventKind::AgentEnd,
                    format!("ralph completion promise met at iter {}", i + 1),
                ));
                break;
            }

            if self.ralph.stop_on_idle && last.tool_calls == 0 && i > 0 {
                status = RalphStatus::Idle;
                all_events.push(SuperEvent::new(
                    SuperEventKind::Warn,
                    "ralph idle (no tool calls) — stopping",
                ));
                break;
            }

            if last.stopped_reason.contains("llm error") {
                status = RalphStatus::Failed;
                break;
            }
        }

        last.events = all_events;
        last.stopped_reason = format!("ralph:{status:?} · {}", last.stopped_reason);
        Ok((last, status))
    }
}
