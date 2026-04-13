//! Report panel module
//!
//! Provides report panel functionality for displaying session analytics and metrics.
//! Includes daily/weekly trends, completion rates, focus streaks, and productivity scores.

use anyhow::Result;
use chrono::{Local, NaiveDate};
use rusqlite::Connection;

/// Report metrics for display on the Report panel
#[derive(Debug, Clone)]
pub struct ReportMetrics {
    /// Number of sessions completed today
    pub count_today: u32,
    /// Number of sessions completed this week
    pub count_week: u32,
    /// Total sessions completed (all time)
    pub count_all_time: u32,
    /// Total duration of sessions today (seconds)
    pub total_duration_today: u64,
    /// Total duration of sessions this week (seconds)
    pub total_duration_week: u64,
    /// Total duration of sessions all time (seconds)
    pub total_duration_all_time: u64,
    /// Percentage of TODOs completed (0–100)
    pub completion_rate: u32,
    /// Number of consecutive days with at least one session
    pub focus_streak_days: u32,
}

impl ReportMetrics {
    /// Create new metrics with specified values
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        count_today: u32,
        count_week: u32,
        count_all_time: u32,
        total_duration_today: u64,
        total_duration_week: u64,
        total_duration_all_time: u64,
        completion_rate: u32,
        focus_streak_days: u32,
    ) -> Self {
        Self {
            count_today,
            count_week,
            count_all_time,
            total_duration_today,
            total_duration_week,
            total_duration_all_time,
            completion_rate,
            focus_streak_days,
        }
    }

    /// Compute metrics by querying the database.
    ///
    /// Queries: session counts (today/week/all-time), duration sums, TODO
    /// completion rate, and consecutive-day focus streak.
    pub fn compute(conn: &Connection) -> Result<Self> {
        let today_ts = today_start_ts();
        let week_ts = week_start_ts();

        // Session counts
        let count_today: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions \
             WHERE end_time IS NOT NULL AND start_time >= ?1",
            rusqlite::params![today_ts],
            |r| r.get(0),
        )?;

        let count_week: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions \
             WHERE end_time IS NOT NULL AND start_time >= ?1",
            rusqlite::params![week_ts],
            |r| r.get(0),
        )?;

        let count_all_time: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE end_time IS NOT NULL",
            [],
            |r| r.get(0),
        )?;

        // Duration sums (COALESCE handles NULL when no rows match)
        let total_duration_today: u64 = conn
            .query_row(
                "SELECT COALESCE(SUM(end_time - start_time), 0) FROM sessions \
                 WHERE end_time IS NOT NULL AND start_time >= ?1",
                rusqlite::params![today_ts],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_duration_week: u64 = conn
            .query_row(
                "SELECT COALESCE(SUM(end_time - start_time), 0) FROM sessions \
                 WHERE end_time IS NOT NULL AND start_time >= ?1",
                rusqlite::params![week_ts],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_duration_all_time: u64 = conn
            .query_row(
                "SELECT COALESCE(SUM(end_time - start_time), 0) FROM sessions \
                 WHERE end_time IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // TODO completion rate: completed / total * 100
        let total_todos: u32 = conn
            .query_row("SELECT COUNT(*) FROM todos", [], |r| r.get(0))
            .unwrap_or(0);
        let completed_todos: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM todos WHERE status = 'completed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let completion_rate = if total_todos > 0 {
            completed_todos * 100 / total_todos
        } else {
            0
        };

        // Focus streak: consecutive days (ending today) with ≥1 completed session
        let focus_streak_days = compute_focus_streak(conn);

        Ok(Self::new(
            count_today,
            count_week,
            count_all_time,
            total_duration_today,
            total_duration_week,
            total_duration_all_time,
            completion_rate,
            focus_streak_days,
        ))
    }

    /// Format today's duration as HH:MM:SS
    pub fn format_duration_today(&self) -> String {
        Self::format_duration(self.total_duration_today)
    }

    /// Format this week's duration as HH:MM:SS
    pub fn format_duration_week(&self) -> String {
        Self::format_duration(self.total_duration_week)
    }

    /// Format all-time duration as HH:MM:SS
    pub fn format_duration_all_time(&self) -> String {
        Self::format_duration(self.total_duration_all_time)
    }

    /// Format duration in seconds to HH:MM:SS format
    fn format_duration(secs: u64) -> String {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Compute productivity score based on metrics.
    /// Formula: (completion_rate * 0.4) + (streak/30 * 30) + (sessions_today/5 * 30)
    /// Max score: 100 (completion=100%, streak≥30 days, ≥5 sessions today)
    pub fn compute_productivity_score(&self) -> u32 {
        let completion_score = (self.completion_rate as f64) * 0.4;
        let streak_score = ((self.focus_streak_days as f64 / 30.0) * 30.0).min(30.0);
        let session_score = ((self.count_today as f64 / 5.0) * 30.0).min(30.0);

        (completion_score + streak_score + session_score) as u32
    }
}

impl Default for ReportMetrics {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0, 0)
    }
}

/// Unix timestamp for the start of today (local midnight).
fn today_start_ts() -> i64 {
    crate::commands::report::today_start()
}

