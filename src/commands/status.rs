use anyhow::Result;
use colored::Colorize;
use rusqlite::Connection;

use crate::db::session_store;
use crate::display::format::format_elapsed;

pub fn run(conn: &Connection) -> Result<()> {
    match session_store::get_active_session(conn)? {
        Some(session) => {
            let tag_display = session
                .tag
                .as_deref()
                .map(|t| format!("  [tag: {t}]"))
                .unwrap_or_default();
            println!(
                "{}",
                format!("Working on: \"{}\" {}", session.task, tag_display).cyan()
            );
            println!("Elapsed: {}", format_elapsed(session.start_time).bold());
        }
        None => {
            println!("No active session.");
        }
    }
    Ok(())
}
