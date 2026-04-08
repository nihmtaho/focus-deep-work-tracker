use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::display::format::format_duration;
use crate::tui::app::{App, MessageKind, MessageOverlay};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Three-zone layout for 007-ui-refresh:
    // Top: Controls zone (hotkey legend)
    // Middle: Timer zone (40%) + TODO zone (60%)
    // Bottom: Today's summary

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),      // controls/help
            Constraint::Percentage(40), // timer + todo
            Constraint::Min(5),         // today summary
        ])
        .split(area);

    // Render controls zone at top
    crate::tui::ui::render_controls_zone(frame, chunks[0], app);

    // Split middle section into timer (40%) and TODO list (60%)
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Render timer zone (left/center)
    crate::tui::ui::render_timer_zone(frame, middle_chunks[0], app);

    // Render TODO zone (right)
    crate::tui::ui::render_todo_zone(frame, middle_chunks[1], app);

    // Render today's summary (bottom)
    let summary_lines: Vec<Line> = if app.today_sessions.is_empty() {
        vec![Line::from(Span::styled(
            "  No sessions today.",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        let mut lines = Vec::new();
        let mut total_secs: i64 = 0;
        for session in &app.today_sessions {
            let secs = session.duration().map(|d| d.num_seconds()).unwrap_or(0);
            total_secs += secs;
            let dur = format_duration(chrono::Duration::seconds(secs));
            let tag_suffix = session
                .tag
                .as_deref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            let label = format!("  {}{}", session.task, tag_suffix);
            lines.push(Line::from(vec![
                Span::styled(format!("{:<30}", label), Style::default().fg(Color::Cyan)),
                Span::raw(dur),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<30}", "TOTAL"),
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

    // Message overlay
    if let Some(msg) = &app.message {
        render_message_overlay(frame, app, msg);
    }
}

pub fn render_message_overlay_pub(frame: &mut Frame, app: &App, msg: &MessageOverlay) {
    render_message_overlay(frame, app, msg);
}

fn render_message_overlay(frame: &mut Frame, app: &App, msg: &MessageOverlay) {
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
