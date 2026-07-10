//! Lightweight findings TUI for Bugbee.

use std::io::{self, stdout};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

use bugbee_core::{Finding, FindingStore};

pub fn run(root: &Path) -> Result<()> {
    let store_path = root.join(".bugbee").join("findings.db");
    if !store_path.exists() {
        anyhow::bail!("No findings DB. Run `bugbee hunt` first.");
    }
    let store = FindingStore::open(&store_path)?;
    let findings = store.list_by_brs(500)?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut state = ListState::default();
    if !findings.is_empty() {
        state.select(Some(0));
    }

    let res = run_loop(&mut terminal, &findings, &mut state);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    findings: &[Finding],
    state: &mut ListState,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Percentage(45),
                    Constraint::Percentage(55),
                ])
                .split(f.area());

            let indigo = Color::Rgb(108, 124, 255);
            let charcoal = Color::Rgb(17, 19, 24);
            let muted = Color::Rgb(189, 193, 198);
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " ◈ ",
                    Style::default().fg(indigo).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "BUGBEE",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  /  SECURITY CONSOLE", Style::default().fg(indigo)),
                Span::styled(
                    "    findings · j/k move · q quit",
                    Style::default().fg(muted),
                ),
            ]))
            .style(Style::default().bg(charcoal))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(indigo))
                    .title(Span::styled(" BUG HUNT ", Style::default().fg(indigo))),
            );
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = findings
                .iter()
                .map(|find| {
                    let loc = find
                        .locations
                        .first()
                        .map(|l| format!("{}:{}", l.file, l.start_line))
                        .unwrap_or_default();
                    ListItem::new(format!(
                        "[{:5.1}] {:?}  {}  — {}",
                        find.brs, find.severity, loc, find.title
                    ))
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("By BRS"))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");
            f.render_stateful_widget(list, chunks[1], state);

            let detail = if let Some(i) = state.selected() {
                let f = &findings[i];
                let mut text = format!(
                    "ID: {}\nTitle: {}\nSeverity: {:?}\nBRS: {:.1}  ECS: {:.2}  Confidence: {:.2}\nStatus: {:?}\nCWE: {:?}\nOWASP: {:?}\n\n{}\n\nEvidence:\n",
                    f.id, f.title, f.severity, f.brs, f.ecs, f.confidence, f.status, f.cwe, f.owasp, f.description
                );
                for t in &f.evidence.traces {
                    text.push_str(t);
                    text.push('\n');
                }
                if let Some(df) = &f.evidence.dataflow {
                    text.push_str(&format!("\nDataflow: {df}\n"));
                }
                text
            } else {
                "No findings".into()
            };
            let para = Paragraph::new(detail)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title("Evidence / Review"));
            f.render_widget(para, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = state.selected().unwrap_or(0);
                        if !findings.is_empty() {
                            state.select(Some((i + 1).min(findings.len() - 1)));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = state.selected().unwrap_or(0);
                        if i > 0 {
                            state.select(Some(i - 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
