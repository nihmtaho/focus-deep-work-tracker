pub mod session_store;

use anyhow::Result;
use rusqlite::Connection;

use crate::error::FocusError;

pub fn open_db() -> Result<Connection> {
    let home = dirs::home_dir().ok_or_else(|| FocusError::DataFileCorrupted {
        path: "~/.local/share/focus/focus.db".to_string(),
    })?;
    let dir = home.join(".local/share/focus");
    std::fs::create_dir_all(&dir)?;
    let db_path = dir.join("focus.db");

    let conn = Connection::open(&db_path).map_err(|_| FocusError::DataFileCorrupted {
        path: db_path.display().to_string(),
    })?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            task       TEXT    NOT NULL,
            tag        TEXT,
            start_time INTEGER NOT NULL,
            end_time   INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_end_time
            ON sessions(end_time);
        CREATE INDEX IF NOT EXISTS idx_sessions_start_time
            ON sessions(start_time DESC);",
    )
    .map_err(|_| FocusError::DataFileCorrupted {
        path: db_path.display().to_string(),
    })?;

    Ok(conn)
}
