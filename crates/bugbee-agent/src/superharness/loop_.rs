//! SuperHarness agent loop — Pi runLoop + OpenCode processor + Claude hooks.

use std::collections::VecDeque;
use std::sync::Arc;

use bugbee_core::{Redactor, Result};
use bugbee_llm::{ChatMessage, ChatRequest, LlmClient, ToolCall};
use parking_lot::Mutex;

use crate::harness::{Budget, RunLimits};
use crate::roles::AgentRole;
use crate::session::Session;
use crate::superharness::compaction::{compact_messages, CompactionConfig};
use crate::superharness::hooks::HookBus;
use crate::superharness::parallel::{execute_tool_batch, ToolExecMode};
use crate::superharness::types::{SuperEvent, SuperEventKind};
use crate::tools::{tool_specs, ToolExecutor};

#[derive(Clone)]
pub struct SuperHarnessConfig {
    pub limits: RunLimits,
    pub include_shell: bool,
    pub include_review: bool,
    pub tool_mode: ToolExecMode,
    pub compaction: CompactionConfig,
    pub hooks: HookBus,
    /// Pi-style: when true, drain steering between turns.
    pub enable_steering: bool,
    /// Fail tool calls if finish_reason is length (Pi truncated-message safety).
    pub fail_tools_on_length: bool,
}

impl Default for SuperHarnessConfig {
    fn default() -> Self {
        Self {
            limits: RunLimits::default(),
            include_shell: false,
            include_review: true,
            tool_mode: ToolExecMode::ParallelReads,
            compaction: CompactionConfig::default(),
            hooks: HookBus::new().with_security_defaults(),
            enable_steering: true,
            fail_tools_on_length: true,
        }
    }
}

