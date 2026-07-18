//! Stage 4: Evidence-aware adjudication matrix.

use bugbee_core::{adjudicate, AdjudicationResult, Finding, NeuralSignal, SymbolicVerdict};

use crate::ast::{extract_structural_signals, symbolic_strength_for_line};
use crate::hypothesis::hypothesize;
use std::fs;
use std::path::Path;

/// Full NSAE pass on one finding: structure → hypothesis → matrix.
pub fn adjudicate_finding(root: &Path, finding: &mut Finding) -> AdjudicationResult {
    let path = root.join(&finding.location.path);
    let slice = fs::read_to_string(&path)
        .ok()
        .map(|c| extract_structural_signals(&path, &c));

    let symbolic = if let Some(ref s) = slice {
        symbolic_strength_for_line(s, finding.location.start_line)
    } else {
        // No file → weak symbolic from rule severity
        if matches!(
            finding.severity,
            bugbee_core::Severity::Critical | bugbee_core::Severity::High
        ) {
            SymbolicVerdict::Noisy
        } else {
            SymbolicVerdict::None
        }
    };

    let hypo = hypothesize(finding, slice.as_ref());
    finding.neural_confidence = hypo.confidence;
    finding.symbolic_verdict = symbolic;

    // Prefer matrix neural signal; boost if rule is high-precision class
    let neural = if finding.rule_id.starts_with("secrets.") && finding.ecs >= 70 {
        NeuralSignal::Strong
    } else {
        hypo.signal
    };

    let result = adjudicate(symbolic, neural);
    finding.apply_adjudication(result.clone());
    finding.push_evidence(bugbee_core::Evidence {
        kind: "nsae".into(),
        detail: format!(
            "symbolic={} neural={} conf={:.2} → {} ({})",
            symbolic.as_str(),
            neural.as_str(),
            hypo.confidence,
            result.state.as_str(),
            result.reason
        ),
        location: None,
    });
    result
}

/// Batch adjudicate all findings.
pub fn adjudicate_all(root: &Path, findings: &mut [Finding]) -> Vec<AdjudicationResult> {
    findings
        .iter_mut()
        .map(|f| adjudicate_finding(root, f))
        .collect()
}
