// TODO management event handlers for dashboard
use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::models::todo;
use crate::tui::app::App;

/// Handle TODO-related keyboard input.
pub fn handle_todo_key(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Char('a') if !app.todo_input_mode => {
            // Start adding TODO
            app.todo_input_mode = true;
            app.todo_input_buffer.clear();
        }
        KeyCode::Char('d') if !app.todo_input_mode && app.selected_todo_idx.is_some() => {
            // Delete selected TODO
            if let Some(idx) = app.selected_todo_idx {
                if idx < app.todos.len() {
                    let todo_id = app.todos[idx].id;
                    // Check if can delete (not linked to active session)
                    if todo::can_delete(db, todo_id)? {
                        todo::delete(db, todo_id)?;
                        app.load_todos(db)?;
                        app.selected_todo_idx = None;
                    } else {
                        app.message = Some(crate::tui::app::MessageOverlay::error(
                            "Cannot delete TODO linked to active session",
                        ));
                    }
                }
            }
        }
        KeyCode::Char('c') if !app.todo_input_mode && app.selected_todo_idx.is_some() => {
            // Mark selected TODO as complete
            if let Some(idx) = app.selected_todo_idx {
                if idx < app.todos.len() {
                    let todo_id = app.todos[idx].id;
                    todo::update_status(db, todo_id, "completed")?;
                    app.load_todos(db)?;
                }
            }
        }
        KeyCode::Char('s') if !app.todo_input_mode => {
            // Open mode selector to choose Freeform or Pomodoro
            // The selected_todo_idx will be used when creating the session
            app.overlay = crate::tui::app::Overlay::ModeSelector {
                cursor: 0,
            };
        }
        KeyCode::Enter if app.todo_input_mode => {
            // Confirm TODO add
            if !app.todo_input_buffer.is_empty() {
                todo::insert(db, &app.todo_input_buffer)?;
                app.todo_input_buffer.clear();
                app.todo_input_mode = false;
                app.load_todos(db)?;
            }
        }
        KeyCode::Esc => {
            app.todo_input_mode = false;
            app.todo_input_buffer.clear();
        }
        KeyCode::Up if !app.todo_input_mode && !app.todos.is_empty() => {
            // Navigate up in TODO list
            match app.selected_todo_idx {
                None => app.selected_todo_idx = Some(0),
                Some(idx) if idx > 0 => app.selected_todo_idx = Some(idx - 1),
                _ => {} // Stay at top
            }
        }
        KeyCode::Down if !app.todo_input_mode && !app.todos.is_empty() => {
            // Navigate down in TODO list
            let len = app.todos.len();
            match app.selected_todo_idx {
                None => app.selected_todo_idx = Some(0),
                Some(idx) if idx < len - 1 => app.selected_todo_idx = Some(idx + 1),
                _ => {} // Stay at bottom
            }
        }
        KeyCode::Char(c) if app.todo_input_mode => {
            // Accumulate characters for TODO input
            if app.todo_input_buffer.len() < 256 {
                app.todo_input_buffer.push(c);
            }
        }
        KeyCode::Backspace if app.todo_input_mode => {
            app.todo_input_buffer.pop();
        }
        _ => {}
    }

    Ok(())
}
