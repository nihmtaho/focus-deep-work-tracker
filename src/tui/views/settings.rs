use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::themes::get_colors_for_theme;

/// Number of settings rows (0=vim, 1=theme, 2=work, 3=break, 4=long_break, 5=long_break_after).
pub const SETTINGS_ROW_COUNT: usize = 6;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let tc = get_colors_for_theme(app.config.theme.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(7), // General section (vim mode + theme)
            Constraint::Length(8), // Pomodoro section
            Constraint::Length(2), // help bar
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(" Settings ")
        .style(
            Style::default()
                .fg(tc.accent)
                .bg(tc.background)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(title, chunks[0]);

    // ── General section ───────────────────────────────────────────────────────
    let vim_selected = app.settings_selected == 0;
    let vim_label = if app.config.vim_mode {
        "Vim Mode: [ ON  ]"
    } else {
        "Vim Mode: [ OFF ]"
    };
    let vim_style = row_style(vim_selected, app.config.vim_mode, app.no_color, &tc);

    let theme_selected = app.settings_selected == 1;
    let theme_name = match app.config.theme.as_deref() {
        None => "Auto    ",
        Some("onedark") => "OneDark ",
        Some("material") => "Material",
        Some("light") => "Light   ",
        Some("dark") => "Dark    ",
        Some(other) => other,
    };
    let theme_label = format!("Theme:    [ {theme_name} ]");
    let theme_style = row_style(theme_selected, app.config.theme.is_some(), app.no_color, &tc);

    let general_content = Paragraph::new(vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(vim_label, vim_style)),
        Line::from(Span::raw("")),
        Line::from(Span::styled(theme_label.as_str(), theme_style)),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tc.panel_border))
            .style(Style::default().bg(tc.background))
            .title(" General "),
    );
    frame.render_widget(general_content, chunks[1]);

    // ── Pomodoro Defaults section ─────────────────────────────────────────────
    let p = &app.pomo_config;
    let rows: &[(usize, &str, u32, &str)] = &[
        (2, "Work Duration    ", p.work_duration_mins, "min  (1–120)"),
        (
            3,
            "Break Duration   ",
            p.break_duration_mins,
            "min  (1–60) ",
        ),
        (
            4,
            "Long Break       ",
            p.long_break_duration_mins,
            "min  (1–60) ",
        ),
        (5, "Long Break After ", p.long_break_after, "sessions     "),
    ];

    let pomo_lines: Vec<Line> = std::iter::once(Line::from(Span::raw("")))
        .chain(rows.iter().map(|(idx, label, val, unit)| {
            let selected = app.settings_selected == *idx;
            let style = if selected {
                Style::default()
                    .fg(tc.background)
                    .bg(tc.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(tc.foreground)
            };
            let prefix = if selected { " ▶ " } else { "   " };
            Line::from(Span::styled(
                format!("{prefix}{label}: {:>3} {unit}", val),
                style,
            ))
        }))
        .collect();

    let pomo_block = Paragraph::new(pomo_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tc.panel_border))
            .style(Style::default().bg(tc.background))
            .title(" Pomodoro Defaults "),
    );
    frame.render_widget(pomo_block, chunks[2]);

    // ── Help bar ──────────────────────────────────────────────────────────────
    let help = Paragraph::new(
        " [↑/↓] Select   [+/-] Adjust / Cycle theme   [V] Toggle Vim Mode ",
    )
    .style(Style::default().fg(tc.panel_border).bg(tc.background))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);

    if let Some(msg) = &app.message {
        crate::tui::views::dashboard::render_message_overlay_pub(frame, app, msg);
    }
}

fn row_style(selected: bool, active: bool, no_color: bool, tc: &crate::theme::ThemeColors) -> Style {
    if selected {
        Style::default()
            .fg(tc.background)
            .bg(tc.accent)
            .add_modifier(Modifier::BOLD)
    } else if active && !no_color {
        Style::default()
            .fg(tc.success)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(tc.panel_border)
    }
}
