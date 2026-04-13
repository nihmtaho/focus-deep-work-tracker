// TODO management event handlers for dashboard
use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::models::todo;
use crate::tui::app::{App, VimInputMode};

/// Handle TODO-related keyboard input.
pub fn handle_todo_key(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    if app.todo_input_mode {
        if app.config.vim_mode {
            handle_vim_input(app, db, key)?;
        } else {
            handle_simple_input(app, db, key)?;
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

// ── Simple (non-vim) input ─────────────────────────────────────────────────────

fn handle_simple_input(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Enter => {
            let text = app.todo_input_buffer.trim().to_string();
            if !text.is_empty() {
                todo::insert(db, &text)?;
                app.exit_todo_input_mode();
                app.load_todos(db)?;
            }
        }
        KeyCode::Esc => {
            app.exit_todo_input_mode();
        }
        KeyCode::Char(c) => {
            if app.todo_input_buffer.len() < 256 {
                let pos = app.todo_cursor_pos;
                app.todo_input_buffer.insert(pos, c);
                app.todo_cursor_pos += c.len_utf8();
            }
        }
        KeyCode::Backspace => {
            buf_backspace(app);
        }
        KeyCode::Delete => {
            buf_delete_at_cursor(app);
        }
        KeyCode::Left => {
            buf_move_left(app);
        }
        KeyCode::Right => {
            buf_move_right(app);
        }
        _ => {}
    }
    Ok(())
}

// ── Vim-aware input ────────────────────────────────────────────────────────────

fn handle_vim_input(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    match app.vim_input_mode {
        VimInputMode::Insert => handle_vim_insert(app, db, key),
        VimInputMode::Normal => handle_vim_normal(app, db, key),
    }
}

/// Insert mode: type normally, Esc → Normal mode.
fn handle_vim_insert(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    match key {
        KeyCode::Esc => {
            // Back to Normal; clamp cursor so it's on a char (vim convention)
            app.vim_input_mode = VimInputMode::Normal;
            buf_clamp_normal(app);
        }
        KeyCode::Enter => {
            let text = app.todo_input_buffer.trim().to_string();
            if !text.is_empty() {
                todo::insert(db, &text)?;
                app.exit_todo_input_mode();
                app.load_todos(db)?;
            }
        }
        KeyCode::Char(c) => {
            if app.todo_input_buffer.len() < 256 {
                let pos = app.todo_cursor_pos;
                app.todo_input_buffer.insert(pos, c);
                app.todo_cursor_pos += c.len_utf8();
            }
        }
        KeyCode::Backspace => {
            buf_backspace(app);
        }
        KeyCode::Delete => {
            buf_delete_at_cursor(app);
        }
        KeyCode::Left => {
            buf_move_left(app);
        }
        KeyCode::Right => {
            buf_move_right(app);
        }
        _ => {}
    }
    Ok(())
}

/// Normal mode: motion and operator keys; 'i'/'a' → Insert, Esc → cancel.
fn handle_vim_normal(app: &mut App, db: &Connection, key: KeyCode) -> anyhow::Result<()> {
    match key {
        // Enter Insert mode
        KeyCode::Char('i') => {
            app.vim_input_mode = VimInputMode::Insert;
        }
        KeyCode::Char('a') => {
            // Insert after cursor
            buf_move_right(app);
            app.vim_input_mode = VimInputMode::Insert;
        }
        KeyCode::Char('A') => {
            // Insert at end of line
            app.todo_cursor_pos = app.todo_input_buffer.len();
            app.vim_input_mode = VimInputMode::Insert;
        }
        KeyCode::Char('I') => {
            // Insert at beginning
            app.todo_cursor_pos = 0;
            app.vim_input_mode = VimInputMode::Insert;
        }
        // Submit
        KeyCode::Enter => {
            let text = app.todo_input_buffer.trim().to_string();
            if !text.is_empty() {
                todo::insert(db, &text)?;
                app.exit_todo_input_mode();
                app.load_todos(db)?;
            }
        }
        // Cancel
        KeyCode::Esc => {
            app.exit_todo_input_mode();
        }
        // Motions
        KeyCode::Char('h') | KeyCode::Left => {
            buf_move_left(app);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            buf_move_right_normal(app);
        }
        KeyCode::Char('0') | KeyCode::Home => {
            app.todo_cursor_pos = 0;
        }
        KeyCode::Char('$') | KeyCode::End => {
            buf_end_normal(app);
        }
        KeyCode::Char('w') => {
            buf_word_forward(app);
        }
        KeyCode::Char('b') => {
            buf_word_backward(app);
        }
        // Operators
        KeyCode::Char('x') => {
            buf_delete_at_cursor(app);
            buf_clamp_normal(app);
        }
        KeyCode::Char('D') => {
            // Delete to end of line
            let pos = app.todo_cursor_pos;
            app.todo_input_buffer.truncate(pos);
            buf_clamp_normal(app);
        }
        KeyCode::Char('d') => {
            // 'dd' would be handled by keyboard_handler; here treat 'd' alone as no-op
        }
        _ => {}
    }
    Ok(())
}

// ── Buffer helpers ─────────────────────────────────────────────────────────────

/// Move cursor one grapheme left (respects UTF-8 boundaries).
fn buf_move_left(app: &mut App) {
    if app.todo_cursor_pos == 0 {
        return;
    }
    let s = &app.todo_input_buffer;
    // Step back one char boundary
    let mut pos = app.todo_cursor_pos - 1;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    app.todo_cursor_pos = pos;
}

/// Move cursor one grapheme right (Insert mode: may go past last char).
fn buf_move_right(app: &mut App) {
    let s = &app.todo_input_buffer;
    let len = s.len();
    if app.todo_cursor_pos >= len {
        return;
    }
    let mut pos = app.todo_cursor_pos + 1;
    while pos <= len && !s.is_char_boundary(pos) {
        pos += 1;
    }
    app.todo_cursor_pos = pos;
}

/// Move cursor one grapheme right (Normal mode: stop at last char, not after).
fn buf_move_right_normal(app: &mut App) {
    let s = &app.todo_input_buffer;
    let len = s.len();
    if s.is_empty() || app.todo_cursor_pos >= len.saturating_sub(1) {
        return;
    }
    buf_move_right(app);
}

/// In Normal mode cursor must rest on a valid char; clamp if needed.
fn buf_clamp_normal(app: &mut App) {
    let s = &app.todo_input_buffer;
    if s.is_empty() {
        app.todo_cursor_pos = 0;
        return;
    }
    // Cursor cannot be past last char in Normal mode
    let last_char_start = s
        .char_indices()
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    if app.todo_cursor_pos > last_char_start {
        app.todo_cursor_pos = last_char_start;
    }
    // Ensure it's on a boundary
    while !s.is_char_boundary(app.todo_cursor_pos) && app.todo_cursor_pos > 0 {
        app.todo_cursor_pos -= 1;
    }
}

/// Delete the char before the cursor (Backspace behaviour).
fn buf_backspace(app: &mut App) {
    if app.todo_cursor_pos == 0 {
        return;
    }
    buf_move_left(app);
    let pos = app.todo_cursor_pos;
    let ch = app.todo_input_buffer[pos..].chars().next().unwrap();
    app.todo_input_buffer.remove(pos);
    let _ = ch; // char removed, cursor already at right position
}

/// Delete the char at the cursor (Delete / 'x' behaviour).
fn buf_delete_at_cursor(app: &mut App) {
    let pos = app.todo_cursor_pos;
    let s = &app.todo_input_buffer;
    if pos >= s.len() {
        return;
    }
    app.todo_input_buffer.remove(pos);
    // Clamp if now past end
    let new_len = app.todo_input_buffer.len();
    if app.todo_cursor_pos > new_len {
        app.todo_cursor_pos = new_len;
    }
}

/// Place cursor on last char (Normal mode convention for '$').
fn buf_end_normal(app: &mut App) {
    let s = &app.todo_input_buffer;
    if s.is_empty() {
        app.todo_cursor_pos = 0;
        return;
    }
    app.todo_cursor_pos = s.char_indices().last().map(|(i, _)| i).unwrap_or(0);
}

/// Jump to start of next word.
fn buf_word_forward(app: &mut App) {
    let s = &app.todo_input_buffer;
    let mut pos = app.todo_cursor_pos;
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let idx = chars.iter().position(|&(i, _)| i == pos).unwrap_or(0);
    // Skip current word chars
    let mut i = idx;
    while i < chars.len() && !chars[i].1.is_whitespace() {
        i += 1;
    }
    // Skip whitespace
    while i < chars.len() && chars[i].1.is_whitespace() {
        i += 1;
    }
    pos = chars.get(i).map(|&(j, _)| j).unwrap_or(s.len());
    app.todo_cursor_pos = pos;
    buf_clamp_normal(app);
}

/// Jump to start of previous word.
fn buf_word_backward(app: &mut App) {
    let s = &app.todo_input_buffer;
    if app.todo_cursor_pos == 0 {
        return;
    }
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let idx = chars
        .iter()
        .position(|&(i, _)| i >= app.todo_cursor_pos)
        .unwrap_or(chars.len())
        .saturating_sub(1);
    let mut i = idx;
    // Skip whitespace backwards
    while i > 0 && chars[i].1.is_whitespace() {
        i -= 1;
    }
    // Skip word chars backwards
    while i > 0 && !chars[i - 1].1.is_whitespace() {
        i -= 1;
    }
    app.todo_cursor_pos = chars.get(i).map(|&(j, _)| j).unwrap_or(0);
}

