use bugbee_core::{Finding, PocClass};

/// Structural sandbox — verifies PoC payloads are syntactically valid
/// and structurally match the expected vulnerability class WITHOUT
/// executing them. Safe alternative to full WASM sandboxing (Horizon B).
pub struct StructuralSandbox;

impl StructuralSandbox {
    pub fn verify(finding: &Finding) -> SandboxVerdict {
        let Some(ref poc) = finding.poc else {
            return SandboxVerdict::Skipped("no poc to verify".into());
        };

        let mut checks = Vec::new();

        match poc.class {
            PocClass::SqlInjection => {
                checks.push(check_payload_patterns(&poc.steps, &["'", "\"", "OR ", " UNION ", "--"]));
                checks.push(check_has_statement(&poc.steps, &["SELECT", "WHERE", "INSERT"]));
            }
            PocClass::Xss => {
                checks.push(check_payload_patterns(&poc.steps, &["<script>", "onerror=", "onload=", "alert("]));
                checks.push(check_has_context(&poc.steps, &["input", "parameter", "query", "reflect"]));
            }
            PocClass::CommandInjection => {
                checks.push(check_payload_patterns(&poc.steps, &[";", "|", "`", "$("]));
                checks.push(check_has_context(&poc.steps, &["shell", "exec", "command", "system"]));
            }
            PocClass::PathTraversal => {
                checks.push(check_payload_patterns(&poc.steps, &["../", "..\\", "%2e%2e"]));
                checks.push(check_has_context(&poc.steps, &["file", "path", "read", "include"]));
            }
            PocClass::Ssrf => {
                checks.push(check_has_context(&poc.steps, &["http", "url", "fetch", "request", "internal"]));
            }
            PocClass::Idor => {
                checks.push(check_has_context(&poc.steps, &["id", "user", "account", "object", "param"]));
            }
            PocClass::HardcodedSecret => {
                checks.push(check_has_context(&poc.steps, &["password", "secret", "key", "token", "credential", "hardcoded"]));
            }
            PocClass::WeakCrypto => {
                checks.push(check_has_context(&poc.steps, &["md5", "sha1", "weak", "cipher", "hash"]));
            }
            PocClass::DangerousEval => {
                checks.push(check_payload_patterns(&poc.steps, &["eval(", "exec(", "system(", "popen("]));
                checks.push(check_has_context(&poc.steps, &["input", "user", "dynamic", "runtime"]));
            }
            PocClass::Other => {
                checks.push(CheckResult::Pass("generic poc, no class-specific checks".into()));
            }
        }

        if let Some(ref curl) = poc.curl_template {
            checks.push(check_curl_syntax(curl));
        }

        let all_pass = checks.iter().all(|c| matches!(c, CheckResult::Pass(_)));
        let details: Vec<String> = checks.iter().map(|c| c.summary()).collect();

        if all_pass {
            SandboxVerdict::Valid { checks: details }
        } else {
            SandboxVerdict::Invalid { failed: details }
        }
    }

    pub fn validate_curl_template(template: &str) -> CheckResult {
        check_curl_syntax(template)
    }
}

#[derive(Debug, Clone)]
pub enum SandboxVerdict {
    Valid { checks: Vec<String> },
    Invalid { failed: Vec<String> },
    Skipped(String),
}

impl SandboxVerdict {
    pub fn is_valid(&self) -> bool {
        matches!(self, SandboxVerdict::Valid { .. })
    }

