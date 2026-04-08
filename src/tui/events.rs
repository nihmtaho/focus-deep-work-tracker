use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::save_config;
use crate::db::{pomodoro_store, session_store};
use crate::display::format::format_elapsed;
use crate::error::FocusError;
use crate::pomodoro::config::PomodoroConfig;
use crate::pomodoro::timer::PomodoroTimer;
use crate::tui::app::{idx_to_window, App, MessageOverlay, Overlay, PromptAction, Tab};

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key_event(app: &mut App, conn: &rusqlite::Connection, key: KeyEvent) -> Result<bool> {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }

    // If overlay active, dispatch to overlay handler
    if app.overlay.is_active() {
        return handle_overlay(app, conn, key);
    }

    // Global keys (no overlay)
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            // If Pomodoro timer is active, let the Pomodoro tab handle Q
            // so it can show the confirm-stop dialog instead of quitting.
            if app.active_tab == Tab::Pomodoro && app.pomodoro_timer.is_some() {
                return handle_pomodoro_tab(app, conn, key);
            }
            return Ok(true);
        }
        KeyCode::Char('?') => {
            app.overlay = Overlay::Help;
            return Ok(false);
        }
        KeyCode::Esc => {
            app.message = None;
            return Ok(false);
        }
        KeyCode::Char('1') => {
            app.active_tab = Tab::Dashboard;
            return Ok(false);
        }
        KeyCode::Char('2') => {
            app.active_tab = Tab::Log;
            app.load_log(conn)?;
            return Ok(false);
        }
        KeyCode::Char('3') => {
            app.active_tab = Tab::Report;
            let window = app.report_window.clone();
            app.load_report(conn, &window)?;
            return Ok(false);
        }
        KeyCode::Char('4') => {
            app.active_tab = Tab::Settings;
            return Ok(false);
        }
        KeyCode::Char('5') => {
            app.active_tab = Tab::Pomodoro;
            return Ok(false);
        }
        KeyCode::Tab => {
            app.active_tab = match app.active_tab {
                Tab::Dashboard => Tab::Log,
                Tab::Log => Tab::Report,
                Tab::Report => Tab::Settings,
                Tab::Settings => Tab::Pomodoro,
                Tab::Pomodoro => Tab::Dashboard,
            };
            // Load data when switching to data tabs
            match app.active_tab {
                Tab::Log => {
                    app.load_log(conn)?;
                }
                Tab::Report => {
                    let window = app.report_window.clone();
                    app.load_report(conn, &window)?;
                }
                _ => {}
            }
            return Ok(false);
        }
        KeyCode::BackTab => {
            app.active_tab = match app.active_tab {
                Tab::Dashboard => Tab::Pomodoro,
                Tab::Log => Tab::Dashboard,
                Tab::Report => Tab::Log,
                Tab::Settings => Tab::Report,
                Tab::Pomodoro => Tab::Settings,
            };
            match app.active_tab {
                Tab::Log => {
                    app.load_log(conn)?;
                }
                Tab::Report => {
                    let window = app.report_window.clone();
                    app.load_report(conn, &window)?;
                }
                _ => {}
            }
            return Ok(false);
        }
        _ => {}
    }

    // Tab-specific dispatch
    let tab = app.active_tab.clone();
    match tab {
        Tab::Dashboard => handle_dashboard_tab(app, conn, key),
        Tab::Log => handle_log_tab(app, conn, key),
        Tab::Report => handle_report_tab(app, conn, key),
        Tab::Settings => handle_settings_tab(app, key),
        Tab::Pomodoro => handle_pomodoro_tab(app, conn, key),
    }
}

// ── Overlay handlers ───────────────────────────────────────────────────────────

fn handle_overlay(app: &mut App, conn: &rusqlite::Connection, key: KeyEvent) -> Result<bool> {
    match &app.overlay.clone() {
        Overlay::Prompt { .. } => handle_overlay_prompt(app, conn, key),
        Overlay::ConfirmDelete { .. } => handle_overlay_confirm(app, conn, key),
        Overlay::Help => {
            // Any key dismisses help
            app.overlay = Overlay::None;
            Ok(false)
        }
        Overlay::ModeSelector { .. } => handle_overlay_mode_selector(app, conn, key),
        Overlay::PomodoroConfirmStop => handle_overlay_pomodoro_confirm_stop(app, conn, key),
        Overlay::None => Ok(false),
    }
}

