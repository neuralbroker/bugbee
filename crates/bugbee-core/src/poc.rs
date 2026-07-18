//! Proof-of-concept and verification types (defense-only, authorized targets).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::finding::FindingId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PocClass {
    SqlInjection,
    Xss,
    CommandInjection,
    PathTraversal,
    Ssrf,
    Idor,
    HardcodedSecret,
    WeakCrypto,
    DangerousEval,
    Other,
}

impl PocClass {
    pub fn as_str(self) -> &'static str {
        match self {
            PocClass::SqlInjection => "sqli",
            PocClass::Xss => "xss",
            PocClass::CommandInjection => "cmdi",
            PocClass::PathTraversal => "path",
            PocClass::Ssrf => "ssrf",
            PocClass::Idor => "idor",
            PocClass::HardcodedSecret => "secret",
            PocClass::WeakCrypto => "crypto",
            PocClass::DangerousEval => "eval",
            PocClass::Other => "other",
        }
    }

    pub fn from_rule_or_cwe(rule_id: &str, cwe: Option<&str>) -> Self {
        let s = format!("{} {}", rule_id, cwe.unwrap_or("")).to_ascii_lowercase();
        if s.contains("sql") || s.contains("cwe-89") {
            PocClass::SqlInjection
        } else if s.contains("xss") || s.contains("cwe-79") {
            PocClass::Xss
        } else if s.contains("shell") || s.contains("command") || s.contains("cwe-78") {
            PocClass::CommandInjection
        } else if s.contains("ssrf") || s.contains("cwe-918") {
            PocClass::Ssrf
        } else if s.contains("eval") || s.contains("cwe-95") {
            PocClass::DangerousEval
        } else if s.contains("secret") || s.contains("password") || s.contains("key") {
            PocClass::HardcodedSecret
        } else if s.contains("md5") || s.contains("crypto") || s.contains("cwe-328") {
            PocClass::WeakCrypto
        } else {
            PocClass::Other
        }
    }
}

/// Educational / authorized reproduction artifact — never live third-party weaponization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfConcept {
    pub class: PocClass,
    /// Human-readable steps (local/fixture/authorized only).
    pub steps: Vec<String>,
    /// Safe curl/template against local fixture (no third-party targets).
    pub curl_template: Option<String>,
    /// Hash of PoC body for AKG dedup.
    pub poc_hash: String,
    pub created_at: DateTime<Utc>,
}

impl ProofOfConcept {
    pub fn new(class: PocClass, steps: Vec<String>, curl_template: Option<String>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(class.as_str().as_bytes());
        for s in &steps {
            hasher.update(s.as_bytes());
        }
        if let Some(ref c) = curl_template {
            hasher.update(c.as_bytes());
        }
        let poc_hash = hex::encode(hasher.finalize());
        Self {
            class,
            steps,
            curl_template,
            poc_hash,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationOutcome {
    Confirmed,
    Unconfirmed,
    FalsePositive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub finding_id: FindingId,
    pub outcome: VerificationOutcome,
    pub evidence_blob: String,
    pub method: String,
    pub verified_at: DateTime<Utc>,
}

impl VerificationResult {
    pub fn confirmed(
        id: FindingId,
        method: impl Into<String>,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: id,
            outcome: VerificationOutcome::Confirmed,
            evidence_blob: evidence.into(),
            method: method.into(),
            verified_at: Utc::now(),
        }
    }

    pub fn false_positive(
        id: FindingId,
        method: impl Into<String>,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: id,
            outcome: VerificationOutcome::FalsePositive,
            evidence_blob: evidence.into(),
            method: method.into(),
            verified_at: Utc::now(),
        }
    }

    pub fn unconfirmed(
        id: FindingId,
        method: impl Into<String>,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            finding_id: id,
            outcome: VerificationOutcome::Unconfirmed,
            evidence_blob: evidence.into(),
            method: method.into(),
            verified_at: Utc::now(),
        }
    }
}
