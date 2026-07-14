use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::scoring::{score_brs, score_ecs, BrsWeights, EcsInputs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_f64(self) -> f64 {
        match self {
            Severity::Critical => 1.0,
            Severity::High => 0.8,
            Severity::Medium => 0.5,
            Severity::Low => 0.25,
            Severity::Info => 0.1,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    New,
    Triaged,
    Confirmed,
    FalsePositive,
    Fixed,
    WontFix,
}

impl FindingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingStatus::New => "new",
            FindingStatus::Triaged => "triaged",
            FindingStatus::Confirmed => "confirmed",
            FindingStatus::FalsePositive => "false_positive",
            FindingStatus::Fixed => "fixed",
            FindingStatus::WontFix => "wont_fix",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationRole {
    Source,
    Path,
    Sink,
    Fix,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingLocation {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_col: Option<u32>,
    pub end_col: Option<u32>,
    pub role: LocationRole,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Evidence {
    pub dataflow: Option<String>,
    pub traces: Vec<String>,
    pub rule_id: Option<String>,
    pub agent_notes: Option<String>,
    pub has_source: bool,
    pub has_sink: bool,
    pub has_path: bool,
    pub has_repro: bool,
    pub missing_sanitizer_check: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewBy {
    Human,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub by: ReviewBy,
    pub verdict: String,
    pub rationale: String,
    pub ts: DateTime<Utc>,
}

/// Model-assisted explanation of a detector finding. Every field is optional:
/// deterministic engines remain useful without a configured model, and old
/// stores deserialize safely as this surface evolves.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiReview {
    pub summary: Option<String>,
    pub root_cause: Option<String>,
    pub attack_scenario: Option<String>,
    pub why_it_matters: Option<String>,
    pub suggested_fix: Option<String>,
    pub alternative_fixes: Vec<String>,
    pub related_finding_ids: Vec<Uuid>,
    pub model_confidence: Option<f64>,
    pub model: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: f64,
    pub brs: f64,
    pub ecs: f64,
    pub cwe: Vec<String>,
    pub owasp: Vec<String>,
    pub category: String,
    pub locations: Vec<FindingLocation>,
    pub evidence: Evidence,
    pub blast_radius: f64,
    pub exploitability: f64,
    pub status: FindingStatus,
    pub reviews: Vec<Review>,
    pub patch_diff: Option<String>,
    #[serde(default)]
    pub ai_review: AiReview,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Finding {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        severity: Severity,
        category: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let mut f = Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            severity,
            confidence: 0.5,
            brs: 0.0,
            ecs: 0.0,
            cwe: Vec::new(),
            owasp: Vec::new(),
            category: category.into(),
            locations: Vec::new(),
            evidence: Evidence::default(),
            blast_radius: 0.3,
            exploitability: 0.3,
            status: FindingStatus::New,
            reviews: Vec::new(),
            patch_diff: None,
            ai_review: AiReview::default(),
            created_at: now,
            updated_at: now,
        };
        f.recompute_scores(&BrsWeights::default());
        f
    }

    pub fn recompute_scores(&mut self, weights: &BrsWeights) {
        let ecs_in = EcsInputs {
            has_source: self.evidence.has_source
                || self
                    .locations
                    .iter()
                    .any(|l| l.role == LocationRole::Source),
            has_sink: self.evidence.has_sink
                || self.locations.iter().any(|l| l.role == LocationRole::Sink),
            has_path: self.evidence.has_path
                || self.locations.iter().any(|l| l.role == LocationRole::Path),
            has_repro: self.evidence.has_repro,
            missing_sanitizer_check: self.evidence.missing_sanitizer_check,
        };
        self.ecs = score_ecs(&ecs_in);
        self.brs = score_brs(
            self.severity.as_f64(),
            self.exploitability,
            self.confidence,
            0.5,
            self.blast_radius,
            0.1,
            weights,
        );
        self.id = Uuid::new_v5(&Uuid::NAMESPACE_URL, self.fingerprint().as_bytes());
        self.updated_at = Utc::now();
    }

    /// Stable, content-addressed identity for deduplicating a finding across scans.
    /// It deliberately excludes mutable scores, descriptions, and timestamps.
    pub fn fingerprint(&self) -> String {
        let mut locations: Vec<_> = self.locations.iter().collect();
        locations.sort_by(|left, right| {
            (
                &left.file,
                left.start_line,
                left.end_line,
                format!("{:?}", left.role),
            )
                .cmp(&(
                    &right.file,
                    right.start_line,
                    right.end_line,
                    format!("{:?}", right.role),
                ))
        });

        let mut hasher = Sha256::new();
        hasher.update(b"bugbee-finding-v1\0");
        hasher.update(
            self.evidence
                .rule_id
                .as_deref()
                .unwrap_or(self.category.as_str())
                .as_bytes(),
        );
        hasher.update(b"\0");
        hasher.update(self.category.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.title.as_bytes());
        for location in locations {
            hasher.update(b"\0");
            hasher.update(location.file.as_bytes());
            hasher.update(b":");
            hasher.update(location.start_line.to_be_bytes());
            hasher.update(b":");
            hasher.update(location.end_line.to_be_bytes());
            hasher.update(b":");
            hasher.update(format!("{:?}", location.role).as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn add_location(&mut self, loc: FindingLocation) {
        match loc.role {
            LocationRole::Source => self.evidence.has_source = true,
            LocationRole::Sink => self.evidence.has_sink = true,
            LocationRole::Path => self.evidence.has_path = true,
            _ => {}
        }
        self.locations.push(loc);
    }

    pub fn add_human_review(&mut self, verdict: impl Into<String>, rationale: impl Into<String>) {
        self.reviews.push(Review {
            by: ReviewBy::Human,
            verdict: verdict.into(),
            rationale: rationale.into(),
            ts: Utc::now(),
        });
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> Finding {
        let mut finding = Finding::new(
            "Unsafe dynamic evaluation",
            "Potential dynamic code execution",
            Severity::High,
            "injection",
        );
        finding.evidence.rule_id = Some("python.eval".into());
        finding.add_location(FindingLocation {
            file: "app.py".into(),
            start_line: 12,
            end_line: 12,
            start_col: None,
            end_col: None,
            role: LocationRole::Sink,
            snippet: None,
        });
        finding.recompute_scores(&BrsWeights::default());
        finding
    }

    #[test]
    fn equivalent_findings_keep_the_same_identity() {
        assert_eq!(candidate().id, candidate().id);
    }

    #[test]
    fn changed_location_changes_identity() {
        let original = candidate();
        let mut moved = candidate();
        moved.locations[0].start_line = 13;
        moved.locations[0].end_line = 13;
        moved.recompute_scores(&BrsWeights::default());
        assert_ne!(original.id, moved.id);
    }
}
