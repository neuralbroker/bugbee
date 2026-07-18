//! Multi-phase godmode pipeline for maximum hunt efficiency.
//!
//! ```text
//! Phase 0  ENGINE     deterministic rules+secrets (parallel file walk)
//! Phase 1  ENRICH     rayon context windows around findings
//! Phase 2  SCOUT      optional LLM breadth (grep/glob guided)
//! Phase 3  DEEPEN     optional LLM taint/evidence on critical/high
//! Phase 4  REVIEW     optional adversarial reviewer (kill FPs)
//! Phase 5  REPORT     digest + SARIF-ready store
//! ```

use std::sync::Arc;
use std::time::Instant;

use bugbee_core::{FindingStatus, ProjectConfig, Redactor, Result, Severity, Store};
use bugbee_llm::LlmClient;
use parking_lot::Mutex;

use crate::harness::budget::RunLimits;
use crate::harness::events::{HarnessEvent, HarnessEventKind};
use crate::harness::prompts::{findings_digest, godmode_user_goal, project_brief};
use crate::harness::runtime::{offline_agent_pass, AgentRunner, RunnerConfig};
use crate::permissions::PermissionGate;
use crate::roles::{AgentRole, RoleKind};
use crate::session::Session;
use crate::tools::{parallel_enrich, ToolContext, ToolExecutor};

#[derive(Debug, Clone)]
pub struct GodmodeOptions {
    /// Run LLM agent phases when a client is provided.
    pub use_llm: bool,
    /// Aggressive budgets.
    pub aggressive: bool,
    /// Run adversarial review phase.
    pub adversarial: bool,
    /// Max findings to enrich in parallel.
    pub enrich_max: usize,
}

impl Default for GodmodeOptions {
    fn default() -> Self {
        Self {
            use_llm: true,
            aggressive: true,
            adversarial: true,
            enrich_max: 64,
        }
    }
}

#[derive(Debug)]
pub struct GodmodeReport {
    pub phases: Vec<String>,
    pub events: Vec<HarnessEvent>,
    pub findings_total: usize,
    pub findings_critical_high: usize,
    pub elapsed_ms: u128,
    pub summary: String,
    pub used_llm: bool,
}

