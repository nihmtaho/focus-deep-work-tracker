use anyhow::Result;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rusqlite::Connection;
use std::io::Write;
use std::time::{Duration, Instant};

use crate::db::{pomodoro_store, session_store};
use crate::display::format::format_elapsed;
use crate::error::FocusError;
use crate::pomodoro::{config::PomodoroConfig, notify, timer::PomodoroTimer};

pub fn run(conn: &Connection, task: String, tag: Option<String>) -> Result<()> {
    let trimmed = task.trim().to_string();
    if trimmed.is_empty() {
        return Err(FocusError::EmptyTask.into());
    }

    if let Some(active) = session_store::get_active_session(conn)? {
        let elapsed = format_elapsed(active.start_time);
        let tag_display = active
            .tag
            .as_deref()
            .map(|t| format!(" [tag: {t}]"))
            .unwrap_or_default();
        eprintln!(
            "Error: Session already running: \"{}\"{}  — elapsed: {}",
            active.task, tag_display, elapsed
        );
        return Err(FocusError::AlreadyRunning {
            task: active.task,
            elapsed,
        }
        .into());
    }

    session_store::insert_session(conn, &trimmed, tag.as_deref())?;

    let tag_display = tag
        .as_deref()
        .map(|t| format!("  [tag: {t}]"))
        .unwrap_or_default();
    println!(
        "{}",
        format!("Session started: {}{}", trimmed, tag_display).green()
    );

    Ok(())
}

/// Run an interactive Pomodoro session in the terminal.
#[allow(clippy::too_many_arguments)]
pub fn run_pomodoro(
    conn: &Connection,
    task: String,
    tag: Option<String>,
    work: Option<u32>,
    break_mins: Option<u32>,
    long_break: Option<u32>,
    long_break_after: Option<u32>,
) -> Result<()> {
    let trimmed = task.trim().to_string();
    if trimmed.is_empty() {
        return Err(FocusError::EmptyTask.into());
    }

    if let Some(active) = session_store::get_active_session(conn)? {
        let elapsed = format_elapsed(active.start_time);
        return Err(FocusError::AlreadyRunning {
            task: active.task,
            elapsed,
        }
        .into());
    }

    let cfg = PomodoroConfig::resolve(work, break_mins, long_break, long_break_after)?;
    let mut timer = PomodoroTimer::new(trimmed.clone(), tag.clone(), cfg);

    let tag_display = tag
        .as_deref()
        .map(|t| format!(" [tag: {t}]"))
        .unwrap_or_default();
    println!(
        "{}",
        format!("Pomodoro session started: {}{}", trimmed, tag_display).green()
    );
    println!("Press P to pause/resume, S to skip break, Q or Ctrl+C to stop.\n");

    enable_raw_mode()?;
    let cleanup = || {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), crossterm::cursor::Show);
    };

    let result = run_pomodoro_loop(conn, &mut timer);

    cleanup();

    match result {
        Ok(auto_abandoned) => {
            if auto_abandoned {
                println!(
                    "\n{}",
                    "Session abandoned: paused for more than 60 minutes.".yellow()
                );
            }
        }
        Err(ref e) => {
            eprintln!("\nError during Pomodoro session: {e}");
        }
    }

    // Record abandonment if stopped mid-work-phase.
    if timer.is_in_work_phase() && timer.completed_work > 0 || timer.is_in_work_phase() {
        let date = pomodoro_store::today_local_date();
        if let Err(e) = pomodoro_store::increment_abandoned(conn, &date) {
            eprintln!("Error saving abandonment stat: {e}");
        }
    }

    let abandoned = if timer.is_in_work_phase() { 1 } else { 0 };
    println!(
        "\n{} completed, {} abandoned.",
        timer.completed_work, abandoned
    );

    result.map(|_| ())
}

/// Inner loop: returns Ok(true) if auto-abandoned due to pause timeout.
fn run_pomodoro_loop(conn: &Connection, timer: &mut PomodoroTimer) -> Result<bool> {
    let mut stdout = std::io::stdout();
    let mut last_tick = Instant::now();

    loop {
        // Advance timer by elapsed time.
        let delta = last_tick.elapsed();
        last_tick = Instant::now();
        let delta_secs = delta.as_secs();

        let events = timer.tick_secs(delta_secs, conn)?;
        let mut auto_abandoned = false;

        for event in &events {
            match event {
                crate::pomodoro::timer::TimerEvent::PhaseComplete {
                    from,
                    to,
                    work_saved,
                } => {
                    let msg = match from {
                        crate::models::pomodoro::PomodoroPhase::Work => {
                            format!("Work phase complete! Starting {}.", to.label())
                        }
                        _ => format!("Break over! Starting {}.", to.label()),
                    };
                    if *work_saved {
                        notify::send_notification("Focus", &msg);
                    }
                    print!("\r\x1b[K{}", msg.green());
                    let _ = stdout.flush();
                }
                crate::pomodoro::timer::TimerEvent::AutoAbandoned { .. } => {
                    auto_abandoned = true;
                }
                _ => {}
            }
        }

        if auto_abandoned {
            return Ok(true);
        }

        // Redraw status line.
        let paused_marker = if timer.paused { " [PAUSED]" } else { "" };
        let status = format!(
            "\r\x1b[K{} | {} | Pomodoro {}/{}{}  [P] pause  [S] skip  [Q] stop",
            timer.phase_label(),
            timer.format_remaining(),
            timer.completed_work,
            timer.config.long_break_after,
            paused_marker,
        );
        print!("{status}");
        let _ = stdout.flush();

        // Poll for key events.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c')
                    || key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Char('Q')
                {
                    break;
                }
                match key.code {
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        if timer.paused {
                            timer.resume();
                        } else {
                            timer.pause();
                        }
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        timer.skip_break();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(false)
}