impl SuperHarnessConfig {
    pub fn aggressive() -> Self {
        Self {
            limits: RunLimits::aggressive(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct SuperRunResult {
    pub final_text: String,
    pub steps: u32,
    pub tool_calls: u32,
    pub events: Vec<SuperEvent>,
    pub stopped_reason: String,
    pub compacted: bool,
}

/// SuperHarness: production agent loop for Bugbee.
pub struct SuperHarness {
    pub client: Arc<dyn LlmClient>,
    pub tools: ToolExecutor,
    pub role: AgentRole,
    pub redactor: Redactor,
    pub config: SuperHarnessConfig,
    pub session: Arc<Mutex<Session>>,
    /// Pi steering queue — inject mid-run.
    pub steering: Arc<Mutex<VecDeque<String>>>,
    /// Pi follow-up queue — inject when agent would stop.
    pub follow_ups: Arc<Mutex<VecDeque<String>>>,
}

impl SuperHarness {
    pub fn push_steering(&self, msg: impl Into<String>) {
        self.steering.lock().push_back(msg.into());
    }

    pub fn push_follow_up(&self, msg: impl Into<String>) {
        self.follow_ups.lock().push_back(msg.into());
    }

    pub async fn run(&self, user_goal: &str, extra_context: &str) -> Result<SuperRunResult> {
        let mut budget = Budget::new(self.config.limits.clone());
        if self.role.max_steps < budget.limits.max_steps {
            budget.limits.max_steps = self.role.max_steps;
        }

        let mut events = Vec::new();
        events.push(SuperEvent::new(
            SuperEventKind::AgentStart,
            format!("superharness role={}", self.role.kind.as_str()),
        ));

        let tool_specs = tool_specs(self.config.include_shell, self.config.include_review);
        let system = self.redactor.redact(&format!(
            "{}\n\n## SuperHarness\n\
             You have tools. Prefer parallel-safe reads (read/grep/glob). \
             Defense only. Never attack unauthorized systems.",
            self.role.system_prompt
        ));
        let user = self
            .redactor
            .redact(&format!("{extra_context}\n\n## Goal\n{user_goal}"));

        let mut messages = vec![ChatMessage::system(system), ChatMessage::user(user)];
        let mut final_text = String::new();
        let mut stopped_reason = "completed".to_string();
        let mut compacted = false;
        let mut first_turn = true;

        // Outer loop: follow-ups (Pi)
        'outer: loop {
            let mut has_more_tools = true;

            // Inner loop: tool calls + steering
            while has_more_tools || !self.steering.lock().is_empty() {
                if let Err(e) = budget.tick_step() {
                    stopped_reason = e;
                    break 'outer;
                }

                if !first_turn {
                    events.push(SuperEvent::new(
                        SuperEventKind::TurnStart,
                        format!("turn {}", budget.steps),
                    ));
                } else {
                    first_turn = false;
                    events.push(SuperEvent::new(
                        SuperEventKind::TurnStart,
                        format!("turn {}", budget.steps),
                    ));
                }

                // Drain steering (Pi getSteeringMessages)
                if self.config.enable_steering {
                    let mut q = self.steering.lock();
                    while let Some(s) = q.pop_front() {
                        events.push(SuperEvent::new(SuperEventKind::Steering, &s));
                        messages.push(ChatMessage::user(format!("[steering] {s}")));
                    }
                }

                // Compaction transform (Pi transformContext + OpenCode prune)
                let (compacted_msgs, did) = compact_messages(&messages, &self.config.compaction);
                if did {
                    compacted = true;
                    messages = compacted_msgs;
                    events.push(SuperEvent::new(
                        SuperEventKind::Compaction,
                        format!("context compacted ({} msgs)", messages.len()),
                    ));
                }

                let req = ChatRequest {
                    temperature: Some(0.15),
                    max_tokens: Some(4096),
                    ..ChatRequest::new(messages.clone()).with_tools(tool_specs.clone())
                };

                let resp = match self.client.chat(req).await {
                    Ok(r) => r,
                    Err(e) => {
                        stopped_reason = format!("llm error: {e}");
                        events.push(SuperEvent::new(
                            SuperEventKind::Warn,
                            stopped_reason.clone(),
                        ));
                        break 'outer;
                    }
                };

                // Pi: fail tools if output truncated
                let truncated = resp.finish_reason.as_deref().is_some_and(|r| r == "length");

                if resp.has_tool_calls() {
                    let sig = resp
                        .tool_calls
                        .iter()
                        .map(|t| {
                            format!("{}:{}", t.function.name, short_hash(&t.function.arguments))
                        })
                        .collect::<Vec<_>>()
                        .join("|");

                    if budget.note_signature(sig) {
                        // OpenCode doom loop
                        stopped_reason =
                            "doom loop detected (repeated identical tool calls)".into();
                        events.push(SuperEvent::new(
                            SuperEventKind::DoomLoop,
                            stopped_reason.clone(),
                        ));
                        messages.push(ChatMessage::assistant_tools(
                            Some(resp.content.clone()),
                            resp.tool_calls.clone(),
                        ));
                        messages.push(ChatMessage::user(
                            "STOP: you are repeating the same tool calls. \
                             Provide your final summary now with no tools.",
                        ));
                        let fin = ChatRequest::new(messages.clone());
                        if let Ok(r) = self.client.chat(fin).await {
                            final_text = r.content;
                        }
                        break 'outer;
                    }

                    if let Err(e) = budget.tick_tools(resp.tool_calls.len() as u32) {
                        stopped_reason = e;
                        break 'outer;
                    }

                    messages.push(ChatMessage::assistant_tools(
                        if resp.content.is_empty() {
                            None
                        } else {
                            Some(resp.content.clone())
                        },
                        resp.tool_calls.clone(),
                    ));

                    let executed = if truncated && self.config.fail_tools_on_length {
                        events.push(SuperEvent::new(
                            SuperEventKind::Warn,
                            "response hit length limit — failing tool batch (Pi safety)",
                        ));
                        fail_truncated_tools(&resp.tool_calls, &mut events)
                    } else {
                        execute_tool_batch(
                            &self.tools,
                            &resp.tool_calls,
                            &self.config.hooks,
                            self.config.tool_mode,
                            budget.steps,
                            &mut events,
                        )
                    };

                    let mut terminate_batch = false;
                    for ex in &executed {
                        let content = self.redactor.redact(&format!(
                            "ok={} title={}\n{}",
                            ex.result.ok, ex.result.title, ex.result.output
                        ));
                        messages.push(ChatMessage::tool_result(
                            &ex.call.id,
                            &ex.call.function.name,
                            content,
                        ));
                        if ex.terminate {
                            terminate_batch = true;
                        }
                    }

                    events.push(SuperEvent::new(
                        SuperEventKind::TurnEnd,
                        format!("{} tools", executed.len()),
                    ));

                    if terminate_batch {
                        stopped_reason = "terminated by hook".into();
                        has_more_tools = false;
                    } else {
                        has_more_tools = true;
                    }
                    continue;
                }

                // No tool calls — assistant final for this turn
                final_text = resp.content.clone();
                messages.push(ChatMessage::assistant(&resp.content));
                events.push(SuperEvent::new(
                    SuperEventKind::Message,
                    format!("assistant ({} chars)", final_text.len()),
                ));
                events.push(SuperEvent::new(SuperEventKind::TurnEnd, "no tools"));
                has_more_tools = false;
            }

            // Pi follow-ups when agent would stop
            let follow = self.follow_ups.lock().pop_front();
            if let Some(fu) = follow {
                events.push(SuperEvent::new(SuperEventKind::FollowUp, &fu));
                messages.push(ChatMessage::user(format!("[follow-up] {fu}")));
                continue;
            }
            break;
        }

        events.push(SuperEvent::new(
            SuperEventKind::AgentEnd,
            format!(
                "steps={} tools={} reason={} compacted={compacted}",
                budget.steps, budget.tool_calls, stopped_reason
            ),
        ));
        self.session.lock().log("superharness", &stopped_reason);

        Ok(SuperRunResult {
            final_text,
            steps: budget.steps,
            tool_calls: budget.tool_calls,
            events,
            stopped_reason,
            compacted,
        })
    }
}

fn fail_truncated_tools(
    calls: &[ToolCall],
    events: &mut Vec<SuperEvent>,
) -> Vec<crate::superharness::parallel::ExecutedCall> {
    use crate::superharness::parallel::ExecutedCall;
    use crate::tools::ToolResult;
    calls
        .iter()
        .map(|call| {
            events.push(SuperEvent::tool(
                SuperEventKind::ToolBlocked,
                &call.function.name,
                "truncated response — tool not executed",
            ));
            ExecutedCall {
                call: call.clone(),
                result: ToolResult::err(
                    &call.function.name,
                    "Tool call was not executed: the response hit the output token limit, \
                     so its arguments may be truncated. Re-issue with complete arguments.",
                ),
                terminate: false,
            }
        })
        .collect()
}

fn short_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:x}", h.finish() & 0xffff)
}
