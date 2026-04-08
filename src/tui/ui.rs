use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, MessageKind, Overlay, Tab};
use crate::tui::views;

const HELP_TEXT: &str = "\
Global
  1-5       Switch tab
  Tab       Next tab
  ?         Show this help
  q         Quit
  Esc       Clear message

Dashboard
  n         New session (choose mode)
  s/Enter   Stop active session

Pomodoro
  P         Pause / Resume
  S         Skip break
  +         Extend phase by 5 min
  Q         Stop timer

Log
  ↑/↓       Select row
  ←/→       Change page
  d         Delete selected session
  r         Rename selected session
  j/k/g/G   Vim navigation (when enabled)

Report
  h/l       Change time window

Settings
  v         Toggle vim mode

Overlays
  Enter     Confirm
  Esc       Cancel
  y/n       Confirm/Cancel delete
";

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Split into tab bar + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_tab_bar(frame, app, chunks[0]);

    match &app.active_tab {
        Tab::Dashboard => views::dashboard::render(frame, app, chunks[1]),
        Tab::Log => views::log::render(frame, app, app.log_page, app.log_selected, chunks[1]),
        Tab::Report => views::report::render(
            frame,
            app,
            &app.report_window,
            app.report_selected_window,
            chunks[1],
        ),
        Tab::Settings => views::settings::render(frame, app, chunks[1]),
        Tab::Pomodoro => {
            if let Some(ref timer) = app.pomodoro_timer {
                views::pomodoro::render(frame, timer, app.no_color, chunks[1]);
            } else {
                let hint = Paragraph::new(
                    "No Pomodoro session active. Press [N] on Dashboard to start one.",
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(hint, chunks[1]);
            }
        }
    }

    // Render overlay on top
    if app.overlay.is_active() {
        render_overlay(frame, app, area);
    }

    // Render message notification if present (and no modal overlay)
    if app.message.is_some() && !app.overlay.is_active() {
        if let Some(msg) = &app.message {
            render_message_overlay(frame, app, msg, area);
        }
    }
}

fn render_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let tabs = [
        (Tab::Dashboard, "[1]Dashboard"),
        (Tab::Log, "[2]Log"),
        (Tab::Report, "[3]Report"),
        (Tab::Settings, "[4]Settings"),
        (Tab::Pomodoro, "[5]Pomodoro"),
    ];

    let spans: Vec<Span> = tabs
        .iter()
        .flat_map(|(tab, label)| {
            let active = &app.active_tab == tab;
            let span = if active {
                Span::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {label} "), Style::default().fg(Color::DarkGray))
            };
            vec![span, Span::raw("")]
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    frame.render_widget(bar, area);
}

fn render_overlay(frame: &mut Frame, app: &App, area: Rect) {
    match &app.overlay {
        Overlay::Prompt { label, value, .. } => {
            render_prompt_overlay(frame, area, label, value);
        }
        Overlay::ConfirmDelete { session_name, .. } => {
            render_confirm_delete_overlay(frame, area, session_name);
        }
        Overlay::Help => {
            render_help_overlay(frame, area);
        }
        Overlay::ModeSelector { cursor } => {
            render_mode_selector_overlay(frame, area, *cursor);
        }
        Overlay::PomodoroConfirmStop => {
            render_pomodoro_confirm_stop_overlay(frame, area);
        }
        Overlay::None => {}
    }

    // Also show message notification inside overlays if present
    if let Some(msg) = &app.message {
        render_message_overlay(frame, app, msg, area);
    }
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let width = (area.width * percent_x / 100).min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height: height.min(area.height),
    }
}

fn render_prompt_overlay(frame: &mut Frame, area: Rect, label: &str, value: &str) {
    let block_area = centered_rect(55, 5, area);
    frame.render_widget(Clear, block_area);

    let display_value = format!("{value}█");
    let content = vec![
        Line::from(Span::styled(label, Style::default().fg(Color::Yellow))),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            &display_value,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[Enter] Confirm  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Input ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, block_area);
}

