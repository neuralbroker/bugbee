//! Hierarchical swarm orchestrator — HPTSA-style task delegation.

use std::path::{Path, PathBuf};
use std::time::Instant;

use bugbee_akg::AttackKnowledgeGraph;
use bugbee_core::{
    AdjudicationState, Finding, FindingStatus, ProjectConfig, Result, Severity, Store, Target,
};
use bugbee_engine::{hunt, HuntOptions};
use bugbee_nsae::{adjudicate_finding, verify_finding};
use tracing::info;

use crate::harness::{HarnessEvent, HarnessEventKind};
use crate::swarm::carlin::carlini_batch;
use crate::swarm::checkpoint::{Checkpoint, CheckpointStore};
use crate::swarm::recon::recon_repo;
use crate::swarm::scribe::render_bounty_reports;
use crate::tools::parallel_enrich;

#[derive(Debug, Clone)]
pub struct SwarmOptions {
    pub resume: bool,
    pub carlini_max: usize,
    pub write_report: Option<PathBuf>,
}

impl Default for SwarmOptions {
    fn default() -> Self {
        Self {
            resume: true,
            carlini_max: 12,
            write_report: None,
        }
    }
}

#[derive(Debug)]
pub struct SwarmReport {
    pub phases: Vec<String>,
    pub events: Vec<HarnessEvent>,
    pub findings_total: usize,
    pub vulnerable: usize,
    pub verified: usize,
    pub kill_chains: usize,
    pub elapsed_ms: u128,
    pub summary: String,
    pub bounty_markdown: String,
    pub akg_nodes: usize,
    pub akg_edges: usize,
}

