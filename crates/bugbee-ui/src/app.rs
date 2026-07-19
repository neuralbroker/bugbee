//! OpenCode-class terminal IDE for Bugbee.
//!
//! Screens: Home (logo + prompt) → Session (transcript + prompt + sidebar + footer).
//! Keymap loosely follows OpenCode: Tab agent, / commands, Ctrl+B sidebar, Esc back.

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bugbee_agent::stream::{run_godmode_streaming, run_swarm_streaming, StreamEvent};
use bugbee_agent::{GodmodeOptions, SwarmOptions, Workspace};
use bugbee_core::{Finding, FindingStatus, Severity};
use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};
use ratatui::Frame;
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::security_panel::SecurityPanel;

use crate::logo::{bugbee_logo_lines, small_brand};
use crate::theme;

// ── Message model (session transcript) ───────────────────────────

#[derive(Clone)]
enum MsgKind {
    User,
    Assistant,
    System,
    Tool,
    Finding,
}

#[derive(Clone)]
struct ChatMsg {
    kind: MsgKind,
    text: String,
    at: String,
}

// ── App state ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Session,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Prompt,
    Transcript,
    Sidebar,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidebarMode {
    Findings,
    Detail,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AgentMode {
    Hunt,   // OpenCode "build"
    Review, // OpenCode "plan"
}

impl AgentMode {
    fn label(self) -> &'static str {
        match self {
            AgentMode::Hunt => "hunt",
            AgentMode::Review => "review",
        }
    }

    fn opencode(self) -> &'static str {
        match self {
            AgentMode::Hunt => "build",
            AgentMode::Review => "plan",
        }
    }

    fn toggle(self) -> Self {
        match self {
            AgentMode::Hunt => AgentMode::Review,
            AgentMode::Review => AgentMode::Hunt,
        }
    }
}

struct App {
    workspace: Workspace,
    screen: Screen,
    focus: Focus,
    agent: AgentMode,
    sidebar_mode: SidebarMode,
    sec_panel: SecurityPanel,
    prompt: String,
    cursor: usize,
    history: Vec<String>,
    history_idx: Option<usize>,
    messages: Vec<ChatMsg>,
    scroll: usize,
    findings: Vec<Finding>,
    find_state: ListState,
    sidebar: bool,
    status: String,
    busy: bool,
    spinner_i: usize,
    toast: Option<(String, Instant)>,
    stream_rx: Option<mpsc::Receiver<StreamEvent>>,
    placeholders: Vec<&'static str>,
    placeholder_i: usize,
    tick: Instant,
}

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const SLASH: &[(&str, &str)] = &[
    ("/help",            "Show keyboard shortcuts"),
    ("/hunt",            "Run deterministic vulnerability hunt"),
    ("/deep-hunt",       "Deep analysis (full pipeline + enrichment)"),
    ("/swarm",           "Multi-agent neuro-symbolic analysis"),
    ("/godmode",         "Deep analysis pipeline (offline)"),
    ("/findings",        "Refresh findings sidebar"),
    ("/report",          "Export SARIF + bounty report"),
    ("/doctor",          "Config readiness check"),
    ("/connect",         "Show provider connect hints"),
    ("/status",          "Session + project status"),
    ("/new",             "New session (home)"),
    ("/sidebar",         "Toggle findings sidebar"),
    ("/agent",           "Toggle hunt <-> review agent"),
    ("/clear",           "Clear transcript"),
    ("/quit",            "Exit"),
];

// ── Public entry ─────────────────────────────────────────────────

