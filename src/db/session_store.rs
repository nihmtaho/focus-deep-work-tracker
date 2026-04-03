use anyhow::Result;
use chrono::{TimeZone, Utc};
use rusqlite::Connection;

use crate::models::session::Session;

fn row_to_session(
    id: i64,
    task: String,
    tag: Option<String>,
    start_epoch: i64,
    end_epoch: Option<i64>,
) -> Session {
    let start_time = Utc
        .timestamp_opt(start_epoch, 0)
        .single()
        .unwrap_or_default();
    let end_time = end_epoch.and_then(|e| Utc.timestamp_opt(e, 0).single());
    Session {
        id,
        task,
        tag,
        start_time,
        end_time,
    }
}

pub fn insert_session(conn: &Connection, task: &str, tag: Option<&str>) -> Result<()> {
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO sessions (task, tag, start_time) VALUES (?1, ?2, ?3)",
        rusqlite::params![task, tag, now],
    )?;
    Ok(())
}

pub fn get_active_session(conn: &Connection) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time IS NULL LIMIT 1",
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;

    if let Some(row) = rows.next() {
        let (id, task, tag, start_epoch, end_epoch) = row?;
        Ok(Some(row_to_session(id, task, tag, start_epoch, end_epoch)))
    } else {
        Ok(None)
    }
}

pub fn stop_session(conn: &Connection) -> Result<Session> {
    // Capture the active session's ID before updating so we can fetch it back
    // precisely. Using a timestamp as the join key is unsafe — two sessions
    // could share the same end_time second.
    let active = get_active_session(conn)?
        .ok_or_else(|| anyhow::anyhow!("No active session to stop"))?;

    let now = Utc::now().timestamp();
    conn.execute(
        "UPDATE sessions SET end_time = ?1 WHERE id = ?2",
        rusqlite::params![now, active.id],
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE id = ?1",
    )?;
    let (id, task, tag, start_epoch, end_epoch) =
        stmt.query_row(rusqlite::params![active.id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        })?;

    Ok(row_to_session(id, task, tag, start_epoch, end_epoch))
}

pub fn list_sessions(conn: &Connection, limit: u32) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;

    let mut sessions = Vec::new();
    for row in rows {
        let (id, task, tag, start_epoch, end_epoch) = row?;
        sessions.push(row_to_session(id, task, tag, start_epoch, end_epoch));
    }
    Ok(sessions)
}

pub fn aggregate_by_tag(conn: &Connection, since: i64) -> Result<Vec<(Option<String>, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT tag, SUM(end_time - start_time) FROM sessions WHERE end_time IS NOT NULL AND start_time >= ?1 GROUP BY tag ORDER BY SUM(end_time - start_time) DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![since], |row| {
        Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?))
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn count_completed(conn: &Connection) -> Result<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sessions WHERE end_time IS NOT NULL",
        [],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Delete a completed session by id. Returns SessionNotFound if id does not exist
/// or the session is still active (end_time IS NULL).
pub fn delete_session(conn: &Connection, id: i64) -> Result<()> {
    let rows = conn.execute(
        "DELETE FROM sessions WHERE id = ?1 AND end_time IS NOT NULL",
        rusqlite::params![id],
    )?;
    if rows == 0 {
        return Err(crate::error::FocusError::SessionNotFound { id }.into());
    }
    Ok(())
}

/// Rename a completed (or active) session's task. Returns SessionNotFound if id
/// does not exist; returns EmptyTask if new_task is blank.
pub fn rename_session(conn: &Connection, id: i64, new_task: &str) -> Result<()> {
    if new_task.trim().is_empty() {
        return Err(crate::error::FocusError::EmptyTask.into());
    }
    let rows = conn.execute(
        "UPDATE sessions SET task = ?1 WHERE id = ?2",
        rusqlite::params![new_task.trim(), id],
    )?;
    if rows == 0 {
        return Err(crate::error::FocusError::SessionNotFound { id }.into());
    }
    Ok(())
}

pub fn list_all_completed(conn: &Connection) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;

    let mut sessions = Vec::new();
    for row in rows {
        let (id, task, tag, start_epoch, end_epoch) = row?;
        sessions.push(row_to_session(id, task, tag, start_epoch, end_epoch));
    }
    Ok(sessions)
}

/// List completed sessions that started at or after `since` (Unix epoch), newest first.
pub fn list_completed_since(conn: &Connection, since: i64) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions \
         WHERE end_time IS NOT NULL AND start_time >= ?1 \
         ORDER BY start_time DESC, id DESC",
    )?;
    let rows = stmt.query_map(rusqlite::params![since], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<i64>>(4)?,
        ))
    })?;
    let mut sessions = Vec::new();
    for row in rows {
        let (id, task, tag, start_epoch, end_epoch) = row?;
        sessions.push(row_to_session(id, task, tag, start_epoch, end_epoch));
    }
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use tempfile::NamedTempFile;

    fn test_db() -> (Connection, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let conn = open_db_at(f.path()).unwrap();
        (conn, f)
    }

    fn insert_completed(conn: &Connection, task: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (task, tag, start_time, end_time) VALUES (?1, NULL, ?2, ?3)",
            rusqlite::params![task, now - 60, now],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_active(conn: &Connection, task: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (task, tag, start_time) VALUES (?1, NULL, ?2)",
            rusqlite::params![task, now - 60],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    // stop_session tests
    #[test]
    fn stop_session_returns_correct_session() {
        let (conn, _f) = test_db();
        let id = insert_active(&conn, "my task");
        let session = stop_session(&conn).expect("should stop active session");
        assert_eq!(session.id, id);
        assert_eq!(session.task, "my task");
        assert!(session.end_time.is_some());
    }

    #[test]
    fn stop_session_no_active_returns_error() {
        let (conn, _f) = test_db();
        let err = stop_session(&conn).unwrap_err();
        assert!(err.to_string().contains("No active session"));
    }

    // delete_session tests (T003)
    #[test]
    fn delete_session_completed_ok() {
        let (conn, _f) = test_db();
        let id = insert_completed(&conn, "task a");
        delete_session(&conn, id).expect("should delete completed session");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn delete_session_not_found_returns_error() {
        let (conn, _f) = test_db();
        let err = delete_session(&conn, 9999).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("9999"));
    }

    #[test]
    fn delete_session_does_not_delete_active() {
        let (conn, _f) = test_db();
        let id = insert_active(&conn, "active task");
        let err = delete_session(&conn, id).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains(&id.to_string()));
        // Row still exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    // rename_session tests (T004)
    #[test]
    fn rename_session_updates_task_name() {
        let (conn, _f) = test_db();
        let id = insert_completed(&conn, "old name");
        rename_session(&conn, id, "new name").expect("should rename");
        let task: String = conn
            .query_row("SELECT task FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(task, "new name");
    }

    #[test]
    fn rename_session_not_found_returns_error() {
        let (conn, _f) = test_db();
        let err = rename_session(&conn, 9999, "x").unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("9999"));
    }

    #[test]
    fn rename_session_empty_task_returns_error() {
        let (conn, _f) = test_db();
        let id = insert_completed(&conn, "name");
        let err = rename_session(&conn, id, "   ").unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("cannot be empty"));
    }
}