pub async fn run_godmode(
    root: impl Into<std::path::PathBuf>,
    config: ProjectConfig,
    store: Store,
    client: Option<Arc<dyn LlmClient>>,
    opts: GodmodeOptions,
) -> Result<GodmodeReport> {
    let root = root.into();
    let t0 = Instant::now();
    let mut events = Vec::new();
    let mut phases = Vec::new();
    let store = Arc::new(Mutex::new(store));
    let session = Arc::new(Mutex::new(Session::new(AgentRole::hunt_mode())));
    let redactor = Redactor::new(config.policy.redact_secrets);

    let gate = if opts.aggressive {
        let mut g = PermissionGate::hunt_mode(&root);
        g.auto_approve = true;
        g
    } else {
        PermissionGate::review_mode(&root)
    };

    let tools = ToolExecutor::new(ToolContext {
        root: root.clone(),
        gate,
        store: Arc::clone(&store),
        session: Arc::clone(&session),
        hunt_config: config.clone(),
    });

    // ── Phase 0: Engine ───────────────────────────────────────────
    phases.push("engine".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "phase0 engine hunt (deterministic)",
    ));
    let hunt_res = tools.execute("hunt", &serde_json::json!({}));
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        hunt_res
            .output
            .lines()
            .next()
            .unwrap_or("hunt done")
            .to_string(),
    ));

    // ── Phase 1: Parallel enrich ──────────────────────────────────
    phases.push("enrich".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "phase1 parallel context enrich (rayon)",
    ));
    let enriched = parallel_enrich(&store, &root, opts.enrich_max)?;
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!("enriched {enriched} findings with context windows"),
    ));

    let mut summary = String::new();
    let used_llm;

    if opts.use_llm {
        if let Some(client) = client {
            used_llm = true;
            let findings = store.lock().list(None)?;
            let brief = project_brief(
                &config.project.name,
                &root.display().to_string(),
                findings.len(),
            );
            let digest = findings_digest(&findings, 30);
            let ctx = format!("{brief}\n\n{digest}");

            let limits = if opts.aggressive {
                RunLimits::aggressive()
            } else {
                RunLimits::default()
            };

            // ── Phase 2: Hunt Lead ────────────────────────────────
            phases.push("hunt_lead".into());
            events.push(HarnessEvent::new(
                HarnessEventKind::Phase,
                "phase2 hunt lead agent (tool loop)",
            ));
            let lead = AgentRunner {
                client: Arc::clone(&client),
                tools: ToolExecutor::new(ToolContext {
                    root: root.clone(),
                    gate: PermissionGate::hunt_mode(&root),
                    store: Arc::clone(&store),
                    session: Arc::clone(&session),
                    hunt_config: config.clone(),
                }),
                role: AgentRole::builtin(RoleKind::HuntLead),
                redactor: redactor.clone(),
                config: RunnerConfig {
                    limits: limits.clone(),
                    include_shell: false,
                    include_review: true,
                },
                session: Arc::clone(&session),
            };
            let lead_result = lead.run(godmode_user_goal(), &ctx).await?;
            events.extend(lead_result.events);
            summary.push_str("## Hunt Lead\n");
            summary.push_str(&lead_result.final_text);
            summary.push_str("\n\n");

            // ── Phase 3: Taint on high/critical only ──────────────
            let hot: Vec<_> = store
                .lock()
                .list(None)?
                .into_iter()
                .filter(|f| {
                    matches!(f.severity, Severity::Critical | Severity::High)
                        && f.status == FindingStatus::Draft
                })
                .take(8)
                .collect();

            if !hot.is_empty() {
                phases.push("taint".into());
                events.push(HarnessEvent::new(
                    HarnessEventKind::Phase,
                    format!("phase3 taint analyst on {} hot findings", hot.len()),
                ));
                let hot_digest = findings_digest(&hot, 8);
                let taint = AgentRunner {
                    client: Arc::clone(&client),
                    tools: ToolExecutor::new(ToolContext {
                        root: root.clone(),
                        gate: PermissionGate::review_mode(&root),
                        store: Arc::clone(&store),
                        session: Arc::clone(&session),
                        hunt_config: config.clone(),
                    }),
                    role: AgentRole::builtin(RoleKind::TaintAnalyst),
                    redactor: redactor.clone(),
                    config: RunnerConfig {
                        limits: RunLimits::scout(),
                        include_shell: false,
                        include_review: true,
                    },
                    session: Arc::clone(&session),
                };
                let r = taint
                    .run(
                        "Deepen evidence on the listed high/critical findings. add_evidence with dataflow. Do not invent paths.",
                        &format!("{brief}\n\n{hot_digest}"),
                    )
                    .await?;
                events.extend(r.events);
                summary.push_str("## Taint Analyst\n");
                summary.push_str(&r.final_text);
                summary.push_str("\n\n");
            }

            // ── Phase 4: Adversarial review ───────────────────────
            if opts.adversarial {
                phases.push("adversarial".into());
                events.push(HarnessEvent::new(
                    HarnessEventKind::Phase,
                    "phase4 adversarial reviewer",
                ));
                let all = store.lock().list(None)?;
                let digest = findings_digest(&all, 40);
                let rev = AgentRunner {
                    client: Arc::clone(&client),
                    tools: ToolExecutor::new(ToolContext {
                        root: root.clone(),
                        gate: PermissionGate::review_mode(&root),
                        store: Arc::clone(&store),
                        session: Arc::clone(&session),
                        hunt_config: config.clone(),
                    }),
                    role: AgentRole::builtin(RoleKind::AdversarialReviewer),
                    redactor: redactor.clone(),
                    config: RunnerConfig {
                        limits: RunLimits::scout(),
                        include_shell: false,
                        include_review: true,
                    },
                    session: Arc::clone(&session),
                };
                let r = rev
                    .run(
                        "Kill weak findings (review_finding fp). Confirm only ironclad issues. Summarize survivors.",
                        &format!("{brief}\n\n{digest}"),
                    )
                    .await?;
                events.extend(r.events);
                summary.push_str("## Adversarial Review\n");
                summary.push_str(&r.final_text);
                summary.push_str("\n\n");
            }
        } else {
            used_llm = false;
            phases.push("offline".into());
            let text = offline_agent_pass(&tools, &mut events, false)?;
            summary = text;
        }
    } else {
        used_llm = false;
        phases.push("offline".into());
        let text = offline_agent_pass(&tools, &mut events, false)?;
        summary = text;
    }

    // ── Phase 5: Report stats ─────────────────────────────────────
    phases.push("report".into());
    let findings = store.lock().list(None)?;
    let ch = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .count();
    events.push(HarnessEvent::new(
        HarnessEventKind::Done,
        format!(
            "godmode done: {} findings ({} crit/high) in {}ms llm={}",
            findings.len(),
            ch,
            t0.elapsed().as_millis(),
            used_llm
        ),
    ));

    let footer = format!(
        "\n---\nGodmode complete · {} findings · {} critical/high · {}ms · phases: {}\n",
        findings.len(),
        ch,
        t0.elapsed().as_millis(),
        phases.join(" → ")
    );
    summary.push_str(&footer);

    Ok(GodmodeReport {
        phases,
        events,
        findings_total: findings.len(),
        findings_critical_high: ch,
        elapsed_ms: t0.elapsed().as_millis(),
        summary,
        used_llm,
    })
}
