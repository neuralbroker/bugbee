use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use bugbee_core::{Finding, FindingStatus, Severity};

use crate::theme;

pub struct SecurityPanel {
    pub finding: Option<Finding>,
    pub chain_text: String,
    pub show_chain: bool,
}

impl Default for SecurityPanel {
    fn default() -> Self {
        Self {
            finding: None,
            chain_text: String::new(),
            show_chain: false,
        }
    }
}

impl SecurityPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_finding(&mut self, finding: Finding) {
        self.finding = Some(finding);
        self.chain_text.clear();
        self.show_chain = false;
    }

    pub fn clear(&mut self) {
        self.finding = None;
        self.chain_text.clear();
        self.show_chain = false;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let Some(ref finding) = self.finding else {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border())
                .title(Span::styled(" finding detail ", theme::muted()))
                .style(Style::default().bg(theme::BG_PANEL));
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "  Select a finding in the sidebar to view details",
                    theme::muted(),
                )))
                .block(block),
                area,
            );
            return;
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(4),
                Constraint::Length(4),
            ])
            .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_active())
            .title(Span::styled(" finding detail ", theme::primary()))
            .style(Style::default().bg(theme::BG_PANEL));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut lines = Vec::new();

        let (sev_label, sev_color) = match finding.severity {
            Severity::Critical => ("CRITICAL", theme::ERROR),
            Severity::High => ("HIGH", theme::WARNING),
            Severity::Medium => ("MEDIUM", theme::PRIMARY),
            Severity::Low => ("LOW", theme::SUCCESS),
            Severity::Info => ("INFO", theme::TEXT_MUTED),
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", sev_label), Style::default().fg(sev_color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(&finding.title, theme::text().add_modifier(Modifier::BOLD)),
        ]));

        let status_str = match finding.status {
            FindingStatus::Draft => "draft",
            FindingStatus::Confirmed => "confirmed",
            FindingStatus::FalsePositive => "false positive",
            FindingStatus::Fixed => "fixed",
        };
        let status_color = match finding.status {
            FindingStatus::Confirmed => theme::SUCCESS,
            FindingStatus::FalsePositive => theme::WARNING,
            FindingStatus::Fixed => theme::INFO,
            _ => theme::TEXT_MUTED,
        };

        lines.push(Line::from(vec![
            Span::styled(" CWE: ", theme::muted()),
            Span::styled(finding.cwe.as_deref().unwrap_or("—"), theme::text()),
            Span::styled("  Status: ", theme::muted()),
            Span::styled(status_str, Style::default().fg(status_color)),
            Span::styled("  BRS: ", theme::muted()),
            Span::styled(format!("{}", finding.brs), theme::text()),
            Span::styled("  ECS: ", theme::muted()),
            Span::styled(format!("{}", finding.ecs), theme::text()),
        ]));

        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled(" Location: ", theme::muted()),
            Span::styled(
                format!("{}:{}", finding.location.path, finding.location.start_line),
                theme::secondary(),
            ),
        ]));

        if let Some(ref snippet) = finding.location.snippet {
            for line in snippet.lines().take(6) {
                lines.push(Line::from(Span::styled(
                    format!("   {}", line),
                    theme::muted(),
                )));
            }
        }

        lines.push(Line::from(""));

        if let Some(ref poc) = finding.poc {
            lines.push(Line::from(vec![
                Span::styled(" PoC Class: ", theme::muted()),
                Span::styled(format!("{:?}", poc.class), theme::warning()),
            ]));
            if let Some(ref curl) = poc.curl_template {
                lines.push(Line::from(Span::styled(
                    format!(" curl: {}", curl),
                    theme::tool_msg(),
                )));
            }
            if !poc.steps.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!(" steps: {}", poc.steps.join(" → ")),
                    theme::text(),
                )));
            }
        }

        // Evidence section
        if !finding.evidence.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Evidence ({}):", finding.evidence.len()),
                theme::info(),
            )));
            for ev in finding.evidence.iter().take(5) {
                let first_line = ev.detail.lines().next().unwrap_or("");
                let preview = if first_line.len() > 80 {
                    format!("{}…", &first_line[..80])
                } else {
                    first_line.to_string()
                };
                lines.push(Line::from(Span::styled(
                    format!("   {}: {}", ev.kind, preview),
                    theme::muted(),
                )));
            }
        }

        // Kill chain
        if self.show_chain && !self.chain_text.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Kill Chain:",
                theme::warning().add_modifier(Modifier::BOLD),
            )));
            for line in self.chain_text.lines() {
                lines.push(Line::from(Span::styled(
                    format!("   {}", line),
                    theme::text(),
                )));
            }
        }

        // Actions footer
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" [c] confirm  ", theme::success()),
            Span::styled(" [f] false-pos  ", theme::warning()),
            Span::styled(" [x] fixed  ", theme::info()),
            Span::styled(" [k] chain  ", theme::primary()),
            Span::styled(" [v] verify  ", theme::secondary()),
        ]));

        f.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }),
            inner,
        );
    }

    pub fn severity_color(severity: &Severity) -> Style {
        match severity {
            Severity::Critical => Style::default().fg(theme::ERROR),
            Severity::High => Style::default().fg(theme::WARNING),
            Severity::Medium => Style::default().fg(theme::PRIMARY),
            Severity::Low => Style::default().fg(theme::SUCCESS),
            Severity::Info => Style::default().fg(theme::TEXT_MUTED),
        }
    }
}
