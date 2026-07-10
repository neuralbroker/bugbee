//! Agent harness: permissions, tools, hunt orchestration.
//! Defensive only — no exploit generation against live systems.

pub mod agents;
pub mod hunt;
pub mod permissions;
pub mod prompts;
pub mod tools;

pub use hunt::{HuntCampaign, HuntReport};
pub use permissions::{Action, PermissionDecision, PermissionPolicy};
