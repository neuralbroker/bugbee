//! AgentRunner — thin adapter over SuperHarness (backward compatible).

use std::sync::Arc;

use bugbee_core::{Error, Redactor, Result};
use bugbee_llm::LlmClient;
use parking_lot::Mutex;

use crate::harness::budget::RunLimits;
use crate::harness::events::{HarnessEvent, HarnessEventKind};
use crate::roles::AgentRole;
use crate::session::Session;
use crate::superharness::{
    SuperEventKind, SuperHarness, SuperHarnessConfig, SuperRunResult, ToolExecMode,
};
use crate::tools::ToolExecutor;

pub struct RunnerConfig {
    pub limits: RunLimits,
    pub include_shell: bool,
    pub include_review: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            limits: RunLimits::default(),
            include_shell: false,
            include_review: true,
        }
    }
}

#[derive(Debug)]
pub struct AgentRunResult {
    pub final_text: String,
    pub steps: u32,
    pub tool_calls: u32,
    pub events: Vec<HarnessEvent>,
    pub stopped_reason: String,
}

pub struct AgentRunner {
    pub client: Arc<dyn LlmClient>,
    pub tools: ToolExecutor,
    pub role: AgentRole,
    pub redactor: Redactor,
    pub config: RunnerConfig,
    pub session: Arc<Mutex<Session>>,
}

impl AgentRunner {
    pub async fn run(&self, user_goal: &str, extra_context: &str) -> Result<AgentRunResult> {
        let sh = SuperHarness {
            client: Arc::clone(&self.client),
            tools: ToolExecutor::new(crate::tools::ToolContext {
                root: self.tools.ctx.root.clone(),
                gate: self.tools.ctx.gate.clone(),
                store: Arc::clone(&self.tools.ctx.store),
                session: Arc::clone(&self.session),
                hunt_config: self.tools.ctx.hunt_config.clone(),
            }),
            role: self.role.clone(),
            redactor: self.redactor.clone(),
            config: SuperHarnessConfig {
                limits: self.config.limits.clone(),
                include_shell: self.config.include_shell,
                include_review: self.config.include_review,
                tool_mode: ToolExecMode::ParallelReads,
                ..SuperHarnessConfig::default()
            },
            session: Arc::clone(&self.session),
            steering: Arc::new(Mutex::new(std::collections::VecDeque::new())),
            follow_ups: Arc::new(Mutex::new(std::collections::VecDeque::new())),
        };

        let SuperRunResult {
            final_text,
            steps,
            tool_calls,
            events,
            stopped_reason,
            ..
        } = sh.run(user_goal, extra_context).await?;

        Ok(AgentRunResult {
            final_text,
            steps,
            tool_calls,
            events: events.into_iter().map(map_event).collect(),
            stopped_reason,
        })
    }
}

fn map_event(e: crate::superharness::SuperEvent) -> HarnessEvent {
    let kind = match e.kind {
        SuperEventKind::Phase | SuperEventKind::AgentStart => HarnessEventKind::Phase,
        SuperEventKind::TurnStart | SuperEventKind::TurnEnd | SuperEventKind::Message => {
            HarnessEventKind::Step
        }
        SuperEventKind::ToolStart => HarnessEventKind::ToolStart,
        SuperEventKind::ToolEnd | SuperEventKind::ToolBlocked => HarnessEventKind::ToolEnd,
        SuperEventKind::Warn | SuperEventKind::DoomLoop | SuperEventKind::Hook => {
            HarnessEventKind::Warn
        }
        SuperEventKind::AgentEnd | SuperEventKind::Compaction => HarnessEventKind::Done,
        SuperEventKind::Steering | SuperEventKind::FollowUp => HarnessEventKind::Model,
    };
    HarnessEvent::new(kind, e.message)
}

/// Offline agent: no LLM — runs hunt + list (enrich happens in pipeline phase1).
pub fn offline_agent_pass(
    tools: &ToolExecutor,
    events: &mut Vec<HarnessEvent>,
    run_hunt: bool,
) -> Result<String> {
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "offline godmode (no LLM) — digest",
    ));
    if run_hunt {
        let hunt = tools.execute("hunt", &serde_json::json!({}));
        events.push(HarnessEvent::new(
            HarnessEventKind::ToolEnd,
            hunt.output.chars().take(200).collect::<String>(),
        ));
    }

    let list = tools.execute("list_findings", &serde_json::json!({ "limit": 50 }));
    let findings = tools.ctx.store.lock().list(None)?;
    let hot = findings
        .iter()
        .filter(|f| {
            matches!(
                f.severity,
                bugbee_core::Severity::Critical | bugbee_core::Severity::High
            )
        })
        .count();

    Ok(format!(
        "Offline godmode complete (deterministic engines + enrich).\n\
         Findings ({} total, {} crit/high):\n{}\n\
         Human review required — no auto-confirm without LLM dual-gate.\n\
         Tip: `bugbee connect` + `bugbee super` / `bugbee godmode` for multi-agent pass.",
        findings.len(),
        hot,
        list.output
    ))
}

// keep Error import used if needed
#[allow(dead_code)]
fn _e() -> Error {
    Error::Other("x".into())
}
