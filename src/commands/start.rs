use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;

use crate::db::session_store;
use crate::display::format::format_elapsed;
use crate::error::FocusError;

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