fn render_confirm_delete_overlay(frame: &mut Frame, area: Rect, session_name: &str) {
    let block_area = centered_rect(60, 5, area);
    frame.render_widget(Clear, block_area);

    let name_truncated: String = session_name.chars().take(40).collect();
    let content = vec![
        Line::from(Span::styled(
            "Are you sure you want to delete:",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("  \"{}\"", name_truncated),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[Y]es  [N]o  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(content).block(
        Block::default()
            .title(" Confirm Delete ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    frame.render_widget(widget, block_area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let height = (HELP_TEXT.lines().count() as u16 + 2).min(area.height.saturating_sub(2));
    let block_area = centered_rect(65, height, area);
    frame.render_widget(Clear, block_area);

    let widget = Paragraph::new(HELP_TEXT)
        .block(
            Block::default()
                .title(" Key Bindings — any key to dismiss ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, block_area);
}

fn render_mode_selector_overlay(frame: &mut Frame, area: Rect, cursor: usize) {
    let block_area = centered_rect(50, 8, area);
    frame.render_widget(Clear, block_area);

    let modes = ["Freeform session", "Pomodoro session"];
    let content: Vec<Line> = modes
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == cursor {
                Line::from(Span::styled(
                    format!(" ▶ {label}"),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!("   {label}"),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .chain(std::iter::once(Line::from(Span::raw(""))))
        .chain(std::iter::once(Line::from(Span::styled(
            "[↑/↓] Select  [Enter] Confirm  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        ))))
        .collect();

    let widget = Paragraph::new(content).block(
        Block::default()
            .title(" New Session ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(widget, block_area);
}

fn render_pomodoro_confirm_stop_overlay(frame: &mut Frame, area: Rect) {
    let block_area = centered_rect(55, 5, area);
    frame.render_widget(Clear, block_area);

    let content = vec![
        Line::from(Span::styled(
            "Stop the current work phase?",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "This will count as an abandoned session.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[Y]es  [N]o  [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(content).block(
        Block::default()
            .title(" Confirm Stop ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    frame.render_widget(widget, block_area);
}

fn render_message_overlay(
    frame: &mut Frame,
    app: &App,
    msg: &crate::tui::app::MessageOverlay,
    area: Rect,
) {
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

/// Render the timer zone displaying active session countdown in HH:MM:SS format.
/// Takes a significant portion of the layout (40% width) for visual prominence.
pub fn render_timer_zone(frame: &mut Frame, area: Rect, app: &App) {
    let timer_text = if let Some(session) = &app.active_session {
        let elapsed = session.elapsed();
        let hours = elapsed.num_seconds() / 3600;
        let minutes = (elapsed.num_seconds() % 3600) / 60;
        let seconds = elapsed.num_seconds() % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        "--:--:--".to_string()
    };

    let timer_widget = Paragraph::new(timer_text)
        .block(
            Block::default()
                .title(" Timer ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    frame.render_widget(timer_widget, area);
}

/// Render the TODO list zone displaying all todos with visual distinction
/// for active vs completed items.
pub fn render_todo_zone(frame: &mut Frame, area: Rect, app: &App) {
    if app.todos.is_empty() {
        let empty_text = Paragraph::new("No TODOs. Press [a] to add one.")
            .block(
                Block::default()
                    .title(" TODOs ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_text, area);
        return;
    }

    let todos_display: Vec<Line> = app
        .todos
        .iter()
        .enumerate()
        .map(|(idx, todo)| {
            let is_selected = app.selected_todo_idx == Some(idx);
            let status_icon = if todo.is_completed() { "✓" } else { "•" };

            let (text_style, text) = if todo.is_completed() {
                (
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                    format!("  {} {}", status_icon, todo.title),
                )
            } else {
                (Style::default().fg(Color::White), format!("  {} {}", status_icon, todo.title))
            };

            if is_selected {
                Line::from(vec![Span::styled(
                    text,
                    text_style
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )])
            } else {
                Line::from(Span::styled(text, text_style))
            }
        })
        .collect();

    let todo_widget = Paragraph::new(todos_display)
        .block(
            Block::default()
                .title(" TODOs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(todo_widget, area);
}

/// Render the controls/help zone displaying available hotkeys.
pub fn render_controls_zone(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.todo_input_mode {
        " [Enter] confirm  [Esc] cancel "
    } else {
        " [a] add  [d] delete  [c] complete  [s] start  [↑↓] navigate "
    };

    let controls_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(controls_widget, area);
}
