use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Stable identifier for a finding (deterministic when possible).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FindingId(pub String);

impl FindingId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Content-addressed id so re-hunts update rather than duplicate.
    pub fn from_parts(rule_id: &str, path: &str, start_line: u32, message: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(start_line.to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(message.as_bytes());
        let digest = hasher.finalize();
        Self(hex::encode(&digest[..16]))
    }
}

impl Default for FindingId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "info" | "informational" => Some(Severity::Info),
            "low" => Some(Severity::Low),
            "medium" | "med" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" | "crit" => Some(Severity::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Draft,
    Confirmed,
    FalsePositive,
    Fixed,
}

impl FindingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingStatus::Draft => "draft",
            FindingStatus::Confirmed => "confirmed",
            FindingStatus::FalsePositive => "false_positive",
            FindingStatus::Fixed => "fixed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "draft" => Some(FindingStatus::Draft),
            "confirm" | "confirmed" | "c" => Some(FindingStatus::Confirmed),
            "fp" | "false_positive" | "false-positive" => Some(FindingStatus::FalsePositive),
            "fixed" | "fix" | "x" => Some(FindingStatus::Fixed),
            _ => None,
        }
    }
}

impl std::fmt::Display for FindingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    #[default]
    Rule,
    Secrets,
    Taint,
    Agent,
    Manual,
    Neural,
    Symbolic,
    Swarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: Option<u32>,
    pub end_column: Option<u32>,
    pub snippet: Option<String>,
}

impl Location {
    pub fn line(path: impl Into<String>, line: u32) -> Self {
        Self {
            path: path.into(),
            start_line: line,
            end_line: line,
            start_column: None,
            end_column: None,
            snippet: None,
        }
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }
}

/// Supporting proof for a finding (must be re-verifiable offline).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    pub detail: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: FindingId,
    pub rule_id: String,
    pub title: String,
    pub message: String,
    pub severity: Severity,
    pub status: FindingStatus,
    pub source: SourceKind,
    pub location: Location,
    pub evidence: Vec<Evidence>,
    /// Bugbee Risk Score 0–100.
    pub brs: u8,
    /// Evidence Completeness Score 0–100.
    pub ecs: u8,
    pub cwe: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // ── Neuro-symbolic swarm fields (serde-default for store compat) ──
    #[serde(default)]
    pub neural_confidence: f32,
    #[serde(default)]
    pub symbolic_verdict: crate::adjudication::SymbolicVerdict,
    #[serde(default)]
    pub adjudicated_state: crate::adjudication::AdjudicationState,
    #[serde(default)]
    pub adjudication_reason: Option<String>,
    #[serde(default)]
    pub poc: Option<crate::poc::ProofOfConcept>,
    #[serde(default)]
    pub chain_dependencies: Vec<FindingId>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub swarm_agent: Option<String>,
}

impl Finding {
    pub fn new(
        rule_id: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        severity: Severity,
        source: SourceKind,
        location: Location,
    ) -> Self {
        let rule_id = rule_id.into();
        let title = title.into();
        let message = message.into();
        let id = FindingId::from_parts(&rule_id, &location.path, location.start_line, &message);
        let now = Utc::now();
        let mut f = Self {
            id,
            rule_id,
            title,
            message,
            severity,
            status: FindingStatus::Draft,
            source,
            location,
            evidence: Vec::new(),
            brs: 0,
            ecs: 0,
            cwe: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            neural_confidence: 0.0,
            symbolic_verdict: crate::adjudication::SymbolicVerdict::None,
            adjudicated_state: crate::adjudication::AdjudicationState::Pending,
            adjudication_reason: None,
            poc: None,
            chain_dependencies: Vec::new(),
            verified: false,
            swarm_agent: None,
        };
        f.recompute_scores();
        f
    }

    /// Apply NSAE matrix result onto this finding.
    pub fn apply_adjudication(&mut self, result: crate::adjudication::AdjudicationResult) {
        self.adjudicated_state = result.state;
        self.adjudication_reason = Some(result.reason);
        if result.state == crate::adjudication::AdjudicationState::Vulnerable {
            // Boost ECS when matrix promotes
            if self.ecs < 70 {
                self.ecs = 70;
            }
        }
        if result.state == crate::adjudication::AdjudicationState::Safe {
            self.status = FindingStatus::FalsePositive;
        }
        self.recompute_scores();
        self.touch();
    }

    pub fn push_evidence(&mut self, evidence: Evidence) {
        self.evidence.push(evidence);
        self.recompute_scores();
        self.touch();
    }

    pub fn set_status(&mut self, status: FindingStatus) {
        self.status = status;
        self.touch();
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn recompute_scores(&mut self) {
        self.brs = crate::scoring::compute_brs(self.severity, self.ecs);
        self.ecs = crate::scoring::compute_ecs(self);
        // BRS depends on ECS — recompute once more after ECS settles.
        self.brs = crate::scoring::compute_brs(self.severity, self.ecs);
    }
}
