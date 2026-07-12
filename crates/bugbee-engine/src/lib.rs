//! Deterministic detection: pattern rules, secrets, simple taint heuristics.

pub mod rules;
pub mod secrets;
pub mod taint;

use bugbee_core::{BrsWeights, Finding};
use bugbee_index::RepoIndex;

use crate::rules::{builtin_rule_packs, load_rules_dir, RulePack};
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
        // External files take precedence over the embedded baseline, allowing
        // enterprises to replace a built-in rule by id without duplicating it.
        packs.extend(builtin_rule_packs()?);
        let mut seen_rule_ids = std::collections::HashSet::new();
        for pack in &mut packs {
            pack.dedup_rules(&mut seen_rule_ids);
        }
        packs.retain(|pack| !pack.is_empty());
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::*;
    use bugbee_index::Indexer;

    fn workspace_path(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
    }

    fn scan_fixture(name: &str) -> Vec<Finding> {
        let root = workspace_path(&format!("fixtures/{name}"));
        let rules = workspace_path("rules/owasp-2025");
        let index = Indexer::new(root).build().expect("index fixture");
        HuntEngine::with_rules_dirs(&[rules], BrsWeights::default())
            .expect("load rules")
            .run(&index)
            .expect("scan fixture")
    }

    fn scan_builtin_fixture(name: &str) -> Vec<Finding> {
        let root = workspace_path(&format!("fixtures/{name}"));
        let index = Indexer::new(root).build().expect("index fixture");
        HuntEngine::with_rules_dirs(&[], BrsWeights::default())
            .expect("load built-in rules")
            .run(&index)
            .expect("scan fixture")
    }

    fn rule_ids(findings: &[Finding]) -> BTreeSet<String> {
        findings
            .iter()
            .filter_map(|finding| finding.evidence.rule_id.clone())
            .collect()
    }

    #[test]
    fn python_fixture_has_expected_security_findings_without_duplicates() {
        let findings = scan_fixture("python-vuln");
        let expected = BTreeSet::from([
            "py-eval-injection".into(),
            "py-flask-debug".into(),
            "py-pickle-load".into(),
            "taint.py-command-injection".into(),
            "taint.py-sql-injection".into(),
        ]);
        assert_eq!(rule_ids(&findings), expected);

        let identities = findings
            .iter()
            .map(|finding| finding.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(identities.len(), findings.len());
    }

    #[test]
    fn javascript_fixture_has_expected_security_findings() {
        let findings = scan_fixture("js-vuln");
        let expected = BTreeSet::from([
            "js-disable-tls-verify".into(),
            "js-eval".into(),
            "taint.js-sql-injection".into(),
            "taint.js-xss".into(),
        ]);
        assert_eq!(rule_ids(&findings), expected);
    }

    #[test]
    fn go_fixture_has_expected_security_findings() {
        let findings = scan_fixture("go-vuln");
        let expected = BTreeSet::from([
            "go-http-listen-all".into(),
            "go-md5-password".into(),
            "taint.go-command-injection".into(),
        ]);
        assert_eq!(rule_ids(&findings), expected);
    }

    #[test]
    fn separated_python_source_and_sink_do_not_create_a_taint_finding() {
        let findings = scan_fixture("python-safe");
        assert!(!rule_ids(&findings).contains("taint.py-command-injection"));
    }

    #[test]
    fn embedded_rule_pack_keeps_release_binaries_useful_without_rule_files() {
        let findings = scan_builtin_fixture("python-vuln");
        let ids = rule_ids(&findings);
        assert!(ids.contains("py-eval-injection"));
        assert!(ids.contains("py-pickle-load"));
        assert!(ids.contains("py-flask-debug"));
    }
}
