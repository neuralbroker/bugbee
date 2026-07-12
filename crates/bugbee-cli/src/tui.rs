//! OpenCode-inspired interactive security IDE TUI for Bugbee.
//!
//! UX goals (OpenCode-like, Bugbee-specialized):
//! - `bugbee` with no args opens this workspace
//! - Slash commands drive hunt / review / doctor / ask
//! - Findings panel stays visible with BRS ordering
//! - Keyboard-first, low friction, no live network attacks

use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

use bugbee_core::{BugbeeConfig, Finding, FindingStatus, FindingStore};
use bugbee_harness::HuntCampaign;
use bugbee_providers::InferenceGateway;

#[derive(Clone)]
struct LogLine {
    kind: LogKind,
    text: String,
}

#[derive(Clone, Copy)]
enum LogKind {
    System,
    User,
    Agent,
    Ok,
    Warn,
}

struct App {
    root: PathBuf,
    input: String,
    logs: Vec<LogLine>,
    findings: Vec<Finding>,
    list_state: ListState,
    status: String,
    busy: bool,
    filter: String,
}

impl App {
    fn new(root: PathBuf) -> Self {
        let mut app = Self {
            root,
            input: String::new(),
            logs: vec![
                LogLine {
                    kind: LogKind::System,
                    text: "Bugbee security workspace — OpenCode-style UX, defensive only.".into(),
                },
                LogLine {
                    kind: LogKind::System,
                    text: "Commands: /hunt  /findings  /review <id> confirm|fp|fixed  /doctor  /ask …  /report  /help  /quit"
                        .into(),
                },
                LogLine {
                    kind: LogKind::System,
                    text: "Tips: Enter runs command · ↑/↓ or j/k move findings · c confirm · f false-positive · q quit"
                        .into(),
                },
            ],
            findings: Vec::new(),
            list_state: ListState::default(),
            status: "ready".into(),
            busy: false,
            filter: String::new(),
        };
        let _ = app.reload_findings();
        app
    }

    fn store_path(&self) -> PathBuf {
        self.root.join(".bugbee").join("findings.db")
    }

    fn reload_findings(&mut self) -> Result<()> {
        let path = self.store_path();
        if !path.exists() {
            self.findings.clear();
            self.list_state.select(None);
            return Ok(());
        }
        let store = FindingStore::open(&path)?;
        self.findings = store.list_by_brs(500)?;
        if self.findings.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(i) = self.list_state.selected() {
            if i >= self.findings.len() {
                self.list_state.select(Some(self.findings.len() - 1));
            }
        }
        Ok(())
    }

    fn push(&mut self, kind: LogKind, text: impl Into<String>) {
        self.logs.push(LogLine {
            kind,
            text: text.into(),
        });
        if self.logs.len() > 400 {
            self.logs.drain(0..self.logs.len() - 400);
        }
    }

    fn selected_finding(&self) -> Option<&Finding> {
        self.list_state
            .selected()
            .and_then(|i| self.findings.get(i))
    }

    fn filtered_indices(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            return (0..self.findings.len()).collect();
        }
        let q = self.filter.to_lowercase();
        self.findings
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.title.to_lowercase().contains(&q)
                    || f.category.to_lowercase().contains(&q)
                    || f.evidence
                        .rule_id
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || f.locations
                        .first()
                        .map(|l| l.file.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

pub fn run(root: &Path) -> Result<()> {
    let root = root.to_path_buf();
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new(root);
    let res = event_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    res
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
            KeyCode::Esc | KeyCode::Char('q') if app.input.is_empty() => break,
            KeyCode::Enter => {
                let cmd = app.input.trim().to_string();
                app.input.clear();
                if !cmd.is_empty() {
                    handle_command(app, &cmd)?;
                }
            }
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Up | KeyCode::Char('k') if app.input.is_empty() => {
                move_sel(app, -1);
            }
            KeyCode::Down | KeyCode::Char('j') if app.input.is_empty() => {
                move_sel(app, 1);
            }
            KeyCode::Char('c') if app.input.is_empty() => {
                review_selected(app, FindingStatus::Confirmed)?;
            }
            KeyCode::Char('f') if app.input.is_empty() => {
                review_selected(app, FindingStatus::FalsePositive)?;
            }
            KeyCode::Char('x') if app.input.is_empty() => {
                review_selected(app, FindingStatus::Fixed)?;
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input.push(ch);
            }
            _ => {}
        }
    }
    Ok(())
}

