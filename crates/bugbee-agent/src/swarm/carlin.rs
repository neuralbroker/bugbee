//! HunterAgent Carlini Loop: hypothesize → test (local) → observe → refine.
//!
//! Defense-only: iterates on source/engine evidence, not live third-party exploitation.

use std::path::Path;

use bugbee_core::{Evidence, Finding};
use bugbee_nsae::{adjudicate_finding, hypothesize, verify_finding};
use tracing::debug;

/// Max refinement iterations (spec: 5).
pub const MAX_CARLINI_ITERS: usize = 5;

#[derive(Debug, Clone)]
pub struct CarliniResult {
    pub iterations: usize,
    pub final_confidence: f32,
    pub refined: bool,
}

/// Refine a single finding with iterative local observation.
pub fn carlini_refine(root: &Path, finding: &mut Finding) -> CarliniResult {
    let mut iterations = 0;
    let mut last_conf = finding.neural_confidence;
    let mut refined = false;

    for i in 0..MAX_CARLINI_ITERS {
        iterations = i + 1;
        // Hypothesize (stage 2)
        let path = root.join(&finding.location.path);
        let slice = std::fs::read_to_string(&path)
            .ok()
            .map(|c| bugbee_nsae::extract_structural_signals(&path, &c));
        let hypo = hypothesize(finding, slice.as_ref());
        finding.neural_confidence = hypo.confidence;

        // Observe via NSAE + prover
        let _adj = adjudicate_finding(root, finding);
        let ver = verify_finding(root, finding);

        finding.push_evidence(Evidence {
            kind: "carlini".into(),
            detail: format!(
                "iter={i} conf={:.2} adj={} ver={:?} reason={}",
                hypo.confidence,
                finding.adjudicated_state.as_str(),
                ver.outcome,
                hypo.reasoning
            ),
            location: None,
        });
        finding.swarm_agent = Some("hunter".into());

        if (hypo.confidence - last_conf).abs() < 0.02 && i > 0 {
            break; // converged
        }
        if hypo.confidence >= 0.85 && finding.verified {
            refined = true;
            break;
        }
        last_conf = hypo.confidence;
        refined = true;
        debug!(iter = i, conf = hypo.confidence, "carlini");
    }

    CarliniResult {
        iterations,
        final_confidence: finding.neural_confidence,
        refined,
    }
}

/// Run Carlini on high/critical draft findings (bounded).
pub fn carlini_batch(root: &Path, findings: &mut [Finding], max: usize) -> usize {
    let mut n = 0;
    for f in findings.iter_mut() {
        if n >= max {
            break;
        }
        if !matches!(
            f.severity,
            bugbee_core::Severity::Critical | bugbee_core::Severity::High
        ) {
            continue;
        }
        if f.status != bugbee_core::FindingStatus::Draft {
            continue;
        }
        carlini_refine(root, f);
        n += 1;
    }
    n
}
