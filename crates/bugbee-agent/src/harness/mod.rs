//! Godmode harness — OpenCode-style agent loop + multi-phase security pipeline.

mod budget;
mod events;
mod pipeline;
mod prompts;
mod runtime;

pub use budget::{Budget, RunLimits};
// Budget used by superharness
pub use events::{HarnessEvent, HarnessEventKind};
// HarnessEventKind used by swarm
pub use pipeline::{run_godmode, GodmodeOptions, GodmodeReport};
pub use runtime::{AgentRunResult, AgentRunner, RunnerConfig};
