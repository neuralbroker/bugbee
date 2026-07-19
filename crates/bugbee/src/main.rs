//! Bugbee CLI — terminal-first security agent with godmode harness.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use bugbee_agent::{
    run_godmode, run_swarm, GodmodeOptions, PermissionGate, RalphConfig, RalphRunner, SuperHarness,
    SuperHarnessConfig, SwarmOptions, Workspace,
};
use bugbee_core::{config, FindingStatus, VERSION};
use bugbee_llm::{ChatMessage, ChatRequest};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "bugbee",
    version = VERSION,
    about = "Bugbee — security analysis for your source code",
    long_about = "\
Bugbee analyzes source code for vulnerabilities using deterministic rules,
secrets scanning, and optional AI-powered analysis.

  Deterministic engines (no model required)
  Defense-only — no exploitation or weaponized payloads
  Bring your own LLM (OpenAI, Anthropic, Ollama, and many more)
  Single Rust binary — no runtime dependencies
  Secrets redacted before outbound calls",
    after_help = "\
Examples:
  bugbee init              Set up your project
  bugbee hunt              Run rules + secrets engines
  bugbee findings          List findings
  bugbee report -o report.sarif.json
  bugbee connect --provider ollama --model qwen2.5-coder
  bugbee swarm             Full multi-agent pipeline
  bugbee                   Launch interactive terminal

