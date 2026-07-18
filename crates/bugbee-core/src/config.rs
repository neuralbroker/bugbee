use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::STATE_DIR;

/// User-level defaults (optional; project config wins).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BugbeeConfig {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub base_url: Option<String>,
}

/// Per-repository configuration (`bugbee.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub hunt: HuntConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub provider: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntConfig {
    /// Rule pack globs relative to repo or absolute.
    #[serde(default = "default_packs")]
    pub packs: Vec<String>,
    #[serde(default = "default_true")]
    pub secrets: bool,
    #[serde(default = "default_true")]
    pub rules: bool,
    /// Aggressive mode widens match surface (more candidates → human review).
    #[serde(default = "default_true")]
    pub aggressive: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for HuntConfig {
    fn default() -> Self {
        Self {
            packs: default_packs(),
            secrets: true,
            rules: true,
            aggressive: true,
            exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Block paths matching these globs from LLM context.
    #[serde(default = "default_sensitive")]
    pub sensitive_paths: Vec<String>,
    /// Never send raw secrets to providers.
    #[serde(default = "default_true")]
    pub redact_secrets: bool,
    /// Defense-only: reject live exploit tooling (always true in product).
    #[serde(default = "default_true")]
    pub defense_only: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            sensitive_paths: default_sensitive(),
            redact_secrets: true,
            defense_only: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub name: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    /// Env var name holding the API key (never store the key in toml).
    pub api_key_env: Option<String>,
}

fn default_packs() -> Vec<String> {
    vec!["rules/**/*.yaml".into(), "rules/**/*.yml".into()]
}

fn default_sensitive() -> Vec<String> {
    vec![
        "**/.env".into(),
        "**/.env.*".into(),
        "**/secrets/**".into(),
        "**/*credential*".into(),
        "**/id_rsa".into(),
        "**/*.pem".into(),
    ]
}

fn default_true() -> bool {
    true
}

impl ProjectConfig {
    pub fn default_for(name: &str) -> Self {
        Self {
            project: ProjectMeta {
                name: name.to_string(),
                description: "Bugbee security project".into(),
            },
            hunt: HuntConfig::default(),
            policy: PolicyConfig::default(),
            provider: ProviderConfig::default(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path.as_ref())?;
        let cfg: ProjectConfig = toml::from_str(&text)?;
        if !cfg.policy.defense_only {
            return Err(Error::Config(
                "policy.defense_only cannot be disabled — Bugbee is defensive only".into(),
            ));
        }
        Ok(cfg)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let text = toml::to_string_pretty(self).map_err(|e| Error::Toml(e.to_string()))?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text)?;
        Ok(())
    }
}

/// Resolve project root: walk up looking for `bugbee.toml` or `.git`.
pub fn find_project_root(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut dir = start.as_ref().canonicalize().ok()?;
    loop {
        if dir.join("bugbee.toml").is_file() {
            return Some(dir);
        }
        if dir.join(".git").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn state_dir(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(STATE_DIR)
}

pub fn store_path(root: impl AsRef<Path>) -> PathBuf {
    state_dir(root).join("findings.db")
}

pub fn config_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("bugbee.toml")
}

/// Initialize a project: write config + state dir + starter docs pointer.
pub fn init_project(root: impl AsRef<Path>, name: Option<&str>) -> Result<ProjectConfig> {
    let root = root.as_ref();
    fs::create_dir_all(root)?;
    let name = name.map(|s| s.to_string()).unwrap_or_else(|| {
        root.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let cfg_path = config_path(root);
    let cfg = if cfg_path.is_file() {
        ProjectConfig::load(&cfg_path)?
    } else {
        let cfg = ProjectConfig::default_for(&name);
        cfg.save(&cfg_path)?;
        cfg
    };

    fs::create_dir_all(state_dir(root))?;
    let gitignore = root.join(".gitignore");
    if gitignore.is_file() {
        let content = fs::read_to_string(&gitignore)?;
        if !content
            .lines()
            .any(|l| l.trim() == STATE_DIR || l.trim() == ".bugbee/")
        {
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("{STATE_DIR}/\n"));
            fs::write(gitignore, content)?;
        }
    }

    Ok(cfg)
}
