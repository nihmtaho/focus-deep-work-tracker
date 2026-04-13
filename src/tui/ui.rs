use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

use crate::pomodoro::stats::PomodoroPanelState;
use crate::tui::app::{App, MessageKind, Overlay, Tab};
use crate::tui::timer_display::TimerDisplay;
use crate::tui::views;

const HELP_TEXT: &str = "\
Global
  d/l/s     Dashboard/Log/Settings
  Tab       Next tab
  ?         Show this help
  q         Quit
  Esc       Clear message

Dashboard
  n         New session (choose mode)
  e/Enter   Stop active session
  1/2/3     Focus panel (Timer · TODOs · Report)
  p         Pause/Resume Pomodoro
  + / S     Extend / Skip break
  q         Stop Pomodoro (when active)

TODOs (Dashboard)
  a         Add TODO
  c         Complete selected
  →         Start session with selected TODO
  ↑/↓       Navigate

Log
  ↑/↓       Select row
  ←/→       Change page
  d         Delete selected session
  r         Rename selected session
  j/k/g/G   Vim navigation (when enabled)

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
        Tab::Settings => views::settings::render(frame, app, chunks[1]),
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
        (Tab::Dashboard, "[d]Dashboard"),
        (Tab::Log, "[l]Log"),
        (Tab::Settings, "[s]Settings"),
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

