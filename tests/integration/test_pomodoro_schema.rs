// Integration tests for the pomodoro DB schema and store (T008, T024).
use focus::db::{open_db_at, pomodoro_store};

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

// T008: sessions table has mode column with default 'freeform'
#[test]
fn sessions_table_has_mode_column() {
    let conn = temp_conn();
    let col_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name='mode'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
        > 0;
    assert!(col_exists, "sessions table must have mode column");
}

#[test]
fn pomodoro_stats_table_exists() {
    let conn = temp_conn();
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='pomodoro_stats'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
        > 0;
    assert!(exists, "pomodoro_stats table must exist");
}

#[test]
fn existing_sessions_get_freeform_mode() {
    let conn = temp_conn();
    // Insert a row using raw SQL (simulating pre-migration row).
    conn.execute(
        "INSERT INTO sessions (task, start_time) VALUES ('old task', 1000)",
        [],
    )
    .unwrap();
    let mode: String = conn
        .query_row(
            "SELECT mode FROM sessions WHERE task = 'old task'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(mode, "freeform");
}

// T024: pomodoro_store CRUD
#[test]
fn increment_completed_upserts_row() {
    let conn = temp_conn();
    pomodoro_store::increment_completed(&conn, "2026-04-06", 25, 5).unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, "2026-04-06").unwrap();
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.work_minutes, 25);
    assert_eq!(stats.break_minutes, 5);
}

#[test]
fn increment_completed_accumulates() {
    let conn = temp_conn();
    pomodoro_store::increment_completed(&conn, "2026-04-06", 25, 0).unwrap();
    pomodoro_store::increment_completed(&conn, "2026-04-06", 25, 0).unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, "2026-04-06").unwrap();
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.work_minutes, 50);
}

#[test]
fn increment_abandoned_increments_field() {
    let conn = temp_conn();
    pomodoro_store::increment_abandoned(&conn, "2026-04-06").unwrap();
    let stats = pomodoro_store::get_stats_for_date(&conn, "2026-04-06").unwrap();
    assert_eq!(stats.abandoned, 1);
}

#[test]
fn get_stats_for_date_empty_returns_zeroed() {
    let conn = temp_conn();
    let stats = pomodoro_store::get_stats_for_date(&conn, "2026-01-01").unwrap();
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.abandoned, 0);
    assert_eq!(stats.work_minutes, 0);
    assert_eq!(stats.break_minutes, 0);
}

#[test]
fn get_stats_range_returns_all_rows_asc() {
    let conn = temp_conn();
    pomodoro_store::increment_completed(&conn, "2026-04-04", 25, 0).unwrap();
    pomodoro_store::increment_completed(&conn, "2026-04-05", 25, 0).unwrap();
    pomodoro_store::increment_completed(&conn, "2026-04-06", 25, 0).unwrap();
    let rows = pomodoro_store::get_stats_range(&conn, "2026-04-04", "2026-04-06").unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].date, "2026-04-04");
    assert_eq!(rows[2].date, "2026-04-06");
}

#[test]
fn calculate_streak_three_consecutive_days() {
    // Build stats with today, yesterday, day-before-yesterday all having completed > 0.
    use chrono::Local;
    let today = Local::now().date_naive();
    let mut stats = Vec::new();
    for i in (0..3).rev() {
        let d = (today - chrono::Duration::days(i))
            .format("%Y-%m-%d")
            .to_string();
        stats.push(focus::models::pomodoro::PomodoroStats {
            date: d,
            completed: 2,
            ..Default::default()
        });
    }
    let streak = pomodoro_store::calculate_streak(&stats);
    assert_eq!(streak, 3);
}

#[test]
fn calculate_streak_gap_resets() {
    use chrono::Local;
    let today = Local::now().date_naive();
    // today and 2 days ago but NOT yesterday → streak should be 1 (only today).
    let day_before_yesterday = (today - chrono::Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let today_str = today.format("%Y-%m-%d").to_string();
    let stats = vec![
        focus::models::pomodoro::PomodoroStats {
            date: day_before_yesterday,
            completed: 2,
            ..Default::default()
        },
        focus::models::pomodoro::PomodoroStats {
            date: today_str,
            completed: 2,
            ..Default::default()
        },
    ];
    let streak = pomodoro_store::calculate_streak(&stats);
    assert_eq!(streak, 1);
}

#[test]
fn calculate_best_streak_finds_maximum() {
    let stats = vec![
        focus::models::pomodoro::PomodoroStats {
            date: "2026-04-01".into(),
            completed: 2,
            ..Default::default()
        },
        focus::models::pomodoro::PomodoroStats {
            date: "2026-04-02".into(),
            completed: 2,
            ..Default::default()
        },
        focus::models::pomodoro::PomodoroStats {
            date: "2026-04-03".into(),
            completed: 0, // gap
            ..Default::default()
        },
        focus::models::pomodoro::PomodoroStats {
            date: "2026-04-04".into(),
            completed: 3,
            ..Default::default()
        },
    ];
    let best = pomodoro_store::calculate_best_streak(&stats);
    assert_eq!(best, 2);
}
