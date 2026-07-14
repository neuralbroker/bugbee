//! Bugbee core: findings, scoring, config, redact, store.

pub mod config;
pub mod error;
pub mod finding;
pub mod redact;
pub mod scoring;
pub mod store;

pub use config::{BugbeeConfig, HuntConfig, InferenceConfig, PermissionConfig, ProviderConfig};
pub use error::{BugbeeError, Result};
pub use finding::{
    AiReview, Evidence, Finding, FindingLocation, FindingStatus, LocationRole, Review, ReviewBy,
    Severity,
};
pub use redact::Redactor;
pub use scoring::{
    dual_review_decision, score_brs, score_ecs, BrsWeights, DualReviewDecision, EcsInputs,
};
pub use store::FindingStore;
