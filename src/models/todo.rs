use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

/// Represents a user-defined task that can be linked to sessions.
#[derive(Debug, Clone, Serialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub status: String, // "active" | "completed"
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl Todo {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }
}

/// Insert a new TODO with the given title.
/// Returns an error if title is empty or exceeds 256 characters.
pub fn insert(conn: &Connection, title: &str) -> Result<Todo> {
    if title.is_empty() || title.len() > 256 {
        anyhow::bail!("Title must be 1-256 characters");
    }

    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO todos (title, status, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![title, "active", now],
    )?;

    let id = conn.last_insert_rowid() as u64;
    Ok(Todo {
        id,
        title: title.to_string(),
        status: "active".to_string(),
        created_at: now,
        completed_at: None,
    })
}

/// Mark a TODO as completed.
pub fn update_status(conn: &Connection, id: u64, status: &str) -> Result<Todo> {
    if status != "active" && status != "completed" {
        anyhow::bail!("Status must be 'active' or 'completed'");
    }

    let now = Utc::now().timestamp();
    let completed_at = if status == "completed" { Some(now) } else { None };

    conn.execute(
        "UPDATE todos SET status = ?1, completed_at = ?2 WHERE id = ?3",
        rusqlite::params![status, completed_at, id as i64],
    )?;

    // Fetch the updated todo
    let mut stmt = conn.prepare("SELECT id, title, status, created_at, completed_at FROM todos WHERE id = ?1")?;
    let todo = stmt.query_row(rusqlite::params![id as i64], |row| {
        Ok(Todo {
            id: row.get::<_, i64>(0)? as u64,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            completed_at: row.get(4)?,
        })
    })?;

    Ok(todo)
}

/// Delete a TODO by id.
pub fn delete(conn: &Connection, id: u64) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", rusqlite::params![id as i64])?;
    Ok(())
}

/// List all TODOs, ordered by status (active first) then by created_at (newest first).
pub fn list_all(conn: &Connection) -> Result<Vec<Todo>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, status, created_at, completed_at FROM todos ORDER BY status ASC, created_at DESC"
    )?;

    let todos = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get::<_, i64>(0)? as u64,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            completed_at: row.get(4)?,
        })
    })?;

    let mut result = Vec::new();
    for todo in todos {
        result.push(todo?);
    }
    Ok(result)
}

/// Check if a TODO can be deleted (not linked to active session).
pub fn can_delete(conn: &Connection, id: u64) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sessions WHERE todo_id = ?1 AND end_time IS NULL",
        rusqlite::params![id as i64],
        |row| row.get(0),
    )?;
    Ok(count == 0)
}
