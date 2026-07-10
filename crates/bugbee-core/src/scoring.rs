//! Bugbee Risk Score (BRS) and Evidence Completeness Score (ECS).
//!
//! BRS = 100 * sigmoid(w_s*S + w_e*E + w_c*C + w_r*R + w_b*B - w_f*F)
//! ECS = (1/4)*(src+sink+path+repro) * (1 - 0.5*missing_sanitizer)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrsWeights {
    pub severity: f64,
    pub exploitability: f64,
    pub confidence: f64,
    pub recency: f64,
    pub blast_radius: f64,
    pub false_positive_prior: f64,
}

impl Default for BrsWeights {
    fn default() -> Self {
        Self {
            severity: 0.28,
            exploitability: 0.22,
            confidence: 0.20,
            recency: 0.08,
            blast_radius: 0.14,
            false_positive_prior: 0.18,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EcsInputs {
    pub has_source: bool,
    pub has_sink: bool,
    pub has_path: bool,
    pub has_repro: bool,
    pub missing_sanitizer_check: bool,
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Compute Bugbee Risk Score in [0, 100].
/// All component inputs should be in [0, 1].
pub fn score_brs(
    severity: f64,
    exploitability: f64,
    confidence: f64,
    recency: f64,
    blast_radius: f64,
    false_positive_prior: f64,
    weights: &BrsWeights,
) -> f64 {
    let s = severity.clamp(0.0, 1.0);
    let e = exploitability.clamp(0.0, 1.0);
    let c = confidence.clamp(0.0, 1.0);
    let r = recency.clamp(0.0, 1.0);
    let b = blast_radius.clamp(0.0, 1.0);
    let f = false_positive_prior.clamp(0.0, 1.0);

    let linear = weights.severity * s
        + weights.exploitability * e
        + weights.confidence * c
        + weights.recency * r
        + weights.blast_radius * b
        - weights.false_positive_prior * f;

    // Center around 0 for sigmoid: map typical range to sharper distribution
    let centered = (linear - 0.35) * 6.0;
    (100.0 * sigmoid(centered)).clamp(0.0, 100.0)
}

/// Evidence Completeness Score in [0, 1].
pub fn score_ecs(inputs: &EcsInputs) -> f64 {
    let parts = [
        inputs.has_source,
        inputs.has_sink,
        inputs.has_path,
        inputs.has_repro,
    ];
    let base = parts.iter().filter(|&&p| p).count() as f64 / 4.0;
    let penalty = if inputs.missing_sanitizer_check {
        0.5
    } else {
        1.0
    };
    (base * penalty).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualReviewDecision {
    AutoConfirm,
    HumanQueue,
    Drop,
}

/// Dual-review consensus gate.
pub fn dual_review_decision(
    auto_confirms: bool,
    brs: f64,
    ecs: f64,
    theta_high: f64,
    theta_low: f64,
) -> DualReviewDecision {
    if auto_confirms && brs >= theta_high && ecs >= 0.75 {
        DualReviewDecision::AutoConfirm
    } else if brs >= theta_low {
        DualReviewDecision::HumanQueue
    } else if brs < theta_low && ecs < 0.25 {
        DualReviewDecision::Drop
    } else {
        DualReviewDecision::HumanQueue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brs_critical_high_confidence() {
        let w = BrsWeights::default();
        let score = score_brs(1.0, 0.9, 0.9, 0.8, 0.8, 0.05, &w);
        assert!(score > 70.0, "expected high BRS, got {score}");
    }

    #[test]
    fn brs_low_noise() {
        let w = BrsWeights::default();
        let score = score_brs(0.1, 0.1, 0.2, 0.1, 0.1, 0.8, &w);
        assert!(score < 40.0, "expected low BRS, got {score}");
    }

    #[test]
    fn ecs_full() {
        let ecs = score_ecs(&EcsInputs {
            has_source: true,
            has_sink: true,
            has_path: true,
            has_repro: true,
            missing_sanitizer_check: false,
        });
        assert!((ecs - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ecs_partial_with_sanitizer_gap() {
        let ecs = score_ecs(&EcsInputs {
            has_source: true,
            has_sink: true,
            has_path: false,
            has_repro: false,
            missing_sanitizer_check: true,
        });
        assert!((ecs - 0.25).abs() < 1e-9);
    }

    #[test]
    fn dual_review_auto() {
        let d = dual_review_decision(true, 85.0, 0.8, 80.0, 30.0);
        assert_eq!(d, DualReviewDecision::AutoConfirm);
    }

    #[test]
    fn dual_review_human() {
        let d = dual_review_decision(false, 50.0, 0.5, 80.0, 30.0);
        assert_eq!(d, DualReviewDecision::HumanQueue);
    }
}
