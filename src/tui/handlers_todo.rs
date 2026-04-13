// TODO management event handlers for dashboard
use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::models::todo;
use crate::tui::app::App;
use crate::tui::text_input::TextInputEvent;

/// Handle TODO-related keyboard input.
pub fn handle_todo_key(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    if app.todo_input_mode {
        match app.todo_input.handle_key(key) {
            TextInputEvent::Submit(text) if !text.is_empty() => {
                todo::insert(db, &text)?;
                app.exit_todo_input_mode();
                app.load_todos(db)?;
            }
            TextInputEvent::Submit(_) | TextInputEvent::Cancel => {
                app.exit_todo_input_mode();
            }
            TextInputEvent::Continue => {}
        }
        return Ok(());
    }

    // ── Viewing mode keys ──────────────────────────────────────────────────────
    match key {
        KeyCode::Char('a') => {
            app.enter_todo_input_mode();
        }
        KeyCode::Delete | KeyCode::Backspace if app.selected_todo_idx.is_some() => {
            if let Some(idx) = app.selected_todo_idx {
                if idx < app.todos.len() {
                    let todo_id = app.todos[idx].id;
                    if todo::can_delete(db, todo_id)? {
                        todo::delete(db, todo_id)?;
                        app.load_todos(db)?;
                        app.selected_todo_idx = if app.todos.is_empty() {
                            None
                        } else {
                            Some(idx.min(app.todos.len().saturating_sub(1)))
                        };
                        app.message =
                            Some(crate::tui::app::MessageOverlay::success("Todo deleted."));
                    } else {
                        app.message = Some(crate::tui::app::MessageOverlay::error(
                            "Cannot delete TODO linked to active session",
                        ));
                    }
                }
            }
        }
        KeyCode::Char('c') if app.selected_todo_idx.is_some() => {
            if let Some(idx) = app.selected_todo_idx {
                if idx < app.todos.len() {
                    let todo_id = app.todos[idx].id;
                    todo::update_status(db, todo_id, "completed")?;
                    app.load_todos(db)?;
                }
            }
        }
        KeyCode::Char('s') => {
            app.overlay = crate::tui::app::Overlay::ModeSelector { cursor: 0 };
        }
        KeyCode::Right if app.selected_todo_idx.is_some() => {
            app.overlay = crate::tui::app::Overlay::ModeSelector { cursor: 0 };
        }
        KeyCode::Up if !app.todos.is_empty() => match app.selected_todo_idx {
            None => app.selected_todo_idx = Some(0),
            Some(i) if i > 0 => app.selected_todo_idx = Some(i - 1),
            _ => {}
        },
        KeyCode::Down if !app.todos.is_empty() => {
            let len = app.todos.len();
            match app.selected_todo_idx {
                None => app.selected_todo_idx = Some(0),
                Some(i) if i + 1 < len => app.selected_todo_idx = Some(i + 1),
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