pub fn handle_overlay_prompt(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    let Overlay::Prompt {
        label,
        value,
        action,
    } = app.overlay.clone()
    else {
        return Ok(false);
    };
    match key.code {
        KeyCode::Esc => {
            app.overlay = Overlay::None;
        }
        KeyCode::Backspace => {
            let mut v = value;
            v.pop();
            app.overlay = Overlay::Prompt {
                label,
                value: v,
                action,
            };
        }
        KeyCode::Enter => {
            let task_name = if value.trim().is_empty() {
                "Untitled Session".to_string()
            } else {
                value.trim().to_string()
            };
            match action {
                PromptAction::StartSession => {
                    // Step 1 confirmed: move to tag prompt (step 2)
                    app.overlay = Overlay::Prompt {
                        label: "Tag (optional, press Enter to skip):".to_string(),
                        value: String::new(),
                        action: PromptAction::StartSessionTag { task: task_name },
                    };
                }
                PromptAction::StartSessionTag { task } => {
                    let tag_opt = if value.trim().is_empty() {
                        None
                    } else {
                        Some(value.trim().to_string())
                    };
                    match session_store::get_active_session(conn)? {
                        Some(existing) => {
                            let elapsed = format_elapsed(existing.start_time);
                            app.message = Some(MessageOverlay::error(
                                FocusError::AlreadyRunning {
                                    task: existing.task,
                                    elapsed,
                                }
                                .to_string(),
                            ));
                        }
                        None => {
                            session_store::insert_session(conn, &task, tag_opt.as_deref())?;
                            app.message = Some(MessageOverlay::success(format!(
                                "Session started: \"{task}\""
                            )));
                            let _ = app.load_dashboard(conn);
                        }
                    }
                    app.overlay = Overlay::None;
                }
                PromptAction::RenameSession { id } => {
                    match session_store::rename_session(conn, id, &task_name) {
                        Ok(()) => {
                            app.message = Some(MessageOverlay::success(format!(
                                "Renamed to \"{task_name}\""
                            )));
                            let sel = app.log_selected;
                            app.load_log(conn)?;
                            // Restore selection
                            let max = app.log_page_entries(app.log_page).len().saturating_sub(1);
                            app.log_selected = sel.min(max);
                        }
                        Err(e) => {
                            app.message = Some(MessageOverlay::error(e.to_string()));
                        }
                    }
                    app.overlay = Overlay::None;
                }
                PromptAction::StartPomodoroName => {
                    // Move to tag step
                    app.overlay = Overlay::Prompt {
                        label: "Pomodoro — tag (optional):".to_string(),
                        value: String::new(),
                        action: PromptAction::StartPomodoroTag { task: task_name },
                    };
                }
                PromptAction::StartPomodoroTag { task } => {
                    let tag_opt = if value.trim().is_empty() {
                        None
                    } else {
                        Some(value.trim().to_string())
                    };
                    let config =
                        PomodoroConfig::resolve(None, None, None, None).unwrap_or_default();
                    let timer = PomodoroTimer::new(task, tag_opt, config);
                    app.pomodoro_timer = Some(timer);
                    app.active_tab = Tab::Pomodoro;
                    app.overlay = Overlay::None;
                    app.message = Some(MessageOverlay::success("Pomodoro started!"));
                }
            }
        }
        KeyCode::Char(c) => {
            let mut v = value;
            v.push(c);
            app.overlay = Overlay::Prompt {
                label,
                value: v,
                action,
            };
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_overlay_confirm(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    let Overlay::ConfirmDelete {
        session_id,
        session_name,
    } = app.overlay.clone()
    else {
        return Ok(false);
    };
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            match session_store::delete_session(conn, session_id) {
                Ok(()) => {
                    app.message = Some(MessageOverlay::success(format!(
                        "Deleted \"{session_name}\""
                    )));
                    let prev_sel = app.log_selected;
                    app.load_log(conn)?;
                    app.log_selected = prev_sel;
                    app.clamp_log_selected();
                }
                Err(e) => {
                    app.message = Some(MessageOverlay::error(e.to_string()));
                }
            }
            app.overlay = Overlay::None;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.overlay = Overlay::None;
        }
        _ => {}
    }
    Ok(false)
}

// ── Mode selector overlay ──────────────────────────────────────────────────────

fn handle_overlay_mode_selector(
    app: &mut App,
    _conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    let Overlay::ModeSelector { cursor } = app.overlay.clone() else {
        return Ok(false);
    };
    match key.code {
        KeyCode::Esc => {
            app.overlay = Overlay::None;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.overlay = Overlay::ModeSelector {
                cursor: if cursor == 0 { 1 } else { cursor - 1 },
            };
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.overlay = Overlay::ModeSelector {
                cursor: (cursor + 1) % 2,
            };
        }
        KeyCode::Enter => {
            app.overlay = Overlay::None;
            if cursor == 0 {
                // Freeform: open task name prompt
                if app.active_session.is_some() {
                    app.message = Some(MessageOverlay::warning("Session already running."));
                } else {
                    app.overlay = Overlay::Prompt {
                        label: "Session name:".to_string(),
                        value: String::new(),
                        action: PromptAction::StartSession,
                    };
                }
            } else {
                // Pomodoro: gather task name
                app.overlay = Overlay::Prompt {
                    label: "Pomodoro — task name:".to_string(),
                    value: String::new(),
                    action: PromptAction::StartPomodoroName,
                };
            }
        }
        _ => {}
    }
    Ok(false)
}

// ── Pomodoro confirm stop overlay ──────────────────────────────────────────────

fn handle_overlay_pomodoro_confirm_stop(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(ref timer) = app.pomodoro_timer {
                if timer.is_in_work_phase() {
                    let date = pomodoro_store::today_local_date();
                    let _ = pomodoro_store::increment_abandoned(conn, &date);
                }
            }
            app.pomodoro_timer = None;
            app.overlay = Overlay::None;
            app.active_tab = Tab::Dashboard;
            app.message = Some(MessageOverlay::warning("Pomodoro stopped."));
            let _ = app.load_dashboard(conn);
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.overlay = Overlay::None;
        }
        _ => {}
    }
    Ok(false)
}

