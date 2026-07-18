use std::fs;
use std::path::{Path, PathBuf};

use bugbee_core::{Finding, ProjectConfig, Result};
use ignore::WalkBuilder;
use tracing::{info, warn};

use crate::rules::{discover_rule_dirs, embedded_rules, load_rules_from_dir, Rule};
use crate::secrets::scan_secrets;

#[derive(Debug, Clone)]
pub struct HuntOptions {
    pub root: PathBuf,
    pub run_secrets: bool,
    pub run_rules: bool,
    pub extra_rule_dirs: Vec<String>,
}

impl HuntOptions {
    pub fn from_config(root: impl Into<PathBuf>, cfg: &ProjectConfig) -> Self {
        Self {
            root: root.into(),
            run_secrets: cfg.hunt.secrets,
            run_rules: cfg.hunt.rules,
            extra_rule_dirs: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct HuntSummary {
    pub files_scanned: usize,
    pub findings: Vec<Finding>,
    pub rules_loaded: usize,
}

/// Run deterministic engines over a repository.
pub fn hunt(opts: &HuntOptions) -> Result<HuntSummary> {
    let mut summary = HuntSummary::default();

    let rules: Vec<Rule> = if opts.run_rules {
        let mut all = Vec::new();
        for dir in discover_rule_dirs(&opts.root, &opts.extra_rule_dirs) {
            match load_rules_from_dir(&dir) {
                Ok(mut r) => all.append(&mut r),
                Err(e) => warn!(error = %e, "rule load failed"),
            }
        }
        // Also load packs shipped next to the binary via BUGBEE_RULES
        if let Ok(extra) = std::env::var("BUGBEE_RULES") {
            for part in extra.split(':') {
                if let Ok(mut r) = load_rules_from_dir(part) {
                    all.append(&mut r);
                }
            }
        }
        // Always include embedded foundation pack (dedup by rule id later)
        match embedded_rules() {
            Ok(embedded) => {
                for r in embedded {
                    if !all.iter().any(|x| x.id == r.id) {
                        all.push(r);
                    }
                }
            }
            Err(e) => warn!(error = %e, "embedded rules failed"),
        }
        summary.rules_loaded = all.len();
        all
    } else {
        Vec::new()
    };

    info!(
        root = %opts.root.display(),
        rules = summary.rules_loaded,
        secrets = opts.run_secrets,
        "starting hunt"
    );

    let walker = WalkBuilder::new(&opts.root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "target" | "node_modules" | ".git" | ".bugbee" | "dist" | "build" | ".next"
            )
        })
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "walk error");
                continue;
            }
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if !is_textish(path) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        summary.files_scanned += 1;

        let rel = pathstrip(&opts.root, path);

        if opts.run_secrets {
            for mut f in scan_secrets(path, &content) {
                f.location.path = rel.clone();
                summary.findings.push(f);
            }
        }

        for rule in &rules {
            match rule.apply(path, &content) {
                Ok(mut found) => {
                    for f in &mut found {
                        f.location.path = rel.clone();
                    }
                    summary.findings.append(&mut found);
                }
                Err(e) => warn!(rule = %rule.id, error = %e, "rule apply failed"),
            }
        }
    }

    // Dedup by id (keep highest ecs)
    summary.findings.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    summary.findings.dedup_by(|a, b| {
        if a.id == b.id {
            if a.ecs < b.ecs {
                *a = b.clone();
            }
            true
        } else {
            false
        }
    });

    info!(
        files = summary.files_scanned,
        findings = summary.findings.len(),
        "hunt complete"
    );
    Ok(summary)
}

fn pathstrip(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn is_textish(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "go"
            | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "java"
            | "kt"
            | "php"
            | "rb"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "swift"
            | "scala"
            | "sh"
            | "bash"
            | "zsh"
            | "ps1"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "jsonc"
            | "env"
            | "ini"
            | "cfg"
            | "conf"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "scss"
            | "sql"
            | "md"
            | "txt"
            | "dockerfile"
            | "tf"
            | "hcl"
            | "vue"
            | "svelte"
            | "gradle"
            | "properties"
    ) || path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        matches!(
            n,
            "Dockerfile" | "Makefile" | "Jenkinsfile" | ".env" | ".env.local" | ".env.example"
        )
    })
}
