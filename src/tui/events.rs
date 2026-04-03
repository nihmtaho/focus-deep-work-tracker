use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::db::session_store;
use crate::display::format::format_elapsed;
use crate::error::FocusError;
use crate::tui::app::{App, InputField, MessageOverlay, TimeWindow, View};

/// Handle a key event. Returns true if the app should quit.
pub fn handle_key_event(app: &mut App, conn: &rusqlite::Connection, key: KeyEvent) -> Result<bool> {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }

    match &app.view.clone() {
        View::Dashboard => handle_dashboard(app, conn, key),
        View::Menu { selected } => handle_menu(app, conn, key, *selected),
        View::StartForm {
            task,
            tag,
            active_field,
        } => handle_start_form(
            app,
            conn,
            key,
            task.clone(),
            tag.clone(),
            active_field.clone(),
        ),
        View::Log { page } => handle_log(app, conn, key, *page),
        View::Report {
            window,
            selected_window,
        } => handle_report(app, conn, key, window.clone(), *selected_window),
    }
}

fn handle_dashboard(app: &mut App, _conn: &rusqlite::Connection, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('m') | KeyCode::Char('M') => {
            app.view = View::Menu { selected: 0 };
        }
        KeyCode::Esc => {
            app.message = None;
        }
        _ => {}
    }
    Ok(false)
}

fn handle_menu(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
    selected: usize,
) -> Result<bool> {
    // Menu items: 0=Start, 1=Stop, 2=Log, 3=Report, 4=Back
    const MENU_ITEMS: usize = 5;

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.view = View::Dashboard;
        }
        KeyCode::Up => {
            let new_selected = if selected == 0 {
                MENU_ITEMS - 1
            } else {
                selected - 1
            };
            app.view = View::Menu {
                selected: new_selected,
            };
        }
        KeyCode::Down => {
            let new_selected = (selected + 1) % MENU_ITEMS;
            app.view = View::Menu {
                selected: new_selected,
            };
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            match selected {
                0 => {
                    // Start
                    app.view = View::StartForm {
                        task: String::new(),
                        tag: String::new(),
                        active_field: InputField::Task,
                    };
                }
                1 => {
                    // Stop
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
                            app.view = View::Dashboard;
                            let _ = app.load_dashboard(conn);
                        }
                        Err(e) => {
                            let fe = e.downcast::<rusqlite::Error>();
                            let msg = match fe {
                                Ok(db_err) => format!("DB error: {db_err}"),
                                Err(e2) => {
                                    if e2.to_string().contains("No active session") {
                                        "No active session to stop.".to_string()
                                    } else {
                                        format!("Error: {e2}")
                                    }
                                }
                            };
                            app.message = Some(MessageOverlay::error(msg));
                            app.view = View::Dashboard;
                        }
                    }
                }
                2 => {
                    // Log
                    app.load_log(conn)?;
                    app.view = View::Log { page: 0 };
                }
                3 => {
                    // Report
                    let window = TimeWindow::Today;
                    app.load_report(conn, &window)?;
                    app.view = View::Report {
                        window,
                        selected_window: 0,
                    };
                }
                4 => {
                    // Back
                    app.view = View::Dashboard;
                }
                _ => {}
            }
        }
        KeyCode::Char('1') => {
            app.view = View::StartForm {
                task: String::new(),
                tag: String::new(),
                active_field: InputField::Task,
            };
        }
        KeyCode::Char('2') => match session_store::stop_session(conn) {
            Ok(session) => {
                let elapsed = session
                    .duration()
                    .map(crate::display::format::format_duration)
                    .unwrap_or_else(|| "unknown".to_string());
                app.message = Some(MessageOverlay::success(format!(
                    "Session stopped: \"{}\" — {}",
                    session.task, elapsed
                )));
                app.view = View::Dashboard;
                let _ = app.load_dashboard(conn);
            }
            Err(_) => {
                app.message = Some(MessageOverlay::error("No active session to stop."));
                app.view = View::Dashboard;
            }
        },
        KeyCode::Char('3') => {
            app.load_log(conn)?;
            app.view = View::Log { page: 0 };
        }
        KeyCode::Char('4') => {
            let window = TimeWindow::Today;
            app.load_report(conn, &window)?;
            app.view = View::Report {
                window,
                selected_window: 0,
            };
        }
        _ => {}
    }
    Ok(false)
}

