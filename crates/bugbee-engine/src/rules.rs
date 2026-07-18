use std::fs;
use std::path::{Path, PathBuf};

use bugbee_core::{Error, Evidence, Finding, Location, Result, Severity, SourceKind};
use regex::Regex;
use serde::Deserialize;
use tracing::{debug, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct RulePack {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    /// Regex matched against each line.
    pub pattern: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub cwe: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// File path globs (simple suffix / contains checks for v0).
    #[serde(default)]
    pub paths: Vec<String>,
}

impl Rule {
    pub fn severity(&self) -> Severity {
        Severity::parse(&self.severity).unwrap_or(Severity::Medium)
    }

    fn path_matches(&self, path: &Path) -> bool {
        if self.paths.is_empty() {
            return true;
        }
        let s = path.to_string_lossy();
        self.paths.iter().any(|p| {
            if let Some(ext) = p.strip_prefix("*.") {
                s.ends_with(&format!(".{ext}")) || s.ends_with(ext)
            } else {
                s.contains(p.trim_start_matches('*'))
            }
        })
    }

    pub fn apply(&self, path: &Path, content: &str) -> Result<Vec<Finding>> {
        if !self.path_matches(path) {
            return Ok(Vec::new());
        }
        let re = Regex::new(&self.pattern)
            .map_err(|e| Error::Engine(format!("bad regex in {}: {e}", self.id)))?;
        let mut out = Vec::new();
        let rel = path.to_string_lossy();
        for (idx, line) in content.lines().enumerate() {
            if re.is_match(line) {
                let line_no = (idx + 1) as u32;
                let mut finding = Finding::new(
                    &self.id,
                    &self.title,
                    &self.message,
                    self.severity(),
                    SourceKind::Rule,
                    Location::line(rel.as_ref(), line_no).with_snippet(line.trim()),
                );
                finding.cwe = self.cwe.clone();
                finding.tags = self.tags.clone();
                finding.push_evidence(Evidence {
                    kind: "pattern_match".into(),
                    detail: format!("rule {} matched line {line_no}", self.id),
                    location: Some(Location::line(rel.as_ref(), line_no)),
                });
                out.push(finding);
            }
        }
        Ok(out)
    }
}

pub fn load_rules_from_dir(dir: impl AsRef<Path>) -> Result<Vec<Rule>> {
    let dir = dir.as_ref();
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut rules = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "yaml" && ext != "yml" {
            continue;
        }
        match load_pack(path) {
            Ok(pack) => {
                debug!(pack = %pack.id, n = pack.rules.len(), "loaded rule pack");
                rules.extend(pack.rules);
            }
            Err(e) => warn!(path = %path.display(), error = %e, "skip rule pack"),
        }
    }
    Ok(rules)
}

/// Built-in foundation pack so a single binary works without external files.
pub fn embedded_rules() -> Result<Vec<Rule>> {
    const YAML: &str = include_str!("../../../rules/owasp-basics/injection.yaml");
    let pack: RulePack =
        serde_yaml::from_str(YAML).map_err(|e| Error::Engine(format!("embedded rules: {e}")))?;
    Ok(pack.rules)
}

pub fn load_pack(path: impl AsRef<Path>) -> Result<RulePack> {
    let text = fs::read_to_string(path.as_ref())?;
    let pack: RulePack = serde_yaml::from_str(&text)
        .map_err(|e| Error::Engine(format!("yaml {}: {e}", path.as_ref().display())))?;
    Ok(pack)
}

/// Discover rule directories: project `rules/`, then bundled relative to executable optional.
pub fn discover_rule_dirs(project_root: &Path, extra: &[String]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let local = project_root.join("rules");
    if local.is_dir() {
        dirs.push(local);
    }
    // Workspace rules when developing Bugbee itself / monorepo layouts.
    if let Ok(cwd) = std::env::current_dir() {
        let cand = cwd.join("rules");
        if cand.is_dir() && !dirs.contains(&cand) {
            dirs.push(cand);
        }
    }
    for e in extra {
        let p = PathBuf::from(e);
        if p.is_dir() {
            dirs.push(p);
        }
    }
    dirs
}
