//! Deterministic local analysis engines.
//!
//! No network. No LLM. Reproducible candidates for the agent review queue.

mod hunt;
mod rules;
mod secrets;

pub use hunt::{hunt, HuntOptions, HuntSummary};
pub use rules::{load_rules_from_dir, Rule, RulePack};
pub use secrets::scan_secrets;
