// Placeholder — CLI Pomodoro integration tests (T027).
// Full tests require the CLI runner which is implemented in Phase 3.
// These tests validate DB-level behaviour that the CLI runner produces.

use focus::db::{open_db_at, pomodoro_store, session_store};

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

#[test]
fn validation_work_zero_returns_error() {
    let cfg = focus::pomodoro::config::PomodoroConfig {
        work_duration_mins: 0,
        ..focus::pomodoro::config::PomodoroConfig::default()
    };
    assert!(cfg.validate().is_err());
    let msg = cfg.validate().unwrap_err().to_string();
    assert!(msg.contains("work"), "error message should mention 'work'");
}

#[test]
fn validation_work_too_large_returns_error() {
    let cfg = focus::pomodoro::config::PomodoroConfig {
        work_duration_mins: 121,
        ..focus::pomodoro::config::PomodoroConfig::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn validation_break_too_large_returns_error() {
    let cfg = focus::pomodoro::config::PomodoroConfig {
        break_duration_mins: 61,
        ..focus::pomodoro::config::PomodoroConfig::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn pomodoro_session_saved_with_correct_mode() {
    let conn = temp_conn();
    // Simulate what the timer does: insert a completed pomodoro session.
    let now = chrono::Utc::now().timestamp();
    session_store::insert_session_with_times(&conn, "refactor", None, "pomodoro", now - 60, now)
        .unwrap();
    let sessions = session_store::list_sessions(&conn, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].mode, "pomodoro");
    assert_eq!(sessions[0].task, "refactor");
}

#[test]
fn freeform_session_has_freeform_mode() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "freeform task", None).unwrap();
    session_store::stop_session(&conn).unwrap();
    let sessions = session_store::list_sessions(&conn, 10).unwrap();
    assert_eq!(sessions[0].mode, "freeform");
}

#[test]
fn focus_stop_increments_abandoned_stat_when_mid_work() {
    let conn = temp_conn();
    let date = pomodoro_store::today_local_date();
    // Simulate 1 completed + 1 abandoned
    pomodoro_store::increment_completed(&conn, &date, 25, 0).unwrap();
    pomodoro_store::increment_abandoned(&conn, &date).unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, &date).unwrap();
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.abandoned, 1);
}