pub fn run_workspace(workspace: Workspace) -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let findings = workspace.store.list(None).unwrap_or_default();
    let mut app = App {
        workspace,
        screen: Screen::Home,
        focus: Focus::Prompt,
        agent: AgentMode::Hunt,
        sidebar_mode: SidebarMode::Findings,
        sec_panel: SecurityPanel::new(),
        prompt: String::new(),
        cursor: 0,
        history: Vec::new(),
        history_idx: None,
        messages: Vec::new(),
        scroll: 0,
        findings,
        find_state: ListState::default(),
        sidebar: true,
        status: "ready".into(),
        busy: false,
        spinner_i: 0,
        toast: None,
        stream_rx: None,
        placeholders: vec![
            "Hunt this repo for injection and secrets…",
            "Summarize top security risks with file:line",
            "Run swarm and open findings for review",
            "What is the attack surface of this project?",
        ],
        placeholder_i: 0,
        tick: Instant::now(),
    };
    if !app.findings.is_empty() {
        app.find_state.select(Some(0));
    }

    let res = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        if app.tick.elapsed() > Duration::from_millis(80) {
            app.spinner_i = (app.spinner_i + 1) % SPINNER.len();
            if app.tick.elapsed() > Duration::from_secs(4) {
                app.placeholder_i = (app.placeholder_i + 1) % app.placeholders.len();
            }
            app.tick = Instant::now();
            if let Some((_, t)) = &app.toast {
                if t.elapsed() > Duration::from_secs(4) {
                    app.toast = None;
                }
            }
        }

        // Poll stream events from background tasks
        if let Some(ref mut rx) = app.stream_rx {
            match rx.try_recv() {
                Ok(ev) => {
                    app.busy = true;
                    app.status = "streaming…".into();
                    match &ev {
                        StreamEvent::Phase { name } => {
                            push_msg(app, MsgKind::Tool, format!("> phase: {name}"));
                        }
                        StreamEvent::Step { message } => {
                            push_msg(app, MsgKind::Tool, message.clone());
                        }
                        StreamEvent::Finding { count } => {
                            push_msg(
                                app,
                                MsgKind::Finding,
                                format!("{} findings", count),
                            );
                            refresh_findings(app);
                        }
                        StreamEvent::ToolCall { name, args } => {
                            let preview = if args.len() > 80 {
                                format!("{}…", &args[..80])
                            } else {
                                args.clone()
                            };
                            push_msg(
                                app,
                                MsgKind::Tool,
                                format!("[tool] {name}({preview})"),
                            );
                        }
                        StreamEvent::ToolResult { name, ok, preview } => {
                            let status = if *ok { "+" } else { "-" };
                            push_msg(
                                app,
                                MsgKind::Tool,
                                format!("  {status} {name}: {preview}"),
                            );
                        }
                        StreamEvent::Warn { message } => {
                            push_msg(app, MsgKind::System, format!("warn: {message}"));
                        }
                        StreamEvent::Done { summary, elapsed_ms } => {
                            push_msg(
                                app,
                                MsgKind::Assistant,
                                format!("{}\n{}ms", summary, elapsed_ms),
                            );
                            app.busy = false;
                            app.stream_rx = None;
                            app.status = "ready".into();
                            refresh_findings(app);
                            app.sidebar = true;
                        }
                        StreamEvent::Error { message } => {
                            push_msg(app, MsgKind::System, message.clone());
                            app.busy = false;
                            app.stream_rx = None;
                            app.status = "error".into();
                        }
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    app.busy = false;
                    app.stream_rx = None;
                    app.status = "disconnected".into();
                }
            }
        }

        terminal.draw(|f| draw(f, app))?;

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Global
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => return Ok(()),
                KeyCode::Char('b') => {
                    app.sidebar = !app.sidebar;
                    toast(
                        app,
                        if app.sidebar {
                            "sidebar on"
                        } else {
                            "sidebar off"
                        },
                    );
                    continue;
                }
                KeyCode::Char('n') => {
                    new_session(app);
                    continue;
                }
                _ => {}
            }
        }

        if app.focus == Focus::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter) {
                app.focus = Focus::Prompt;
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') if app.prompt.is_empty() && app.screen == Screen::Home => {
                return Ok(());
            }
            KeyCode::Esc => {
                if app.screen == Screen::Session && app.prompt.is_empty() {
                    app.screen = Screen::Home;
                    app.focus = Focus::Prompt;
                } else {
                    app.prompt.clear();
                    app.cursor = 0;
                }
            }
            KeyCode::Tab => {
                app.agent = app.agent.toggle();
                toast(
                    app,
                    &format!("agent mode: {}", app.agent.label()),
                );
            }
            KeyCode::F(1) => app.focus = Focus::Help,
            KeyCode::Char('?') if app.prompt.is_empty() => app.focus = Focus::Help,
            KeyCode::Up if app.focus == Focus::Prompt && key.modifiers.is_empty() => {
                history_up(app);
            }
            KeyCode::Down if app.focus == Focus::Prompt && key.modifiers.is_empty() => {
                history_down(app);
            }
            KeyCode::PageUp => {
                app.scroll = app.scroll.saturating_add(5);
                app.focus = Focus::Transcript;
            }
            KeyCode::PageDown => {
                app.scroll = app.scroll.saturating_sub(5);
                app.focus = Focus::Transcript;
            }
            KeyCode::Char('j') if app.focus == Focus::Sidebar => select_finding(app, 1),
            KeyCode::Char('k') if app.focus == Focus::Sidebar => select_finding(app, -1),
            KeyCode::Down if app.focus == Focus::Sidebar => select_finding(app, 1),
            KeyCode::Up if app.focus == Focus::Sidebar => select_finding(app, -1),
            KeyCode::Char('c') if app.focus == Focus::Sidebar && app.sidebar_mode == SidebarMode::Findings => {
                set_finding_status(app, FindingStatus::Confirmed);
            }
            KeyCode::Char('f') if app.focus == Focus::Sidebar && app.sidebar_mode == SidebarMode::Findings => {
                set_finding_status(app, FindingStatus::FalsePositive);
            }
            KeyCode::Char('x') if app.focus == Focus::Sidebar && app.sidebar_mode == SidebarMode::Findings => {
                set_finding_status(app, FindingStatus::Fixed);
            }
            KeyCode::Char('d') if app.focus == Focus::Sidebar => {
                if app.sidebar_mode == SidebarMode::Findings {
                    if let Some(i) = app.find_state.selected() {
                        if let Some(f) = app.findings.get(i) {
                            app.sec_panel.set_finding(f.clone());
                            app.sidebar_mode = SidebarMode::Detail;
                            toast(app, &format!("detail: {}", f.id));
                        }
                    }
                } else {
                    app.sidebar_mode = SidebarMode::Findings;
                    toast(app, "findings list");
                }
            }
            KeyCode::Char('k') if app.focus == Focus::Sidebar && app.sidebar_mode == SidebarMode::Detail => {
                app.sec_panel.show_chain = !app.sec_panel.show_chain;
                toast(app, if app.sec_panel.show_chain { "kill chain on" } else { "kill chain off" });
            }
            KeyCode::Char('s') if app.prompt.is_empty() && key.modifiers.is_empty() => {
                app.focus = Focus::Sidebar;
                app.sidebar = true;
            }
            KeyCode::Enter if app.focus == Focus::Prompt => {
                if submit_prompt(app) {
                    return Ok(());
                }
            }
            KeyCode::Backspace if app.focus == Focus::Prompt => {
                if app.cursor > 0 {
                    let i = app.cursor - 1;
                    app.prompt.remove(i);
                    app.cursor = i;
                }
            }
            KeyCode::Left if app.focus == Focus::Prompt => {
                app.cursor = app.cursor.saturating_sub(1);
            }
            KeyCode::Right if app.focus == Focus::Prompt => {
                app.cursor = (app.cursor + 1).min(app.prompt.len());
            }
            KeyCode::Home if app.focus == Focus::Prompt => app.cursor = 0,
            KeyCode::End if app.focus == Focus::Prompt => app.cursor = app.prompt.len(),
            KeyCode::Char(c)
                if app.focus == Focus::Prompt && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                // slash at empty starts palette-style filtering in prompt
                app.prompt.insert(app.cursor, c);
                app.cursor += 1;
                app.focus = Focus::Prompt;
            }
            _ => {}
        }
    }
}

