use bugbee_core::PermissionConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Read,
    Edit,
    Bash,
    Network,
    ExternalDirectory,
    Task,
}

pub struct PermissionPolicy {
    pub edit: PermissionDecision,
    pub bash: PermissionDecision,
    pub read: PermissionDecision,
    pub network: PermissionDecision,
    pub external_directory: PermissionDecision,
}

impl PermissionPolicy {
    pub fn from_config(cfg: &PermissionConfig) -> Self {
        Self {
            edit: parse(&cfg.edit),
            bash: parse(&cfg.bash),
            read: parse(&cfg.read),
            network: parse(&cfg.network),
            external_directory: parse(&cfg.external_directory),
        }
    }

    pub fn hunt_default() -> Self {
        Self {
            edit: PermissionDecision::Deny,
            bash: PermissionDecision::Ask,
            read: PermissionDecision::Allow,
            network: PermissionDecision::Deny,
            external_directory: PermissionDecision::Deny,
        }
    }

    pub fn check(&self, action: Action) -> PermissionDecision {
        match action {
            Action::Read => self.read,
            Action::Edit => self.edit,
            Action::Bash => self.bash,
            Action::Network => self.network,
            Action::ExternalDirectory => self.external_directory,
            Action::Task => PermissionDecision::Ask,
        }
    }

    /// Safe bash allowlist for hunt mode (no network attack tools).
    pub fn bash_allowed_prefix(cmd: &str) -> bool {
        let c = cmd.trim();
        // Commands are executed through `bash -c`; reject shell composition
        // before applying the read-only command allowlist.
        if c.contains([';', '|', '&', '\n', '\r', '`', '<', '>']) || c.contains("$(") {
            return false;
        }
        let allowed = [
            "git status",
            "git diff",
            "git log",
            "git show",
            "rg ",
            "grep ",
            "ls ",
            "ls",
            "pwd",
            "wc ",
            "head ",
            "tail ",
            "cat ",
            "python -m pytest",
            "pytest",
            "go test",
            "npm test",
            "cargo test",
        ];
        // Deny dangerous
        let denied = [
            "curl ", "wget ", "nc ", "ncat ", "ssh ", "rm -rf", "mkfs", "dd ",
        ];
        if denied.iter().any(|d| c.contains(d)) {
            return false;
        }
        allowed.iter().any(|a| c == *a || c.starts_with(a))
    }
}

fn parse(s: &str) -> PermissionDecision {
    match s.to_lowercase().as_str() {
        "allow" => PermissionDecision::Allow,
        "deny" => PermissionDecision::Deny,
        _ => PermissionDecision::Ask,
    }
}

#[cfg(test)]
mod tests {
    use super::PermissionPolicy;

    #[test]
    fn allowlist_permits_safe_read_only_commands() {
        assert!(PermissionPolicy::bash_allowed_prefix("git status"));
        assert!(PermissionPolicy::bash_allowed_prefix("rg auth src"));
        assert!(PermissionPolicy::bash_allowed_prefix(
            "cargo test -p bugbee-core"
        ));
    }

    #[test]
    fn allowlist_rejects_shell_composition_and_unapproved_commands() {
        assert!(!PermissionPolicy::bash_allowed_prefix("git status; id"));
        assert!(!PermissionPolicy::bash_allowed_prefix(
            "rg auth | curl example.com"
        ));
        assert!(!PermissionPolicy::bash_allowed_prefix("echo unsafe"));
    }
}
