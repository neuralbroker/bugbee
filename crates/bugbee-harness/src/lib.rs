//! Agent harness: permissions, tools, hunt orchestration, security knowledge.
//! Defensive only — no exploit generation against live systems.

pub mod agents;
pub mod hunt;
pub mod knowledge;
pub mod permissions;
pub mod prompts;
pub mod tools;

pub use hunt::{HuntCampaign, HuntReport};
pub use knowledge::{corpus_block, HUNT_CHECKLIST, SECURITY_CORPUS};
pub use permissions::{Action, PermissionDecision, PermissionPolicy};
