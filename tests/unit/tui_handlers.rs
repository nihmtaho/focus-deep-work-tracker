// TUI event handlers for TODO management - TDD first
#[cfg(test)]
mod tests {
    use focus::config::AppConfig;
    use focus::models::todo;
    use focus::tui::app::App;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> rusqlite::Connection {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();
        let db = focus::db::open_db_at(path).expect("Failed to open test DB");
        db
    }

    fn create_test_app() -> App {
        let config = AppConfig::default();
        App::new(false, config)
    }

    // T036: Test that 'a' key opens the AddTodo prompt overlay
    #[test]
    fn test_add_todo_hotkey_opens_prompt() {
        use crossterm::event::KeyCode;
        use focus::tui::app::{Overlay, PromptAction};
        use focus::tui::handlers_todo::handle_todo_key;

        let db = setup_test_db();
        let mut app = create_test_app();

        assert!(matches!(app.overlay, Overlay::None));
        handle_todo_key(&mut app, &db, KeyCode::Char('a')).unwrap();
        assert!(matches!(
            app.overlay,
            Overlay::Prompt { action: PromptAction::AddTodo, .. }
        ));
    }

    // T037: Test handle_mark_complete() on 'c' key
    #[test]
    fn test_mark_complete_hotkey_completes_todo() {
        let db = setup_test_db();
        let app = create_test_app();

        // Create a TODO first
        let todo = todo::insert(&db, "Test task").expect("Failed to insert TODO");
        let mut app = app;
        app.todos.push(todo.clone());
        app.selected_todo_idx = Some(0);

        // Simulate pressing 'c' key to mark complete
        // Expected: app.todos[0].status should change to "completed"
    }

    // T038: Test handle_delete_todo() on 'd' key
    #[test]
    fn test_delete_todo_hotkey_deletes_todo() {
        let db = setup_test_db();
        let app = create_test_app();

        // Create a TODO
        let todo = todo::insert(&db, "Test task").expect("Failed to insert TODO");
        let mut app = app;
        app.todos.push(todo.clone());
        app.selected_todo_idx = Some(0);

        // Simulate pressing 'd' key
        // Expected: TODO should be deleted from both DB and app.todos
    }

    // Additional: Test navigation with arrow keys
    #[test]
    fn test_navigate_todos_with_arrow_keys() {
        let _db = setup_test_db();
        let mut app = create_test_app();

        // Add some TODOs to app state
        let todo1 = focus::models::todo::Todo {
            id: 1,
            title: "Task 1".to_string(),
            status: "active".to_string(),
            created_at: 0,
            completed_at: None,
        };
        let todo2 = focus::models::todo::Todo {
            id: 2,
            title: "Task 2".to_string(),
            status: "active".to_string(),
            created_at: 0,
            completed_at: None,
        };

        app.todos.push(todo1);
        app.todos.push(todo2);
        app.selected_todo_idx = Some(0);

        // Simulate Down arrow: should move to index 1
        // Simulate Up arrow: should move back to index 0
    }
}
