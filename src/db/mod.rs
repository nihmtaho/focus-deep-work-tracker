pub mod pomodoro_store;
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
    open_db_at(&db_path)
}

/// Open (or create) a database at an explicit path.
/// Used by integration tests to open a temporary isolated database.
pub fn open_db_at(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path).map_err(|_| FocusError::DataFileCorrupted {
        path: db_path.display().to_string(),
    })?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS sessions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            task       TEXT    NOT NULL,
            tag        TEXT,
            start_time INTEGER NOT NULL,
            end_time   INTEGER,
            mode       TEXT    NOT NULL DEFAULT 'freeform'
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_end_time
            ON sessions(end_time);
        CREATE INDEX IF NOT EXISTS idx_sessions_start_time
            ON sessions(start_time DESC);
        CREATE TABLE IF NOT EXISTS pomodoro_stats (
            date          TEXT    PRIMARY KEY,
            completed     INTEGER NOT NULL DEFAULT 0,
            abandoned     INTEGER NOT NULL DEFAULT 0,
            work_minutes  INTEGER NOT NULL DEFAULT 0,
            break_minutes INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_pomodoro_stats_date
            ON pomodoro_stats(date);",
    )
    .map_err(|_| FocusError::DataFileCorrupted {
        path: db_path.display().to_string(),
    })?;

    // Idempotent migration: add mode column to pre-existing sessions tables
    // (the CREATE TABLE above already includes it for new DBs).
    let mode_col_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name='mode'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    if !mode_col_exists {
        conn.execute_batch(
            "ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform';",
        )
        .map_err(|_| FocusError::DataFileCorrupted {
            path: db_path.display().to_string(),
        })?;
    }

    Ok(conn)
}