// ── Commands ─────────────────────────────────────────────────────

fn toast(app: &mut App, msg: &str) {
    app.toast = Some((msg.to_string(), Instant::now()));
    app.status = msg.to_string();
}

fn push_msg(app: &mut App, kind: MsgKind, text: impl Into<String>) {
    app.messages.push(ChatMsg {
        kind,
        text: text.into(),
        at: Local::now().format("%H:%M:%S").to_string(),
    });
    app.scroll = 0;
}

fn new_session(app: &mut App) {
    app.screen = Screen::Home;
    app.messages.clear();
    app.prompt.clear();
    app.cursor = 0;
    app.focus = Focus::Prompt;
    toast(app, "new session");
}

fn history_up(app: &mut App) {
    if app.history.is_empty() {
        return;
    }
    let i = match app.history_idx {
        None => app.history.len() - 1,
        Some(0) => 0,
        Some(i) => i - 1,
    };
    app.history_idx = Some(i);
    app.prompt = app.history[i].clone();
    app.cursor = app.prompt.len();
}

fn history_down(app: &mut App) {
    let Some(i) = app.history_idx else {
        return;
    };
    if i + 1 >= app.history.len() {
        app.history_idx = None;
        app.prompt.clear();
        app.cursor = 0;
    } else {
        app.history_idx = Some(i + 1);
        app.prompt = app.history[i + 1].clone();
        app.cursor = app.prompt.len();
    }
}

