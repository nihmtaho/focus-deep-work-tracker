use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::session_store;
use crate::display::format::format_duration;
use crate::error::FocusError;
use crate::models::session::Session;

#[derive(Serialize)]
struct ExportSession {
    id: i64,
    task: String,
    tag: Option<String>,
    start_time: String,
    end_time: String,
    duration_seconds: i64,
}

pub fn export_json(sessions: &[Session]) -> String {
    if sessions.is_empty() {
        return "[]".to_string();
    }
    let export: Vec<ExportSession> = sessions
        .iter()
        .filter_map(|s| {
            s.end_time.map(|end| ExportSession {
                id: s.id,
                task: s.task.clone(),
                tag: s.tag.clone(),
                start_time: s.start_time.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                end_time: end.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                duration_seconds: (end - s.start_time).num_seconds(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&export).unwrap_or_else(|_| "[]".to_string())
}

pub fn export_markdown(sessions: &[Session]) -> String {
    let mut out = String::from("| Date | Task | Tag | Start | End | Duration |\n");
    out.push_str("|------|------|-----|-------|-----|----------|\n");
    for s in sessions {
        if let Some(end) = s.end_time {
            let date = s.start_time.format("%Y-%m-%d").to_string();
            let start = s.start_time.format("%H:%M").to_string();
            let end_str = end.format("%H:%M").to_string();
            let tag = s.tag.as_deref().unwrap_or("—");
            let duration = format_duration(end - s.start_time);
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                date, s.task, tag, start, end_str, duration
            ));
        }
    }
    out
}

pub fn run(conn: &Connection, format: String) -> Result<()> {
    let fmt = format.trim().to_lowercase();
    if fmt != "json" && fmt != "markdown" {
        return Err(FocusError::InvalidFormat.into());
    }

    let sessions = session_store::list_all_completed(conn)?;

    let output = if fmt == "json" {
        export_json(&sessions)
    } else {
        export_markdown(&sessions)
    };

    println!("{}", output);
    Ok(())
}
