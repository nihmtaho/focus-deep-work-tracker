/// Integration tests for US5: Export Session Data (focus export)
///
/// FR-015: list_all_completed returns sessions in start_time ASC order.
/// FR-015: export_json produces valid JSON; export_markdown produces valid Markdown.
/// FR-015: empty session list produces valid empty structures.
use focus::commands::export::{export_json, export_markdown};
use focus::db::open_db_at;
use focus::db::session_store;

fn temp_conn() -> rusqlite::Connection {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("test.db");
    open_db_at(&path).expect("open temp db")
}

fn insert_and_stop(conn: &rusqlite::Connection, task: &str, tag: Option<&str>) {
    session_store::insert_session(conn, task, tag).expect("insert");
    session_store::stop_session(conn).expect("stop");
}

// FR-015: list_all_completed returns ASC order by start_time
#[test]
fn test_list_all_completed_asc_order() {
    let conn = temp_conn();
    insert_and_stop(&conn, "first", None);
    insert_and_stop(&conn, "second", None);
    insert_and_stop(&conn, "third", None);

    let sessions = session_store::list_all_completed(&conn).expect("list");
    assert_eq!(sessions.len(), 3);
    assert_eq!(sessions[0].task, "first");
    assert_eq!(sessions[2].task, "third");
}

// FR-015: active session is excluded from export
#[test]
fn test_list_all_completed_excludes_active() {
    let conn = temp_conn();
    insert_and_stop(&conn, "done", None);
    session_store::insert_session(&conn, "still running", None).expect("insert active");

    let sessions = session_store::list_all_completed(&conn).expect("list");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].task, "done");
}

// FR-015: export_json produces valid JSON array
#[test]
fn test_export_json_valid_structure() {
    let conn = temp_conn();
    insert_and_stop(&conn, "json task", Some("rust"));
    let sessions = session_store::list_all_completed(&conn).expect("list");

    let json = export_json(&sessions);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["task"], "json task");
    assert_eq!(arr[0]["tag"], "rust");
    assert!(arr[0]["start_time"].is_string());
    assert!(arr[0]["end_time"].is_string());
    assert!(arr[0]["duration_seconds"].is_number());
}

// FR-015: export_json with no sessions returns "[]"
#[test]
fn test_export_json_empty() {
    let json = export_json(&[]);
    assert_eq!(json.trim(), "[]");
}

// FR-015: export_markdown produces a table with header and separator
#[test]
fn test_export_markdown_contains_header() {
    let conn = temp_conn();
    insert_and_stop(&conn, "md task", None);
    let sessions = session_store::list_all_completed(&conn).expect("list");

    let md = export_markdown(&sessions);
    assert!(md.contains("| Date |"), "must contain Date column header");
    assert!(md.contains("| Task |"), "must contain Task column header");
    assert!(md.contains("md task"), "must contain session task");
}

// FR-015: export_markdown with no sessions returns table with headers only
#[test]
fn test_export_markdown_empty_has_headers() {
    let md = export_markdown(&[]);
    assert!(md.contains("| Date |"));
    assert!(md.contains("|---"));
    // No data rows beyond the header and separator
    let data_rows = md
        .lines()
        .filter(|l| l.starts_with('|') && !l.contains("Date") && !l.contains("---"))
        .count();
    assert_eq!(data_rows, 0);
}
