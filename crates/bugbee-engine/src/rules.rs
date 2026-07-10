use std::fs;
use std::path::Path;

use bugbee_core::{BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Severity};
use bugbee_index::{Lang, RepoIndex};
use regex::Regex;
use serde::Deserialize;

const BUILTIN_OWASP_2025_RULES: &str =
    include_str!("../../../rules/owasp-2025/injection-crypto-misconfig.yaml");

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    #[serde(default)]
    pub cwe: Vec<String>,
    #[serde(default)]
    pub owasp: Vec<String>,
    pub category: String,
    pub languages: Vec<String>,
    pub pattern: String,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub exploitability: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct RulePack {
    pub name: String,
    pub rules: Vec<Rule>,
}

impl RulePack {
    pub fn scan(&self, index: &RepoIndex, weights: &BrsWeights) -> anyhow::Result<Vec<Finding>> {
        let mut out = Vec::new();
        for rule in &self.rules {
            let re = Regex::new(&rule.pattern)
                .map_err(|e| anyhow::anyhow!("bad regex in {}: {e}", rule.id))?;
            for file in &index.files {
                let lang = file.lang.as_str();
                if !rule.languages.is_empty()
                    && !rule.languages.iter().any(|l| l == lang || l == "generic")
                {
                    continue;
                }
                // Skip Other
                if file.lang == Lang::Other {
                    continue;
                }
                let content = match index.read_file(&file.path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        let sev = parse_severity(&rule.severity);
                        let mut f = Finding::new(
                            &rule.title,
                            format!("{} — matched in {}:{}", rule.message, file.path, i + 1),
                            sev,
                            &rule.category,
                        );
                        f.cwe = rule.cwe.clone();
                        f.owasp = rule.owasp.clone();
                        f.confidence = rule.confidence.unwrap_or(0.65);
                        f.exploitability = rule.exploitability.unwrap_or(0.5);
                        f.evidence = Evidence {
                            rule_id: Some(rule.id.clone()),
                            has_sink: true,
                            has_source: false,
                            has_path: false,
                            has_repro: false,
                            missing_sanitizer_check: true,
                            traces: vec![format!("{}:{}: {}", file.path, i + 1, line.trim())],
                            dataflow: None,
                            agent_notes: Some(format!("rule pack: {}", self.name)),
                        };
                        f.add_location(FindingLocation {
                            file: file.path.clone(),
                            start_line: (i + 1) as u32,
                            end_line: (i + 1) as u32,
                            start_col: None,
                            end_col: None,
                            role: LocationRole::Sink,
                            snippet: Some(line.trim().to_string()),
                        });
                        f.recompute_scores(weights);
                        out.push(f);
                    }
                }
            }
        }
        Ok(out)
    }
}

pub fn load_rules_dir(dir: &Path) -> anyhow::Result<Vec<RulePack>> {
    let mut packs = Vec::new();
    if !dir.exists() {
        return Ok(packs);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml")
            && path.extension().and_then(|e| e.to_str()) != Some("yml")
        {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        let rules: Vec<Rule> = serde_yaml::from_str(&raw)?;
        packs.push(RulePack {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("rules")
                .to_string(),
            rules,
        });
    }
    Ok(packs)
}

/// The default defensive rule pack is embedded in every Bugbee binary so a
/// downloaded release has the same baseline coverage as a source checkout.
/// Filesystem rule packs remain supported for organization-specific rules.
pub fn builtin_rule_packs() -> anyhow::Result<Vec<RulePack>> {
    let rules: Vec<Rule> = serde_yaml::from_str(BUILTIN_OWASP_2025_RULES)
        .map_err(|error| anyhow::anyhow!("invalid embedded OWASP 2025 rule pack: {error}"))?;
    Ok(vec![RulePack {
        name: "builtin-owasp-2025".into(),
        rules,
    }])
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Info,
    }
}
