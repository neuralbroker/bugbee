//! Stage 3: Deterministic translator — hypothesis → formal verification script (IR).
//!
//! Direct LLM→Lean fails often (LeanGuard). We emit a **deterministic IR** that
//! the static prover always executes without compilation lottery.

use serde::{Deserialize, Serialize};

use crate::hypothesis::NeuralHypothesis;
use bugbee_core::Finding;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalScript {
    pub ir_version: u32,
    pub finding_id: String,
    pub checks: Vec<IrCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)] // IR ops are intentionally Assert*
pub enum IrCheck {
    /// Assert snippet / pattern still exists at location.
    AssertPattern {
        path: String,
        line: u32,
        pattern: String,
    },
    /// Assert no sanitizer within N lines before sink.
    AssertNoGuard {
        path: String,
        line: u32,
        window: u32,
    },
    /// Assert taint source exists earlier in file.
    AssertSourceBefore { path: String, line: u32 },
    /// Assert secret-like assignment remains.
    AssertSecretAssign { path: String, line: u32 },
}

pub fn translate_hypothesis(finding: &Finding, hypo: &NeuralHypothesis) -> FormalScript {
    let path = finding.location.path.clone();
    let line = finding.location.start_line;
    let mut checks = Vec::new();

    let pattern = finding
        .location
        .snippet
        .clone()
        .filter(|s| s.len() >= 3)
        .unwrap_or_else(|| finding.rule_id.clone());

    checks.push(IrCheck::AssertPattern {
        path: path.clone(),
        line,
        pattern: pattern.chars().take(80).collect(),
    });

    if finding.rule_id.contains("secret") || finding.rule_id.contains("password") {
        checks.push(IrCheck::AssertSecretAssign {
            path: path.clone(),
            line,
        });
    } else {
        checks.push(IrCheck::AssertNoGuard {
            path: path.clone(),
            line,
            window: 12,
        });
        if hypo.confidence >= 0.5 {
            checks.push(IrCheck::AssertSourceBefore { path, line });
        }
    }

    FormalScript {
        ir_version: 1,
        finding_id: finding.id.0.clone(),
        checks,
    }
}
