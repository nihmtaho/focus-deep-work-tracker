use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::display::format::{format_duration, format_elapsed};
use crate::tui::app::{App, MessageKind};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(5), // active session
            Constraint::Min(5),    // today summary
            Constraint::Length(3), // help bar
        ])
        .split(area);

    // Title
    let title = Paragraph::new(" focus — deep work tracker ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    // Active session block
    let session_text = match &app.active_session {
        Some(s) => {
            let elapsed = format_elapsed(s.start_time);
            let tag_str = s
                .tag
                .as_deref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            vec![
                Line::from(vec![
                    Span::styled("  Task: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}{}", s.task, tag_str)),
                ]),
                Line::from(vec![
                    Span::styled("  Elapsed: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        elapsed,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ]
        }
        None => vec![Line::from(Span::styled(
            "  No active session",
            Style::default().fg(Color::DarkGray),
        ))],
    };

    let session_block = Paragraph::new(session_text)
        .block(
            Block::default()
                .title(" Active Session ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(session_block, chunks[1]);

    // Today summary
    let summary_lines: Vec<Line> = if app.today_summary.is_empty() {
        vec![Line::from(Span::styled(
            "  No sessions today.",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        let mut lines = Vec::new();
        let total_secs: i64 = app.today_summary.iter().map(|(_, s)| s).sum();
        for (tag, secs) in &app.today_summary {
            let label = tag.as_deref().unwrap_or("untagged");
            let dur = format_duration(chrono::Duration::seconds(*secs));
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<20}", label), Style::default().fg(Color::Cyan)),
                Span::raw(dur),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<20}", "TOTAL"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format_duration(chrono::Duration::seconds(total_secs)),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        lines
    };

    let summary_block = Paragraph::new(summary_lines)
        .block(
            Block::default()
                .title(" Today's Summary ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(summary_block, chunks[2]);

    // Help bar
    let help = Paragraph::new(" [M] Menu  [Q] Quit ")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[3]);

    // Message overlay
    if let Some(msg) = &app.message {
        render_message_overlay(frame, app, msg);
    }
}

pub fn render_message_overlay_pub(
    frame: &mut Frame,
    app: &App,
    msg: &crate::tui::app::MessageOverlay,
) {
    render_message_overlay(frame, app, msg);
}

fn render_message_overlay(frame: &mut Frame, app: &App, msg: &crate::tui::app::MessageOverlay) {
    use ratatui::layout::Rect;

    let area = frame.area();
    let msg_width = (msg.text.len() as u16 + 4).min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(msg_width)) / 2;
    let y = area.y + area.height.saturating_sub(4);

    let overlay_area = Rect {
        x,
        y,
        width: msg_width,
        height: 3,
    };

    let (fg, title) = if app.no_color {
        (
            Color::White,
            match msg.kind {
                MessageKind::Success => " OK ",
                MessageKind::Warning => " WARN ",
                MessageKind::Error => " ERROR ",
            },
        )
    } else {
        match msg.kind {
            MessageKind::Success => (Color::Green, " OK "),
            MessageKind::Warning => (Color::Yellow, " WARN "),
            MessageKind::Error => (Color::Red, " ERROR "),
        }
    };

    let overlay = Paragraph::new(msg.text.clone())
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(fg)),
        )
        .style(Style::default().fg(fg));
    frame.render_widget(overlay, overlay_area);
}
