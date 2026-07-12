use std::fs;
use std::path::Path;

use bugbee_core::{
    BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Redactor, Severity,
};
use bugbee_index::{Lang, RepoIndex};
use regex::Regex;
use serde::Deserialize;

const BUILTIN_OWASP_CORE: &str =
    include_str!("../../../rules/owasp-2025/injection-crypto-misconfig.yaml");
const BUILTIN_OWASP_WEB: &str = include_str!("../../../rules/owasp-2025/web-auth-ssrf-xss.yaml");
const BUILTIN_INDIA_APPSEC: &str =
    include_str!("../../../rules/india-appsec/india-gov-edu-enterprise.yaml");

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

/// A rule with a precompiled regex so packs are not recompiled per file.
#[derive(Debug, Clone)]
struct CompiledRule {
    meta: Rule,
    re: Regex,
    severity: Severity,
}

#[derive(Debug, Clone)]
pub struct RulePack {
    pub name: String,
    pub rules: Vec<Rule>,
    compiled: Vec<CompiledRule>,
}

impl RulePack {
    pub fn from_rules(name: impl Into<String>, rules: Vec<Rule>) -> anyhow::Result<Self> {
        let name = name.into();
        let mut compiled = Vec::with_capacity(rules.len());
        for rule in &rules {
            let re = Regex::new(&rule.pattern)
                .map_err(|e| anyhow::anyhow!("bad regex in {}: {e}", rule.id))?;
            compiled.push(CompiledRule {
                meta: rule.clone(),
                re,
                severity: parse_severity(&rule.severity),
            });
        }
        Ok(Self {
            name,
            rules,
            compiled,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.compiled.is_empty()
    }

    /// Keep the first occurrence of each rule id and drop later duplicates.
    pub fn dedup_rules(&mut self, seen: &mut std::collections::HashSet<String>) {
        let mut keep = Vec::with_capacity(self.compiled.len());
        let mut keep_meta = Vec::with_capacity(self.rules.len());
        for compiled in self.compiled.drain(..) {
            if seen.insert(compiled.meta.id.clone()) {
                keep_meta.push(compiled.meta.clone());
                keep.push(compiled);
            }
        }
        self.rules = keep_meta;
        self.compiled = keep;
    }

    pub fn scan(&self, index: &RepoIndex, weights: &BrsWeights) -> anyhow::Result<Vec<Finding>> {
        let redactor = Redactor::enterprise();
        let mut out = Vec::new();

        for file in &index.files {
            if file.lang == Lang::Other {
                continue;
            }
            let lang = file.lang.as_str();
            let content = match index.read_file(&file.path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for rule in &self.compiled {
                if !rule.meta.languages.is_empty()
                    && !rule
                        .meta
                        .languages
                        .iter()
                        .any(|l| l == lang || l == "generic")
                {
                    continue;
                }

                for (i, line) in content.lines().enumerate() {
                    if !rule.re.is_match(line) {
                        continue;
                    }
                    let line_no = (i + 1) as u32;
                    let safe_snippet = redactor.redact(line.trim());
                    let mut f = Finding::new(
                        &rule.meta.title,
                        format!(
                            "{} — matched in {}:{}",
                            rule.meta.message, file.path, line_no
                        ),
                        rule.severity,
                        &rule.meta.category,
                    );
                    f.cwe = rule.meta.cwe.clone();
                    f.owasp = rule.meta.owasp.clone();
                    f.confidence = rule.meta.confidence.unwrap_or(0.65);
                    f.exploitability = rule.meta.exploitability.unwrap_or(0.5);
                    f.evidence = Evidence {
                        rule_id: Some(rule.meta.id.clone()),
                        has_sink: true,
                        has_source: false,
                        has_path: false,
                        has_repro: false,
                        missing_sanitizer_check: true,
                        traces: vec![format!("{}:{}: {}", file.path, line_no, safe_snippet)],
                        dataflow: None,
                        agent_notes: Some(format!("rule pack: {}", self.name)),
                    };
                    f.add_location(FindingLocation {
                        file: file.path.clone(),
                        start_line: line_no,
                        end_line: line_no,
                        start_col: None,
                        end_col: None,
                        role: LocationRole::Sink,
                        snippet: Some(safe_snippet),
                    });
                    f.recompute_scores(weights);
                    out.push(f);
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
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("rules")
            .to_string();
        packs.push(RulePack::from_rules(name, rules)?);
    }
    Ok(packs)
}

/// Embedded baselines ship inside every release binary:
///
/// - OWASP-focused core + web rules
/// - India AppSec pack (gov / edu / BFSI / enterprise hygiene, CERT-In oriented)
///
/// Filesystem rule packs remain supported for organization-specific rules.
pub fn builtin_rule_packs() -> anyhow::Result<Vec<RulePack>> {
    let mut packs = Vec::new();
    for (name, raw) in [
        ("builtin-owasp-2025-core", BUILTIN_OWASP_CORE),
        ("builtin-owasp-2025-web", BUILTIN_OWASP_WEB),
        ("builtin-india-appsec", BUILTIN_INDIA_APPSEC),
    ] {
        let rules: Vec<Rule> = serde_yaml::from_str(raw)
            .map_err(|error| anyhow::anyhow!("invalid embedded pack {name}: {error}"))?;
        packs.push(RulePack::from_rules(name, rules)?);
    }
    Ok(packs)
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
