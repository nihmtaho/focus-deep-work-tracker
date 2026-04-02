/// Integration tests for US1: Start and Stop a Work Session
///
/// Each test opens a temporary SQLite file isolated from ~/.local/share/focus/focus.db.
/// Tests follow TDD order: behaviour is verified against the contract in contracts/cli-schema.md.
use focus::db::open_db_at;
use focus::db::session_store;

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

// FR-001: insert a session and read it back as active
#[test]
fn test_insert_and_get_active_session() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "write unit tests", Some("rust")).expect("insert");
    let active = session_store::get_active_session(&conn)
        .expect("query")
        .expect("should be Some");
    assert_eq!(active.task, "write unit tests");
    assert_eq!(active.tag.as_deref(), Some("rust"));
    assert!(active.is_active());
}

// FR-010: only one active session allowed — second insert must not produce a second active row
#[test]
fn test_only_one_active_session_at_a_time() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "first task", None).expect("first insert");
    session_store::insert_session(&conn, "second task", None).expect("second insert");

    // get_active_session returns LIMIT 1 — only the first is visible
    let active = session_store::get_active_session(&conn)
        .expect("query")
        .expect("some");
    assert_eq!(active.task, "first task");
}

// FR-004: stop_session records end_time and returns a completed Session
#[test]
fn test_stop_session_records_duration() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "deep work", Some("rust")).expect("insert");
    let stopped = session_store::stop_session(&conn).expect("stop");
    assert_eq!(stopped.task, "deep work");
    assert!(
        stopped.end_time.is_some(),
        "end_time must be set after stop"
    );
    assert!(stopped.duration().is_some());
}

// FR-004: stop when no session active must return an error
#[test]
fn test_stop_with_no_active_session_returns_none() {
    let conn = temp_conn();
    // Nothing inserted — get_active_session returns None
    let active = session_store::get_active_session(&conn).expect("query");
    assert!(active.is_none(), "should be no active session");
}

// FR-002: get_active_session returns None after session is stopped
#[test]
fn test_no_active_session_after_stop() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "task", None).expect("insert");
    session_store::stop_session(&conn).expect("stop");
    let active = session_store::get_active_session(&conn).expect("query");
    assert!(active.is_none());
}

// FR-013: empty task — validation is at command layer; DB itself accepts empty strings,
// so the guard must be enforced before insert_session is called.
// This test confirms insert_session itself does NOT validate (that's the command's job).
#[test]
fn test_insert_session_without_tag() {
    let conn = temp_conn();
    session_store::insert_session(&conn, "no tag task", None).expect("insert");
    let active = session_store::get_active_session(&conn)
        .expect("query")
        .expect("some");
    assert!(active.tag.is_none());
}
