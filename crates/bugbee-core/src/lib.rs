//! Bugbee core: domain model, configuration, persistence, scoring, redaction.
//!
//! This crate has **no network** and **no UI**. It is the stable foundation
//! every other crate depends on.

pub mod adjudication;
pub mod config;
pub mod error;
pub mod finding;
pub mod poc;
pub mod redact;
pub mod scoring;
pub mod store;
pub mod target;

pub use adjudication::{
    adjudicate, AdjudicationResult, AdjudicationState, NeuralSignal, SymbolicVerdict,
};
pub use config::{BugbeeConfig, ProjectConfig};
pub use error::{Error, Result};
pub use finding::{Evidence, Finding, FindingId, FindingStatus, Location, Severity, SourceKind};
pub use poc::{PocClass, ProofOfConcept, VerificationOutcome, VerificationResult};
pub use redact::Redactor;
pub use scoring::{BugbeeRiskScore, EvidenceCompleteness};
pub use store::Store;
pub use target::{AuthMechanism, Target, TargetId, TargetKind};

/// Current product version (workspace).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Local project state directory name.
pub const STATE_DIR: &str = ".bugbee";
