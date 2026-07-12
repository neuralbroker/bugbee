use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing_subscriber::EnvFilter;

use bugbee_core::{BugbeeConfig, FindingStatus, FindingStore};
use bugbee_harness::HuntCampaign;
use bugbee_providers::InferenceGateway;

mod tui;

#[derive(Parser, Debug)]
#[command(
    name = "bugbee",
    version,
    about = "Agentic bug & vulnerability hunting IDE (terminal-first, OpenCode-style UX)"
)]
struct Cli {
    /// Project root (default: cwd)
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Subcommand. When omitted, opens the interactive workspace (OpenCode-style).
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize bugbee.toml and .bugbee/ in the project
    Init {
        #[arg(long)]
        name: Option<String>,
    },
    /// Show how to connect any model provider (BYOK — any model)
    Connect {
        /// Provider id (e.g. xai, deepseek, openai, ollama, or custom)
        provider: Option<String>,
        /// API key to store in the OS keychain (never written to bugbee.toml)
        #[arg(long)]
        api_key: Option<String>,
        /// Custom OpenAI-compatible base URL
        #[arg(long)]
        base_url: Option<String>,
        /// Model id to set as default hunt model
        #[arg(long)]
        model: Option<String>,
    },
    /// Run a deterministic (+ optional LLM review) hunt
    Hunt {
        /// Primary rule pack name under rules/ (additional packs load from config)
        #[arg(long, default_value = "owasp-2025")]
        pack: String,
        /// Also load India AppSec pack (gov/edu/BFSI/enterprise)
        #[arg(long, default_value_t = true)]
        india: bool,
        /// Maximize finding surface (lower drop threshold, queue more candidates)
        #[arg(long, default_value_t = true)]
        aggressive: bool,
        /// Enable LLM adversarial review (requires configured model)
        #[arg(long)]
        llm_review: bool,
        /// Auto-approve safe permission asks
        #[arg(long)]
        auto: bool,
    },
    /// List findings sorted by BRS
    Findings {
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Review a finding: confirm | fp | fixed
    Review {
        id: String,
        #[arg(value_enum)]
        verdict: ReviewVerdict,
    },
    /// Export SARIF report
    Report {
        #[arg(long, default_value = "bugbee.sarif.json")]
        output: PathBuf,
    },
    /// Ask the agent (any configured model) a question about the codebase
    Ask {
        question: String,
        #[arg(long, default_value = "hunt")]
        role: String,
    },
    /// List configured providers and models
    Models { provider: Option<String> },
    /// Check local configuration, model routing, and safe defaults without making network calls
    Doctor,
    /// Launch terminal UI
    Tui,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ReviewVerdict {
    Confirm,
    Fp,
    Fixed,
    WontFix,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("bugbee=info".parse()?))
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let root_arg = cli.root.clone().unwrap_or(std::env::current_dir()?);
    let root = root_arg.canonicalize().unwrap_or(root_arg);

    match cli.cmd {
        None | Some(Commands::Tui) => tui::run_workspace(&root)?,
        Some(Commands::Init { name }) => cmd_init(&root, name)?,
        Some(Commands::Connect {
            provider,
            api_key,
            base_url,
            model,
        }) => cmd_connect(&root, provider, api_key, base_url, model)?,
        Some(Commands::Hunt {
            pack,
            india,
            aggressive,
            llm_review,
            auto,
        }) => cmd_hunt(&root, &pack, india, aggressive, llm_review, auto).await?,
        Some(Commands::Findings { limit }) => cmd_findings(&root, limit)?,
        Some(Commands::Review { id, verdict }) => cmd_review(&root, &id, verdict)?,
        Some(Commands::Report { output }) => cmd_report(&root, &output)?,
        Some(Commands::Ask { question, role }) => cmd_ask(&root, &question, &role).await?,
        Some(Commands::Models { provider }) => cmd_models(&root, provider.as_deref()).await?,
        Some(Commands::Doctor) => cmd_doctor(&root)?,
    }
    Ok(())
}

fn store_path(root: &std::path::Path) -> PathBuf {
    root.join(".bugbee").join("findings.db")
}

fn cmd_init(root: &std::path::Path, name: Option<String>) -> Result<()> {
    let cfg_path = BugbeeConfig::project_config_path(root);
    let mut cfg = if cfg_path.exists() {
        BugbeeConfig::load(&cfg_path)?
    } else {
        BugbeeConfig::default_project()
    };
    cfg.project_name = name.or_else(|| root.file_name().map(|s| s.to_string_lossy().to_string()));
    cfg.save(&cfg_path)?;
    fs::create_dir_all(root.join(".bugbee"))?;
    fs::create_dir_all(root.join(".bugbee").join("patches"))?;
    let agents = root.join("BUGBEE.md");
    if !agents.exists() {
        fs::write(
            &agents,
            r#"# Bugbee project guide

## Scope
Defensive bug fixing and vulnerability hunting only.

## Languages
python, javascript, typescript, go

## Notes for agents
- Prefer evidence and tests over speculation
- Never commit secrets
- Human review required for production changes
"#,
        )?;
    }
    println!("{} {}", "initialized".green().bold(), cfg_path.display());
    println!("  store: {}", store_path(root).display());
    println!("  next:  bugbee                  # OpenCode-style workspace");
    println!("         bugbee hunt             # aggressive + India packs");
    println!("         bugbee connect --provider xai --model grok-4.5");
    Ok(())
}

fn cmd_connect(
    root: &std::path::Path,
    provider: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<()> {
    let cfg_path = BugbeeConfig::project_config_path(root);
    let mut cfg = if cfg_path.exists() {
        BugbeeConfig::load(&cfg_path)?
    } else {
        BugbeeConfig::default_project()
    };

    if provider.is_none() {
        println!("{}", "Bugbee is model-agnostic — use ANY model.".bold());
        println!("Providers (examples; add any OpenAI-compatible endpoint):\n");
        for (id, p) in &cfg.providers {
            println!(
                "  {}  {}  ({})",
                id.cyan(),
                p.name.clone().unwrap_or_default(),
                p.base_url
            );
        }
        println!("\n{}", "Examples:".bold());
        println!("  bugbee connect --provider xai --api-key \"$XAI_API_KEY\" --model grok-4.5");
        println!("  bugbee connect --provider deepseek --model deepseek-v4-pro");
        println!("  bugbee connect --provider ollama --base-url http://127.0.0.1:11434/v1 --model qwen2.5-coder");
        println!("  bugbee connect --provider mygw --base-url https://gateway/v1 --api-key KEY --model any-model-id");
        println!("\nOr set env vars: XAI_API_KEY, OPENAI_API_KEY, DEEPSEEK_API_KEY, OPENROUTER_API_KEY, …");
        return Ok(());
    }

    let pid = provider.unwrap();
    let entry = cfg
        .providers
        .entry(pid.clone())
        .or_insert_with(|| bugbee_core::ProviderConfig {
            name: Some(pid.clone()),
            base_url: base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".into()),
            api_key_env: Some(format!("{}_API_KEY", pid.to_uppercase().replace('-', "_"))),
            api_key: None,
            models: vec![],
            headers: Default::default(),
            protocol: "openai_compat".into(),
        });
    if let Some(u) = base_url {
        entry.base_url = u;
    }
    if let Some(m) = model {
        cfg.inference.hunt = Some(format!("{pid}/{m}"));
        cfg.inference.scout = Some(format!("{pid}/{m}"));
        cfg.inference.review = Some(format!("{pid}/{m}"));
        cfg.inference.patch = Some(format!("{pid}/{m}"));
        cfg.inference.default_provider = Some(pid.clone());
        if !entry.models.contains(&m) {
            entry.models.push(m);
        }
    }
    if let Some(k) = api_key {
        cfg.store_api_key(&pid, &k)?;
    }
    cfg.save(&cfg_path)?;
    println!(
        "{} provider `{}` → {}",
        "connected".green().bold(),
        pid,
        cfg_path.display()
    );
    if let Some(h) = &cfg.inference.hunt {
        println!("  default hunt model: {h}");
    }
    Ok(())
}

async fn cmd_hunt(
    root: &std::path::Path,
    pack: &str,
    india: bool,
    aggressive: bool,
    llm_review: bool,
    auto: bool,
) -> Result<()> {
    let mut cfg = BugbeeConfig::load_layered(Some(root))?;
    cfg.hunt.aggressive = aggressive;
    cfg.hunt.india_profile = india;
    if india && !cfg.hunt.packs.iter().any(|p| p == "india-appsec") {
        cfg.hunt.packs.push("india-appsec".into());
    }
    if !cfg.hunt.packs.iter().any(|p| p == pack) {
        cfg.hunt.packs.insert(0, pack.to_string());
    }

    let store = FindingStore::open(store_path(root))?;
    let mut campaign = HuntCampaign::new(root, cfg);
    campaign.use_llm_review = llm_review;
    campaign.auto_approve = auto;
    let pack_dir = root.join("rules").join(pack);
    if pack_dir.exists() {
        campaign.rules_dirs.insert(0, pack_dir);
    }
    if india {
        let india_dir = root.join("rules").join("india-appsec");
        if india_dir.exists() {
            campaign.rules_dirs.insert(0, india_dir);
        }
    }

    println!(
        "{} hunting in {}  (aggressive={} india={}) …",
        "bugbee".magenta().bold(),
        root.display(),
        aggressive,
        india
    );
    let report = campaign.run(&store).await?;
    println!("{}", "── Hunt report ──".bold());
    println!("  files indexed : {}", report.files_indexed);
    println!("  findings      : {}", report.findings);
    println!("  human queue   : {}", report.human_queue);
    println!("  auto-confirm  : {}", report.auto_confirmed);
    println!("  dropped       : {}", report.dropped);
    println!("  duration      : {} ms", report.duration_ms);
    println!("\n  bugbee findings");
    println!("  bugbee            # OpenCode-style workspace");
    Ok(())
}

fn cmd_findings(root: &std::path::Path, limit: usize) -> Result<()> {
    let store = FindingStore::open(store_path(root))?;
    let list = store.list_by_brs(limit)?;
    if list.is_empty() {
        println!("No findings. Run `bugbee hunt` first.");
        return Ok(());
    }
    for f in list {
        let sev = match f.severity {
            bugbee_core::Severity::Critical => "CRIT".red().bold(),
            bugbee_core::Severity::High => "HIGH".red(),
            bugbee_core::Severity::Medium => "MED ".yellow(),
            bugbee_core::Severity::Low => "LOW ".blue(),
            bugbee_core::Severity::Info => "INFO".normal(),
        };
        let loc = f
            .locations
            .first()
            .map(|l| format!("{}:{}", l.file, l.start_line))
            .unwrap_or_else(|| "-".into());
        println!(
            "{} BRS={:5.1} ECS={:.2} status={} id={}  {}  {}",
            sev,
            f.brs,
            f.ecs,
            f.status.as_str(),
            &f.id.to_string()[..8],
            loc,
            f.title
        );
    }
    Ok(())
}

fn cmd_review(root: &std::path::Path, id: &str, verdict: ReviewVerdict) -> Result<()> {
    let store = FindingStore::open(store_path(root))?;
    let f = store
        .find_by_prefix(id)
        .with_context(|| format!("could not resolve finding id: {id}"))?;
    let status = match verdict {
        ReviewVerdict::Confirm => FindingStatus::Confirmed,
        ReviewVerdict::Fp => FindingStatus::FalsePositive,
        ReviewVerdict::Fixed => FindingStatus::Fixed,
        ReviewVerdict::WontFix => FindingStatus::WontFix,
    };
    store.update_status(&f.id, status)?;
    println!("{} {} → {:?}", "reviewed".green().bold(), f.id, status);
    Ok(())
}

fn cmd_report(root: &std::path::Path, output: &std::path::Path) -> Result<()> {
    let store = FindingStore::open(store_path(root))?;
    let sarif = store.export_sarif()?;
    fs::write(output, serde_json::to_string_pretty(&sarif)?)?;
    println!("{} {}", "wrote".green().bold(), output.display());
    Ok(())
}

async fn cmd_ask(root: &std::path::Path, question: &str, role: &str) -> Result<()> {
    let cfg = BugbeeConfig::load_layered(Some(root))?;
    let gw = InferenceGateway::from_config(cfg.clone())?;
    if gw.available_providers().is_empty() {
        anyhow::bail!("No providers connected. Run `bugbee connect` and set a model.");
    }
    let campaign = HuntCampaign::new(root, cfg);
    print!("{}", "thinking…\n".dimmed());
    io::stdout().flush()?;
    let answer = campaign.ask(&gw, question, role).await?;
    println!("{answer}");
    Ok(())
}

async fn cmd_models(root: &std::path::Path, provider: Option<&str>) -> Result<()> {
    let cfg = BugbeeConfig::load_layered(Some(root))?;
    if let Some(p) = provider {
        let gw = InferenceGateway::from_config(cfg)?;
        match gw.list_models(p).await {
            Ok(models) => {
                println!("Models from {p} (any id may still work if not listed):");
                for m in models {
                    println!("  {m}");
                }
            }
            Err(e) => {
                println!("Could not list models: {e}\nYou can still set any model id manually.")
            }
        }
    } else {
        println!(
            "{}",
            "Configured providers (platform accepts ANY model):".bold()
        );
        for (id, p) in &cfg.providers {
            println!(
                "  {}  base={}  models={:?}",
                id.cyan(),
                p.base_url,
                p.models
            );
        }
        println!(
            "\nhunt={} scout={} review={}",
            cfg.inference.hunt.as_deref().unwrap_or("-"),
            cfg.inference.scout.as_deref().unwrap_or("-"),
            cfg.inference.review.as_deref().unwrap_or("-"),
        );
    }
    Ok(())
}

fn cmd_doctor(root: &std::path::Path) -> Result<()> {
    let config_path = BugbeeConfig::project_config_path(root);
    let cfg = BugbeeConfig::load_layered(Some(root))?;
    let mut warnings = 0usize;

    println!("{}", "Bugbee doctor".bold());
    println!("  root: {}", root.display());

    if config_path.exists() {
        pass("project configuration", &config_path.display().to_string());
    } else {
        warnings += 1;
        warn("project configuration", "missing — run `bugbee init`");
    }

    if cfg.permissions.network.eq_ignore_ascii_case("deny") {
        pass("agent network policy", "deny by default");
    } else {
        warnings += 1;
        warn(
            "agent network policy",
            "not set to deny; review [permissions].network",
        );
    }

    if cfg
        .permissions
        .external_directory
        .eq_ignore_ascii_case("deny")
    {
        pass("external directory policy", "deny by default");
    } else {
        warnings += 1;
        warn(
            "external directory policy",
            "not set to deny; review [permissions].external_directory",
        );
    }

    for (role, model_ref) in [
        ("hunt", cfg.inference.hunt.as_deref()),
        ("scout", cfg.inference.scout.as_deref()),
        ("review", cfg.inference.review.as_deref()),
        ("patch", cfg.inference.patch.as_deref()),
    ] {
        let Some(model_ref) = model_ref else {
            warnings += 1;
            warn(&format!("{role} model"), "not configured");
            continue;
        };

        let (provider_id, model_id) = match cfg.parse_model_ref(model_ref) {
            Ok(reference) => reference,
            Err(error) => {
                warnings += 1;
                warn(&format!("{role} model"), &error.to_string());
                continue;
            }
        };

        let Some(provider) = cfg.providers.get(&provider_id) else {
            warnings += 1;
            warn(
                &format!("{role} model"),
                &format!("provider `{provider_id}` is not configured"),
            );
            continue;
        };

        if provider.protocol != "openai_compat" {
            warnings += 1;
            warn(
                &format!("{role} model"),
                &format!(
                    "`{provider_id}` uses `{}`; this native adapter is not implemented yet",
                    provider.protocol
                ),
            );
            continue;
        }

        // Local OpenAI-compatible stubs (Ollama et al.) use a non-secret placeholder.
        let is_local_stub = provider.base_url.contains("127.0.0.1")
            || provider.base_url.contains("localhost")
            || provider.api_key.as_deref() == Some("ollama")
            || provider.api_key.as_deref() == Some("local");
        if provider.api_key.is_some() && !is_local_stub {
            warnings += 1;
            warn(
                &format!("{role} provider"),
                "legacy inline API key is configured; migrate it to the OS keychain",
            );
        }

        match cfg.resolve_api_key(&provider_id) {
            Ok(_) => pass(
                &format!("{role} model"),
                &format!("{provider_id}/{model_id} is ready"),
            ),
            Err(_) if is_local_stub => pass(
                &format!("{role} model"),
                &format!("{provider_id}/{model_id} local endpoint (no key required)"),
            ),
            Err(_) => {
                warnings += 1;
                let source = provider
                    .api_key_env
                    .as_deref()
                    .unwrap_or("a local provider key");
                warn(
                    &format!("{role} model"),
                    &format!("{provider_id}/{model_id} needs {source}"),
                );
            }
        }
    }

    if store_path(root).exists() {
        pass("finding store", "ready");
    } else {
        println!(
            "  {} finding store: created on the first hunt",
            "•".dimmed()
        );
    }

    if warnings == 0 {
        println!("\n{}", "No issues found.".green().bold());
    } else {
        println!(
            "\n{} {} item(s) need attention before model-backed workflows.",
            "Review:".yellow().bold(),
            warnings
        );
    }
    Ok(())
}

fn pass(label: &str, detail: &str) {
    println!("  {} {}: {}", "✓".green().bold(), label, detail);
}

fn warn(label: &str, detail: &str) {
    println!("  {} {}: {}", "!".yellow().bold(), label, detail);
}