fn select_finding(app: &mut App, delta: i32) {
    if app.findings.is_empty() {
        return;
    }
    let n = app.findings.len() as i32;
    let i = app.find_state.selected().unwrap_or(0) as i32;
    app.find_state
        .select(Some((i + delta).rem_euclid(n) as usize));
}

fn set_finding_status(app: &mut App, status: FindingStatus) {
    let Some(i) = app.find_state.selected() else {
        return;
    };
    let Some(f) = app.findings.get(i) else {
        return;
    };
    let id = f.id.clone();
    match app.workspace.store.set_status(&id, status) {
        Ok(u) => {
            app.findings[i] = u;
            toast(app, &format!("{id} → {}", status.as_str()));
            push_msg(
                app,
                MsgKind::System,
                format!("review: {id} marked {}", status.as_str()),
            );
        }
        Err(e) => toast(app, &format!("error: {e}")),
    }
}

fn refresh_findings(app: &mut App) {
    match app.workspace.store.list(None) {
        Ok(f) => {
            app.findings = f;
            if app.findings.is_empty() {
                app.find_state.select(None);
            } else if app.find_state.selected().is_none() {
                app.find_state.select(Some(0));
            }
            toast(app, &format!("{} findings", app.findings.len()));
        }
        Err(e) => toast(app, &format!("refresh error: {e}")),
    }
}

/// Returns true if app should exit.
fn submit_prompt(app: &mut App) -> bool {
    let raw = app.prompt.trim().to_string();
    if raw.is_empty() {
        return false;
    }
    app.history.push(raw.clone());
    app.history_idx = None;
    app.prompt.clear();
    app.cursor = 0;

    if raw.starts_with('/') {
        return run_slash(app, &raw);
    }

    // Freeform → session message (+ offline agent stub response)
    app.screen = Screen::Session;
    push_msg(app, MsgKind::User, raw.clone());
    app.busy = true;
    app.status = "thinking…".into();

    // Local grounded reply without requiring LLM for beta UX completeness
    let reply = local_reply(app, &raw);
    push_msg(app, MsgKind::Assistant, reply);
    app.busy = false;
    app.status = "ready".into();
    false
}

fn local_reply(app: &App, q: &str) -> String {
    let n = app.findings.len();
    let vul = app
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .count();
    let mut s = format!(
        "Got it. Project **{}** · agent `{}` · {} findings ({} high/critical).\n\n",
        app.workspace.config.project.name,
        app.agent.label(),
        n,
        vul
    );
    s.push_str("For a full analysis run `/hunt` or `/swarm`. ");
    s.push_str("Connect a model with `bugbee connect` then `bugbee super \"...\"` for interactive analysis.\n\n");
    if !app.findings.is_empty() {
        s.push_str("Top findings:\n");
        for f in app.findings.iter().take(5) {
            s.push_str(&format!(
                "• [{}] {}:{} — {}\n",
                f.severity.as_str(),
                f.location.path,
                f.location.start_line,
                f.title
            ));
        }
    } else {
        s.push_str(&format!(
            "You asked: {q}\nNo findings yet — run `/swarm` or `/hunt`."
        ));
    }
    s
}

