use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::{Error, Result};

/// Authorization gate — HARD RULE.
/// Every command that touches a live target MUST check this gate.
/// Without explicit permission + matching scope, the command MUST refuse.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Glob-style host patterns that are authorized targets.
    /// e.g. ["*.local", "localhost", "*.example.com"]
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Optional: specific URL prefixes allowed.
    #[serde(default)]
    pub allowed_urls: Vec<String>,
    /// Description of who authorized this scope.
    pub authorized_by: Option<String>,
    /// Expiry date for this authorization.
    pub expires_at: Option<String>,
}

impl ScopeConfig {
    pub fn is_empty(&self) -> bool {
        self.allowed_hosts.is_empty() && self.allowed_urls.is_empty()
    }
}

/// Load scope from a YAML/TOML/JSON file.
pub fn load_scope(path: &Path) -> Result<ScopeConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Invalid(format!("cannot read scope file {}: {e}", path.display())))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("toml");

    match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|e| Error::Invalid(format!("invalid scope yaml: {e}"))),
        "json" => serde_json::from_str(&content)
            .map_err(|e| Error::Invalid(format!("invalid scope json: {e}"))),
        _ => toml::from_str(&content)
            .map_err(|e| Error::Invalid(format!("invalid scope toml: {e}"))),
    }
}

/// Check if a URL is within the authorized scope.
pub fn url_in_scope(url: &str, scope: &ScopeConfig, i_have_permission: bool) -> ScopeResult {
    if !i_have_permission {
        return ScopeResult::Denied {
            reason: "missing --i-have-permission flag (HARD RULE)".into(),
        };
    }
    if scope.is_empty() {
        return ScopeResult::Denied {
            reason: "scope file is empty or not loaded — no targets authorized".into(),
        };
    }

    let url_lower = url.to_ascii_lowercase();

    // Check explicit URL prefixes
    for allowed in &scope.allowed_urls {
        if url_lower.starts_with(&allowed.to_ascii_lowercase()) {
            return ScopeResult::Authorized;
        }
    }

    // Extract host from URL
    let host = match url::Url::parse(url) {
        Ok(parsed) => parsed
            .host_str()
            .unwrap_or("")
            .to_string(),
        Err(_) => {
            // Try to extract host from raw string
            let without_proto = url_lower
                .trim_start_matches("http://")
                .trim_start_matches("https://");
            without_proto
                .split('/')
                .next()
                .unwrap_or(without_proto)
                .split(':')
                .next()
                .unwrap_or(without_proto)
                .to_string()
        }
    };

    if host.is_empty() {
        return ScopeResult::Denied {
            reason: format!("could not parse host from url: {url}"),
        };
    }

    for pattern in &scope.allowed_hosts {
        if host_match(&host, pattern) {
            return ScopeResult::Authorized;
        }
    }

    ScopeResult::Denied {
        reason: format!(
            "host `{host}` is not in authorized scope (allowed: {:?})",
            scope.allowed_hosts
        ),
    }
}