Learn more: https://github.com/neuralbroker/bugbee"
)]
struct Cli {
    /// Project root directory (auto-detects from bugbee.toml otherwise)
    #[arg(long, global = true, help = "Project root directory (default: auto-detect from bugbee.toml)")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize bugbee.toml and local state in your project
    Init {
        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Run deterministic vulnerability scan (rules + secrets, no LLM needed)
    Hunt,
    /// Multi-phase AI analysis pipeline
    Godmode {
        /// Skip LLM — engines only
        #[arg(long, help = "Skip LLM — run deterministic engines only")]
        offline: bool,
        /// Bypass false-positive review phase
        #[arg(long, help = "Skip false-positive review phase")]
        no_review: bool,
        /// Show detailed progress output
        #[arg(long, short, help = "Show detailed progress output")]
        verbose: bool,
    },
    /// Multi-agent neuro-symbolic analysis pipeline
    Swarm {
        /// Start fresh (ignore saved state)
        #[arg(long, help = "Start fresh — ignore saved state")]
        no_resume: bool,
        /// Write bounty markdown report to this path
        #[arg(long, help = "Write bounty-format markdown report")]
        report: Option<PathBuf>,
        /// Show detailed progress output
        #[arg(long, short, help = "Show detailed progress output")]
        verbose: bool,
    },
    /// Interactive agent with tool-calling LLM
    Super {
        /// Goal or task for the agent
        #[arg(required = true, num_args = 1.., help = "Goal or task for the agent")]
        goal: Vec<String>,
        /// Outer retry iterations (0 = single pass)
        #[arg(long, default_value_t = 0, help = "Outer retry iterations (0 = single pass)")]
        ralph: u32,
        /// Show detailed progress output
        #[arg(long, short, help = "Show detailed progress output")]
        verbose: bool,
    },
    /// List all findings (optionally filter by status)
    Findings {
        /// Filter by status: draft, confirmed, false_positive, fixed
        #[arg(long, help = "Filter by status: draft, confirmed, false_positive, fixed")]
        status: Option<String>,
    },
    /// Review a finding: confirm | fp | fixed
    Review {
        /// Finding ID
        id: String,
        /// Action: confirm, fp (false positive), or fixed
        action: String,
    },
    /// Export findings (SARIF, JSON, or bounty markdown)
    Report {
        /// Output file path
        #[arg(long, short, default_value = "findings.sarif.json", help = "Output file path")]
        output: PathBuf,
        /// Output format: sarif, json, or bounty
        #[arg(long, default_value = "sarif", help = "Output format: sarif, json, or bounty")]
        format: String,
    },
    /// Check configuration and system readiness
    Doctor,
    /// Ask the configured AI model a question about your repo
    Ask {
        /// Your question
        #[arg(required = true, num_args = 1.., help = "Your question about the codebase")]
        question: Vec<String>,
    },
    /// Configure an AI provider (key stays in env vars)
    Connect {
        /// Provider name: openai, anthropic, ollama, xai, deepseek, etc.
        #[arg(long, help = "Provider: openai, anthropic, ollama, xai, deepseek, ...")]
        provider: String,
        /// Model name (e.g. gpt-4o, claude-3-haiku, qwen2.5-coder)
        #[arg(long, help = "Model name (e.g. gpt-4o, claude-3-haiku)")]
        model: String,
        /// Custom base URL (for self-hosted gateways)
        #[arg(long, help = "Custom base URL for self-hosted/private gateways")]
        base_url: Option<String>,
        /// Environment variable holding the API key
        #[arg(long, help = "Env var name for API key (e.g. OPENAI_API_KEY)")]
        api_key_env: Option<String>,
    },
    /// Open the interactive terminal workspace
    Workspace,
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    let root = resolve_root(cli.root.as_deref(), &cwd)?;

    match cli.command {
        None | Some(Commands::Workspace) => {
            let ws = Workspace::open(&root)
                .with_context(|| format!("open workspace at {}", root.display()))?;
            bugbee_ui::run_workspace(ws).context("workspace ui")?;
        }
        Some(Commands::Init { name }) => {
            let ws = Workspace::init(&root, name.as_deref())?;
            println!(
                "initialized {} at {}",
                ws.config.project.name,
                root.display()
            );
            println!("  config: {}", config::config_path(&root).display());
            println!("  state:  {}", config::state_dir(&root).display());
            println!("run `bugbee` for workspace, `bugbee godmode` for full harness.");
        }
        Some(Commands::Hunt) => {
            let ws = Workspace::open(&root)?;
            let summary = ws.run_hunt()?;
            println!(
                "hunt complete: {} files scanned, {} findings ({} rules loaded)",
                summary.files_scanned,
                summary.findings.len(),
                summary.rules_loaded
            );
            for f in summary.findings.iter().take(20) {
                println!(
                    "  [{:<8}] brs={:>3}  {}:{}  {}",
                    f.severity.as_str(),
                    f.brs,
                    f.location.path,
                    f.location.start_line,
                    f.title
                );
            }
            if summary.findings.len() > 20 {
                println!("  … {} more (bugbee findings)", summary.findings.len() - 20);
            }
        }
        Some(Commands::Godmode {
            offline,
            no_review,
            verbose,
        }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let ws = Workspace::open(&root)?;
                let client = if offline {
                    None
                } else {
                    match bugbee_llm::from_env(
                        ws.config.provider.name.as_deref(),
                        ws.config.provider.model.as_deref(),
                        ws.config.provider.base_url.as_deref(),
                        ws.config.provider.api_key_env.as_deref(),
                    ) {
                        Ok(c) => {
                            // Only use if key present or ollama
                            let has_key = ws
                                .config
                                .provider
                                .api_key_env
                                .as_ref()
                                .map(|e| std::env::var(e).is_ok())
                                .unwrap_or(false);
                            let is_local = ws
                                .config
                                .provider
                                .name
                                .as_deref()
                                .is_some_and(|n| n == "ollama" || n == "local");
                            if has_key || is_local || ws.config.provider.name.is_none() {
                                // If no provider configured, skip LLM
                                if ws.config.provider.name.is_none() {
                                    None
                                } else {
                                    Some(Arc::from(c))
                                }
                            } else {
                                eprintln!("warning: provider configured but API key is missing — running offline");
                                None
                            }
                        }
                        Err(e) => {
                            eprintln!("warning: LLM unavailable ({e}) — running offline");
                            None
                        }
                    }
                };

                let opts = GodmodeOptions {
                    use_llm: client.is_some(),
                    aggressive: ws.config.hunt.aggressive,
                    adversarial: !no_review,
                    enrich_max: 64,
                };

                println!(
                    "Bugbee analysis · {} · llm={}",
                    ws.config.project.name,
                    if client.is_some() { "on" } else { "off" }
                );

                // Take store out — run_godmode opens via path; pass owned store
                let store = bugbee_core::Store::open(config::store_path(&root))?;
                let report =
                    run_godmode(root.clone(), ws.config.clone(), store, client, opts).await?;

                if verbose {
                    for e in &report.events {
                        println!("  [{:?}] {}", e.kind, e.message);
                    }
                }

                println!("{}", report.summary);
                println!(
                    "phases: {} · {} findings · {} crit/high · {}ms",
                    report.phases.join(" → "),
                    report.findings_total,
                    report.findings_critical_high,
                    report.elapsed_ms
                );
                Ok::<(), anyhow::Error>(())
            })?;
        }
        Some(Commands::Swarm {
            no_resume,
            report,
            verbose,
        }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let ws = Workspace::open(&root)?;
                let store = bugbee_core::Store::open(config::store_path(&root))?;
                let report_path = report.unwrap_or_else(|| root.join("bugbee-report.md"));
                println!(
                    "Bugbee swarm · neuro-symbolic · {}",
                    ws.config.project.name
                );
                let opts = SwarmOptions {
                    resume: !no_resume,
                    carlini_max: 12,
                    write_report: Some(report_path.clone()),
                };
                let result = run_swarm(root.clone(), ws.config.clone(), store, opts).await?;
                if verbose {
                    for e in &result.events {
                        println!("  [{:?}] {}", e.kind, e.message);
                    }
                }
                println!("{}", result.summary);
                println!(
                    "swarm: {} findings · {} vulnerable · {} verified · {} kill-chains · akg {}n/{}e · {}ms",
                    result.findings_total,
                    result.vulnerable,
                    result.verified,
                    result.kill_chains,
                    result.akg_nodes,
                    result.akg_edges,
                    result.elapsed_ms
                );
                println!("report: {}", report_path.display());
                Ok::<(), anyhow::Error>(())
            })?;
        }
        Some(Commands::Super {
            goal,
            ralph,
            verbose,
        }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let ws = Workspace::open(&root)?;
                let goal = goal.join(" ");
                let client = bugbee_llm::from_env(
                    ws.config.provider.name.as_deref(),
                    ws.config.provider.model.as_deref(),
                    ws.config.provider.base_url.as_deref(),
                    ws.config.provider.api_key_env.as_deref(),
                )?;
                let tools = ws.tool_executor(PermissionGate::hunt_mode(&root))?;
                let session = std::sync::Arc::new(parking_lot::Mutex::new(
                    bugbee_agent::Session::new(bugbee_agent::AgentRole::hunt_mode()),
                ));
                let findings = ws.store.list(None)?;
                let mut ctx = format!(
                    "Project: {}\nRoot: {}\nFindings: {}\n",
                    ws.config.project.name,
                    ws.root.display(),
                    findings.len()
                );
                for f in findings.iter().take(20) {
                    ctx.push_str(&format!(
                        "- [{}] {} @ {}:{}\n",
                        f.severity.as_str(),
                        f.title,
                        f.location.path,
                        f.location.start_line
                    ));
                }

                println!("Bugbee agent  ·  iterations={ralph}");

                if ralph > 0 {
                    let runner = RalphRunner::new(
                        std::sync::Arc::from(client),
                        tools,
                        bugbee_agent::AgentRole::hunt_mode(),
                        ws.redactor.clone(),
                        session,
                        SuperHarnessConfig::aggressive(),
                        RalphConfig {
                            max_iterations: ralph,
                            ..Default::default()
                        },
                    );
                    let (result, status) = runner.run(&goal, &ctx).await?;
                    if verbose {
                        for e in &result.events {
                            println!("  [{:?}] {}", e.kind, e.message);
                        }
                    }
                    println!("{}", result.final_text);
                    println!(
                        "super/ralph: {:?} · steps={} tools={} · {}",
                        status, result.steps, result.tool_calls, result.stopped_reason
                    );
                } else {
                    let harness = SuperHarness {
                        client: std::sync::Arc::from(client),
                        tools,
                        role: bugbee_agent::AgentRole::hunt_mode(),
                        redactor: ws.redactor.clone(),
                        config: SuperHarnessConfig::default(),
                        session,
                        steering: std::sync::Arc::new(parking_lot::Mutex::new(
                            std::collections::VecDeque::new(),
                        )),
                        follow_ups: std::sync::Arc::new(parking_lot::Mutex::new(
                            std::collections::VecDeque::new(),
                        )),
                    };
                    let result = harness.run(&goal, &ctx).await?;
                    if verbose {
                        for e in &result.events {
                            println!("  [{:?}] {}", e.kind, e.message);
                        }
                    }
                    println!("{}", result.final_text);
                    println!(
                        "super: steps={} tools={} compacted={} · {}",
                        result.steps, result.tool_calls, result.compacted, result.stopped_reason
                    );
                }
                Ok::<(), anyhow::Error>(())
            })?;
        }
        Some(Commands::Findings { status }) => {
            let ws = Workspace::open(&root)?;
            let filter = match status.as_deref() {
                Some(s) => Some(
                    FindingStatus::parse(s)
                        .ok_or_else(|| anyhow::anyhow!("invalid status: {s}"))?,
                ),
                None => None,
            };
            let findings = ws.store.list(filter)?;
            if findings.is_empty() {
                println!("no findings");
                return Ok(());
            }
            for f in &findings {
                println!(
                    "{}\t{:<8}\t{:<12}\tbrs={}\tecs={}\t{}:{}  {}",
                    f.id,
                    f.severity.as_str(),
                    f.status.as_str(),
                    f.brs,
                    f.ecs,
                    f.location.path,
                    f.location.start_line,
                    f.title
                );
            }
            println!("{} total", findings.len());
        }
        Some(Commands::Review { id, action }) => {
            let ws = Workspace::open(&root)?;
            let status = FindingStatus::parse(&action)
                .ok_or_else(|| anyhow::anyhow!("action must be confirm|fp|fixed (got {action})"))?;
            let finding = ws
                .store
                .set_status(&bugbee_core::FindingId(id.clone()), status)?;
            println!("{} → {}", finding.id, finding.status.as_str());
        }
        Some(Commands::Report { output, format }) => {
            let ws = Workspace::open(&root)?;
            let findings = ws.store.list(None)?;
            match format.as_str() {
                "sarif" | "sarif-json" => {
                    let sarif = bugbee_agent::findings_to_sarif(&findings);
                    let text = serde_json::to_string_pretty(&sarif)?;
                    std::fs::write(&output, text)?;
                    println!("wrote {} ({} results)", output.display(), findings.len());
                }
                "json" => {
                    let text = serde_json::to_string_pretty(&findings)?;
                    std::fs::write(&output, text)?;
                    println!("wrote {} ({} findings)", output.display(), findings.len());
                }
                "bounty" | "md" | "markdown" => {
                    let text = bugbee_agent::render_bounty_reports(&findings);
                    std::fs::write(&output, text)?;
                    println!("wrote bounty report {}", output.display());
                }
                other => bail!("unsupported format: {other} (use sarif|json|bounty)"),
            }
        }
        Some(Commands::Doctor) => {
            let ws = Workspace::open(&root)?;
            println!("bugbee {VERSION}");
            println!("{}", ws.doctor_report());
            println!("tools: hunt read grep glob list_dir list_findings get_finding review_finding add_evidence todo_write");
        }
        Some(Commands::Connect {
            provider,
            model,
            base_url,
            api_key_env,
        }) => {
            let mut cfg = if config::config_path(&root).is_file() {
                config::ProjectConfig::load(config::config_path(&root))?
            } else {
                config::init_project(&root, None)?
            };
            cfg.provider.name = Some(provider.clone());
            cfg.provider.model = Some(model.clone());
            cfg.provider.base_url = base_url;
            cfg.provider.api_key_env = api_key_env
                .or_else(|| bugbee_llm::default_api_key_env(&provider).map(ToOwned::to_owned));
            cfg.save(config::config_path(&root))?;
            println!("connected provider={provider} model={model}");
            if let Some(env) = &cfg.provider.api_key_env {
                println!("set API key via environment variable: {env}");
            }
        }
        Some(Commands::Ask { question }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let ws = Workspace::open(&root)?;
                let q = question.join(" ");
                let client = bugbee_llm::from_env(
                    ws.config.provider.name.as_deref(),
                    ws.config.provider.model.as_deref(),
                    ws.config.provider.base_url.as_deref(),
                    ws.config.provider.api_key_env.as_deref(),
                )?;
                let findings = ws.store.list(None)?;
                let mut ctx = format!(
                    "Project: {}\nRoot: {}\nFindings (top 15):\n",
                    ws.config.project.name,
                    ws.root.display()
                );
                for f in findings.iter().take(15) {
                    ctx.push_str(&format!(
                        "- [{}] {} @ {}:{} — {}\n",
                        f.severity.as_str(),
                        f.title,
                        f.location.path,
                        f.location.start_line,
                        f.message
                    ));
                }
                let ctx = ws.redactor.redact(&ctx);
                let q = ws.redactor.redact(&q);
                let req = ChatRequest::new(vec![
                    ChatMessage::system(
                        "You are Bugbee, a defensive AppSec assistant. \
                         Only discuss authorized code security. No exploit weaponization. \
                         Prefer evidence and file:line references.",
                    ),
                    ChatMessage::user(format!("{ctx}\n\nQuestion: {q}")),
                ]);
                let resp = client.chat(req).await?;
                println!("{}", resp.content);
                Ok::<(), anyhow::Error>(())
            })?;
        }
    }

    Ok(())
}

fn resolve_root(explicit: Option<&std::path::Path>, cwd: &std::path::Path) -> Result<PathBuf> {
    if let Some(r) = explicit {
        return Ok(r.to_path_buf());
    }
    if let Some(found) = config::find_project_root(cwd) {
        return Ok(found);
    }
    Ok(cwd.to_path_buf())
}