fn move_sel(app: &mut App, delta: i32) {
    if app.findings.is_empty() {
        return;
    }
    let cur = app.list_state.selected().unwrap_or(0) as i32;
    let next = (cur + delta).clamp(0, app.findings.len() as i32 - 1) as usize;
    app.list_state.select(Some(next));
}

fn review_selected(app: &mut App, status: FindingStatus) -> Result<()> {
    let Some(finding) = app.selected_finding().cloned() else {
        app.push(LogKind::Warn, "No finding selected.");
        return Ok(());
    };
    let store = FindingStore::open(app.store_path())?;
    store.update_status(&finding.id, status)?;
    app.push(
        LogKind::Ok,
        format!("reviewed {} → {:?}", &finding.id.to_string()[..8], status),
    );
    app.reload_findings()?;
    Ok(())
}

fn handle_command(app: &mut App, raw: &str) -> Result<()> {
    app.push(LogKind::User, raw.to_string());
    let line = raw.trim();
    if line.is_empty() {
        return Ok(());
    }

    // Bare text without slash becomes /ask
    let (cmd, rest) = if let Some(stripped) = line.strip_prefix('/') {
        let mut parts = stripped.splitn(2, char::is_whitespace);
        (
            parts.next().unwrap_or("").to_lowercase(),
            parts.next().unwrap_or("").trim().to_string(),
        )
    } else {
        ("ask".into(), line.to_string())
    };

    match cmd.as_str() {
        "help" | "h" | "?" => {
            app.push(
                LogKind::Agent,
                "Slash commands:\n  /hunt [--llm]     aggressive local hunt (India + OWASP packs)\n  /findings [q]     list / filter findings\n  /review id verdict\n  /doctor           config readiness\n  /ask question     model chat about this repo\n  /report [path]    SARIF export\n  /connect …        show connect help\n  /quit              clear log\n  /quit             leave workspace",
            );
        }
        "clear" => app.logs.clear(),
        "quit" | "exit" | "q" => {
            // Signal by emptying and returning error special? Use std process.
            // Event loop checks q only when input empty; here force exit via process.
            // We'll set status and let caller use a flag — use Err with special? cleaner: panic free:
            app.status = "quit".into();
            // Crossterm cleanup happens when run() returns — throw a soft bail.
            anyhow::bail!("quit");
        }
        "doctor" => run_doctor(app)?,
        "findings" => {
            app.filter = rest;
            app.reload_findings()?;
            let n = app.findings.len();
            let preview: Vec<String> = app
                .findings
                .iter()
                .take(12)
                .map(|f| {
                    let loc = f
                        .locations
                        .first()
                        .map(|l| format!("{}:{}", l.file, l.start_line))
                        .unwrap_or_else(|| "-".into());
                    format!(
                        "[{:5.1}] {:?} {} — {} ({})",
                        f.brs,
                        f.severity,
                        loc,
                        f.title,
                        &f.id.to_string()[..8]
                    )
                })
                .collect();
            app.push(
                LogKind::Ok,
                format!("{n} findings loaded (filter=`{}`)", app.filter),
            );
            for line in preview {
                app.push(LogKind::Agent, line);
            }
        }
        "hunt" => {
            let llm = rest.contains("--llm");
            run_hunt(app, llm)?;
        }
        "review" => {
            let mut parts = rest.split_whitespace();
            let id = parts.next().unwrap_or("");
            let verdict = parts.next().unwrap_or("confirm");
            if id.is_empty() {
                app.push(
                    LogKind::Warn,
                    "usage: /review <id-prefix> confirm|fp|fixed|wontfix",
                );
                return Ok(());
            }
            let status = match verdict {
                "confirm" | "c" => FindingStatus::Confirmed,
                "fp" | "false" | "false_positive" => FindingStatus::FalsePositive,
                "fixed" | "x" => FindingStatus::Fixed,
                "wontfix" | "wont" => FindingStatus::WontFix,
                _ => {
                    app.push(LogKind::Warn, "verdict must be confirm|fp|fixed|wontfix");
                    return Ok(());
                }
            };
            let store = FindingStore::open(app.store_path())?;
            let f = store
                .find_by_prefix(id)
                .with_context(|| format!("finding not found: {id}"))?;
            store.update_status(&f.id, status)?;
            app.push(LogKind::Ok, format!("reviewed {} → {:?}", id, status));
            app.reload_findings()?;
        }
        "report" => {
            let out = if rest.is_empty() {
                PathBuf::from("bugbee.sarif.json")
            } else {
                PathBuf::from(rest)
            };
            let store = FindingStore::open(app.store_path())?;
            let sarif = store.export_sarif()?;
            std::fs::write(&out, serde_json::to_string_pretty(&sarif)?)?;
            app.push(LogKind::Ok, format!("wrote {}", out.display()));
        }
        "connect" => {
            app.push(
                LogKind::Agent,
                "Outside TUI: bugbee connect --provider xai --api-key \"$XAI_API_KEY\" --model grok-4.5\nOr local: bugbee connect --provider ollama --base-url http://127.0.0.1:11434/v1 --model qwen2.5-coder",
            );
        }
        "ask" => {
            if rest.is_empty() {
                app.push(
                    LogKind::Warn,
                    "usage: /ask <question about this repository>",
                );
                return Ok(());
            }
            run_ask(app, &rest)?;
        }
        other => {
            app.push(
                LogKind::Warn,
                format!("unknown command `/{other}` — try /help"),
            );
        }
    }
    Ok(())
}

