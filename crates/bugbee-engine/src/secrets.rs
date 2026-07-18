use std::path::Path;
use std::sync::OnceLock;

use bugbee_core::{Evidence, Finding, Location, Severity, SourceKind};
use regex::Regex;

struct SecretRule {
    id: &'static str,
    title: &'static str,
    severity: Severity,
    re: Regex,
}

fn secret_rules() -> &'static [SecretRule] {
    static RULES: OnceLock<Vec<SecretRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            SecretRule {
                id: "secrets.aws_access_key",
                title: "AWS access key id",
                severity: Severity::Critical,
                re: Regex::new(r"AKIA[0-9A-Z]{16}").expect("re"),
            },
            SecretRule {
                id: "secrets.github_pat",
                title: "GitHub personal access token",
                severity: Severity::Critical,
                re: Regex::new(r"ghp_[A-Za-z0-9]{20,}").expect("re"),
            },
            SecretRule {
                id: "secrets.openai_key",
                title: "OpenAI-style API key",
                severity: Severity::High,
                re: Regex::new(r"sk-[A-Za-z0-9]{20,}").expect("re"),
            },
            SecretRule {
                id: "secrets.xai_key",
                title: "xAI API key",
                severity: Severity::High,
                re: Regex::new(r"xai-[A-Za-z0-9]{20,}").expect("re"),
            },
            SecretRule {
                id: "secrets.private_key_header",
                title: "Private key material",
                severity: Severity::Critical,
                re: Regex::new(r"-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----").expect("re"),
            },
            SecretRule {
                id: "secrets.generic_password_assign",
                title: "Hardcoded password assignment",
                severity: Severity::Medium,
                re: Regex::new(r#"(?i)(password|passwd|pwd)\s*=\s*['"][^'"]{6,}['"]"#).expect("re"),
            },
        ]
    })
}

fn skip_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/target/")
        || s.contains("/node_modules/")
        || s.contains("/.git/")
        || s.contains("/.bugbee/")
        || s.ends_with(".lock")
        || s.ends_with(".min.js")
}

/// Scan file content for high-signal secret patterns.
pub fn scan_secrets(path: &Path, content: &str) -> Vec<Finding> {
    if skip_path(path) {
        return Vec::new();
    }
    // Skip huge files
    if content.len() > 1_500_000 {
        return Vec::new();
    }
    let rel = path.to_string_lossy();
    let mut out = Vec::new();
    for rule in secret_rules() {
        for (idx, line) in content.lines().enumerate() {
            if rule.re.is_match(line) {
                let line_no = (idx + 1) as u32;
                // Avoid reporting the secret value in snippet — truncate mid-token.
                let snippet = redact_line_snippet(line);
                let mut finding = Finding::new(
                    rule.id,
                    rule.title,
                    format!("{} detected in source", rule.title),
                    rule.severity,
                    SourceKind::Secrets,
                    Location::line(rel.as_ref(), line_no).with_snippet(snippet),
                );
                finding.tags = vec!["secrets".into(), "credentials".into()];
                finding.push_evidence(Evidence {
                    kind: "secret_pattern".into(),
                    detail: format!("{} matched", rule.id),
                    location: Some(Location::line(rel.as_ref(), line_no)),
                });
                out.push(finding);
            }
        }
    }
    out
}

fn redact_line_snippet(line: &str) -> String {
    let t = line.trim();
    if t.len() <= 24 {
        return "***".into();
    }
    format!("{}…***", &t[..12.min(t.len())])
}