fn run_slash(app: &mut App, raw: &str) -> bool {
    app.screen = Screen::Session;
    push_msg(app, MsgKind::User, raw.to_string());

    let mut parts = raw.trim_start_matches('/').split_whitespace();
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "q" | "quit" | "exit" => return true,
        "help" | "?" => {
            app.focus = Focus::Help;
        }
        "new" => new_session(app),
        "clear" => {
            app.messages.clear();
            toast(app, "cleared");
        }
        "sidebar" => {
            app.sidebar = !app.sidebar;
            toast(
                app,
                if app.sidebar {
                    "sidebar on"
                } else {
                    "sidebar off"
                },
            );
        }
        "agent" | "tab" => {
            app.agent = app.agent.toggle();
            push_msg(
                app,
                MsgKind::System,
                format!("agent mode: {}", app.agent.label()),
            );
        }
        "status" | "doctor" => {
            let d = app.workspace.doctor_report();
            push_msg(app, MsgKind::System, d);
        }
        "connect" => {
            push_msg(
                app,
                MsgKind::System,
                "Configure provider from shell:\n  bugbee connect --provider ollama --model qwen2.5-coder --base-url http://127.0.0.1:11434/v1\n  bugbee connect --provider xai --model grok-4.5 --api-key-env XAI_API_KEY\nThen: bugbee super -v \"your goal\"",
            );
        }
        "findings" | "ls" => {
            refresh_findings(app);
            app.sidebar = true;
            let mut body = format!("{} findings\n", app.findings.len());
            for f in app.findings.iter().take(20) {
                body.push_str(&format!(
                    "[{}] {} {}:{} — {}\n",
                    f.severity.as_str(),
                    f.status.as_str(),
                    f.location.path,
                    f.location.start_line,
                    f.title
                ));
            }
            push_msg(app, MsgKind::Finding, body);
        }
        "hunt" => {
            app.busy = true;
            push_msg(app, MsgKind::Tool, "running hunt…");
            match app.workspace.run_hunt() {
                Ok(sum) => {
                    push_msg(
                        app,
                        MsgKind::Tool,
                        format!(
                            "hunt complete: {} files · {} findings · {} rules",
                            sum.files_scanned,
                            sum.findings.len(),
                            sum.rules_loaded
                        ),
                    );
                    for f in sum.findings.iter().take(12) {
                        push_msg(
                            app,
                            MsgKind::Finding,
                            format!(
                                "[{}] brs={} {}:{}  {}",
                                f.severity.as_str(),
                                f.brs,
                                f.location.path,
                                f.location.start_line,
                                f.title
                            ),
                        );
                    }
                    refresh_findings(app);
                    app.sidebar = true;
                }
                Err(e) => push_msg(app, MsgKind::System, format!("hunt error: {e}")),
            }
            app.busy = false;
            app.status = "ready".into();
        }
        "deep-hunt" | "dh" => {
            if app.stream_rx.is_some() {
                toast(app, "already running");
                return false;
            }
            app.busy = true;
            app.status = "deep analysis…".into();
            push_msg(app, MsgKind::Tool, "deep hunt (godmode + streaming)…");
            let root = app.workspace.root.clone();
            let config = app.workspace.config.clone();
            let store = match bugbee_core::Store::open(bugbee_core::config::store_path(&root)) {
                Ok(s) => s,
                Err(e) => {
                    push_msg(app, MsgKind::System, format!("store: {e}"));
                    app.busy = false;
                    app.status = "error".into();
                    return false;
                }
            };
            let opts = GodmodeOptions {
                use_llm: false,
                aggressive: true,
                adversarial: true,
                enrich_max: 64,
            };
            let (tx, rx) = mpsc::channel(256);
            app.stream_rx = Some(rx);
            tokio::spawn(async move {
                run_godmode_streaming(root, config, store, None, opts, tx).await;
            });
        }
        "swarm" => {
            if app.stream_rx.is_some() {
                toast(app, "already running");
                return false;
            }
            app.busy = true;
            app.status = "swarm analysis…".into();
            push_msg(app, MsgKind::Tool, "swarm pipeline (streaming)…");
            let root = app.workspace.root.clone();
            let config = app.workspace.config.clone();
            let store = match bugbee_core::Store::open(bugbee_core::config::store_path(&root)) {
                Ok(s) => s,
                Err(e) => {
                    push_msg(app, MsgKind::System, format!("store: {e}"));
                    app.busy = false;
                    app.status = "error".into();
                    return false;
                }
            };
            let report = root.join("bugbee-report.md");
            let opts = SwarmOptions {
                resume: true,
                carlini_max: 8,
                write_report: Some(report),
            };
            let (tx, rx) = mpsc::channel(256);
            app.stream_rx = Some(rx);
            tokio::spawn(async move {
                run_swarm_streaming(root, config, store, opts, tx).await;
            });
        }
        "godmode" | "gm" => {
            if app.stream_rx.is_some() {
                toast(app, "already running");
                return false;
            }
            app.busy = true;
            app.status = "deep analysis…".into();
            push_msg(app, MsgKind::Tool, "godmode (streaming)…");
            let root = app.workspace.root.clone();
            let config = app.workspace.config.clone();
            let store = match bugbee_core::Store::open(bugbee_core::config::store_path(&root)) {
                Ok(s) => s,
                Err(e) => {
                    push_msg(app, MsgKind::System, format!("store: {e}"));
                    app.busy = false;
                    app.status = "error".into();
                    return false;
                }
            };
            let opts = GodmodeOptions {
                use_llm: false,
                aggressive: true,
                adversarial: false,
                enrich_max: 64,
            };
            let (tx, rx) = mpsc::channel(256);
            app.stream_rx = Some(rx);
            tokio::spawn(async move {
                run_godmode_streaming(root, config, store, None, opts, tx).await;
            });
        }
        "report" => {
            let findings = app.workspace.store.list(None).unwrap_or_default();
            let sarif_path = PathBuf::from("findings.sarif.json");
            let bounty_path = PathBuf::from("bugbee-report.md");
            let sarif = bugbee_agent::findings_to_sarif(&findings);
            let _ = std::fs::write(
                &sarif_path,
                serde_json::to_string_pretty(&sarif).unwrap_or_default(),
            );
            let bounty = bugbee_agent::render_bounty_reports(&findings);
            let _ = std::fs::write(&bounty_path, bounty);
            push_msg(
                app,
                MsgKind::System,
                format!(
                    "wrote {} and {} ({} findings)",
                    sarif_path.display(),
                    bounty_path.display(),
                    findings.len()
                ),
            );
        }
        other => {
            push_msg(
                app,
                MsgKind::System,
                format!("unknown command: /{other} -- try /help"),
            );
        }
    }
    false
}

