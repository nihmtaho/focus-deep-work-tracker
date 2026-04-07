use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::pomodoro::timer::{format_secs, PomodoroTimer};

/// Render the full-screen Pomodoro timer view.
pub fn render(frame: &mut Frame, timer: &PomodoroTimer, no_color: bool, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Phase label
            Constraint::Length(3), // MM:SS countdown
            Constraint::Length(1), // pomodoro count + total elapsed
            Constraint::Length(3), // progress bar
            Constraint::Length(2), // keyboard hints
            Constraint::Min(0),
        ])
        .split(area);

    // Phase label (with PAUSED badge if paused)
    let phase_text = if timer.paused {
        format!("{} ⏸ PAUSED", timer.phase_label())
    } else {
        timer.phase_label().to_string()
    };
    let phase_color = if !no_color {
        match timer.phase {
            crate::models::pomodoro::PomodoroPhase::Work => Color::Red,
            crate::models::pomodoro::PomodoroPhase::Break => Color::Green,
            crate::models::pomodoro::PomodoroPhase::LongBreak => Color::Blue,
        }
    } else {
        Color::Reset
    };
    let phase_widget = Paragraph::new(phase_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(phase_widget, chunks[0]);

    // MM:SS countdown (large)
    let countdown = Paragraph::new(timer.format_remaining())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(if !no_color { Color::Cyan } else { Color::Reset })
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(countdown, chunks[1]);

    // Pomodoro count + elapsed
    let elapsed_total = format_secs(timer.total_elapsed_secs());
    let count_line = format!(
        "{}/{} pomodoros  •  Total elapsed: {}",
        timer.completed_work, timer.config.long_break_after, elapsed_total
    );
    let count_widget = Paragraph::new(count_line).alignment(Alignment::Center);
    frame.render_widget(count_widget, chunks[2]);

    // Progress bar
    let progress = timer.phase_progress();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(if !no_color { Color::Cyan } else { Color::Reset })
                .bg(if !no_color {
                    Color::DarkGray
                } else {
                    Color::Reset
                }),
        )
        .ratio(progress);
    frame.render_widget(gauge, chunks[3]);

    // Keyboard hints
    let hints = Line::from(vec![
        Span::styled("[P]", Style::default().fg(Color::Yellow)),
        Span::raw(" pause  "),
        Span::styled("[S]", Style::default().fg(Color::Yellow)),
        Span::raw(" skip break  "),
        Span::styled("[+]", Style::default().fg(Color::Yellow)),
        Span::raw(" extend 5 min  "),
        Span::styled("[Q]", Style::default().fg(Color::Yellow)),
        Span::raw(" stop"),
    ]);
    let hints_widget = Paragraph::new(hints).alignment(Alignment::Center);
    frame.render_widget(hints_widget, chunks[4]);
}
