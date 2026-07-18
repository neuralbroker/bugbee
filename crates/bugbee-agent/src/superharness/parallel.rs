//! Parallel tool execution — Pi `toolExecution: parallel` for pure tools.

use bugbee_llm::ToolCall;
use serde_json::Value;

use crate::superharness::hooks::{HookAction, HookBus, HookContext};
use crate::superharness::types::{SuperEvent, SuperEventKind};
use crate::tools::{ToolExecutor, ToolResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolExecMode {
    /// Always sequential (safest for store mutations).
    #[default]
    Sequential,
    /// Parallel for pure-read tools; sequential otherwise.
    ParallelReads,
}

/// Tools that are safe to run concurrently (no store writes).
pub fn is_parallel_safe(name: &str) -> bool {
    matches!(
        name,
        "read" | "grep" | "glob" | "list_dir" | "get_finding" | "list_findings"
    )
}

pub struct ExecutedCall {
    pub call: ToolCall,
    pub result: ToolResult,
    pub terminate: bool,
}

/// Execute a batch of tool calls with hooks and optional parallelization.
pub fn execute_tool_batch(
    tools: &ToolExecutor,
    calls: &[ToolCall],
    hooks: &HookBus,
    mode: ToolExecMode,
    turn: u32,
    events: &mut Vec<SuperEvent>,
) -> Vec<ExecutedCall> {
    // Truncated/empty args already handled by caller
    let all_parallel = matches!(mode, ToolExecMode::ParallelReads)
        && calls.iter().all(|c| is_parallel_safe(&c.function.name));

    if all_parallel && calls.len() > 1 {
        execute_parallel(tools, calls, hooks, turn, events)
    } else {
        execute_sequential(tools, calls, hooks, turn, events)
    }
}

fn parse_args(call: &ToolCall) -> Value {
    if call.function.arguments.trim().is_empty() {
        return Value::Object(Default::default());
    }
    serde_json::from_str(&call.function.arguments).unwrap_or_else(
        |e| serde_json::json!({ "_parse_error": e.to_string(), "raw": call.function.arguments }),
    )
}

fn execute_sequential(
    tools: &ToolExecutor,
    calls: &[ToolCall],
    hooks: &HookBus,
    turn: u32,
    events: &mut Vec<SuperEvent>,
) -> Vec<ExecutedCall> {
    let mut out = Vec::new();
    for call in calls {
        let (executed, stop) = run_one(tools, call, hooks, turn, events);
        out.push(executed);
        if stop {
            break;
        }
    }
    out
}

fn execute_parallel(
    tools: &ToolExecutor,
    calls: &[ToolCall],
    hooks: &HookBus,
    turn: u32,
    events: &mut Vec<SuperEvent>,
) -> Vec<ExecutedCall> {
    // Pre-hook sequential (may block), then parallel execute, then post-hook sequential
    use rayon::prelude::*;

    let mut prepared: Vec<(ToolCall, Value, bool, String)> = Vec::new();
    for call in calls {
        let args = parse_args(call);
        let ctx = HookContext {
            tool_name: call.function.name.clone(),
            tool_call_id: call.id.clone(),
            args: args.clone(),
            result_preview: None,
            is_error: false,
            turn,
        };
        match hooks.emit_pre(&ctx, events) {
            HookAction::Block { reason } => {
                events.push(SuperEvent::tool(
                    SuperEventKind::ToolBlocked,
                    &call.function.name,
                    &reason,
                ));
                prepared.push((call.clone(), args, true, reason));
            }
            _ => prepared.push((call.clone(), args, false, String::new())),
        }
    }

    let results: Vec<(ToolCall, ToolResult)> = prepared
        .par_iter()
        .map(|(call, args, blocked, reason)| {
            if *blocked {
                (
                    call.clone(),
                    ToolResult::err(&call.function.name, reason.clone()),
                )
            } else {
                let r = tools.execute(&call.function.name, args);
                (call.clone(), r)
            }
        })
        .collect();

    let mut out = Vec::new();
    let mut terminate = false;
    for (call, mut result) in results {
        events.push(SuperEvent::tool(
            SuperEventKind::ToolEnd,
            &call.function.name,
            format!("ok={} ({} chars)", result.ok, result.output.len()),
        ));
        let ctx = HookContext {
            tool_name: call.function.name.clone(),
            tool_call_id: call.id.clone(),
            args: parse_args(&call),
            result_preview: Some(result.output.chars().take(200).collect()),
            is_error: !result.ok,
            turn,
        };
        match hooks.emit_post(&ctx, events) {
            HookAction::OverrideResult { content, is_error } => {
                result.output = content;
                result.ok = !is_error;
            }
            HookAction::Terminate => terminate = true,
            _ => {}
        }
        out.push(ExecutedCall {
            call,
            result,
            terminate,
        });
        if terminate {
            break;
        }
    }
    out
}

fn run_one(
    tools: &ToolExecutor,
    call: &ToolCall,
    hooks: &HookBus,
    turn: u32,
    events: &mut Vec<SuperEvent>,
) -> (ExecutedCall, bool) {
    let args = parse_args(call);
    events.push(SuperEvent::tool(
        SuperEventKind::ToolStart,
        &call.function.name,
        compact_args(&args),
    ));

    let ctx = HookContext {
        tool_name: call.function.name.clone(),
        tool_call_id: call.id.clone(),
        args: args.clone(),
        result_preview: None,
        is_error: false,
        turn,
    };

    match hooks.emit_pre(&ctx, events) {
        HookAction::Block { reason } => {
            events.push(SuperEvent::tool(
                SuperEventKind::ToolBlocked,
                &call.function.name,
                &reason,
            ));
            return (
                ExecutedCall {
                    call: call.clone(),
                    result: ToolResult::err(&call.function.name, reason),
                    terminate: false,
                },
                false,
            );
        }
        HookAction::Terminate => {
            return (
                ExecutedCall {
                    call: call.clone(),
                    result: ToolResult::err(&call.function.name, "terminated by hook"),
                    terminate: true,
                },
                true,
            );
        }
        _ => {}
    }

    let mut result = if args.get("_parse_error").is_some() {
        ToolResult::err(
            &call.function.name,
            format!(
                "invalid JSON arguments: {}",
                args.get("_parse_error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
            ),
        )
    } else {
        tools.execute(&call.function.name, &args)
    };

    events.push(SuperEvent::tool(
        SuperEventKind::ToolEnd,
        &call.function.name,
        format!("ok={} ({} chars)", result.ok, result.output.len()),
    ));

    let ctx = HookContext {
        tool_name: call.function.name.clone(),
        tool_call_id: call.id.clone(),
        args,
        result_preview: Some(result.output.chars().take(200).collect()),
        is_error: !result.ok,
        turn,
    };

    let mut terminate = false;
    match hooks.emit_post(&ctx, events) {
        HookAction::OverrideResult { content, is_error } => {
            result.output = content;
            result.ok = !is_error;
        }
        HookAction::Terminate => terminate = true,
        _ => {}
    }

    (
        ExecutedCall {
            call: call.clone(),
            result,
            terminate,
        },
        terminate,
    )
}

fn compact_args(v: &Value) -> String {
    let s = v.to_string();
    if s.len() > 140 {
        format!("{}…", &s[..140])
    } else {
        s
    }
}
