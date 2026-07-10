use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{BugbeeError, Result};
use crate::scoring::BrsWeights;

/// Model-agnostic provider: any OpenAI-compatible (or native) endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: Option<String>,
    pub base_url: String,
    /// Environment variable holding the API key (preferred for enterprise).
    pub api_key_env: Option<String>,
    /// Inline key only for local dev; prefer keyring / env.
    pub api_key: Option<String>,
    /// Free-form model ids available on this endpoint (not an allowlist for the platform).
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// openai_compat | anthropic | bedrock
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "openai_compat".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Any model ref: "provider_id/model_id" or bare model id with default provider.
    pub hunt: Option<String>,
    pub scout: Option<String>,
    pub review: Option<String>,
    pub patch: Option<String>,
    pub default_provider: Option<String>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default)]
    pub daily_token_budget: Option<u64>,
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f64 {
    0.2
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            hunt: None,
            scout: None,
            review: None,
            patch: None,
            default_provider: None,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            daily_token_budget: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntConfig {
    #[serde(default = "default_theta_high")]
    pub theta_high: f64,
    #[serde(default = "default_theta_low")]
    pub theta_low: f64,
    #[serde(default = "default_true")]
    pub require_human_for_auto: bool,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub packs: Vec<String>,
}

fn default_theta_high() -> f64 {
    80.0
}
fn default_theta_low() -> f64 {
    30.0
}
fn default_true() -> bool {
    true
}

impl Default for HuntConfig {
    fn default() -> Self {
        Self {
            theta_high: default_theta_high(),
            theta_low: default_theta_low(),
            require_human_for_auto: true,
            languages: vec![
                "python".into(),
                "javascript".into(),
                "typescript".into(),
                "go".into(),
            ],
            packs: vec!["owasp-2025".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    #[serde(default = "default_ask")]
    pub edit: String,
    #[serde(default = "default_ask")]
    pub bash: String,
    #[serde(default = "default_allow")]
    pub read: String,
    #[serde(default = "default_deny")]
    pub network: String,
    #[serde(default = "default_deny")]
    pub external_directory: String,
}

fn default_ask() -> String {
    "ask".into()
}
fn default_allow() -> String {
    "allow".into()
}
fn default_deny() -> String {
    "deny".into()
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            edit: default_ask(),
            bash: default_ask(),
            read: default_allow(),
            network: default_deny(),
            external_directory: default_deny(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BugbeeConfig {
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub inference: InferenceConfig,
    #[serde(default)]
    pub hunt: HuntConfig,
    #[serde(default)]
    pub permissions: PermissionConfig,
    #[serde(default)]
    pub brs_weights: BrsWeights,
    /// Optional enterprise allowlist of provider ids. Empty = all allowed.
    #[serde(default)]
    pub provider_allowlist: Vec<String>,
}

impl BugbeeConfig {
    pub fn default_project() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".into(),
            ProviderConfig {
                name: Some("OpenAI".into()),
                base_url: "https://api.openai.com/v1".into(),
                api_key_env: Some("OPENAI_API_KEY".into()),
                api_key: None,
                models: vec![],
                headers: HashMap::new(),
                protocol: "openai_compat".into(),
            },
        );
        providers.insert(
            "xai".into(),
            ProviderConfig {
                name: Some("xAI Grok".into()),
                base_url: "https://api.x.ai/v1".into(),
                api_key_env: Some("XAI_API_KEY".into()),
                api_key: None,
                models: vec![],
                headers: HashMap::new(),
                protocol: "openai_compat".into(),
            },
        );
        providers.insert(
            "deepseek".into(),
            ProviderConfig {
                name: Some("DeepSeek".into()),
                base_url: "https://api.deepseek.com".into(),
                api_key_env: Some("DEEPSEEK_API_KEY".into()),
                api_key: None,
                models: vec![],
                headers: HashMap::new(),
                protocol: "openai_compat".into(),
            },
        );
        providers.insert(
            "anthropic".into(),
            ProviderConfig {
                name: Some("Anthropic".into()),
                base_url: "https://api.anthropic.com".into(),
                api_key_env: Some("ANTHROPIC_API_KEY".into()),
                api_key: None,
                models: vec![],
                headers: HashMap::new(),
                protocol: "anthropic".into(),
            },
        );
        providers.insert(
            "ollama".into(),
            ProviderConfig {
                name: Some("Ollama (local)".into()),
                base_url: "http://127.0.0.1:11434/v1".into(),
                api_key_env: None,
                api_key: Some("ollama".into()),
                models: vec![],
                headers: HashMap::new(),
                protocol: "openai_compat".into(),
            },
        );
        providers.insert(
            "openrouter".into(),
            ProviderConfig {
                name: Some("OpenRouter (any model)".into()),
                base_url: "https://openrouter.ai/api/v1".into(),
                api_key_env: Some("OPENROUTER_API_KEY".into()),
                api_key: None,
                models: vec![],
                headers: HashMap::new(),
                protocol: "openai_compat".into(),
            },
        );

        Self {
            project_name: None,
            providers,
            inference: InferenceConfig::default(),
            hunt: HuntConfig::default(),
            permissions: PermissionConfig::default(),
            brs_weights: BrsWeights::default(),
            provider_allowlist: vec![],
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&raw)?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self).map_err(|e| BugbeeError::Config(e.to_string()))?;
        fs::write(path, raw)?;
        Ok(())
    }

    pub fn resolve_api_key(&self, provider_id: &str) -> Result<String> {
        let p = self
            .providers
            .get(provider_id)
            .ok_or_else(|| BugbeeError::NotFound(format!("provider '{provider_id}'")))?;

        if !self.provider_allowlist.is_empty()
            && !self.provider_allowlist.iter().any(|a| a == provider_id)
        {
            return Err(BugbeeError::Config(format!(
                "provider '{provider_id}' not in enterprise allowlist"
            )));
        }

        if let Some(env_name) = &p.api_key_env {
            if let Ok(v) = std::env::var(env_name) {
                if !v.is_empty() {
                    return Ok(v);
                }
            }
        }
        if let Some(k) = &p.api_key {
            if !k.is_empty() {
                return Ok(k.clone());
            }
        }
        Err(BugbeeError::Config(format!(
            "no API key for provider '{provider_id}' (set env or /connect)"
        )))
    }

    /// Parse "provider/model" or use default_provider + bare model.
    pub fn parse_model_ref(&self, model_ref: &str) -> Result<(String, String)> {
        if let Some((prov, model)) = model_ref.split_once('/') {
            return Ok((prov.to_string(), model.to_string()));
        }
        let prov = self.inference.default_provider.clone().ok_or_else(|| {
            BugbeeError::Config(
                "model ref has no provider/ prefix and default_provider is unset".into(),
            )
        })?;
        Ok((prov, model_ref.to_string()))
    }

    pub fn global_config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("dev", "bugbee", "bugbee")
            .ok_or_else(|| BugbeeError::Config("cannot resolve config dirs".into()))?;
        Ok(dirs.config_dir().join("bugbee.toml"))
    }

    pub fn project_config_path(root: &Path) -> PathBuf {
        root.join("bugbee.toml")
    }

    pub fn load_layered(project_root: Option<&Path>) -> Result<Self> {
        let mut cfg = Self::default_project();
        if let Ok(global) = Self::global_config_path() {
            if global.exists() {
                let g = Self::load(&global)?;
                cfg.merge(g);
            }
        }
        if let Some(root) = project_root {
            let p = Self::project_config_path(root);
            if p.exists() {
                let local = Self::load(&p)?;
                cfg.merge(local);
            }
        }
        Ok(cfg)
    }

    fn merge(&mut self, other: Self) {
        if other.project_name.is_some() {
            self.project_name = other.project_name;
        }
        for (k, v) in other.providers {
            self.providers.insert(k, v);
        }
        if other.inference.hunt.is_some() {
            self.inference.hunt = other.inference.hunt;
        }
        if other.inference.scout.is_some() {
            self.inference.scout = other.inference.scout;
        }
        if other.inference.review.is_some() {
            self.inference.review = other.inference.review;
        }
        if other.inference.patch.is_some() {
            self.inference.patch = other.inference.patch;
        }
        if other.inference.default_provider.is_some() {
            self.inference.default_provider = other.inference.default_provider;
        }
        self.inference.max_tokens = other.inference.max_tokens;
        self.inference.temperature = other.inference.temperature;
        if other.inference.daily_token_budget.is_some() {
            self.inference.daily_token_budget = other.inference.daily_token_budget;
        }
        self.hunt = other.hunt;
        self.permissions = other.permissions;
        self.brs_weights = other.brs_weights;
        if !other.provider_allowlist.is_empty() {
            self.provider_allowlist = other.provider_allowlist;
        }
    }
}