    pub fn summary(&self) -> String {
        match self {
            SandboxVerdict::Valid { checks } => {
                format!("structural sandbox: {} checks passed", checks.len())
            }
            SandboxVerdict::Invalid { failed } => {
                format!("structural sandbox: {} checks failed: {}", failed.len(), failed.join("; "))
            }
            SandboxVerdict::Skipped(reason) => {
                format!("sandbox skipped: {reason}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheckResult {
    Pass(String),
    Fail(String),
}

impl CheckResult {
    pub fn summary(&self) -> String {
        match self {
            CheckResult::Pass(msg) => format!("ok: {msg}"),
            CheckResult::Fail(msg) => format!("fail: {msg}"),
        }
    }
}

fn check_payload_patterns(steps: &[String], patterns: &[&str]) -> CheckResult {
    let all_text = steps.join(" ");
    let found: Vec<&str> = patterns
        .iter()
        .filter(|p| all_text.to_ascii_lowercase().contains(&p.to_ascii_lowercase()))
        .copied()
        .collect();
    if found.is_empty() {
        CheckResult::Fail(format!("no expected payload patterns found: {:?}", patterns))
    } else {
        CheckResult::Pass(format!("found payload patterns: {:?}", found))
    }
}

fn check_has_statement(steps: &[String], keywords: &[&str]) -> CheckResult {
    let all_text = steps.join(" ");
    let found: Vec<&str> = keywords
        .iter()
        .filter(|k| all_text.to_ascii_lowercase().contains(&k.to_ascii_lowercase()))
        .copied()
        .collect();
    if found.is_empty() {
        CheckResult::Fail(format!("no expected statements found: {:?}", keywords))
    } else {
        CheckResult::Pass(format!("found statements: {:?}", found))
    }
}

fn check_has_context(steps: &[String], keywords: &[&str]) -> CheckResult {
    let all_text = steps.join(" ");
    let found: Vec<&str> = keywords
        .iter()
        .filter(|k| all_text.to_ascii_lowercase().contains(&k.to_ascii_lowercase()))
        .copied()
        .collect();
    if found.is_empty() {
        CheckResult::Fail(format!("no context keywords found: {:?}", keywords))
    } else {
        CheckResult::Pass(format!("found context: {:?}", found))
    }
}

fn check_curl_syntax(curl: &str) -> CheckResult {
    let trimmed = curl.trim();
    if !trimmed.starts_with("curl ") && !trimmed.starts_with("http") {
        return CheckResult::Fail("not a curl command or http url".into());
    }

    if trimmed.contains("internal") || trimmed.contains("127.0.0.1") || trimmed.contains("localhost") {
        return CheckResult::Pass("targets local/fixture — safe".into());
    }

    let url_part = trimmed
        .split_whitespace()
        .find(|s| s.starts_with("http"))
        .unwrap_or("");
    if url_part.is_empty() {
        return CheckResult::Fail("no url found in curl template".into());
    }

    CheckResult::Pass(format!("valid curl targeting {}", url_part))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bugbee_core::{Finding, FindingId, Location, Severity};

    fn make_test_finding(class: PocClass, steps: Vec<&str>, curl: Option<&str>) -> Finding {
        Finding {
            id: FindingId("test-1".into()),
            rule_id: "test".into(),
            title: "test".into(),
            message: "test".into(),
            severity: Severity::High,
            status: bugbee_core::FindingStatus::Draft,
            source: bugbee_core::SourceKind::Rule,
            location: Location::line("test.py", 1),
            evidence: vec![],
            brs: 50,
            ecs: 0,
            cwe: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            neural_confidence: 0.0,
            symbolic_verdict: bugbee_core::adjudication::SymbolicVerdict::None,
            adjudicated_state: bugbee_core::adjudication::AdjudicationState::Pending,
            adjudication_reason: None,
            poc: Some(bugbee_core::ProofOfConcept::new(
                class,
                steps.into_iter().map(String::from).collect(),
                curl.map(String::from),
            )),
            chain_dependencies: vec![],
            verified: false,
            swarm_agent: None,
        }
    }

    #[test]
    fn test_valid_sqli_poc() {
        let f = make_test_finding(
            PocClass::SqlInjection,
            vec!["inject ' OR '1'='1 in the username parameter", "SELECT * FROM users WHERE username = '' OR '1'='1'", "observe authentication bypass"],
            Some("curl -X POST http://localhost:8080/login -d \"username=' OR '1'='1\""),
        );
        let v = StructuralSandbox::verify(&f);
        assert!(v.is_valid(), "sqli poc should be valid: {:?}", v);
    }

    #[test]
    fn test_invalid_sqli_poc() {
        let f = make_test_finding(
            PocClass::SqlInjection,
            vec!["click the button", "see what happens"],
            None,
        );
        let v = StructuralSandbox::verify(&f);
        assert!(!v.is_valid(), "should fail: no sql patterns");
    }

    #[test]
    fn test_valid_xss_poc() {
        let f = make_test_finding(
            PocClass::Xss,
            vec!["inject <script>alert(1)</script> in the search parameter", "observe execution"],
            Some("curl 'http://localhost:8080/search?q=<script>alert(1)</script>'"),
        );
        let v = StructuralSandbox::verify(&f);
        assert!(v.is_valid(), "xss poc should be valid: {:?}", v);
    }

    #[test]
    fn test_command_injection_poc() {
        let f = make_test_finding(
            PocClass::CommandInjection,
            vec!["inject ; id in the host parameter", "observe command output"],
            Some("curl 'http://localhost:8080/ping?host=127.0.0.1;id'"),
        );
        let v = StructuralSandbox::verify(&f);
        assert!(v.is_valid(), "cmdi poc should be valid: {:?}", v);
    }
}
