//! OpenCode-style permission envelope (allow / ask / deny).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bugbee_core::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Read,
    Grep,
    Glob,
    ListDir,
    Hunt,
    Findings,
    Review,
    Edit,
    Shell,
    Network,
    Todo,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Ask,
    Deny,
}

/// Callback for interactive approval of Ask-level permissions.
/// Returns true if approved.
pub type ApprovalFn = Arc<dyn Fn(&str, Permission) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct PermissionGate {
    pub root: PathBuf,
    pub read_only: bool,
    pub allow_shell: bool,
    pub allow_network: bool,
    pub allow_edit: bool,
    /// Auto-approve "ask" in headless godmode when true.
    pub auto_approve: bool,
    pub sensitive_globs: Vec<String>,
    /// Interactive approval hook — called for Ask decisions when !auto_approve.
    pub approval: Option<ApprovalFn>,
}

impl PermissionGate {
    pub fn hunt_mode(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            read_only: false,
            allow_shell: false,
            allow_network: false,
            allow_edit: false,
            auto_approve: true,
            sensitive_globs: default_sensitive(),
            approval: None,
        }
    }

    pub fn review_mode(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            read_only: true,
            allow_shell: false,
            allow_network: false,
            allow_edit: false,
            auto_approve: true,
            sensitive_globs: default_sensitive(),
            approval: None,
        }
    }

    pub fn patch_mode(root: impl Into<PathBuf>) -> Self {
        let mut g = Self::hunt_mode(root);
        g.allow_edit = true;
        g
    }

    pub fn decide(&self, perm: Permission) -> Decision {
        match perm {
            Permission::Edit if !self.allow_edit || self.read_only => Decision::Deny,
            Permission::Shell if !self.allow_shell || self.read_only => Decision::Deny,
            Permission::Network if !self.allow_network => Decision::Deny,
            Permission::Shell | Permission::Edit | Permission::Network => Decision::Ask,
            _ => Decision::Allow,
        }
    }

    pub fn check(&self, perm: Permission) -> Result<()> {
        match self.decide(perm) {
            Decision::Allow => Ok(()),
            Decision::Ask if self.auto_approve => Ok(()),
            Decision::Ask => {
                if let Some(ref approve) = self.approval {
                    let label = format!("{:?}", perm);
                    if approve(&label, perm) {
                        return Ok(());
                    }
                }
                Err(Error::Permission(format!(
                    "{perm:?} requires approval"
                )))
            }
            Decision::Deny => Err(Error::Permission(format!(
                "{perm:?} denied by policy (read_only={}, shell={}, edit={}, net={})",
                self.read_only, self.allow_shell, self.allow_edit, self.allow_network
            ))),
        }
    }

    pub fn resolve_path(&self, user_path: &str) -> Result<PathBuf> {
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let candidate = if Path::new(user_path).is_absolute() {
            PathBuf::from(user_path)
        } else {
            root.join(user_path)
        };
        // Don't require file to exist for write targets — canonicalize parent.
        let canon = if candidate.exists() {
            candidate.canonicalize().unwrap_or(candidate)
        } else if let Some(parent) = candidate.parent() {
            let p = parent
                .canonicalize()
                .unwrap_or_else(|_| parent.to_path_buf());
            p.join(candidate.file_name().unwrap_or_default())
        } else {
            candidate
        };
        // Normalize: remove `.` and `..` components for lexical comparison.
        let normalized: PathBuf = canon.components().collect();
        if !normalized.starts_with(&root) {
            return Err(Error::Permission(format!(
                "path escapes project root: {user_path}"
            )));
        }
        Ok(canon)
    }

    pub fn is_sensitive(&self, path: &Path) -> bool {
        // Match against file name and each path component for precision.
        let file_name = path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
        let s = path.to_string_lossy();
        self.sensitive_globs.iter().any(|g| {
            let trimmed = g.trim_start_matches('*').trim_start_matches('.');
            // Match against full path (substring) for path-based patterns like .bugbee
            s.contains(trimmed)
                // Match against file name for file patterns
                || file_name.contains(trimmed)
        })
    }

    pub fn rel_display(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned()
    }
}

fn default_sensitive() -> Vec<String> {
    vec![
        ".env".into(),
        "id_rsa".into(),
        ".pem".into(),
        "credentials".into(),
        "secret".into(),
        ".bugbee".into(),
    ]
}