// ── Tab handlers ───────────────────────────────────────────────────────────────

pub fn handle_dashboard_tab(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    match key.code {
        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter if !app.todo_input_mode => {
            match session_store::stop_session(conn) {
                Ok(session) => {
                    let elapsed = session
                        .duration()
                        .map(crate::display::format::format_duration)
                        .unwrap_or_else(|| "unknown".to_string());
                    app.message = Some(MessageOverlay::success(format!(
                        "Session stopped: \"{}\" — {}",
                        session.task, elapsed
                    )));
                    let _ = app.load_dashboard(conn);
                }
                Err(e) => {
                    let msg = if e.to_string().contains("No active session") {
                        "No active session to stop.".to_string()
                    } else {
                        format!("Error: {e}")
                    };
                    app.message = Some(MessageOverlay::error(msg));
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') if !app.todo_input_mode => {
            if app.active_session.is_some() {
                app.message = Some(MessageOverlay::warning("Session already running."));
            } else if app.pomodoro_timer.is_some() {
                app.message = Some(MessageOverlay::warning(
                    "Pomodoro already running. Switch to Pomodoro tab.",
                ));
            } else {
                app.overlay = Overlay::ModeSelector { cursor: 0 };
            }
        }
        KeyCode::Esc if app.todo_input_mode => {
            // Cancel TODO input mode
            app.todo_input_mode = false;
            app.todo_input_buffer.clear();
        }
        _ => {
            // Delegate TODO-related keys to the TODO handler
            crate::tui::handlers_todo::handle_todo_key(app, conn, key.code)?;
        }
    }
    Ok(false)
}

pub fn handle_log_tab(app: &mut App, conn: &rusqlite::Connection, key: KeyEvent) -> Result<bool> {
    let page_len = app.log_page_entries(app.log_page).len();
    match key.code {
        KeyCode::Down => {
            if page_len > 0 && app.log_selected < page_len - 1 {
                app.log_selected += 1;
            }
        }
        KeyCode::Up => {
            if app.log_selected > 0 {
                app.log_selected -= 1;
            }
        }
        KeyCode::Right | KeyCode::PageDown => {
            if app.log_page + 1 < app.log_total_pages {
                app.log_page += 1;
                app.log_selected = 0;
            }
        }
        KeyCode::Left | KeyCode::PageUp => {
            if app.log_page > 0 {
                app.log_page -= 1;
                app.log_selected = 0;
            }
        }
        KeyCode::Char('j') if app.config.vim_mode => {
            if page_len > 0 && app.log_selected < page_len - 1 {
                app.log_selected += 1;
            }
        }
        KeyCode::Char('k') if app.config.vim_mode => {
            if app.log_selected > 0 {
                app.log_selected -= 1;
            }
        }
        KeyCode::Char('g') if app.config.vim_mode => {
            app.log_selected = 0;
        }
        KeyCode::Char('G') if app.config.vim_mode => {
            if page_len > 0 {
                app.log_selected = page_len - 1;
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if page_len > 0 {
                let session = &app.log_page_entries(app.log_page)[app.log_selected];
                app.overlay = Overlay::ConfirmDelete {
                    session_id: session.id,
                    session_name: session.task.clone(),
                };
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if page_len > 0 {
                let session = &app.log_page_entries(app.log_page)[app.log_selected];
                app.overlay = Overlay::Prompt {
                    label: "Rename session:".to_string(),
                    value: session.task.clone(),
                    action: PromptAction::RenameSession { id: session.id },
                };
            }
        }
        _ => {}
    }
    let _ = conn; // not used in most paths
    Ok(false)
}

pub fn handle_report_tab(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    const WINDOW_COUNT: usize = 3;
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            let new_idx = if app.report_selected_window == 0 {
                WINDOW_COUNT - 1
            } else {
                app.report_selected_window - 1
            };
            let new_window = idx_to_window(new_idx);
            app.load_report(conn, &new_window)?;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let new_idx = (app.report_selected_window + 1) % WINDOW_COUNT;
            let new_window = idx_to_window(new_idx);
            app.load_report(conn, &new_window)?;
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_pomodoro_tab(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
) -> Result<bool> {
    match key.code {
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if let Some(ref mut timer) = app.pomodoro_timer {
                if timer.paused {
                    timer.resume();
                } else {
                    timer.pause();
                }
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if let Some(ref mut timer) = app.pomodoro_timer {
                timer.skip_break();
            }
        }
        KeyCode::Char('+') => {
            if let Some(ref mut timer) = app.pomodoro_timer {
                timer.extend();
            }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if app.pomodoro_timer.is_some() {
                let in_work = app
                    .pomodoro_timer
                    .as_ref()
                    .is_some_and(|t| t.is_in_work_phase());
                if in_work {
                    app.overlay = Overlay::PomodoroConfirmStop;
                } else {
                    // In a break phase — stop without abandon penalty
                    app.pomodoro_timer = None;
                    app.active_tab = Tab::Dashboard;
                    let _ = app.load_dashboard(conn);
                    app.message = Some(MessageOverlay::success("Pomodoro finished."));
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_settings_tab(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crate::pomodoro::config::{pomodoro_config_path, save_to_file};
    use crate::tui::views::settings::SETTINGS_ROW_COUNT;

    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if app.settings_selected > 0 {
                app.settings_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.settings_selected + 1 < SETTINGS_ROW_COUNT {
                app.settings_selected += 1;
            }
        }

        // Toggle vim mode (row 0, or 'V' always works)
        KeyCode::Char('v') | KeyCode::Char('V') if app.settings_selected == 0 => {
            app.config.vim_mode = !app.config.vim_mode;
            let msg = if app.config.vim_mode {
                "Vim mode enabled"
            } else {
                "Vim mode disabled"
            };
            let path = crate::config::config_file_path();
            if let Err(e) = save_config(&path, &app.config) {
                app.message = Some(MessageOverlay::error(format!(
                    "Failed to save settings: {e}"
                )));
            } else {
                app.message = Some(MessageOverlay::success(msg));
            }
        }

        // Also allow Enter to toggle vim when selected
        KeyCode::Enter if app.settings_selected == 0 => {
            app.config.vim_mode = !app.config.vim_mode;
            let msg = if app.config.vim_mode {
                "Vim mode enabled"
            } else {
                "Vim mode disabled"
            };
            let path = crate::config::config_file_path();
            if let Err(e) = save_config(&path, &app.config) {
                app.message = Some(MessageOverlay::error(format!("Failed to save: {e}")));
            } else {
                app.message = Some(MessageOverlay::success(msg));
            }
        }

        // Increase value for Pomodoro rows
        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Right => {
            let changed = match app.settings_selected {
                1 => {
                    if app.pomo_config.work_duration_mins < 120 {
                        app.pomo_config.work_duration_mins += 1;
                        true
                    } else {
                        false
                    }
                }
                2 => {
                    if app.pomo_config.break_duration_mins < 60 {
                        app.pomo_config.break_duration_mins += 1;
                        true
                    } else {
                        false
                    }
                }
                3 => {
                    if app.pomo_config.long_break_duration_mins < 60 {
                        app.pomo_config.long_break_duration_mins += 1;
                        true
                    } else {
                        false
                    }
                }
                4 => {
                    if app.pomo_config.long_break_after < 10 {
                        app.pomo_config.long_break_after += 1;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if changed {
                let path = pomodoro_config_path();
                if let Err(e) = save_to_file(&path, &app.pomo_config) {
                    app.message = Some(MessageOverlay::error(format!("Failed to save: {e}")));
                }
            }
        }

        // Decrease value for Pomodoro rows
        KeyCode::Char('-') | KeyCode::Left => {
            let changed = match app.settings_selected {
                1 => {
                    if app.pomo_config.work_duration_mins > 1 {
                        app.pomo_config.work_duration_mins -= 1;
                        true
                    } else {
                        false
                    }
                }
                2 => {
                    if app.pomo_config.break_duration_mins > 1 {
                        app.pomo_config.break_duration_mins -= 1;
                        true
                    } else {
                        false
                    }
                }
                3 => {
                    if app.pomo_config.long_break_duration_mins > 1 {
                        app.pomo_config.long_break_duration_mins -= 1;
                        true
                    } else {
                        false
                    }
                }
                4 => {
                    if app.pomo_config.long_break_after > 2 {
                        app.pomo_config.long_break_after -= 1;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if changed {
                let path = pomodoro_config_path();
                if let Err(e) = save_to_file(&path, &app.pomo_config) {
                    app.message = Some(MessageOverlay::error(format!("Failed to save: {e}")));
                }
            }
        }

        _ => {}
    }
    Ok(false)
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::db::open_db_at;
    use crate::tui::app::App;
    use tempfile::NamedTempFile;

    fn make_app() -> App {
        App::new(false, AppConfig::default())
    }

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_conn() -> (rusqlite::Connection, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let conn = open_db_at(f.path()).unwrap();
        (conn, f)
    }

    fn insert_completed_session(conn: &rusqlite::Connection, task: &str) -> i64 {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (task, tag, start_time, end_time) VALUES (?1, NULL, ?2, ?3)",
            rusqlite::params![task, now - 60, now],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_active_session(conn: &rusqlite::Connection, task: &str) {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (task, tag, start_time) VALUES (?1, NULL, ?2)",
            rusqlite::params![task, now - 60],
        )
        .unwrap();
    }

    // T019: Dashboard tab tests
    #[test]
    fn dashboard_s_with_active_session_stops_it() {
        let (conn, _f) = test_conn();
        insert_active_session(&conn, "my task");
        let mut app = make_app();
        app.load_dashboard(&conn).unwrap();
        handle_dashboard_tab(&mut app, &conn, make_key(KeyCode::Char('s'))).unwrap();
        assert!(app.message.is_some());
        let msg = app.message.unwrap();
        assert_eq!(msg.kind, crate::tui::app::MessageKind::Success);
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn dashboard_s_with_no_active_session_shows_error() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        handle_dashboard_tab(&mut app, &conn, make_key(KeyCode::Char('s'))).unwrap();
        let msg = app.message.unwrap();
        assert_eq!(msg.kind, crate::tui::app::MessageKind::Error);
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn dashboard_s_does_not_open_overlay() {
        let (conn, _f) = test_conn();
        insert_active_session(&conn, "task");
        let mut app = make_app();
        app.load_dashboard(&conn).unwrap();
        handle_dashboard_tab(&mut app, &conn, make_key(KeyCode::Char('s'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
    }

    // T020: Overlay prompt tests
    #[test]
    fn prompt_printable_char_appends_to_value() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "he".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Char('y'))).unwrap();
        let Overlay::Prompt { value, .. } = &app.overlay else {
            panic!()
        };
        assert_eq!(value, "hey");
    }

    #[test]
    fn prompt_backspace_removes_last_char() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "ab".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Backspace)).unwrap();
        let Overlay::Prompt { value, .. } = &app.overlay else {
            panic!()
        };
        assert_eq!(value, "a");
    }

    #[test]
    fn prompt_esc_clears_overlay() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "abc".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn prompt_enter_empty_uses_untitled_session() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        // Step 1: enter empty name → moves to tag prompt with "Untitled Session"
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        // Now in step 2 (tag prompt)
        assert!(matches!(
            app.overlay,
            Overlay::Prompt {
                action: PromptAction::StartSessionTag { .. },
                ..
            }
        ));
        // Step 2: press Enter to skip tag
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        let active = session_store::get_active_session(&conn).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().task, "Untitled Session");
    }

    #[test]
    fn prompt_enter_nonempty_uses_provided_name() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        // Step 1: enter name
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "deep work".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        // Step 2: press Enter to skip tag
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        let active = session_store::get_active_session(&conn).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().task, "deep work");
    }

    #[test]
    fn prompt_enter_with_tag_stores_tag() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        // Step 1: enter name
        app.overlay = Overlay::Prompt {
            label: "test:".into(),
            value: "coding".into(),
            action: PromptAction::StartSession,
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        // Step 2: enter a tag
        if let Overlay::Prompt { ref mut value, .. } = app.overlay {
            *value = "dev".to_string();
        }
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        let active = session_store::get_active_session(&conn).unwrap();
        assert!(active.is_some());
        let s = active.unwrap();
        assert_eq!(s.task, "coding");
        assert_eq!(s.tag.as_deref(), Some("dev"));
    }

    // T025: Global tab switching tests
    #[test]
    fn key_1_sets_dashboard_tab() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.active_tab = Tab::Log;
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('1'))).unwrap();
        assert_eq!(app.active_tab, Tab::Dashboard);
    }

    #[test]
    fn key_2_sets_log_tab() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('2'))).unwrap();
        assert_eq!(app.active_tab, Tab::Log);
    }

    #[test]
    fn key_3_sets_report_tab() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('3'))).unwrap();
        assert_eq!(app.active_tab, Tab::Report);
    }

    #[test]
    fn key_4_sets_settings_tab() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('4'))).unwrap();
        assert_eq!(app.active_tab, Tab::Settings);
    }

    #[test]
    fn tab_key_cycles_through_all_tabs() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        assert_eq!(app.active_tab, Tab::Dashboard);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.active_tab, Tab::Log);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.active_tab, Tab::Report);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.active_tab, Tab::Settings);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.active_tab, Tab::Pomodoro);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Tab)).unwrap();
        assert_eq!(app.active_tab, Tab::Dashboard);
    }

    // T026: Log tab navigation tests
    #[test]
    fn log_down_increments_selected() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Down)).unwrap();
        assert_eq!(app.log_selected, 1);
    }

    #[test]
    fn log_down_does_not_exceed_last_item() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "only");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Down)).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    fn log_up_decrements_selected() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        app.log_selected = 1;
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Up)).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    fn log_up_does_not_go_below_zero() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Up)).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    fn log_right_advances_page() {
        let (conn, _f) = test_conn();
        for i in 0..12 {
            insert_completed_session(&conn, &format!("task {i}"));
        }
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Right)).unwrap();
        assert_eq!(app.log_page, 1);
    }

    #[test]
    fn log_left_goes_back_page() {
        let (conn, _f) = test_conn();
        for i in 0..12 {
            insert_completed_session(&conn, &format!("task {i}"));
        }
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        app.log_page = 1;
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Left)).unwrap();
        assert_eq!(app.log_page, 0);
    }

    // T035: Vim key tests in log tab
    #[test]
    fn vim_j_moves_down_when_enabled() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.config.vim_mode = true;
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.log_selected, 1);
    }

    #[test]
    fn vim_j_has_no_effect_when_disabled() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.config.vim_mode = false;
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    fn vim_k_moves_up_when_enabled() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.config.vim_mode = true;
        app.load_log(&conn).unwrap();
        app.log_selected = 1;
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    fn vim_g_jumps_to_top() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.config.vim_mode = true;
        app.load_log(&conn).unwrap();
        app.log_selected = 1;
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('g'))).unwrap();
        assert_eq!(app.log_selected, 0);
    }

    #[test]
    #[allow(non_snake_case)]
    fn vim_G_jumps_to_bottom() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "a");
        insert_completed_session(&conn, "b");
        let mut app = make_app();
        app.config.vim_mode = true;
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('G'))).unwrap();
        assert_eq!(app.log_selected, 1);
    }

    // T036: Settings toggle tests
    #[test]
    fn settings_v_toggles_vim_mode() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        assert!(!app.config.vim_mode);
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('4'))).unwrap();
        handle_settings_tab(&mut app, make_key(KeyCode::Char('v'))).unwrap();
        assert!(app.config.vim_mode);
        handle_settings_tab(&mut app, make_key(KeyCode::Char('v'))).unwrap();
        assert!(!app.config.vim_mode);
    }

    // T040: Delete flow tests
    #[test]
    fn log_d_with_selection_opens_confirm_overlay() {
        let (conn, _f) = test_conn();
        let id = insert_completed_session(&conn, "to delete");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(
            app.overlay,
            Overlay::ConfirmDelete { session_id, .. } if session_id == id
        ));
    }

    #[test]
    fn log_d_with_empty_log_does_nothing() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('d'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn confirm_y_calls_delete_and_resets_overlay() {
        let (conn, _f) = test_conn();
        let id = insert_completed_session(&conn, "bye");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        app.overlay = Overlay::ConfirmDelete {
            session_id: id,
            session_name: "bye".into(),
        };
        handle_overlay_confirm(&mut app, &conn, make_key(KeyCode::Char('y'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn confirm_n_resets_overlay_only() {
        let (conn, _f) = test_conn();
        let id = insert_completed_session(&conn, "keep");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        app.overlay = Overlay::ConfirmDelete {
            session_id: id,
            session_name: "keep".into(),
        };
        handle_overlay_confirm(&mut app, &conn, make_key(KeyCode::Char('n'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    // T041: Rename flow tests
    #[test]
    fn log_r_with_selection_opens_prompt_prefilled() {
        let (conn, _f) = test_conn();
        insert_completed_session(&conn, "original name");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('r'))).unwrap();
        let Overlay::Prompt { value, action, .. } = &app.overlay else {
            panic!("Expected Prompt")
        };
        assert_eq!(value, "original name");
        assert!(matches!(action, PromptAction::RenameSession { .. }));
    }

    #[test]
    fn log_r_with_empty_log_does_nothing() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        handle_log_tab(&mut app, &conn, make_key(KeyCode::Char('r'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn prompt_rename_enter_calls_rename_session() {
        let (conn, _f) = test_conn();
        let id = insert_completed_session(&conn, "old");
        let mut app = make_app();
        app.load_log(&conn).unwrap();
        app.overlay = Overlay::Prompt {
            label: "Rename session:".into(),
            value: "new name".into(),
            action: PromptAction::RenameSession { id },
        };
        handle_overlay_prompt(&mut app, &conn, make_key(KeyCode::Enter)).unwrap();
        let task: String = conn
            .query_row("SELECT task FROM sessions WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(task, "new name");
    }

    // T042: Help key test
    #[test]
    fn question_mark_sets_help_overlay() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        handle_key_event(&mut app, &conn, make_key(KeyCode::Char('?'))).unwrap();
        assert!(matches!(app.overlay, Overlay::Help));
    }

    #[test]
    fn any_key_dismisses_help_overlay() {
        let (conn, _f) = test_conn();
        let mut app = make_app();
        app.overlay = Overlay::Help;
        handle_overlay(&mut app, &conn, make_key(KeyCode::Char('x'))).unwrap();
        assert!(matches!(app.overlay, Overlay::None));
    }
}
