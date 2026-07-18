use serde::{Deserialize, Serialize};

/// Verified status of a finding after harness analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unknown = 0,
    Confirmed = 1,
    FalsePositive = 2,
    Unreproducible = 3,
    Inconclusive = 4,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationStatus::Unknown => "unknown",
            VerificationStatus::Confirmed => "confirmed",
            VerificationStatus::FalsePositive => "false_positive",
            VerificationStatus::Unreproducible => "unreproducible",
            VerificationStatus::Inconclusive => "inconclusive",
        }
    }
}

impl From<i32> for VerificationStatus {
    fn from(v: i32) -> Self {
        match v {
            1 => VerificationStatus::Confirmed,
            2 => VerificationStatus::FalsePositive,
            3 => VerificationStatus::Unreproducible,
            4 => VerificationStatus::Inconclusive,
            _ => VerificationStatus::Unknown,
        }
    }
}

/// A signal detected during differential analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: SignalType,
    pub evidence: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    TimeDelay,
    ErrorLeak,
    StructuralChange,
    OobTriggered,
    StatusChange,
    ContentChange,
    HeaderChange,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::TimeDelay => "time_delay",
            SignalType::ErrorLeak => "error_leak",
            SignalType::StructuralChange => "structural_change",
            SignalType::OobTriggered => "oob_triggered",
            SignalType::StatusChange => "status_change",
            SignalType::ContentChange => "content_change",
            SignalType::HeaderChange => "header_change",
        }
    }
}

/// A causal chain linking cause to effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalChain {
    pub valid: bool,
    pub links: Vec<CausalLink>,
    pub proof_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub cause: String,
    pub effect: String,
    pub evidence: String,
    pub confidence: f64,
}
