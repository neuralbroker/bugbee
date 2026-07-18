//! Hierarchical multi-agent swarm (spec §2).
//!
//! Topology: Orchestrator → Recon / Hunter / Symbolic / Prover / Chain / Scribe
//! Shared memory: Attack Knowledge Graph + finding store.

mod carlin;
mod checkpoint;
mod orchestrator;
mod recon;
mod scribe;

pub use carlin::carlini_refine;
pub use checkpoint::{Checkpoint, CheckpointStore};
pub use orchestrator::{run_swarm, SwarmOptions, SwarmReport};
pub use recon::recon_repo;
pub use scribe::{render_bounty_report, render_bounty_reports};

// re-export for CLI path bugbee_agent::swarm::render_bounty_reports