/// Simple host glob matching: supports wildcards at start or end.
/// e.g. "*.local" matches "app.local", "*.example.com" matches "api.example.com"
fn host_match(host: &str, pattern: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase();
    let host = host.to_ascii_lowercase();

    if pattern == host {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return host.ends_with(suffix) || host == suffix.trim_start_matches('.');
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return host.starts_with(prefix);
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeResult {
    Authorized,
    Denied { reason: String },
}

impl ScopeResult {
    pub fn is_authorized(&self) -> bool {
        matches!(self, ScopeResult::Authorized)
    }

    pub fn into_result(self) -> Result<()> {
        match self {
            ScopeResult::Authorized => Ok(()),
            ScopeResult::Denied { reason } => {
                Err(Error::Permission(format!("authorization denied: {reason}")))
            }
        }
    }
}

impl fmt::Display for ScopeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeResult::Authorized => write!(f, "authorized"),
            ScopeResult::Denied { reason } => write!(f, "denied: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scope() -> ScopeConfig {
        ScopeConfig {
            allowed_hosts: vec![
                "*.local".into(),
                "localhost".into(),
                "*.example.com".into(),
                "vulnerable-app.internal".into(),
            ],
            allowed_urls: vec![],
            authorized_by: Some("test".into()),
            expires_at: None,
        }
    }

    // ── HARD RULE: Refusal without --i-have-permission ──────────────

    #[test]
    fn test_refuses_without_permission_flag() {
        let scope = make_scope();
        let result = url_in_scope("http://localhost:8080", &scope, false);
        assert!(!result.is_authorized());
        assert!(result.to_string().contains("--i-have-permission"));
    }

    // ── HARD RULE: Refusal on out-of-scope targets ─────────────────

    #[test]
    fn test_refuses_out_of_scope_host() {
        let scope = make_scope();
        let result = url_in_scope("http://evil.com", &scope, true);
        assert!(!result.is_authorized());
        assert!(result.to_string().contains("not in authorized scope"));
    }

    #[test]
    fn test_refuses_out_of_scope_ip() {
        let scope = make_scope();
        let result = url_in_scope("http://192.168.1.1/admin", &scope, true);
        assert!(!result.is_authorized());
    }

    #[test]
    fn test_refuses_empty_scope() {
        let empty = ScopeConfig {
            allowed_hosts: vec![],
            allowed_urls: vec![],
            authorized_by: None,
            expires_at: None,
        };
        let result = url_in_scope("http://localhost:8080", &empty, true);
        assert!(!result.is_authorized());
        assert!(result.to_string().contains("empty"));
    }

    // ── In-scope targets ARE authorized ────────────────────────────

    #[test]
    fn test_allows_localhost() {
        let scope = make_scope();
        let result = url_in_scope("http://localhost:8080/login", &scope, true);
        assert!(result.is_authorized(), "{}", result);
    }

    #[test]
    fn test_allows_wildcard_local() {
        let scope = make_scope();
        let result = url_in_scope("http://app.local/admin", &scope, true);
        assert!(result.is_authorized(), "{}", result);
    }

    #[test]
    fn test_allows_wildcard_example() {
        let scope = make_scope();
        let result = url_in_scope("https://api.example.com/v1/users", &scope, true);
        assert!(result.is_authorized(), "{}", result);
    }

    #[test]
    fn test_allows_exact_host() {
        let scope = make_scope();
        let result = url_in_scope("http://vulnerable-app.internal", &scope, true);
        assert!(result.is_authorized(), "{}", result);
    }

    // ── URL-based matching ─────────────────────────────────────────

    #[test]
    fn test_allows_url_prefix() {
        let scope = ScopeConfig {
            allowed_hosts: vec![],
            allowed_urls: vec!["http://sandbox.local/api".into()],
            authorized_by: None,
            expires_at: None,
        };
        let result = url_in_scope("http://sandbox.local/api/users", &scope, true);
        assert!(result.is_authorized(), "{}", result);
    }

    #[test]
    fn test_refuses_url_prefix_mismatch() {
        let scope = ScopeConfig {
            allowed_hosts: vec![],
            allowed_urls: vec!["http://sandbox.local/api".into()],
            authorized_by: None,
            expires_at: None,
        };
        let result = url_in_scope("http://sandbox.local/admin", &scope, true);
        assert!(!result.is_authorized());
    }

    // ── Host matching edge cases ───────────────────────────────────

    #[test]
    fn test_host_match_exact() {
        assert!(host_match("localhost", "localhost"));
        assert!(!host_match("evil.com", "localhost"));
    }

    #[test]
    fn test_host_match_wildcard_prefix() {
        assert!(host_match("app.local", "*.local"));
        assert!(host_match("deep.app.local", "*.local"));
        assert!(!host_match("app.local", "*.evil"));
    }

    #[test]
    fn test_host_match_wildcard_suffix() {
        assert!(host_match("api.example.com", "api.*"));
        assert!(!host_match("evil.example.com", "api.*"));
    }

    #[test]
    fn test_host_match_case_insensitive() {
        assert!(host_match("LOCALHOST", "localhost"));
        assert!(host_match("localhost", "LOCALHOST"));
    }

    // ── Scope file loading ─────────────────────────────────────────

    #[test]
    fn test_load_toml_scope() {
        let dir = std::env::temp_dir().join(format!("scope-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("scope.toml");
        std::fs::write(
            &path,
            r#"
allowed_hosts = ["*.local", "localhost"]
authorized_by = "test"
"#,
        )
        .unwrap();
        let scope = load_scope(&path).unwrap();
        assert_eq!(scope.allowed_hosts.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_yaml_scope() {
        let dir = std::env::temp_dir().join(format!("scope-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("scope.yaml");
        std::fs::write(
            &path,
            r#"
allowed_hosts:
  - "*.local"
  - "localhost"
authorized_by: "test"
"#,
        )
        .unwrap();
        let scope = load_scope(&path).unwrap();
        assert_eq!(scope.allowed_hosts.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file() {
        let result = load_scope(Path::new("/nonexistent/scope.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_content() {
        let dir = std::env::temp_dir().join(format!("scope-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("scope.toml");
        std::fs::write(&path, "@@@ invalid toml @@@").unwrap();
        let result = load_scope(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