fn handle_start_form(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
    task: String,
    tag: String,
    active_field: InputField,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.view = View::Menu { selected: 0 };
        }
        KeyCode::Tab => {
            let new_field = match active_field {
                InputField::Task => InputField::Tag,
                InputField::Tag => InputField::Task,
            };
            app.view = View::StartForm {
                task,
                tag,
                active_field: new_field,
            };
        }
        KeyCode::Enter => {
            // Submit form
            let task_trimmed = task.trim().to_string();
            if task_trimmed.is_empty() {
                app.message = Some(MessageOverlay::error(FocusError::EmptyTask.to_string()));
                return Ok(false);
            }
            // Check for existing active session
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
                    let tag_opt = if tag.trim().is_empty() {
                        None
                    } else {
                        Some(tag.trim())
                    };
                    session_store::insert_session(conn, &task_trimmed, tag_opt)?;
                    app.message = Some(MessageOverlay::success(format!(
                        "Session started: \"{}\"",
                        task_trimmed
                    )));
                    app.view = View::Dashboard;
                    let _ = app.load_dashboard(conn);
                }
            }
        }
        KeyCode::Backspace => match active_field {
            InputField::Task => {
                let mut t = task;
                t.pop();
                app.view = View::StartForm {
                    task: t,
                    tag,
                    active_field,
                };
            }
            InputField::Tag => {
                let mut tg = tag;
                tg.pop();
                app.view = View::StartForm {
                    task,
                    tag: tg,
                    active_field,
                };
            }
        },
        KeyCode::Char(c) => match active_field {
            InputField::Task => {
                let mut t = task;
                t.push(c);
                app.view = View::StartForm {
                    task: t,
                    tag,
                    active_field,
                };
            }
            InputField::Tag => {
                let mut tg = tag;
                tg.push(c);
                app.view = View::StartForm {
                    task,
                    tag: tg,
                    active_field,
                };
            }
        },
        _ => {}
    }
    Ok(false)
}

fn handle_log(
    app: &mut App,
    _conn: &rusqlite::Connection,
    key: KeyEvent,
    page: usize,
) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.view = View::Dashboard;
        }
        KeyCode::Right | KeyCode::Char('n') | KeyCode::Char('N') => {
            let next_page = page + 1;
            if next_page < app.log_total_pages {
                app.view = View::Log { page: next_page };
            }
        }
        KeyCode::Left | KeyCode::Char('p') | KeyCode::Char('P') => {
            if page > 0 {
                app.view = View::Log { page: page - 1 };
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_report(
    app: &mut App,
    conn: &rusqlite::Connection,
    key: KeyEvent,
    window: TimeWindow,
    selected_window: usize,
) -> Result<bool> {
    // Windows: 0=Today, 1=CurrentWeek, 2=Last7Days
    const WINDOW_COUNT: usize = 3;

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.view = View::Dashboard;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            let new_idx = if selected_window == 0 {
                WINDOW_COUNT - 1
            } else {
                selected_window - 1
            };
            let new_window = idx_to_window(new_idx);
            app.load_report(conn, &new_window)?;
            app.view = View::Report {
                window: new_window,
                selected_window: new_idx,
            };
        }
        KeyCode::Right | KeyCode::Char('l') => {
            let new_idx = (selected_window + 1) % WINDOW_COUNT;
            let new_window = idx_to_window(new_idx);
            app.load_report(conn, &new_window)?;
            app.view = View::Report {
                window: new_window,
                selected_window: new_idx,
            };
        }
        KeyCode::Char('1') => {
            let new_window = TimeWindow::Today;
            app.load_report(conn, &new_window)?;
            app.view = View::Report {
                window: new_window,
                selected_window: 0,
            };
        }
        KeyCode::Char('2') => {
            let new_window = TimeWindow::CurrentWeek;
            app.load_report(conn, &new_window)?;
            app.view = View::Report {
                window: new_window,
                selected_window: 1,
            };
        }
        KeyCode::Char('3') => {
            let new_window = TimeWindow::Last7Days;
            app.load_report(conn, &new_window)?;
            app.view = View::Report {
                window: new_window,
                selected_window: 2,
            };
        }
        _ => {}
    }
    let _ = window; // avoid unused warning
    Ok(false)
}

fn idx_to_window(idx: usize) -> TimeWindow {
    match idx {
        0 => TimeWindow::Today,
        1 => TimeWindow::CurrentWeek,
        _ => TimeWindow::Last7Days,
    }
}
