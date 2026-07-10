//! Deterministic detection: pattern rules, secrets, simple taint heuristics.

pub mod rules;
pub mod secrets;
pub mod taint;

use bugbee_core::{BrsWeights, Finding};
use bugbee_index::RepoIndex;

use crate::rules::{load_rules_dir, RulePack};
use crate::secrets::scan_secrets;
use crate::taint::scan_taint_heuristics;

pub struct HuntEngine {
    pub packs: Vec<RulePack>,
    pub weights: BrsWeights,
}

impl HuntEngine {
    pub fn with_rules_dirs(
        dirs: &[std::path::PathBuf],
        weights: BrsWeights,
    ) -> anyhow::Result<Self> {
        let mut packs = Vec::new();
        let mut seen_dirs = std::collections::HashSet::new();
        for d in dirs {
            let key = d.canonicalize().unwrap_or_else(|_| d.clone());
            if !seen_dirs.insert(key) {
                continue;
            }
            if d.exists() {
                packs.extend(load_rules_dir(d)?);
            }
        }
        // Dedup packs by name (same rules file resolved via multiple paths)
        let mut seen_names = std::collections::HashSet::new();
        packs.retain(|p| seen_names.insert(p.name.clone()));
        Ok(Self { packs, weights })
    }

    pub fn run(&self, index: &RepoIndex) -> anyhow::Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for pack in &self.packs {
            findings.extend(pack.scan(index, &self.weights)?);
        }

        findings.extend(scan_secrets(index, &self.weights)?);
        findings.extend(scan_taint_heuristics(index, &self.weights)?);

        // Dedup by rule_id + primary location
        let mut seen = std::collections::HashSet::new();
        findings.retain(|f| {
            let loc = f
                .locations
                .first()
                .map(|l| format!("{}:{}", l.file, l.start_line))
                .unwrap_or_default();
            let key = format!(
                "{}|{}|{}",
                f.evidence.rule_id.as_deref().unwrap_or(&f.title),
                loc,
                f.title
            );
            seen.insert(key)
        });

        findings.sort_by(|a, b| {
            b.brs
                .partial_cmp(&a.brs)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(findings)
    }
}
