use chrono::{DateTime, Duration, Utc};

use crate::models::session::Session;

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.num_seconds().max(0);
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a duration given in whole minutes (e.g., 65 → "1h 05m", 30 → "30m").
pub fn format_duration_mins(mins: u32) -> String {
    if mins == 0 {
        return "—".to_string();
    }
    let h = mins / 60;
    let m = mins % 60;
    if h > 0 {
        format!("{}h {:02}m", h, m)
    } else {
        format!("{}m", m)
    }
}

pub fn format_elapsed(start: DateTime<Utc>) -> String {
    format_duration(Utc::now() - start)
}

pub struct TableRow {
    pub date: String,
    pub task: String,
    pub tag: String,
    pub duration: String,
    pub mode: String,
}

pub fn build_table_rows(sessions: &[Session]) -> Vec<TableRow> {
    sessions
        .iter()
        .map(|s| TableRow {
            date: s.start_time.format("%Y-%m-%d %H:%M").to_string(),
            task: s.task.clone(),
            tag: s.tag.clone().unwrap_or_else(|| "—".to_string()),
            duration: s
                .duration()
                .map(format_duration)
                .unwrap_or_else(|| "—".to_string()),
            mode: s.mode.clone(),
        })
        .collect()
}

pub fn print_log_table(sessions: &[Session]) {
    let rows = build_table_rows(sessions);

    let task_width = rows.iter().map(|r| r.task.len()).max().unwrap_or(4).max(4);
    let tag_width = rows.iter().map(|r| r.tag.len()).max().unwrap_or(3).max(3);

    println!(
        "{:<20} {:<task_width$} {:<tag_width$} {:<9} DURATION",
        "DATE", "TASK", "TAG", "MODE"
    );
    for row in &rows {
        println!(
            "{:<20} {:<task_width$} {:<tag_width$} {:<9} {}",
            row.date, row.task, row.tag, row.mode, row.duration
        );
    }
}