// ── Drawing ──────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    // Background fill via full-block style
    f.render_widget(Block::default().style(Style::default().bg(theme::BG)), area);

    match app.screen {
        Screen::Home => draw_home(f, app, area),
        Screen::Session => draw_session(f, app, area),
    }

    if app.focus == Focus::Help {
        draw_help_modal(f, area);
    }
    if let Some((msg, _)) = &app.toast {
        draw_toast(f, area, msg);
    }
}

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(1),
        ])
        .split(area);

    // Logo centered
    let logo = Paragraph::new(bugbee_logo_lines())
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme::BG));
    f.render_widget(logo, chunks[1]);

    // Tips
    let tips = Paragraph::new(vec![Line::from(Span::styled(
        "  Tab agent  ·  / commands  ·  Ctrl+B sidebar  ·  F1 help  ·  q quit",
        theme::muted(),
    ))])
    .alignment(Alignment::Center);
    f.render_widget(tips, chunks[2]);

    // Centered prompt (OpenCode home_prompt)
    let prompt_w = (area.width * 7 / 10)
        .max(40)
        .min(area.width.saturating_sub(4));
    let prompt_x = area.x + (area.width.saturating_sub(prompt_w)) / 2;
    let prompt_area = Rect {
        x: prompt_x,
        y: chunks[3].y,
        width: prompt_w,
        height: chunks[3].height,
    };
    draw_prompt_box(f, app, prompt_area, true);

    draw_footer(f, app, chunks[5]);
}

fn draw_session(f: &mut Frame, app: &mut App, area: Rect) {
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header strip
            Constraint::Min(5),    // main
            Constraint::Length(4), // prompt
            Constraint::Length(1), // footer
        ])
        .split(area);

    // Header
    let hdr = Line::from(vec![
        small_brand().spans[0].clone(),
        Span::raw("  "),
        Span::styled(
            app.workspace.config.project.name.clone(),
            theme::text().add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", theme::muted()),
        Span::styled(app.agent.label(), theme::primary_bold()),
        Span::styled(format!(" ({})", app.agent.opencode()), theme::muted()),
        Span::styled("  ·  ", theme::muted()),
        Span::styled(format!("{} findings", app.findings.len()), theme::info()),
        if app.busy {
            Span::styled(
                format!("  {} {}", SPINNER[app.spinner_i], app.status),
                theme::warning(),
            )
        } else {
            match app.status.as_str() {
                "error" => Span::styled("  error", theme::error()),
                "disconnected" => Span::styled("  disconnected", theme::warning()),
                "ready" => Span::styled("  ready", theme::muted()),
                _ => Span::styled(format!("  {}", app.status), theme::muted()),
            }
        },
    ]);
    f.render_widget(
        Paragraph::new(hdr).style(Style::default().bg(theme::BG_PANEL)),
        body[0],
    );

    // Main: optional sidebar + transcript
    if app.sidebar {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(body[1]);
        draw_transcript(f, app, cols[0]);
        draw_sidebar(f, app, cols[1]);
    } else {
        draw_transcript(f, app, body[1]);
    }

    draw_prompt_box(f, app, body[2], false);
    draw_footer(f, app, body[3]);
}