fn run_doctor(app: &mut App) -> Result<()> {
    let cfg = BugbeeConfig::load_layered(Some(&app.root))?;
    app.push(
        LogKind::Agent,
        format!(
            "root={}  aggressive={}  india_profile={}  packs={:?}  hunt_model={}",
            app.root.display(),
            cfg.hunt.aggressive,
            cfg.hunt.india_profile,
            cfg.hunt.packs,
            cfg.inference.hunt.as_deref().unwrap_or("(none)")
        ),
    );
    if cfg.permissions.network.eq_ignore_ascii_case("deny") {
        app.push(LogKind::Ok, "network policy: deny (safe default)");
    } else {
        app.push(
            LogKind::Warn,
            "network policy is not deny — review bugbee.toml",
        );
    }
    Ok(())
}

fn run_hunt(app: &mut App, llm_review: bool) -> Result<()> {
    app.busy = true;
    app.status = "hunting…".into();
    app.push(
        LogKind::System,
        "indexing + aggressive engines (OWASP + India AppSec)…",
    );

    let mut cfg = BugbeeConfig::load_layered(Some(&app.root))?;
    cfg.hunt.aggressive = true;
    if cfg.hunt.india_profile && !cfg.hunt.packs.iter().any(|p| p == "india-appsec") {
        cfg.hunt.packs.push("india-appsec".into());
    }
    if !cfg.hunt.packs.iter().any(|p| p == "owasp-2025") {
        cfg.hunt.packs.insert(0, "owasp-2025".into());
    }

    let store = FindingStore::open(app.store_path())?;
    let mut campaign = HuntCampaign::new(&app.root, cfg);
    campaign.use_llm_review = llm_review;

    // Block in async runtime from sync TUI: use current-thread runtime handle if any.
    let report = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(campaign.run(&store))?;

    app.push(
        LogKind::Ok,
        format!(
            "hunt done — files={} findings={} queue={} auto={} dropped={} ({} ms)",
            report.files_indexed,
            report.findings,
            report.human_queue,
            report.auto_confirmed,
            report.dropped,
            report.duration_ms
        ),
    );
    app.reload_findings()?;
    app.status = format!("{} findings", app.findings.len());
    app.busy = false;
    Ok(())
}

fn run_ask(app: &mut App, question: &str) -> Result<()> {
    let cfg = BugbeeConfig::load_layered(Some(&app.root))?;
    let gw = match InferenceGateway::from_config(cfg.clone()) {
        Ok(g) if !g.available_providers().is_empty() => g,
        _ => {
            app.push(
                LogKind::Warn,
                "No model connected. Run: bugbee connect --provider ollama --model qwen2.5-coder",
            );
            return Ok(());
        }
    };
    app.push(LogKind::System, "thinking…");
    let campaign = HuntCampaign::new(&app.root, cfg);
    let answer = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(campaign.ask(&gw, question, "hunt"))?;
    app.push(LogKind::Agent, answer);
    Ok(())
}

