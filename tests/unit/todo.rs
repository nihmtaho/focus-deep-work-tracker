// TODO unit tests - TDD-first approach
// All tests should be written first and fail before implementation

#[cfg(test)]
mod tests {
    use focus::db;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> rusqlite::Connection {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();
        let db = db::open_db_at(path).expect("Failed to open test DB");
        db
    }

    // T008: Test for todo::insert() with validation
    #[test]
    fn test_insert_todo_creates_active_todo() {
        let db = setup_test_db();
        let result = focus::models::todo::insert(&db, "Implement feature X");
        assert!(result.is_ok());
        let todo = result.unwrap();
        assert_eq!(todo.status, "active");
        assert!(todo.completed_at.is_none());
        assert!(!todo.title.is_empty());
    }

    #[test]
    fn test_insert_empty_title_fails() {
        let db = setup_test_db();
        let result = focus::models::todo::insert(&db, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_long_title_fails() {
        let db = setup_test_db();
        let long_title = "a".repeat(257);
        let result = focus::models::todo::insert(&db, &long_title);
        assert!(result.is_err());
    }

    // T009: Test for todo::update_status()
    #[test]
    fn test_update_status_marks_completed() {
        let db = setup_test_db();
        let todo = focus::models::todo::insert(&db, "Test task").unwrap();
        let updated = focus::models::todo::update_status(&db, todo.id, "completed").unwrap();
        assert_eq!(updated.status, "completed");
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_update_status_invalid_status_fails() {
        let db = setup_test_db();
        let todo = focus::models::todo::insert(&db, "Test task").unwrap();
        let result = focus::models::todo::update_status(&db, todo.id, "invalid");
        assert!(result.is_err());
    }

    // T010: Test for todo::delete()
    #[test]
    fn test_delete_todo_succeeds() {
        let db = setup_test_db();
        let todo = focus::models::todo::insert(&db, "Test task").unwrap();
        let result = focus::models::todo::delete(&db, todo.id);
        assert!(result.is_ok());

        // Verify deletion
        let todos = focus::models::todo::list_all(&db).unwrap();
        assert!(todos.iter().all(|t| t.id != todo.id));
    }

    // T011: Test for todo::list_all()
    #[test]
    fn test_list_all_returns_todos() {
        let db = setup_test_db();
        let _t1 = focus::models::todo::insert(&db, "Task 1").unwrap();
        let t2 = focus::models::todo::insert(&db, "Task 2").unwrap();
        let _t3 = focus::models::todo::insert(&db, "Task 3").unwrap();

        // Mark one as completed
        let _ = focus::models::todo::update_status(&db, t2.id, "completed");

        let todos = focus::models::todo::list_all(&db).unwrap();
        assert_eq!(todos.len(), 3);

        // Active TODOs should come first
        assert_eq!(todos[0].status, "active");
        assert_eq!(todos[1].status, "active");
        assert_eq!(todos[2].status, "completed");
    }
}
