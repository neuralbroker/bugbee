use bugbee_core::{BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Severity};
use bugbee_index::RepoIndex;
use once_cell::sync::Lazy;
use regex::Regex;

struct SecretPattern {
    id: &'static str,
    re: Regex,
    severity: Severity,
}

static PATTERNS: Lazy<Vec<SecretPattern>> = Lazy::new(|| {
    vec![
        SecretPattern {
            id: "aws-access-key",
            re: Regex::new(r"AKIA[0-9A-Z]{16}").expect("valid secrets regex"),
            severity: Severity::Critical,
        },
        SecretPattern {
            id: "generic-api-key",
            re: Regex::new(
                r#"(?i)(api[_-]?key|secret|token)\s*[=:]\s*['"][A-Za-z0-9_\-]{16,}['"]"#,
            )
            .expect("valid secrets regex"),
            severity: Severity::High,
        },
        SecretPattern {
            id: "private-key-header",
            re: Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----")
                .expect("valid secrets regex"),
            severity: Severity::Critical,
        },
        SecretPattern {
            id: "jwt-token",
            re: Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+")
                .expect("valid secrets regex"),
            severity: Severity::Medium,
        },
        SecretPattern {
            id: "github-token",
            re: Regex::new(r"\b(gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
                .expect("valid secrets regex"),
            severity: Severity::Critical,
        },
        SecretPattern {
            id: "openai-style-key",
            re: Regex::new(r"\b(sk-(?:proj-|svcacct-|ant-)?[A-Za-z0-9_-]{20,})\b")
                .expect("valid secrets regex"),
            severity: Severity::Critical,
        },
    ]
});

pub fn scan_secrets(index: &RepoIndex, weights: &BrsWeights) -> anyhow::Result<Vec<Finding>> {
    let mut out = Vec::new();
    for file in &index.files {
        let content = match index.read_file(&file.path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for pattern in PATTERNS.iter() {
            for (i, line) in content.lines().enumerate() {
                if !pattern.re.is_match(line) {
                    continue;
                }
                let line_no = (i + 1) as u32;
                let mut f = Finding::new(
                    format!("Secret detected: {}", pattern.id),
                    format!("Potential secret material in {}:{}", file.path, line_no),
                    pattern.severity,
                    "secrets",
                );
                f.cwe = vec!["CWE-798".into()];
                f.owasp = vec!["A02:2025".into(), "A07:2025".into()];
                f.confidence = 0.8;
                f.exploitability = 0.7;
                f.evidence = Evidence {
                    rule_id: Some(format!("secrets.{}", pattern.id)),
                    has_sink: true,
                    has_source: true,
                    has_path: true,
                    has_repro: false,
                    missing_sanitizer_check: false,
                    // Never persist the raw secret material.
                    traces: vec![format!("{}:{}: [redacted match]", file.path, line_no)],
                    dataflow: None,
                    agent_notes: Some("Secret scanners never forward raw secrets to LLMs".into()),
                };
                f.add_location(FindingLocation {
                    file: file.path.clone(),
                    start_line: line_no,
                    end_line: line_no,
                    start_col: None,
                    end_col: None,
                    role: LocationRole::Sink,
                    snippet: Some("[redacted snippet]".into()),
                });
                f.recompute_scores(weights);
                out.push(f);
            }
        }
    }
    Ok(out)
}
