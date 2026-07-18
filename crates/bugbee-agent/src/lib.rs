//! Bugbee agent — OpenCode-inspired godmode harness for security hunting.
//!
//! Architecture mirrors OpenCode's tool registry + session processor, specialized
//! for AppSec: hunt engines, evidence, multi-role review, defense-only policy.

pub mod crawl;
pub mod harness;
pub mod hunter;
pub mod permissions;
pub mod roles;
pub mod session;
pub mod stream;
pub mod superharness;
pub mod swarm;
pub mod tools;

pub use harness::{
    run_godmode, AgentRunResult, AgentRunner, GodmodeOptions, GodmodeReport, HarnessEvent,
    RunLimits, RunnerConfig,
};
pub use permissions::{Decision, Permission, PermissionGate};
pub use roles::{AgentRole, RoleKind};
pub use session::{Session, SessionEvent, TodoItem};
pub use superharness::{
    compact_messages, CompactionConfig, HookBus, RalphConfig, RalphRunner, RalphStatus, SuperEvent,
    SuperEventKind, SuperHarness, SuperHarnessConfig, SuperRunResult, ToolExecMode,
};
pub use swarm::{
    recon_repo, render_bounty_report, render_bounty_reports, run_swarm, Checkpoint,
    CheckpointStore, SwarmOptions, SwarmReport,
};
pub use tools::{tool_specs, ToolContext, ToolExecutor, ToolResult};

use std::path::PathBuf;
use std::sync::Arc;

use bugbee_core::{config, ProjectConfig, Redactor, Result, Store};
use bugbee_engine::{hunt, HuntOptions, HuntSummary};
use parking_lot::Mutex;

/// High-level workspace bound to a project root.
pub struct Workspace {
    pub root: PathBuf,
    pub config: ProjectConfig,
    pub store: Store,
    pub redactor: Redactor,
}

impl Workspace {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let cfg_path = config::config_path(&root);
        let config = if cfg_path.is_file() {
            ProjectConfig::load(&cfg_path)?
        } else {
            ProjectConfig::default_for(
                root.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("project"),
            )
        };
        let store = Store::open(config::store_path(&root))?;
        let redactor = Redactor::new(config.policy.redact_secrets);
        Ok(Self {
            root,
            config,
            store,
            redactor,
        })
    }

    pub fn init(root: impl AsRef<std::path::Path>, name: Option<&str>) -> Result<Self> {
        let cfg = config::init_project(root.as_ref(), name)?;
        let store = Store::open(config::store_path(root.as_ref()))?;
        let redactor = Redactor::new(cfg.policy.redact_secrets);
        Ok(Self {
            root: root.as_ref().to_path_buf(),
            config: cfg,
            store,
            redactor,
        })
    }

    pub fn run_hunt(&self) -> Result<HuntSummary> {
        let opts = HuntOptions::from_config(&self.root, &self.config);
        let summary = hunt(&opts)?;
        self.store.upsert_many(&summary.findings)?;
        Ok(summary)
    }

    /// Build a tool executor bound to this workspace (shared store mutex).
    pub fn tool_executor(&self, gate: PermissionGate) -> Result<ToolExecutor> {
        // Re-open store connection for mutex-shared access path via same file.
        let store = Store::open(config::store_path(&self.root))?;
        let session = Session::new(if gate.read_only {
            AgentRole::review_mode()
        } else {
            AgentRole::hunt_mode()
        });
        Ok(ToolExecutor::new(ToolContext {
            root: self.root.clone(),
            gate,
            store: Arc::new(Mutex::new(store)),
            session: Arc::new(Mutex::new(session)),
            hunt_config: self.config.clone(),
        }))
    }

    pub fn doctor_report(&self) -> String {
        let mut lines = vec![
            format!("project: {}", self.config.project.name),
            format!("root: {}", self.root.display()),
            format!("defense_only: {}", self.config.policy.defense_only),
            format!("redact_secrets: {}", self.config.policy.redact_secrets),
            format!("hunt.secrets: {}", self.config.hunt.secrets),
            format!("hunt.rules: {}", self.config.hunt.rules),
            format!("hunt.aggressive: {}", self.config.hunt.aggressive),
            "harness: SuperHarness (Pi + OpenCode + Claude Code patterns)".into(),
            "pipeline: swarm (NSAE) · godmode · super · ralph".into(),
        ];
        match self.store.count() {
            Ok(n) => lines.push(format!("findings_in_store: {n}")),
            Err(e) => lines.push(format!("store_error: {e}")),
        }
        let provider = self.config.provider.name.as_deref().unwrap_or("(unset)");
        let model = self.config.provider.model.as_deref().unwrap_or("(unset)");
        lines.push(format!("provider: {provider}"));
        lines.push(format!("model: {model}"));
        if let Some(env) = &self.config.provider.api_key_env {
            let present = std::env::var(env).is_ok();
            lines.push(format!(
                "api_key_env {env}: {}",
                if present { "set" } else { "missing" }
            ));
        }
        lines.join("\n")
    }
}

/// Export findings as SARIF 2.1.0 JSON.
pub fn findings_to_sarif(findings: &[bugbee_core::Finding]) -> serde_json::Value {
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "ruleId": f.rule_id,
                "level": match f.severity {
                    bugbee_core::Severity::Critical | bugbee_core::Severity::High => "error",
                    bugbee_core::Severity::Medium => "warning",
                    _ => "note",
                },
                "message": { "text": format!("{} — {}", f.title, f.message) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.location.path },
                        "region": {
                            "startLine": f.location.start_line,
                            "endLine": f.location.end_line,
                            "snippet": { "text": f.location.snippet.clone().unwrap_or_default() }
                        }
                    }
                }],
                "properties": {
                    "brs": f.brs,
                    "ecs": f.ecs,
                    "status": f.status.as_str(),
                    "findingId": f.id.0,
                }
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "bugbee",
                    "informationUri": "https://github.com/neuralbroker/bugbee",
                    "version": bugbee_core::VERSION,
                }
            },
            "results": results
        }]
    })
}
