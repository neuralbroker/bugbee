//! Secret / sensitive data redaction before any LLM payload leaves the machine.

use once_cell::sync::Lazy;
use regex::Regex;

static PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (
            Regex::new(
                r#"(?i)(api[_-]?key|apikey|secret|token|password|passwd|auth)\s*[=:]\s*['"]?([^\s'"]{8,})['"]?"#,
            )
            .unwrap(),
            "$1=***REDACTED***",
        ),
        (
            Regex::new(
                r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
            )
            .unwrap(),
            "***REDACTED PRIVATE KEY***",
        ),
        (
            Regex::new(r"\b(sk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{20,})\b").unwrap(),
            "***REDACTED_KEY***",
        ),
        (
            Regex::new(r"\b(sk-ant-[A-Za-z0-9_-]{20,})\b").unwrap(),
            "***REDACTED_KEY***",
        ),
        (
            Regex::new(r"\b(xai-[A-Za-z0-9_-]{20,})\b").unwrap(),
            "***REDACTED_KEY***",
        ),
        (
            Regex::new(r"\b(gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
                .unwrap(),
            "***REDACTED_TOKEN***",
        ),
        (
            Regex::new(r"\b(AKIA[0-9A-Z]{16})\b").unwrap(),
            "***REDACTED_AWS_KEY***",
        ),
        (
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap(),
            "***EMAIL***",
        ),
        (
            Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
            "***IP***",
        ),
    ]
});

#[derive(Debug, Default)]
pub struct Redactor {
    pub redact_emails: bool,
    pub redact_ips: bool,
}

impl Redactor {
    pub fn enterprise() -> Self {
        Self {
            redact_emails: true,
            redact_ips: true,
        }
    }

    pub fn redact(&self, input: &str) -> String {
        let mut out = input.to_string();
        for (re, rep) in PATTERNS.iter() {
            let pat = re.as_str();
            if !self.redact_emails && pat.contains('@') {
                continue;
            }
            if !self.redact_ips && pat.contains(r"\d{1,3}") {
                continue;
            }
            out = re.replace_all(&out, *rep).into_owned();
        }
        out
    }

    pub fn is_sensitive_path(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.ends_with(".env")
            || lower.contains(".env.")
            || lower.ends_with("id_rsa")
            || lower.ends_with("id_ed25519")
            || lower.contains("credentials.json")
            || lower.contains("service-account")
            || lower.ends_with(".pem")
            || lower.ends_with(".p12")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key() {
        let r = Redactor::enterprise();
        let s = r.redact("api_key=sk-abcdefghijklmnopqrstuvwxyz123456");
        assert!(!s.contains("sk-abcdef"));
        assert!(s.contains("REDACTED") || s.contains("***"));
    }

    #[test]
    fn redacts_current_provider_and_github_token_formats() {
        let redactor = Redactor::enterprise();
        let tokens = [
            "sk-proj-abcdefghijklmnopqrstuvwxyz123456",
            "sk-ant-api03-abcdefghijklmnopqrstuvwxyz123456",
            "github_pat_abcdefghijklmnopqrstuvwxyz_1234567890",
        ];
        for token in tokens {
            assert!(
                !redactor.redact(token).contains(token),
                "token should be redacted: {token}"
            );
        }
    }

    #[test]
    fn sensitive_paths() {
        assert!(Redactor::is_sensitive_path("/app/.env"));
        assert!(Redactor::is_sensitive_path("secrets/id_rsa"));
        assert!(!Redactor::is_sensitive_path("src/main.rs"));
    }
}