/// Render the Pomodoro panel when idle, showing historical stats and start button.
/// Displays total cycles, cumulative duration, focus streak, and last completion time.
pub fn render_pomodoro_panel(frame: &mut Frame, area: Rect, _app: &App, focused: bool) {
    // For now, create an idle panel state as placeholder
    // TODO: Load actual stats from database in Phase 6 (T060-T061)
    let panel_state = PomodoroPanelState::idle();

    let content = if panel_state.has_activity() {
        // Show stats when there's activity today
        vec![
            Line::from(Span::styled(
                format!("{} pomodoros today", panel_state.total_cycles_today),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                format!("Duration: {}", panel_state.format_duration()),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                format!("Streak: {} days", panel_state.focus_streak_days),
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "[N] Start Pomodoro",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        // Show start prompt when idle
        vec![
            Line::from(Span::styled(
                "No Pomodoro sessions yet.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "Press [N] on Dashboard to start a Pomodoro session.",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let widget = Paragraph::new(content)
        .block(
            Block::default()
                .title(" [1] Pomodoro ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

/// Render the Report panel showing session analytics and productivity metrics.
/// Displays: session counts, total duration, completion rate, focus streak, and productivity score.
pub fn render_report_panel(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let metrics = &app.report_metrics;

    let content = vec![
        Line::from(Span::styled(
            "Today's Report",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(
                format!("Sessions: {}", metrics.count_today),
                Style::default().fg(Color::White),
            ),
            Span::raw(format!("  Duration: {}", metrics.format_duration_today())),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(
                format!("Completion: {}%", metrics.completion_rate),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!("  Streak: {} days", metrics.focus_streak_days)),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("Productivity: {}/100", metrics.compute_productivity_score()),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!(
                "Week: {} sessions • {}",
                metrics.count_week,
                metrics.format_duration_week()
            ),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            format!("All-time: {} sessions", metrics.count_all_time),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let widget = Paragraph::new(content)
        .block(
            Block::default()
                .title(" [3] Report ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

/// Render the timer zone displaying active session countdown in HH:MM:SS format.
/// Uses TimerDisplay component for consistent flip-clock formatting.
/// Takes a significant portion of the layout (40% width) for visual prominence.
pub fn render_timer_zone(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    let timer_text = if let Some(session) = &app.active_session {
        let elapsed = session.elapsed();
        let duration = Duration::from_secs(elapsed.num_seconds() as u64);
        let timer = TimerDisplay::new(duration);
        timer.render()
    } else {
        "--:--:--".to_string()
    };

    let timer_widget = Paragraph::new(timer_text)
        .block(
            Block::default()
                .title(" [1] Timer ")
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                }),
        )
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    frame.render_widget(timer_widget, area);
}

/// Render the TODO list zone displaying all todos with visual distinction
/// for active vs completed items, using theme colors for each state.
pub fn render_todo_zone(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    // If in TODO input mode, render input field instead
    let border_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    if app.todo_input_mode {
        // Vim mode: render VimEditor with mode indicator
        if app.config.vim_mode {
            if let Some(ref editor) = app.todo_vim_editor {
                let mode_str = match editor.mode {
                    vimltui::VimMode::Insert => "-- INSERT --",
                    vimltui::VimMode::Replace => "-- REPLACE --",
                    vimltui::VimMode::Visual(_) => "-- VISUAL --",
                    _ => "-- NORMAL --",
                };
                let content = editor.content();
                let line = content.lines().next().unwrap_or("");
                let display_line = if matches!(editor.mode, vimltui::VimMode::Insert) {
                    format!("{}█", line)
                } else {
                    format!("{line}  ")
                };
                let input_widget = Paragraph::new(vec![
                    Line::from(Span::styled(
                        display_line.as_str(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        mode_str,
                        Style::default().fg(Color::Yellow),
                    )),
                    Line::from(Span::styled(
                        "[i] insert  [Esc] normal  [Enter] save  [:w] save  [:q] cancel",
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
                .block(
                    Block::default()
                        .title(" Add TODO (Vim) ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .wrap(Wrap { trim: true });
                frame.render_widget(input_widget, area);
                return;
            }
        }
        // Non-vim or no editor: simple text input
        let input_display = format!("{}█", app.todo_input_buffer);
        let input_widget = Paragraph::new(vec![
            Line::from(Span::styled(
                &input_display,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "[Enter] save  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .title(" Add TODO ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

        frame.render_widget(input_widget, area);
        return;
    }

    if app.todos.is_empty() {
        let empty_text = Paragraph::new("No TODOs. Press [a] to add one.")
            .block(
                Block::default()
                    .title(" [2] TODOs ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_text, area);
        return;
    }

    // Get current theme colors for TODO rendering
    let theme_colors = crate::tui::themes::get_colors_for_theme(app.config.theme.as_deref());

    let todos_display: Vec<Line> = app
        .todos
        .iter()
        .enumerate()
        .map(|(idx, todo)| {
            let is_selected = app.selected_todo_idx == Some(idx);
            let status_icon = if todo.is_completed() { "✓" } else { "•" };
            let text = format!("  {} {}", status_icon, todo.title);

            // Apply state-based color from theme
            let todo_color = todo.get_color(&theme_colors);
            let base_style = if app.no_color {
                Style::default()
            } else {
                Style::default().fg(todo_color)
            };

            // Add dimming for completed items
            let text_style = if todo.is_completed() {
                base_style.add_modifier(Modifier::DIM)
            } else {
                base_style
            };

            if is_selected {
                Line::from(vec![Span::styled(
                    text,
                    text_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD),
                )])
            } else {
                Line::from(Span::styled(text, text_style))
            }
        })
        .collect();

    let todo_widget = Paragraph::new(todos_display)
        .block(
            Block::default()
                .title(" [2] TODOs ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(todo_widget, area);
}

/// Render the controls/help zone displaying available hotkeys.
pub fn render_controls_zone(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.todo_input_mode {
        if app.config.vim_mode {
            " [i] insert  [Esc] normal  [Enter] save  [:q] cancel "
        } else {
            " [Enter] confirm  [Esc] cancel "
        }
    } else if app.pomodoro_timer.is_some() {
        " [p] pause/resume  [s] skip break  [+] extend  [q] stop Pomodoro "
    } else {
        " [a] add  [c] complete  [→] start session  [↑↓] navigate  [n] new session "
    };

    let controls_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(controls_widget, area);
}