fn draw_transcript(f: &mut Frame, app: &App, area: Rect) {
    let border = if app.focus == Focus::Transcript {
        theme::border_active()
    } else {
        theme::border()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(Span::styled(" session ", theme::muted()))
        .style(Style::default().bg(theme::BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.messages.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No messages yet.  Try: /hunt  /swarm  /help  or type a goal",
                    theme::muted(),
                )),
            ]),
            inner,
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for m in &app.messages {
        let (tag, style) = match m.kind {
            MsgKind::User => ("you", theme::user_msg()),
            MsgKind::Assistant => ("bugbee", theme::assistant_msg()),
            MsgKind::System => ("sys", theme::system_msg()),
            MsgKind::Tool => ("tool", theme::tool_msg()),
            MsgKind::Finding => ("find", theme::warning()),
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", tag), style.add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", m.at), theme::muted()),
        ]));
        for row in m.text.lines() {
            lines.push(Line::from(Span::styled(format!("  {row}"), style)));
        }
        lines.push(Line::from(""));
    }

    let h = inner.height as usize;
    let total = lines.len();
    let max_scroll = total.saturating_sub(h);
    let scroll = app.scroll.min(max_scroll);
    // show bottom by default: scroll from end
    let start = total.saturating_sub(h + scroll);
    let end = (start + h).min(total);
    let view = lines[start..end].to_vec();

    f.render_widget(Paragraph::new(view).wrap(Wrap { trim: false }), inner);

    if max_scroll > 0 {
        let mut state = ScrollbarState::new(max_scroll).position(max_scroll - scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(theme::muted())
                .thumb_style(theme::primary()),
            inner,
            &mut state,
        );
    }
}

fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    match app.sidebar_mode {
        SidebarMode::Detail => {
            app.sec_panel.draw(f, area);
        }
        SidebarMode::Findings => {
            let border = if app.focus == Focus::Sidebar {
                theme::border_active()
            } else {
                theme::border()
            };
            let title = format!(" findings ({}) ", app.findings.len());
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border)
                .title(Span::styled(title, theme::muted()))
                .style(Style::default().bg(theme::BG_PANEL));

            let items: Vec<ListItem> = app
                .findings
                .iter()
                .map(|f| {
                    let sev = SecurityPanel::severity_color(&f.severity);
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{:<8}", f.severity.as_str()), sev),
                        Span::styled(
                            format!(
                                " {} {}:{}",
                                f.title, f.location.path, f.location.start_line
                            ),
                            theme::text(),
                        ),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(block)
                .highlight_style(theme::selected())
                .highlight_symbol("▸ ");
            f.render_stateful_widget(list, area, &mut app.find_state);
        }
    }
}