/// Full neuro-symbolic swarm pipeline (offline-capable MVP).
pub async fn run_swarm(
    root: impl Into<PathBuf>,
    config: ProjectConfig,
    store: Store,
    opts: SwarmOptions,
) -> Result<SwarmReport> {
    let root = root.into();
    let t0 = Instant::now();
    let mut events = Vec::new();
    let mut phases = Vec::new();
    let ckpt_store = CheckpointStore::in_project(&root);
    let mut akg = AttackKnowledgeGraph::new();
    let mut findings: Vec<Finding> = Vec::new();

    // ── Resume ────────────────────────────────────────────────────
    if opts.resume {
        if let Some(ckpt) = ckpt_store.load()? {
            events.push(HarnessEvent::new(
                HarnessEventKind::Phase,
                format!(
                    "resume checkpoint phase={} findings={}",
                    ckpt.phase,
                    ckpt.findings.len()
                ),
            ));
            findings = ckpt.findings.clone();
            akg = ckpt.restore_akg();
        }
    }

    // ── Phase R: Recon ────────────────────────────────────────────
    phases.push("recon".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "ReconAgent: map attack surface",
    ));
    let recon = recon_repo(&root);
    for t in &recon.targets {
        akg.add_asset(t, false);
    }
    // Ensure repo root asset
    akg.add_asset(&Target::repo(root.display().to_string()), false);
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!(
            "recon: {} files, langs={:?}, auth_hints={}, api_specs={}",
            recon.files_indexed,
            recon.by_language,
            recon.auth_hints.len(),
            recon.api_specs.len()
        ),
    ));
    save_ckpt(&ckpt_store, &root, "recon", &findings, &akg)?;

    // ── Phase H: Hunter (engine) ──────────────────────────────────
    phases.push("hunter".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "HunterAgent: deterministic engines + candidates",
    ));
    let hunt_opts = HuntOptions::from_config(&root, &config);
    let summary = hunt(&hunt_opts)?;
    // Merge by id
    for f in summary.findings {
        if let Some(existing) = findings.iter_mut().find(|x| x.id == f.id) {
            // keep richer evidence
            if f.evidence.len() > existing.evidence.len() {
                *existing = f;
            }
        } else {
            findings.push(f);
        }
    }
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!(
            "hunt: {} files, {} findings, {} rules",
            summary.files_scanned,
            findings.len(),
            summary.rules_loaded
        ),
    ));
    save_ckpt(&ckpt_store, &root, "hunter", &findings, &akg)?;

    // ── Enrich ────────────────────────────────────────────────────
    phases.push("enrich".into());
    store.upsert_many(&findings)?;
    let store_mutex = parking_lot::Mutex::new(store);
    let enriched = parallel_enrich(&store_mutex, &root, 64)?;
    findings = store_mutex.into_inner().list(None)?;
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!("enriched {enriched} context windows"),
    ));

    // ── Phase S: Symbolic + NSAE ──────────────────────────────────
    phases.push("symbolic_nsae".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "SymbolicAgent + NSAE adjudication matrix",
    ));
    for f in &mut findings {
        f.swarm_agent = Some("symbolic".into());
        adjudicate_finding(&root, f);
    }
    let vul_n = findings
        .iter()
        .filter(|f| f.adjudicated_state == AdjudicationState::Vulnerable)
        .count();
    let safe_n = findings
        .iter()
        .filter(|f| f.adjudicated_state == AdjudicationState::Safe)
        .count();
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!("nsae: vulnerable={vul_n} safe={safe_n}"),
    ));
    save_ckpt(&ckpt_store, &root, "symbolic_nsae", &findings, &akg)?;

    // ── Phase C: Carlini refine hot set ───────────────────────────
    phases.push("carlini".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "HunterAgent Carlini loop on critical/high",
    ));
    let refined = carlini_batch(&root, &mut findings, opts.carlini_max);
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!("carlini refined {refined} findings"),
    ));

    // ── Phase P: Prover ───────────────────────────────────────────
    phases.push("prover".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "ProverAgent: static IR verification (defense-only)",
    ));
    let mut verified = 0usize;
    for f in &mut findings {
        if f.adjudicated_state == AdjudicationState::Safe {
            continue;
        }
        f.swarm_agent = Some("prover".into());
        let v = verify_finding(&root, f);
        if f.verified {
            verified += 1;
        }
        let _ = v;
    }
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!("prover verified={verified}"),
    ));
    save_ckpt(&ckpt_store, &root, "prover", &findings, &akg)?;

    // ── Phase K: Chain / AKG ──────────────────────────────────────
    phases.push("chain".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "ChainAgent: Attack Knowledge Graph",
    ));
    akg.ingest_findings(&findings);
    akg.synthesize_local_chains(&findings);
    let chains = akg.find_kill_chains(4);
    let paths = akg.list_attack_paths();
    for p in &paths {
        events.push(HarnessEvent::new(
            HarnessEventKind::Finding,
            format!(
                "attack_path {} likelihood={} prereq={}",
                p.label,
                p.likelihood,
                p.prerequisites.len()
            ),
        ));
    }
    events.push(HarnessEvent::new(
        HarnessEventKind::ToolEnd,
        format!(
            "akg nodes={} edges={} paths={} kill_chains={}",
            akg.node_count(),
            akg.edge_count(),
            paths.len(),
            chains.len()
        ),
    ));
    save_ckpt(&ckpt_store, &root, "chain", &findings, &akg)?;

    // ── Phase Scribe ──────────────────────────────────────────────
    phases.push("scribe".into());
    events.push(HarnessEvent::new(
        HarnessEventKind::Phase,
        "ScribeAgent: bounty reports",
    ));
    let reportable: Vec<_> = findings
        .iter()
        .filter(|f| {
            f.adjudicated_state == AdjudicationState::Vulnerable
                || (f.verified && f.status != FindingStatus::FalsePositive)
        })
        .cloned()
        .collect();
    let bounty_markdown = render_bounty_reports(&reportable);
    if let Some(ref path) = opts.write_report {
        std::fs::write(path, &bounty_markdown)?;
        events.push(HarnessEvent::new(
            HarnessEventKind::ToolEnd,
            format!("wrote report {}", path.display()),
        ));
    }

    // Persist findings
    let store = Store::open(bugbee_core::config::store_path(&root))?;
    store.upsert_many(&findings)?;
    save_ckpt(&ckpt_store, &root, "done", &findings, &akg)?;

    let vulnerable = findings
        .iter()
        .filter(|f| f.adjudicated_state == AdjudicationState::Vulnerable)
        .count();
    let crit_high = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .count();

    let mut summary = String::new();
    summary.push_str("## Bugbee Swarm Report\n\n");
    summary.push_str(&format!(
        "- Recon: {} source files, {} auth hints, {} API specs\n",
        recon.files_indexed,
        recon.auth_hints.len(),
        recon.api_specs.len()
    ));
    summary.push_str(&format!(
        "- Findings: {} total, {} NSAE-vulnerable, {} verified, {} crit/high\n",
        findings.len(),
        vulnerable,
        verified,
        crit_high
    ));
    summary.push_str(&format!(
        "- AKG: {} nodes, {} edges, {} kill-chain sketches\n",
        akg.node_count(),
        akg.edge_count(),
        chains.len()
    ));
    if !chains.is_empty() {
        summary.push_str("- Top kill chains:\n");
        for c in chains.iter().take(5) {
            summary.push_str(&format!(
                "  - [{}] {} (difficulty {})\n",
                c.max_severity.as_str(),
                c.labels.join(" → "),
                c.difficulty.score
            ));
        }
    }
    summary.push_str("\n### Top findings\n");
    let mut ranked = findings.clone();
    ranked.sort_by_key(|b| std::cmp::Reverse(b.brs));
    for f in ranked.iter().take(12) {
        summary.push_str(&format!(
            "- [{:<8}] adj={:<12} ver={} brs={} {}:{}  {}\n",
            f.severity.as_str(),
            f.adjudicated_state.as_str(),
            if f.verified { "Y" } else { "n" },
            f.brs,
            f.location.path,
            f.location.start_line,
            f.title
        ));
    }
    summary.push_str(&format!(
        "\n---\nSwarm complete in {}ms · phases: {}\n",
        t0.elapsed().as_millis(),
        phases.join(" → ")
    ));

    info!(
        findings = findings.len(),
        vulnerable,
        verified,
        ms = t0.elapsed().as_millis(),
        "swarm complete"
    );

    events.push(HarnessEvent::new(
        HarnessEventKind::Done,
        format!(
            "swarm done findings={} vul={} ver={} ms={}",
            findings.len(),
            vulnerable,
            verified,
            t0.elapsed().as_millis()
        ),
    ));

    Ok(SwarmReport {
        phases,
        events,
        findings_total: findings.len(),
        vulnerable,
        verified,
        kill_chains: chains.len(),
        elapsed_ms: t0.elapsed().as_millis(),
        summary,
        bounty_markdown,
        akg_nodes: akg.node_count(),
        akg_edges: akg.edge_count(),
    })
}

fn save_ckpt(
    store: &CheckpointStore,
    root: &Path,
    phase: &str,
    findings: &[Finding],
    akg: &AttackKnowledgeGraph,
) -> Result<()> {
    let ckpt = Checkpoint::new(root, phase, findings.to_vec(), akg);
    store.save(&ckpt)
}
