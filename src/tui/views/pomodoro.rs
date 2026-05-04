use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::models::pomodoro::PomodoroPhase;
use crate::pomodoro::timer::PomodoroTimer;
use crate::tui::app::App;
use crate::tui::timer_display::TimerDisplay;

/// Render the Pomodoro timer view with a vertically-centered MM:SS clock and
/// a bottom info panel showing session progress, task, and keyboard hints.
///
/// Layout (inside the panel area):
/// ```text
///  [ phase label - centered, 2 rows        ]
///  [ Fill spacer - top                      ]
///  [ clock rows  - 5 or 1 row(s)           ]
///  [ progress bar with % label - 1 row      ]
///  [ Fill spacer - bottom                   ]
///  [ bottom info area - 3 rows              ]  (omitted when very short)
/// ```
/// The bottom info area (3 lines):
/// ```text
///  Line 0: session dots  ● ● ○ ○   session 3 / 4
///  Line 1: current task name  #tag
///  Line 2: next phase preview   [P] pause [S] skip [Q] stop
/// ```
pub fn render(frame: &mut Frame, timer: &PomodoroTimer, app: &App, area: Rect) {
    let no_color = app.no_color;
    let w = area.width;
    let h = area.height;

    // Animated MM:SS clock — pick rendering tier based on available width.
    // Width thresholds are lower than HH:MM:SS because MM:SS is a shorter string.
    let curr_str = app.pomo_clock_curr_str.as_str();
    let prev_str = app.pomo_clock_prev_str.as_str();
    let anim_frames = &app.pomo_clock_anim_frame;

    let clock_rows = match TimerDisplay::best_digit_width_pomo(w) {
        None => {
            let plain =
                TimerDisplay::new(Duration::from_secs(timer.remaining_secs)).render_pomodoro();
            vec![plain]
        }
        Some(dw) if dw >= 5 => {
            TimerDisplay::render_animated_big_sized(curr_str, prev_str, anim_frames, dw)
        }
        Some(_) => {
            // dw == 3 → thin, no animation
            TimerDisplay::render_big_thin_str(curr_str)
        }
    };
    let clock_height: u16 = clock_rows.len() as u16;

    // Show the bottom info panel only when there is enough vertical space
    // Minimum: phase(2) + clock(clock_height) + progress(1) + info(3) + 2 min spacers
    let show_info = h >= clock_height + 2 + 1 + 3 + 2;

    // Build vertical constraints
    let mut constraints = vec![
        Constraint::Length(2),            // 0: phase label
        Constraint::Fill(1),              // 1: top spacer
        Constraint::Length(clock_height), // 2: clock
        Constraint::Length(1),            // 3: progress bar
        Constraint::Fill(1),              // 4: bottom spacer
    ];
    if show_info {
        constraints.push(Constraint::Length(3)); // 5: info panel
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // ── Phase label ──────────────────────────────────────────────────────────
    let phase_text = if timer.paused {
        format!("{} [PAUSED]", timer.phase_label())
    } else {
        timer.phase_label().to_string()
    };
    let phase_color = if no_color {
        Color::Reset
    } else {
        match timer.phase {
            PomodoroPhase::Work => Color::Red,
            PomodoroPhase::Break => Color::Green,
            PomodoroPhase::LongBreak => Color::Blue,
        }
    };
    let phase_widget = Paragraph::new(phase_text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(phase_widget, chunks[0]);

    // ── Clock (MM:SS big or plain) ────────────────────────────────────────────
    let digit_color = if no_color { Color::Reset } else { Color::Yellow };
    let clock_bg = if no_color { Color::Reset } else { Color::Rgb(64, 64, 64) };
    let countdown_lines: Vec<Line> = clock_rows
        .into_iter()
        .map(|row| {
            Line::from(Span::styled(
                row,
                Style::default()
                    .fg(digit_color)
                    .bg(clock_bg)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let countdown = Paragraph::new(countdown_lines)
        .style(Style::default().bg(clock_bg))
        .alignment(Alignment::Center);
    frame.render_widget(countdown, chunks[2]);

    // ── Progress bar with % label ─────────────────────────────────────────────
    let progress = timer.phase_progress();
    let percent_label = format!(" {:.0}% ", progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(if no_color { Color::Reset } else { Color::Cyan })
                .bg(if no_color {
                    Color::Reset
                } else {
                    Color::DarkGray
                }),
        )
        .label(Span::styled(
            percent_label,
            Style::default()
                .fg(if no_color { Color::Reset } else { Color::White })
                .add_modifier(Modifier::BOLD),
        ))
        .ratio(progress);
    frame.render_widget(gauge, chunks[3]);

    // ── Bottom info panel ─────────────────────────────────────────────────────
    if show_info {
        render_bottom_info(frame, timer, no_color, w, chunks[5]);
    }
}

/// Render the 3-line bottom info panel:
///   Line 0: session dots  ● ● ○ ○   session 3 / 4
///   Line 1: task name  #tag
///   Line 2: next phase preview   [P] pause  [S] skip  [Q] stop
fn render_bottom_info(
    frame: &mut Frame,
    timer: &PomodoroTimer,
    no_color: bool,
    panel_width: u16,
    area: Rect,
) {
    // ── Line 0: session dots ─────────────────────────────────────────────────
    let n = (timer.config.long_break_after as usize).max(1);
    let done_in_cycle = (timer.completed_work as usize) % n;
    // When in a work phase, the current session is in progress (done_in_cycle + 1)
    let current_session = if timer.phase.is_work() {
        done_in_cycle + 1
    } else {
        done_in_cycle
    };

    let mut dot_spans: Vec<Span> = (0..n)
        .map(|i| {
            if i < done_in_cycle {
                Span::styled(
                    "● ",
                    Style::default().fg(if no_color { Color::Reset } else { Color::Green }),
                )
            } else if i == done_in_cycle && timer.phase.is_work() {
                // Current in-progress session shown in phase color
                let c = if no_color {
                    Color::Reset
                } else {
                    match timer.phase {
                        PomodoroPhase::Work => Color::Red,
                        PomodoroPhase::Break => Color::Green,
                        PomodoroPhase::LongBreak => Color::Blue,
                    }
                };
                Span::styled("◉ ", Style::default().fg(c))
            } else {
                Span::styled("○ ", Style::default().fg(Color::DarkGray))
            }
        })
        .collect();

    dot_spans.push(Span::styled(
        format!("  {current_session} / {n}"),
        Style::default().fg(Color::DarkGray),
    ));
    let session_line = Line::from(dot_spans);

    // ── Line 1: task + tag ───────────────────────────────────────────────────
    let task_span = if timer.task.is_empty() {
        Span::styled("No task set", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            timer.task.clone(),
            Style::default().fg(if no_color { Color::Reset } else { Color::White }),
        )
    };
    let mut task_spans = vec![task_span];
    if let Some(ref tag) = timer.tag {
        task_spans.push(Span::styled("  #", Style::default().fg(Color::DarkGray)));
        task_spans.push(Span::styled(
            tag.clone(),
            Style::default().fg(if no_color {
                Color::Reset
            } else {
                Color::Magenta
            }),
        ));
    }
    let task_line = Line::from(task_spans);

    // ── Line 2: next phase + keyboard hints ──────────────────────────────────
    let next_text = next_phase_text(timer);
    let mut hint_spans = vec![
        Span::styled(next_text, Style::default().fg(Color::DarkGray)),
        Span::raw("   "),
        Span::styled("[P]", Style::default().fg(Color::Yellow)),
        Span::raw(" pause  "),
        Span::styled("[S]", Style::default().fg(Color::Yellow)),
        Span::raw(" skip  "),
    ];
    if panel_width >= 50 {
        hint_spans.push(Span::styled("[+]", Style::default().fg(Color::Yellow)));
        hint_spans.push(Span::raw(" +5m  "));
    }
    hint_spans.push(Span::styled("[Q]", Style::default().fg(Color::Yellow)));
    hint_spans.push(Span::raw(" stop"));
    let hints_line = Line::from(hint_spans);

    let info_widget = Paragraph::new(vec![session_line, task_line, hints_line])
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(info_widget, area);
}

/// Build the "Next: Xm break/work/long break" preview string.
fn next_phase_text(timer: &PomodoroTimer) -> String {
    let cfg = &timer.config;
    match timer.phase {
        PomodoroPhase::Work => {
            let n = cfg.long_break_after.max(1);
            if (timer.completed_work + 1).is_multiple_of(n) {
                format!("Next: {}m long break", cfg.long_break_duration_mins)
            } else {
                format!("Next: {}m break", cfg.break_duration_mins)
            }
        }
        PomodoroPhase::Break | PomodoroPhase::LongBreak => {
            format!("Next: {}m work", cfg.work_duration_mins)
        }
    }
}
