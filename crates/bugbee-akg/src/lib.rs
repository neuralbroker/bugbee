//! Attack Knowledge Graph (AKG).
//!
//! Causal graph of the engagement: assets, findings, credentials, attack paths.
//! Prevents Type-B planning failures by making prerequisites explicit.
//!
//! Inverse of Incalmo-style abstraction: low-level findings → high-level kill chains.

mod graph;
mod query;

pub use graph::{AkgEdge, AkgNode, AkgSnapshot, AttackKnowledgeGraph, NodeId};
pub use query::{AttackPath, KillChain, PathDifficulty};
