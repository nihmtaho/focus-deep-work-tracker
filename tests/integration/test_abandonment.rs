// Integration tests for abandonment handling (T052).
use focus::db::{open_db_at, pomodoro_store, session_store};

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

#[test]
fn only_completed_phases_saved_to_sessions() {
    let conn = temp_conn();
    let now = chrono::Utc::now().timestamp();
    // Simulate 2 completed work phases saved by the timer.
    session_store::insert_session_with_times(
        &conn,
        "task",
        None,
        "pomodoro",
        now - 3600,
        now - 2100,
    )
    .unwrap();
    session_store::insert_session_with_times(
        &conn,
        "task",
        None,
        "pomodoro",
        now - 1800,
        now - 300,
    )
    .unwrap();
    // 3rd phase was abandoned — NOT saved.
    let sessions = session_store::list_sessions(&conn, 10).unwrap();
    assert_eq!(
        sessions.len(),
        2,
        "only completed phases should be in sessions table"
    );
}

#[test]
fn abandoned_count_incremented_when_mid_work() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    pomodoro_store::increment_abandoned(&conn, &date).unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, &date).unwrap();
    assert_eq!(stats.abandoned, 1);
}

#[test]
fn no_abandonment_recorded_when_stopping_during_break() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    // Simulate: 1 work phase completed, stopped during break (no increment_abandoned).
    pomodoro_store::increment_completed(&conn, &date, 25, 0).unwrap();
    // Do NOT call increment_abandoned since we stopped during break.
    let stats = pomodoro_store::get_stats_for_date(&conn, &date).unwrap();
    assert_eq!(stats.abandoned, 0);
    assert_eq!(stats.completed, 1);
}

#[test]
fn completed_and_abandoned_counts_are_independent() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    pomodoro_store::increment_completed(&conn, &date, 25, 0).unwrap();
    pomodoro_store::increment_completed(&conn, &date, 25, 0).unwrap();
    pomodoro_store::increment_abandoned(&conn, &date).unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, &date).unwrap();
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.abandoned, 1);
}
