//! # SuperHarness
//!
//! Unified agent harness synthesized from:
//!
//! | Source | Patterns adopted |
//! |--------|------------------|
//! | **Pi** ([earendil-works/pi](https://github.com/earendil-works/pi)) | Turn/event lifecycle, parallel tool batches, before/after tool hooks, steering + follow-up queues, fail-on-truncated tools, context transform, should_stop_after_turn |
//! | **OpenCode** ([anomalyco/opencode](https://github.com/anomalyco/opencode)) | Permission modes, doom-loop detection, max steps, compaction prune, plan/build role gates |
//! | **Claude Code** (public plugins + docs) | Hook bus (pre/post tool), Ralph outer loop with completion promise, subagent permission inheritance |
//!
//! Specialized for Bugbee: defense-only tools, NSAE/swarm integration points.

mod compaction;
mod hooks;
mod loop_;
mod parallel;
mod ralph;
mod types;

pub use compaction::{compact_messages, CompactionConfig};
pub use hooks::{HookAction, HookBus, HookContext, HookEvent};
pub use loop_::{SuperHarness, SuperHarnessConfig, SuperRunResult};
pub use parallel::{execute_tool_batch, ToolExecMode};
pub use ralph::{RalphConfig, RalphRunner, RalphStatus};
pub use types::{SuperEvent, SuperEventKind, TurnOutcome};
