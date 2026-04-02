use anyhow::Result;
use rusqlite::Connection;

use crate::db::session_store;
use crate::display::format::print_log_table;
use crate::error::FocusError;

pub fn run(conn: &Connection, limit: u32) -> Result<()> {
    if limit == 0 {
        return Err(FocusError::InvalidLimit.into());
    }

    let sessions = session_store::list_sessions(conn, limit)?;

    if sessions.is_empty() {
        println!("No sessions recorded yet.");
        return Ok(());
    }

    print_log_table(&sessions);
    Ok(())
}