fn draw_prompt_box(f: &mut Frame, app: &App, area: Rect, home: bool) {
    let active = app.focus == Focus::Prompt;
    let border = if active {
        theme::border_active()
    } else {
        theme::border()
    };
    let title = if home {
        " prompt "
    } else {
        " message · / for commands "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(Span::styled(title, theme::muted()))
        .style(Style::default().bg(theme::BG_ELEMENT));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let display = if app.prompt.is_empty() && !active {
        app.placeholders[app.placeholder_i].to_string()
    } else if app.prompt.is_empty() {
        String::new()
    } else {
        app.prompt.clone()
    };

    let style = if app.prompt.is_empty() {
        theme::muted()
    } else {
        theme::text()
    };

    // caret
    let mut shown = display;
    if active {
        let cur = app.cursor.min(shown.len());
        shown.insert(cur, '▌');
    }

    let prefix = Span::styled(
        format!(" {} ", if home { "›" } else { "❯" }),
        theme::primary_bold(),
    );
    f.render_widget(
        Paragraph::new(Line::from(vec![prefix, Span::styled(shown, style)])),
        inner,
    );

    // slash autocomplete strip
    if app.prompt.starts_with('/') && !app.prompt.contains(' ') {
        let q = app.prompt.to_ascii_lowercase();
        let matches: Vec<_> = SLASH
            .iter()
            .filter(|(c, _)| c.starts_with(&q))
            .take(5)
            .collect();
        if !matches.is_empty() {
            let y = area.y.saturating_sub(matches.len() as u16 + 1);
            if y > 0 {
                let h = matches.len() as u16 + 1;
                let pop = Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: h.min(area.y),
                };
                let items: Vec<Line> = matches
                    .iter()
                    .map(|(c, d)| {
                        Line::from(vec![
                            Span::styled(format!(" {c:<12} "), theme::primary()),
                            Span::styled(*d, theme::muted()),
                        ])
                    })
                    .collect();
                f.render_widget(Clear, pop);
                f.render_widget(
                    Paragraph::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(theme::border_active())
                            .style(Style::default().bg(theme::BG_PANEL)),
                    ),
                    pop,
                );
            }
        }
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let root = app.workspace.root.display().to_string();
    let provider = app
        .workspace
        .config
        .provider
        .name
        .clone()
        .unwrap_or_else(|| "no provider".into());
    let model = app
        .workspace
        .config
        .provider
        .model
        .clone()
        .unwrap_or_else(|| "—".into());

    let left = Span::styled(format!(" {} ", root), theme::muted());
    let right = Line::from(vec![
        Span::styled(format!(" {} ", app.agent.label()), theme::primary()),
        Span::styled("| ", theme::success()),
        Span::styled(format!("{provider}/{model} "), theme::text()),
        Span::styled(format!("{} findings ", app.findings.len()), theme::info()),
        Span::styled("/status ", theme::muted()),
        Span::styled("F1 help ", theme::muted()),
    ]);

    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    f.render_widget(
        Paragraph::new(left).style(Style::default().bg(theme::BG_PANEL)),
        row[0],
    );
    f.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .style(Style::default().bg(theme::BG_PANEL)),
        row[1],
    );
}

fn draw_help_modal(f: &mut Frame, area: Rect) {
    let w = area.width.min(72);
    let h = area.height.min(22);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    f.render_widget(Clear, rect);
    let text = vec![
        Line::from(Span::styled(
            " Bugbee keymap ",
            theme::primary_bold(),
        )),
        Line::from(""),
        Line::from("  Tab       Toggle agent between hunt and review modes"),
        Line::from("  Enter     Send message / run slash command"),
        Line::from("  /...      Slash commands (/hunt /swarm /findings ...)"),
        Line::from("  Ctrl+B    Toggle findings sidebar"),
        Line::from("  Ctrl+N    New session"),
        Line::from("  PgUp/Dn   Scroll transcript"),
        Line::from("  s         Focus sidebar  ·  j/k move  ·  c/f/x review"),
        Line::from("  Esc       Clear prompt / back to home"),
        Line::from("  F1 / ?    This help"),
        Line::from("  Ctrl+C/q  Quit (q on home screen)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Commands: /hunt /swarm /godmode /report /doctor /connect /quit",
            theme::muted(),
        )),
        Line::from(""),
        Line::from(Span::styled("  Esc to close", theme::muted())),
    ];
    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_active())
                .title(" help ")
                .style(Style::default().bg(theme::BG_PANEL).fg(theme::TEXT)),
        ),
        rect,
    );
}

fn draw_toast(f: &mut Frame, area: Rect, msg: &str) {
    let w = (msg.len() as u16 + 4).min(area.width.saturating_sub(2));
    let rect = Rect {
        x: area.x + area.width.saturating_sub(w + 1),
        y: area.y + 1,
        width: w,
        height: 3,
    };
    f.render_widget(Clear, rect);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(format!(" {msg} "), theme::text()))).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::primary())
                .style(Style::default().bg(theme::BG_ELEMENT)),
        ),
        rect,
    );
}
