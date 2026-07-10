use bugbee_core::{BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Severity};
use bugbee_index::RepoIndex;
use regex::Regex;

pub fn scan_secrets(index: &RepoIndex, weights: &BrsWeights) -> anyhow::Result<Vec<Finding>> {
    let patterns: Vec<(&str, Regex, Severity)> = vec![
        (
            "aws-access-key",
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            Severity::Critical,
        ),
        (
            "generic-api-key",
            Regex::new(r#"(?i)(api[_-]?key|secret|token)\s*[=:]\s*['"][A-Za-z0-9_\-]{16,}['"]"#)
                .unwrap(),
            Severity::High,
        ),
        (
            "private-key-header",
            Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----").unwrap(),
            Severity::Critical,
        ),
        (
            "jwt-token",
            Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap(),
            Severity::Medium,
        ),
    ];

    let mut out = Vec::new();
    for file in &index.files {
        let content = match index.read_file(&file.path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (id, re, sev) in &patterns {
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    let mut f = Finding::new(
                        format!("Secret detected: {id}"),
                        format!("Potential secret material in {}:{}", file.path, i + 1),
                        *sev,
                        "secrets",
                    );
                    f.cwe = vec!["CWE-798".into()];
                    f.owasp = vec!["A02:2025".into(), "A07:2025".into()];
                    f.confidence = 0.8;
                    f.exploitability = 0.7;
                    f.evidence = Evidence {
                        rule_id: Some(format!("secrets.{id}")),
                        has_sink: true,
                        has_source: true,
                        has_path: true,
                        has_repro: false,
                        missing_sanitizer_check: false,
                        traces: vec![format!("{}:{}", file.path, i + 1)],
                        dataflow: None,
                        agent_notes: Some(
                            "Secret scanners never forward raw secrets to LLMs".into(),
                        ),
                    };
                    f.add_location(FindingLocation {
                        file: file.path.clone(),
                        start_line: (i + 1) as u32,
                        end_line: (i + 1) as u32,
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
    }
    Ok(out)
}
