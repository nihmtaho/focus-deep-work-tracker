// TODO persistence tests - Integration tests for TODO storage and session linking

#[cfg(test)]
mod tests {
    use focus::db;
    use focus::models::todo;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> rusqlite::Connection {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();
        let db = db::open_db_at(path).expect("Failed to open test DB");
        db
    }

    // T016: Test that todos table is created by migration
    #[test]
    fn test_migration_creates_todos_table() {
        let db = setup_test_db();

        // Try to insert a TODO - this will fail if the table doesn't exist
        let result = todo::insert(&db, "Test TODO");
        assert!(result.is_ok());
    }

    // T017: Test that sessions table has todo_id column after migration
    #[test]
    fn test_migration_adds_todo_id_to_sessions() {
        let db = setup_test_db();

        // Create a session with todo_id
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Insert a TODO first
        let todo = todo::insert(&db, "Test TODO").unwrap();

        // Insert a session with the todo_id
        db.execute(
            "INSERT INTO sessions (task, tag, start_time, mode, todo_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["Test task", None::<String>, now, "freeform", todo.id as i64],
        ).expect("Failed to insert session with todo_id");

        // Verify we can read it back
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM sessions WHERE todo_id IS NOT NULL",
            [],
            |row| row.get(0),
        ).expect("Failed to query");

        assert_eq!(count, 1);
    }

    // T039: Test TODO persistence across app operations
    #[test]
    fn test_todo_persistence_across_operations() {
        let db = setup_test_db();

        // Create multiple TODOs
        let t1 = todo::insert(&db, "Task 1").unwrap();
        let _t2 = todo::insert(&db, "Task 2").unwrap();

        // Mark one as completed
        let _ = todo::update_status(&db, t1.id, "completed");

        // List all and verify persistence
        let todos = todo::list_all(&db).unwrap();
        assert_eq!(todos.len(), 2);

        let completed = todos.iter().find(|t| t.status == "completed");
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().id, t1.id);
    }

    // T050: Test persistence reliability over multiple operations
    #[test]
    fn test_todo_persistence_reliability() {
        let db = setup_test_db();

        // Create 10 TODOs
        for i in 1..=10 {
            let title = format!("Task {}", i);
            let todo = todo::insert(&db, &title).unwrap();

            // Randomly mark some as completed
            if i % 3 == 0 {
                let _ = todo::update_status(&db, todo.id, "completed");
            }
        }

        // Verify all persist
        let todos = todo::list_all(&db).unwrap();
        assert_eq!(todos.len(), 10);

        let completed_count = todos.iter().filter(|t| t.status == "completed").count();
        assert!(completed_count > 0);
    }
}