/// Unix timestamp for the start of the current week (local Monday midnight).
fn week_start_ts() -> i64 {
    crate::commands::report::current_week_start()
}

/// Count consecutive days ending today that have at least one completed session.
fn compute_focus_streak(conn: &Connection) -> u32 {
    // Fetch distinct local dates (YYYY-MM-DD) for all completed sessions, newest first.
    let mut stmt = match conn.prepare(
        "SELECT DISTINCT date(start_time, 'unixepoch', 'localtime') AS d \
         FROM sessions WHERE end_time IS NOT NULL ORDER BY d DESC",
    ) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let dates: Vec<NaiveDate> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .ok()
        .map(|rows| {
            rows.filter_map(|r| r.ok())
                .filter_map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
                .collect()
        })
        .unwrap_or_default();

    if dates.is_empty() {
        return 0;
    }

    let today = Local::now().date_naive();
    let mut streak = 0u32;
    let mut expected = today;

    for date in dates {
        if date == expected {
            streak += 1;
            expected = expected.pred_opt().unwrap_or(expected);
        } else if date < expected {
            // Gap found — streak ends
            break;
        }
        // date > expected means future date (shouldn't happen), skip
    }

    streak
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

    fn insert_completed_session(conn: &Connection, start_offset_secs: i64, duration_secs: i64) {
        let now = chrono::Utc::now().timestamp();
        let start = now - start_offset_secs;
        let end = start + duration_secs;
        conn.execute(
            "INSERT INTO sessions (task, start_time, end_time) VALUES ('test', ?1, ?2)",
            rusqlite::params![start, end],
        )
        .unwrap();
    }

    // ── T090: compute() queries correct session data ─────────────────────────────

    #[test]
    fn test_compute_queries_session_count_today() {
        let (conn, _f) = test_db();
        // 2 sessions completed today (started <1h ago)
        insert_completed_session(&conn, 600, 300);
        insert_completed_session(&conn, 1200, 600);

        let metrics = ReportMetrics::compute(&conn).unwrap();
        assert_eq!(metrics.count_today, 2);
        assert!(metrics.count_week >= 2);
        assert!(metrics.count_all_time >= 2);
    }

    #[test]
    fn test_compute_excludes_active_sessions_from_count() {
        let (conn, _f) = test_db();
        // Active session (no end_time)
        conn.execute(
            "INSERT INTO sessions (task, start_time) VALUES ('active', ?1)",
            rusqlite::params![chrono::Utc::now().timestamp() - 60],
        )
        .unwrap();

        let metrics = ReportMetrics::compute(&conn).unwrap();
        assert_eq!(metrics.count_today, 0); // active session must not be counted
    }

    // ── T091: metric calculations (count, duration, completion_rate) ─────────────

    #[test]
    fn test_compute_duration_sums() {
        let (conn, _f) = test_db();
        // Two sessions: 300s + 600s = 900s total today
        insert_completed_session(&conn, 600, 300);
        insert_completed_session(&conn, 1200, 600);

        let metrics = ReportMetrics::compute(&conn).unwrap();
        assert_eq!(metrics.total_duration_today, 900);
        assert!(metrics.total_duration_all_time >= 900);
    }

    #[test]
    fn test_compute_completion_rate_with_todos() {
        let (conn, _f) = test_db();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO todos (title, status, created_at) VALUES ('t1', 'completed', ?1)",
            rusqlite::params![now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO todos (title, status, created_at) VALUES ('t2', 'active', ?1)",
            rusqlite::params![now],
        )
        .unwrap();

        let metrics = ReportMetrics::compute(&conn).unwrap();
        // 1 completed / 2 total = 50%
        assert_eq!(metrics.completion_rate, 50);
    }

    #[test]
    fn test_compute_completion_rate_no_todos() {
        let (conn, _f) = test_db();
        let metrics = ReportMetrics::compute(&conn).unwrap();
        assert_eq!(metrics.completion_rate, 0);
    }

    #[test]
    fn test_compute_empty_database() {
        let (conn, _f) = test_db();
        let metrics = ReportMetrics::compute(&conn).unwrap();
        assert_eq!(metrics.count_today, 0);
        assert_eq!(metrics.count_week, 0);
        assert_eq!(metrics.count_all_time, 0);
        assert_eq!(metrics.total_duration_today, 0);
        assert_eq!(metrics.focus_streak_days, 0);
    }

    // ── Existing non-DB tests ─────────────────────────────────────────────────────

    #[test]
    fn test_create_default_metrics() {
        let metrics = ReportMetrics::new(0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(metrics.count_today, 0);
        assert_eq!(metrics.completion_rate, 0);
    }

    #[test]
    fn test_format_duration() {
        let metrics = ReportMetrics::new(1, 1, 1, 3661, 7200, 86400, 80, 5);
        assert_eq!(metrics.format_duration_today(), "01:01:01");
        assert_eq!(metrics.format_duration_week(), "02:00:00");
        assert_eq!(metrics.format_duration_all_time(), "24:00:00");
    }

    #[test]
    fn test_productivity_score() {
        let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
        let score = metrics.compute_productivity_score();
        assert!(score > 0);
        assert!(score <= 100);
    }
}
