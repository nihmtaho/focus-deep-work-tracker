use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;

use crate::db::session_store;
use crate::display::format::format_duration;
use crate::error::FocusError;

pub fn run(conn: &Connection) -> Result<()> {
    if session_store::get_active_session(conn)?.is_none() {
        return Err(FocusError::NoActiveSession.into());
    }

    let session = session_store::stop_session(conn)?;
    let tag_display = session
        .tag
        .as_deref()
        .map(|t| format!("  [tag: {t}]"))
        .unwrap_or_default();
    let duration = session
        .duration()
        .map(format_duration)
        .unwrap_or_else(|| "0s".to_string());

    println!(
        "{}",
        format!("Stopped: \"{}\"  {}", session.task, tag_display).yellow()
    );
    println!("Duration: {}", duration.bold());

    Ok(())
}
