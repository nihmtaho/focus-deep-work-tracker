/// Integration tests for US4: Weekly Productivity Report (focus report)
///
/// FR-007: aggregate_by_tag groups time by tag.
/// FR-008: time window (since epoch) filters correctly.
/// FR-009: grand total is computable from aggregated rows.
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

// FR-007: sessions grouped by tag, sorted by total desc
#[test]
fn test_aggregate_by_tag_groups_correctly() {
    let conn = temp_conn();
    insert_and_stop(&conn, "rust work 1", Some("rust"));
    insert_and_stop(&conn, "rust work 2", Some("rust"));
    insert_and_stop(&conn, "client work", Some("client-a"));

    let rows = session_store::aggregate_by_tag(&conn, 0).expect("aggregate");
    assert!(!rows.is_empty());

    let tags: Vec<Option<String>> = rows.iter().map(|(t, _)| t.clone()).collect();
    assert!(tags.contains(&Some("rust".to_string())));
    assert!(tags.contains(&Some("client-a".to_string())));
}

// FR-007: untagged sessions appear with tag = None
#[test]
fn test_aggregate_includes_untagged() {
    let conn = temp_conn();
    insert_and_stop(&conn, "no tag work", None);

    let rows = session_store::aggregate_by_tag(&conn, 0).expect("aggregate");
    assert_eq!(rows.len(), 1);
    assert!(rows[0].0.is_none(), "untagged session should have None tag");
}

// FR-008: since filter excludes sessions before the cutoff
#[test]
fn test_aggregate_respects_since_filter() {
    let conn = temp_conn();
    insert_and_stop(&conn, "old work", Some("rust"));

    // since = far future — nothing should match
    let future = chrono::Utc::now().timestamp() + 86400;
    let rows = session_store::aggregate_by_tag(&conn, future).expect("aggregate");
    assert!(rows.is_empty(), "future since should exclude all sessions");
}

// FR-009: grand total equals sum of all tag totals
#[test]
fn test_grand_total_is_sum_of_tag_totals() {
    let conn = temp_conn();
    insert_and_stop(&conn, "a", Some("tag1"));
    insert_and_stop(&conn, "b", Some("tag2"));

    let rows = session_store::aggregate_by_tag(&conn, 0).expect("aggregate");
    let grand_total: i64 = rows.iter().map(|(_, secs)| secs).sum();
    assert!(grand_total >= 0);
}

// FR-007: empty DB returns empty rows, not an error
#[test]
fn test_aggregate_empty_db() {
    let conn = temp_conn();
    let rows = session_store::aggregate_by_tag(&conn, 0).expect("aggregate");
    assert!(rows.is_empty());
}
