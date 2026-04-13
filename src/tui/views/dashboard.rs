use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, MessageKind, MessageOverlay};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // When full Pomodoro panel mode is active, show expanded view instead of 3-column layout
    if app.full_pomodoro_panel {
        render_full_pomodoro_panel(frame, app, area);
        if let Some(msg) = &app.message {
            render_message_overlay(frame, app, msg);
        }
        return;
    }

    // Three-zone layout for 008-dashboard-ui-enhancements:
    // Top: Controls zone (hotkey legend)
    // Middle: Panel zone (40% timer/pomodoro + 30% TODO + 30% Report)
    // Uses flexible layout to accommodate report analytics

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // controls/help
            Constraint::Min(10),   // main panel area (timer + todo + report)
        ])
        .split(area);

    // Render controls zone at top
    crate::tui::ui::render_controls_zone(frame, chunks[0], app);

    // Split main panel into two sections: left (40% timer/pomodoro), right (60% TODO + Report stacked)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Timer/Pomodoro panel
            Constraint::Percentage(60), // TODO + Report stacked vertically
        ])
        .split(chunks[1]);

    // Split the right side vertically: TODO on top (fills), Report on bottom (fixed height)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),    // TODO list (fills available space)
            Constraint::Length(13), // Report panel (fixed height)
        ])
        .split(main_chunks[1]);

    // Compute panel focus flags from app state
    let panel_focused = |idx: usize| app.focused_panel_idx == Some(idx);

    // Render left panel: Pomodoro timer when active, generic timer for freeform, else idle panel
    if let Some(ref timer) = app.pomodoro_timer {
        crate::tui::views::pomodoro::render(frame, timer, app.no_color, main_chunks[0]);
    } else if app.active_session.is_some() {
        // Show Timer zone during active freeform session
        crate::tui::ui::render_timer_zone(frame, main_chunks[0], app, panel_focused(0));
    } else {
        // Show Pomodoro panel when idle (no active session)
        crate::tui::ui::render_pomodoro_panel(frame, main_chunks[0], app, panel_focused(0));
    }

    // Render TODO zone (top-right)
    crate::tui::ui::render_todo_zone(frame, right_chunks[0], app, panel_focused(1));

    // Render Report panel (bottom-right)
    crate::tui::ui::render_report_panel(frame, right_chunks[1], app, panel_focused(2));

    // Message overlay
    if let Some(msg) = &app.message {
        render_message_overlay(frame, app, msg);
    }
}

/// Render the full-screen Pomodoro panel (US11).
///
/// Draws a titled border then delegates the inner area to `pomodoro::render()`,
/// which handles the vertically-centered clock, progress bar, and info panel.
/// A compact stats row (cycles + elapsed) is shown in the bottom border footer.
pub fn render_full_pomodoro_panel(frame: &mut Frame, app: &App, area: Rect) {
    // Build the title with cycle count when a timer is active
    let title = if let Some(ref timer) = app.pomodoro_timer {
        let elapsed = crate::pomodoro::timer::format_secs(timer.total_elapsed_secs());
        format!(
            " Pomodoro  {}/{}  elapsed: {}  [F] collapse ",
            timer.completed_work, timer.config.long_break_after, elapsed
        )
    } else {
        " Pomodoro — Full View  [F] collapse ".to_string()
    };

    let outer = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    if let Some(ref timer) = app.pomodoro_timer {
        // Delegate the full inner area to the shared pomodoro renderer
        crate::tui::views::pomodoro::render(frame, timer, app.no_color, inner);
    } else {
        // No active Pomodoro — show idle message
        let idle_lines = vec![
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "No Pomodoro session active",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "Start a new session from the Dashboard  [N] → Pomodoro mode",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let idle = Paragraph::new(idle_lines).alignment(Alignment::Center);
        frame.render_widget(idle, inner);
    }
}

// T136: Unit tests for full Pomodoro panel data logic
#[cfg(test)]
mod tests {
    #[test]
    fn test_full_pomodoro_panel_toggle_default_off() {
        use crate::config::AppConfig;
        use crate::tui::app::App;
        let app = App::new(false, AppConfig::default());
        assert!(
            !app.full_pomodoro_panel,
            "full_pomodoro_panel must default to false"
        );
    }

    #[test]
    fn test_full_pomodoro_panel_toggle_on() {
        use crate::config::AppConfig;
        use crate::tui::app::App;
        let mut app = App::new(false, AppConfig::default());
        app.full_pomodoro_panel = true;
        assert!(
            app.full_pomodoro_panel,
            "full_pomodoro_panel must be settable to true"
        );
    }

    #[test]
    fn test_full_pomodoro_panel_toggle_roundtrip() {
        use crate::config::AppConfig;
        use crate::tui::app::App;
        let mut app = App::new(false, AppConfig::default());
        app.full_pomodoro_panel = true;
        app.full_pomodoro_panel = false;
        assert!(
            !app.full_pomodoro_panel,
            "full_pomodoro_panel must be toggleable off"
        );
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
