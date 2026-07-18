//! Stage 2: Neural hypothesis generator (engine-backed proxy + optional LLM later).

use bugbee_core::{Finding, NeuralSignal, Severity};
use serde::{Deserialize, Serialize};

use crate::ast::StructuralSlice;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralHypothesis {
    pub finding_id: String,
    pub confidence: f32,
    pub signal: NeuralSignal,
    pub reasoning: String,
    pub suggested_class: String,
}

/// Generate a neural-style hypothesis from structural + finding metadata.
/// When no LLM is available this is a calibrated proxy (R2Vul-style distillation stand-in).
pub fn hypothesize(finding: &Finding, slice: Option<&StructuralSlice>) -> NeuralHypothesis {
    let mut conf = 0.35_f32;
    let mut reasons = Vec::new();

    // Severity prior
    conf += match finding.severity {
        Severity::Critical => 0.35,
        Severity::High => 0.28,
        Severity::Medium => 0.15,
        Severity::Low => 0.08,
        Severity::Info => 0.02,
    };
    reasons.push(format!("severity={}", finding.severity.as_str()));

    // Evidence completeness
    conf += (finding.ecs as f32 / 100.0) * 0.2;
    reasons.push(format!("ecs={}", finding.ecs));

    // Structural reinforcement
    if let Some(s) = slice {
        if s.taint_sinks
            .iter()
            .any(|x| x.line == finding.location.start_line)
        {
            conf += 0.2;
            reasons.push("line_is_sink".into());
        }
        if !s.taint_sources.is_empty() && !s.taint_sinks.is_empty() {
            conf += 0.1;
            reasons.push("source_and_sink_present".into());
        }
        if s.guards.is_empty() {
            conf += 0.05;
            reasons.push("no_guards_detected".into());
        } else {
            conf -= 0.08;
            reasons.push("guards_present".into());
        }
    }

    // Rule pack confidence
    if finding.rule_id.starts_with("secrets.") {
        conf += 0.15;
        reasons.push("secrets_rule".into());
    }
    if finding.rule_id.contains("eval") || finding.rule_id.contains("shell") {
        conf += 0.1;
        reasons.push("rce_class_rule".into());
    }

    conf = conf.clamp(0.0, 0.99);
    NeuralHypothesis {
        finding_id: finding.id.0.clone(),
        confidence: conf,
        signal: NeuralSignal::from_confidence(conf),
        reasoning: reasons.join("; "),
        suggested_class: finding.rule_id.clone(),
    }
}
