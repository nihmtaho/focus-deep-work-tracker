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
    let now = Utc::now().timestamp();
    conn.execute(
        "UPDATE sessions SET end_time = ?1 WHERE end_time IS NULL",
        rusqlite::params![now],
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time = ?1 ORDER BY id DESC LIMIT 1",
    )?;
    let (id, task, tag, start_epoch, end_epoch) =
        stmt.query_row(rusqlite::params![now], |row| {
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
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time DESC LIMIT ?1",
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

pub fn list_all_completed(conn: &Connection) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, task, tag, start_time, end_time FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time ASC",
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
