use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, MessageKind, MessageOverlay};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Three-zone layout for 008-dashboard-ui-enhancements:
    // Top: Controls zone (hotkey legend)
    // Middle: Panel zone (40% timer/pomodoro + 30% TODO + 30% Report)
    // Uses flexible layout to accommodate report analytics

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),      // controls/help
            Constraint::Min(10),        // main panel area (timer + todo + report)
        ])
        .split(area);

    // Render controls zone at top
    crate::tui::ui::render_controls_zone(frame, chunks[0], app);

    // Split main panel into three sections: left (40% timer/pomodoro), middle (30% TODO), right (30% Report)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Timer/Pomodoro panel
            Constraint::Percentage(30), // TODO list
            Constraint::Percentage(30), // Report panel
        ])
        .split(chunks[1]);

    // Render left panel: Pomodoro panel when idle, Timer zone when active session
    if app.active_session.is_some() {
        // Show Timer zone during active session (freeform or pomodoro)
        crate::tui::ui::render_timer_zone(frame, main_chunks[0], app);
    } else {
        // Show Pomodoro panel when idle (no active session)
        crate::tui::ui::render_pomodoro_panel(frame, main_chunks[0], app);
    }

    // Render TODO zone (middle)
    crate::tui::ui::render_todo_zone(frame, main_chunks[1], app);

    // Render Report panel (right) - replaces Today's Summary
    crate::tui::ui::render_report_panel(frame, main_chunks[2], app);

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
