// Integration tests for `focus pomo-stats` command (T043).
use focus::commands::pomo_stats;
use focus::db::{open_db_at, pomodoro_store};

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

#[test]
fn pomo_stats_today_no_data_returns_ok() {
    let conn = temp_conn();
    // Should print "No Pomodoro sessions today." without error.
    let result = pomo_stats::run(&conn, true, false);
    assert!(result.is_ok());
}

#[test]
fn pomo_stats_week_no_data_returns_ok() {
    let conn = temp_conn();
    let result = pomo_stats::run(&conn, false, true);
    assert!(result.is_ok());
}

#[test]
fn pomo_stats_default_is_today() {
    let conn = temp_conn();
    // When neither today nor week is set, defaults to today view — should not error.
    let result = pomo_stats::run(&conn, false, false);
    assert!(result.is_ok());
}

#[test]
fn pomo_stats_today_with_data_returns_ok() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    pomodoro_store::increment_completed(&conn, &date, 25, 5).unwrap();
    pomodoro_store::increment_abandoned(&conn, &date).unwrap();
    let result = pomo_stats::run(&conn, true, false);
    assert!(result.is_ok());
}

#[test]
fn pomo_stats_week_with_data_returns_ok() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    pomodoro_store::increment_completed(&conn, &date, 25, 5).unwrap();
    let result = pomo_stats::run(&conn, false, true);
    assert!(result.is_ok());
}
