//! Integration tests for report panel metrics
//!
//! Tests report metrics computation via ReportMetrics::compute() and panel display.

use focus::db::open_db_at;
use focus::tui::report::ReportMetrics;
use tempfile::NamedTempFile;

fn test_db() -> (rusqlite::Connection, NamedTempFile) {
    let f = NamedTempFile::new().unwrap();
    let conn = open_db_at(f.path()).unwrap();
    (conn, f)
}

fn insert_completed(conn: &rusqlite::Connection, start_offset: i64, duration: i64) {
    let now = chrono::Utc::now().timestamp();
    let start = now - start_offset;
    conn.execute(
        "INSERT INTO sessions (task, start_time, end_time) VALUES ('task', ?1, ?2)",
        rusqlite::params![start, start + duration],
    )
    .unwrap();
}

// ── T092: Report panel displays all metrics ───────────────────────────────────

#[test]
fn test_report_metrics_panel_shows_count_today() {
    let (conn, _f) = test_db();
    insert_completed(&conn, 600, 300);
    insert_completed(&conn, 1200, 600);

    let metrics = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(metrics.count_today, 2, "panel should show 2 sessions today");
    assert!(metrics.count_week >= 2);
    assert!(metrics.count_all_time >= 2);
}

#[test]
fn test_report_metrics_panel_shows_duration() {
    let (conn, _f) = test_db();
    insert_completed(&conn, 600, 300); // 300s
    insert_completed(&conn, 1200, 600); // 600s

    let metrics = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(metrics.total_duration_today, 900);
    assert_eq!(metrics.format_duration_today(), "00:15:00");
}

#[test]
fn test_report_metrics_panel_shows_completion_rate() {
    let (conn, _f) = test_db();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO todos (title, status, created_at) VALUES ('done', 'completed', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO todos (title, status, created_at) VALUES ('pending', 'active', ?1)",
        rusqlite::params![now],
    )
    .unwrap();

    let metrics = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(metrics.completion_rate, 50);
}

#[test]
fn test_report_metrics_panel_shows_productivity_score() {
    let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
    let score = metrics.compute_productivity_score();
    assert!(score > 0 && score <= 100);
}

// ── T093: Report metrics update when new sessions created ─────────────────────

#[test]
fn test_report_metrics_update_after_new_session() {
    let (conn, _f) = test_db();

    let before = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(before.count_today, 0);

    insert_completed(&conn, 300, 600);

    let after = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(after.count_today, 1, "metrics should reflect new session");
    assert_eq!(after.count_all_time, 1);
    assert_eq!(after.total_duration_today, 600);
}

#[test]
fn test_report_metrics_update_after_multiple_sessions() {
    let (conn, _f) = test_db();

    insert_completed(&conn, 600, 300);
    let m1 = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(m1.count_today, 1);

    insert_completed(&conn, 1200, 600);
    let m2 = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(m2.count_today, 2);
    assert_eq!(m2.total_duration_today, 900);
}

#[test]
fn test_report_metrics_active_session_not_counted() {
    let (conn, _f) = test_db();
    // Insert active session (no end_time)
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO sessions (task, start_time) VALUES ('active', ?1)",
        rusqlite::params![now - 60],
    )
    .unwrap();

    let metrics = ReportMetrics::compute(&conn).unwrap();
    assert_eq!(
        metrics.count_today, 0,
        "active session must not appear in count"
    );
}

// ── Legacy struct tests (kept for backward compat) ────────────────────────────

#[test]
fn test_report_metrics_creation() {
    let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
    assert_eq!(metrics.count_today, 5);
    assert_eq!(metrics.count_week, 12);
    assert_eq!(metrics.completion_rate, 80);
    assert_eq!(metrics.focus_streak_days, 7);
}

#[test]
fn test_report_metrics_default() {
    let metrics = ReportMetrics::default();
    assert_eq!(metrics.count_today, 0);
    assert_eq!(metrics.count_week, 0);
    assert_eq!(metrics.completion_rate, 0);
    assert_eq!(metrics.focus_streak_days, 0);
}

#[test]
fn test_report_metrics_format_duration() {
    let metrics = ReportMetrics::new(1, 1, 1, 3661, 7200, 86400, 80, 5);
    assert_eq!(metrics.format_duration_today(), "01:01:01");
    assert_eq!(metrics.format_duration_week(), "02:00:00");
    assert_eq!(metrics.format_duration_all_time(), "24:00:00");
}

#[test]
fn test_report_metrics_productivity_score() {
    let metrics = ReportMetrics::new(5, 12, 100, 1800, 7200, 86400, 80, 7);
    let score = metrics.compute_productivity_score();
    assert!(score > 0);
    assert!(score <= 10000);
}

#[test]
fn test_report_metrics_with_zero_completion() {
    let metrics = ReportMetrics::new(0, 0, 0, 0, 0, 0, 0, 0);
    assert_eq!(metrics.count_today, 0);
    assert_eq!(metrics.completion_rate, 0);
    assert_eq!(metrics.format_duration_today(), "00:00:00");
}

#[test]
fn test_report_metrics_high_completion() {
    let metrics = ReportMetrics::new(10, 50, 500, 18000, 72000, 360000, 95, 15);
    assert_eq!(metrics.completion_rate, 95);
    assert!(metrics.compute_productivity_score() > 80);
}
