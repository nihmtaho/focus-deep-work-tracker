use anyhow::Result;
use chrono::{Datelike, Duration, Local, TimeZone};
use rusqlite::Connection;

use crate::db::session_store;
use crate::display::format::format_duration;

pub fn current_week_start() -> i64 {
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday() as i64;
    let monday = now.date_naive() - chrono::Duration::days(days_since_monday);
    let midnight = monday.and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&midnight)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn today_start() -> i64 {
    let now = Local::now();
    let today = now.date_naive().and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&today)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn rolling_7d_start() -> i64 {
    (chrono::Utc::now() - Duration::seconds(7 * 86400)).timestamp()
}

pub fn run(conn: &Connection, today: bool, week: bool) -> Result<()> {
    let since = if today {
        today_start()
    } else if week {
        rolling_7d_start()
    } else {
        current_week_start()
    };

    let rows = session_store::aggregate_by_tag(conn, since)?;

    if rows.is_empty() {
        println!("No sessions recorded for this period.");
        return Ok(());
    }

    let tag_width = rows
        .iter()
        .map(|(tag, _)| tag.as_deref().unwrap_or("untagged").len())
        .max()
        .unwrap_or(8)
        .max(8);

    let separator = "─".repeat(22 + tag_width);

    println!("{:<tag_width$} Total", "Tag");
    println!("{}", separator);

    let mut grand_total: i64 = 0;
    for (tag, secs) in &rows {
        let label = tag.as_deref().unwrap_or("untagged");
        let dur = format_duration(Duration::seconds(*secs));
        println!("{:<tag_width$} {}", label, dur);
        grand_total += secs;
    }

    println!("{}", separator);
    println!(
        "{:<tag_width$} {}",
        "TOTAL",
        format_duration(Duration::seconds(grand_total))
    );

    // Mode breakdown (only when both modes exist).
    let mode_rows: Vec<(String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT mode, SUM(end_time - start_time) as secs \
             FROM sessions WHERE end_time IS NOT NULL AND start_time >= ?1 \
             GROUP BY mode ORDER BY secs DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![since], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    if mode_rows.len() > 1 {
        println!();
        println!("Mode Breakdown");
        println!("{}", "─".repeat(24));
        for (mode, secs) in &mode_rows {
            let pct = if grand_total > 0 {
                (*secs as f64 / grand_total as f64 * 100.0) as u32
            } else {
                0
            };
            println!(
                "{:<10}: {} ({}%)",
                mode,
                format_duration(Duration::seconds(*secs)),
                pct
            );
        }
    }

    Ok(())
}
