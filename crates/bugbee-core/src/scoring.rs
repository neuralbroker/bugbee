use crate::finding::{Finding, Severity};

/// Bugbee Risk Score (0–100): severity-weighted, boosted by evidence quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BugbeeRiskScore(pub u8);

/// Evidence Completeness Score (0–100): how re-verifiable the claim is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceCompleteness(pub u8);

pub fn severity_base(severity: Severity) -> u8 {
    match severity {
        Severity::Info => 10,
        Severity::Low => 25,
        Severity::Medium => 50,
        Severity::High => 75,
        Severity::Critical => 95,
    }
}

/// ECS: location + snippet + evidence items + rule id.
pub fn compute_ecs(finding: &Finding) -> u8 {
    let mut score: u16 = 20; // baseline: we have a structured finding

    if !finding.location.path.is_empty() {
        score += 15;
    }
    if finding.location.start_line > 0 {
        score += 15;
    }
    if finding
        .location
        .snippet
        .as_ref()
        .is_some_and(|s| !s.is_empty())
    {
        score += 20;
    }
    if !finding.rule_id.is_empty() {
        score += 10;
    }

    let n = finding.evidence.len().min(4) as u16;
    score += n * 5;

    if finding.cwe.is_some() {
        score += 5;
    }

    score.min(100) as u8
}

/// BRS blends severity with how complete the evidence is.
pub fn compute_brs(severity: Severity, ecs: u8) -> u8 {
    let base = severity_base(severity) as u16;
    // Incomplete evidence pulls risk presentation down (not “hide”, but de-rank).
    let factor = 50 + (ecs as u16 / 2); // 50–100
    let score = base * factor / 100;
    score.min(100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Location, SourceKind};

    #[test]
    fn ecs_grows_with_proof() {
        let mut f = Finding::new(
            "test.rule",
            "Title",
            "msg",
            Severity::High,
            SourceKind::Rule,
            Location::line("app.py", 10).with_snippet("eval(user)"),
        );
        let bare = f.ecs;
        f.push_evidence(crate::finding::Evidence {
            kind: "sink".into(),
            detail: "eval".into(),
            location: None,
        });
        assert!(f.ecs >= bare);
        assert!(f.brs > 0);
    }
}