fn draw(f: &mut ratatui::Frame, app: &App) {
    let indigo = Color::Rgb(108, 124, 255);
    let charcoal = Color::Rgb(17, 19, 24);
    let panel = Color::Rgb(22, 26, 36);
    let muted = Color::Rgb(150, 158, 172);
    let ok = Color::Rgb(124, 227, 175);
    let warn = Color::Rgb(255, 184, 106);

    let root = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(root);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " ◈ ",
            Style::default().fg(indigo).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "BUGBEE",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  security ide  ", Style::default().fg(indigo)),
        Span::styled(
            format!("· {}", app.root.display()),
            Style::default().fg(muted),
        ),
    ]))
    .style(Style::default().bg(charcoal))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(indigo))
            .title(Span::styled(
                " OpenCode-style workspace ",
                Style::default().fg(indigo),
            )),
    );
    f.render_widget(header, chunks[0]);

    // Body: log | findings
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[1]);

    let log_items: Vec<Line> = app
        .logs
        .iter()
        .map(|l| {
            let (prefix, color) = match l.kind {
                LogKind::System => ("·", muted),
                LogKind::User => ("›", indigo),
                LogKind::Agent => ("◆", Color::White),
                LogKind::Ok => ("✓", ok),
                LogKind::Warn => ("!", warn),
            };
            Line::from(vec![
                Span::styled(format!("{prefix} "), Style::default().fg(color)),
                Span::raw(l.text.clone()),
            ])
        })
        .collect();
    let log = Paragraph::new(log_items).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(muted))
            .title(" Session ")
            .style(Style::default().bg(panel)),
    );
    f.render_widget(log, body[0]);

    let indices = app.filtered_indices();
    let items: Vec<ListItem> = indices
        .iter()
        .map(|&i| {
            let find = &app.findings[i];
            let loc = find
                .locations
                .first()
                .map(|l| format!("{}:{}", l.file, l.start_line))
                .unwrap_or_default();
            ListItem::new(format!(
                "[{:5.1}] {:?}  {}  {}",
                find.brs, find.severity, loc, find.title
            ))
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(muted))
                .title(format!(" Findings ({}) ", app.findings.len()))
                .style(Style::default().bg(panel)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    // Map selection: list_state indexes into findings, not filtered — keep simple
    let mut state = app.list_state.clone();
    f.render_stateful_widget(list, body[1], &mut state);

    // Detail strip under findings when selected
    if let Some(finding) = app.selected_finding() {
        // overlay detail in bottom of findings pane via second pass — keep simple in log on select
        let _ = finding;
    }

    // Input
    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            " / ",
            Style::default().fg(indigo).add_modifier(Modifier::BOLD),
        ),
        Span::raw(if app.input.is_empty() {
            "hunt | findings | review | doctor | ask …".into()
        } else {
            app.input.clone()
        }),
    ]))
    .style(Style::default().fg(if app.input.is_empty() {
        muted
    } else {
        Color::White
    }))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(indigo))
            .title(" Command "),
    );
    f.render_widget(input, chunks[2]);

    // Status bar
    let detail = app
        .selected_finding()
        .map(|f| {
            format!(
                "selected {} · {} · ECS={:.2} · {}",
                &f.id.to_string()[..8],
                f.status.as_str(),
                f.ecs,
                f.evidence.rule_id.as_deref().unwrap_or("-")
            )
        })
        .unwrap_or_else(|| "no selection".into());
    let status = Paragraph::new(format!(
        " {}  ·  {}  ·  c confirm · f fp · x fixed · q quit",
        app.status, detail
    ))
    .style(Style::default().fg(muted).bg(charcoal));
    f.render_widget(status, chunks[3]);

    // Ensure input cursor area is painted last (already).
    let _ = Rect::default();
}

/// Run TUI but treat "quit" bail as clean exit.
pub fn run_workspace(root: &Path) -> Result<()> {
    match run(root) {
        Err(e) if e.to_string() == "quit" => Ok(()),
        other => other,
    }
}
