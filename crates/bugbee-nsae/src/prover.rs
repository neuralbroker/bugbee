//! Static prover harness — verifies findings against local source (defense-only).
//!
//! Spec's WASM/Docker prover is Horizon B. This stage closes the lab-to-real gap
//! for **code-level** claims: PoC is a re-checkable evidence pack, not live exploit.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use bugbee_core::{
    Finding, FindingStatus, PocClass, ProofOfConcept, VerificationOutcome, VerificationResult,
};
use regex::Regex;
use tracing::debug;

use crate::ast::extract_structural_signals;
use crate::hypothesis::hypothesize;
use crate::translator::{translate_hypothesis, FormalScript, IrCheck};

fn guard_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(escape|sanitize|parametrize|prepared|allowlist)").expect("re")
    })
}

fn source_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(request\.|req\.|input\(|environ|params|query|body)").expect("re")
    })
}

fn secret_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)(password|secret|api[_-]?key|token)\s*=\s*['"]"#).expect("re")
    })
}

pub struct StaticProver<'a> {
    pub root: &'a Path,
}

impl<'a> StaticProver<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }

    pub fn verify(&self, finding: &mut Finding) -> VerificationResult {
        let hypo = {
            let path = self.root.join(&finding.location.path);
            let slice = fs::read_to_string(&path)
                .ok()
                .map(|c| extract_structural_signals(&path, &c));
            hypothesize(finding, slice.as_ref())
        };
        let script = translate_hypothesis(finding, &hypo);
        let result = execute_ir(self.root, finding, &script);

        if result.outcome == VerificationOutcome::Confirmed {
            finding.verified = true;
            finding.poc = Some(build_poc(finding));
            finding.push_evidence(bugbee_core::Evidence {
                kind: "prover".into(),
                detail: result.evidence_blob.clone(),
                location: Some(finding.location.clone()),
            });
        } else if result.outcome == VerificationOutcome::FalsePositive {
            finding.verified = false;
            finding.set_status(FindingStatus::FalsePositive);
            finding.push_evidence(bugbee_core::Evidence {
                kind: "prover_fp".into(),
                detail: result.evidence_blob.clone(),
                location: None,
            });
        }

        debug!(
            id = %finding.id,
            outcome = ?result.outcome,
            "static prover"
        );
        result
    }
}

pub fn verify_finding(root: &Path, finding: &mut Finding) -> VerificationResult {
    StaticProver::new(root).verify(finding)
}

fn execute_ir(root: &Path, finding: &Finding, script: &FormalScript) -> VerificationResult {
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut notes = Vec::new();

    for check in &script.checks {
        match check {
            IrCheck::AssertPattern {
                path,
                line,
                pattern,
            } => {
                let content = read(root, path);
                let ok = content.as_ref().is_some_and(|c| {
                    c.lines()
                        .nth(line.saturating_sub(1) as usize)
                        .is_some_and(|l| {
                            if pattern.len() >= 4 {
                                l.contains(pattern.trim())
                                    || pattern.split_whitespace().take(2).all(|p| l.contains(p))
                            } else {
                                true
                            }
                        })
                });
                if ok {
                    passed += 1;
                    notes.push(format!("PASS pattern @ {path}:{line}"));
                } else {
                    failed += 1;
                    notes.push(format!("FAIL pattern @ {path}:{line}"));
                }
            }
            IrCheck::AssertNoGuard { path, line, window } => {
                let content = read(root, path);
                let has_guard = content.as_ref().is_some_and(|c| {
                    let lines: Vec<_> = c.lines().collect();
                    if lines.is_empty() {
                        return false;
                    }
                    let idx = line.saturating_sub(1) as usize;
                    let start = idx.saturating_sub(*window as usize);
                    let end = idx.min(lines.len().saturating_sub(1));
                    lines[start..=end].iter().any(|l| guard_re().is_match(l))
                });
                if !has_guard {
                    passed += 1;
                    notes.push("PASS no guard in window".into());
                } else {
                    failed += 1;
                    notes.push("FAIL guard present near sink".into());
                }
            }
            IrCheck::AssertSourceBefore { path, line } => {
                let content = read(root, path);
                // Soft: pass if web source exists earlier OR non-web sink (eval/shell)
                let has_source = content.as_ref().is_some_and(|c| {
                    c.lines()
                        .take(*line as usize)
                        .any(|l| source_re().is_match(l))
                });
                let non_web_sink = finding.rule_id.contains("eval")
                    || finding.rule_id.contains("shell")
                    || finding.rule_id.contains("exec")
                    || finding.rule_id.contains("secret");
                if has_source || non_web_sink {
                    passed += 1;
                    notes.push("PASS source-before (or non-web sink)".into());
                } else {
                    failed += 1;
                    notes.push("FAIL no source before".into());
                }
            }
            IrCheck::AssertSecretAssign { path, line } => {
                let content = read(root, path);
                let ok = content.as_ref().is_some_and(|c| {
                    c.lines()
                        .nth(line.saturating_sub(1) as usize)
                        .is_some_and(|l| secret_re().is_match(l) || l.contains('='))
                });
                if ok {
                    passed += 1;
                    notes.push("PASS secret assign".into());
                } else {
                    failed += 1;
                    notes.push("FAIL secret assign".into());
                }
            }
        }
    }

    let blob = notes.join("\n");
    if failed == 0 && passed > 0 {
        VerificationResult::confirmed(finding.id.clone(), "static_ir_v1", blob)
    } else if passed == 0 {
        VerificationResult::false_positive(finding.id.clone(), "static_ir_v1", blob)
    } else {
        VerificationResult::unconfirmed(finding.id.clone(), "static_ir_v1", blob)
    }
}

fn read(root: &Path, rel: &str) -> Option<String> {
    fs::read_to_string(root.join(rel)).ok()
}

fn build_poc(finding: &Finding) -> ProofOfConcept {
    let class = PocClass::from_rule_or_cwe(&finding.rule_id, finding.cwe.as_deref());
    let steps = vec![
        format!("Open authorized codebase file: {}", finding.location.path),
        format!("Go to line {}", finding.location.start_line),
        format!("Observe: {}", finding.message),
        "Confirm no compensating control in adjacent lines".into(),
        "Document business impact for report".into(),
    ];
    let curl = match class {
        PocClass::DangerousEval | PocClass::CommandInjection | PocClass::SqlInjection => Some(
            "# Local fixture only — do not run against unauthorized hosts\n# curl -s 'http://127.0.0.1:8080/vuln?q=test'".into(),
        ),
        _ => None,
    };
    ProofOfConcept::new(class, steps, curl)
}
