use anyhow::Result;
use chrono::Datelike;
use rusqlite::Connection;

use crate::models::pomodoro::PomodoroStats;

/// Returns today's local date as an ISO 8601 string (e.g., "2026-04-06").
pub fn today_local_date() -> String {
    chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string()
}

/// Increment completed count for `date`, adding work and break minutes.
/// Uses an upsert so missing rows are created automatically.
pub fn increment_completed(
    conn: &Connection,
    date: &str,
    work_mins: u32,
    break_mins: u32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO pomodoro_stats(date, completed, work_minutes, break_minutes) \
         VALUES(?1, 1, ?2, ?3) \
         ON CONFLICT(date) DO UPDATE SET \
           completed    = completed + 1, \
           work_minutes = work_minutes + excluded.work_minutes, \
           break_minutes = break_minutes + excluded.break_minutes",
        rusqlite::params![date, work_mins, break_mins],
    )?;
    Ok(())
}

/// Increment abandoned count for `date`.
pub fn increment_abandoned(conn: &Connection, date: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO pomodoro_stats(date, abandoned) VALUES(?1, 1) \
         ON CONFLICT(date) DO UPDATE SET abandoned = abandoned + 1",
        rusqlite::params![date],
    )?;
    Ok(())
}

/// Get stats for a single date. Returns a zeroed struct if no row exists.
pub fn get_stats_for_date(conn: &Connection, date: &str) -> Result<PomodoroStats> {
    let result = conn.query_row(
        "SELECT date, completed, abandoned, work_minutes, break_minutes \
         FROM pomodoro_stats WHERE date = ?1",
        rusqlite::params![date],
        |row| {
            Ok(PomodoroStats {
                date: row.get(0)?,
                completed: row.get::<_, u32>(1)?,
                abandoned: row.get::<_, u32>(2)?,
                work_minutes: row.get::<_, u32>(3)?,
                break_minutes: row.get::<_, u32>(4)?,
            })
        },
    );

    match result {
        Ok(stats) => Ok(stats),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(PomodoroStats {
            date: date.to_string(),
            ..Default::default()
        }),
        Err(e) => Err(e.into()),
    }
}

/// Get stats for a date range [from_date, to_date] inclusive, ordered ASC.
pub fn get_stats_range(
    conn: &Connection,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<PomodoroStats>> {
    let mut stmt = conn.prepare(
        "SELECT date, completed, abandoned, work_minutes, break_minutes \
         FROM pomodoro_stats \
         WHERE date >= ?1 AND date <= ?2 \
         ORDER BY date ASC",
    )?;
    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok(PomodoroStats {
            date: row.get(0)?,
            completed: row.get::<_, u32>(1)?,
            abandoned: row.get::<_, u32>(2)?,
            work_minutes: row.get::<_, u32>(3)?,
            break_minutes: row.get::<_, u32>(4)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// Calculate current streak: consecutive trailing days (ending today) with completed > 0.
pub fn calculate_streak(stats: &[PomodoroStats]) -> u32 {
    // stats is sorted ASC by date; we scan from the end
    let today = chrono::Local::now().date_naive();
    let mut streak = 0u32;
    let mut expected = today;

    for stat in stats.iter().rev() {
        let Ok(date) = chrono::NaiveDate::parse_from_str(&stat.date, "%Y-%m-%d") else {
            break;
        };
        if date != expected {
            break;
        }
        if stat.completed == 0 {
            break;
        }
        streak += 1;
        expected -= chrono::Duration::days(1);
    }

    streak
}

/// Calculate best streak: longest consecutive run of days with completed > 0.
pub fn calculate_best_streak(stats: &[PomodoroStats]) -> u32 {
    let mut best = 0u32;
    let mut current = 0u32;
    let mut prev_date: Option<chrono::NaiveDate> = None;

    for stat in stats {
        let Ok(date) = chrono::NaiveDate::parse_from_str(&stat.date, "%Y-%m-%d") else {
            current = 0;
            prev_date = None;
            continue;
        };
        if stat.completed == 0 {
            current = 0;
            prev_date = None;
            continue;
        }
        match prev_date {
            Some(prev) if date == prev + chrono::Duration::days(1) => {
                current += 1;
            }
            _ => {
                current = 1;
            }
        }
        if current > best {
            best = current;
        }
        prev_date = Some(date);
    }

    best
}

/// Returns today's local date broken down for use in date arithmetic.
pub fn today_parts() -> (i32, u32, u32) {
    let d = chrono::Local::now().date_naive();
    (d.year(), d.month(), d.day())
}
