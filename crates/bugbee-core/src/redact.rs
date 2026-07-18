use std::sync::OnceLock;

use regex::Regex;

fn patterns() -> &'static [(Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            (
                Regex::new(r#"(?i)(api[_-]?key|secret|password|token|authorization)\s*[:=]\s*['"]?([^\s'"]{8,})"#)
                    .expect("regex"),
                "$1=***REDACTED***",
            ),
            (
                Regex::new(r"(?i)bearer\s+[a-z0-9\-._~+/]+=*").expect("regex"),
                "Bearer ***REDACTED***",
            ),
            (
                Regex::new(r"ghp_[A-Za-z0-9]{20,}").expect("regex"),
                "ghp_***REDACTED***",
            ),
            (
                Regex::new(r"sk-[A-Za-z0-9]{20,}").expect("regex"),
                "sk-***REDACTED***",
            ),
            (
                Regex::new(r"xai-[A-Za-z0-9]{20,}").expect("regex"),
                "xai-***REDACTED***",
            ),
            (
                Regex::new(r"-----BEGIN (?:RSA |OPENSSH |EC )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |OPENSSH |EC )?PRIVATE KEY-----")
                    .expect("regex"),
                "***REDACTED PRIVATE KEY***",
            ),
        ]
    })
}

/// Scrubs likely secrets from text before it leaves the host.
#[derive(Debug, Default, Clone)]
pub struct Redactor {
    enabled: bool,
}

impl Redactor {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn redact(&self, input: &str) -> String {
        if !self.enabled {
            return input.to_string();
        }
        let mut out = input.to_string();
        for (re, rep) in patterns().iter() {
            out = re.replace_all(&out, *rep).into_owned();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key_assignment() {
        let r = Redactor::new(true);
        let s = r.redact("api_key = 'sk-abcdefghijklmnopqrstuvwxyz'");
        assert!(s.contains("REDACTED"));
        assert!(!s.contains("sk-abcdefghijklmnop"));
    }
}
