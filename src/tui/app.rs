use std::time::Instant;

use crate::config::AppConfig;
use crate::models::session::Session;
use crate::models::todo::Todo;
use crate::pomodoro::config::PomodoroConfig;
use crate::pomodoro::timer::PomodoroTimer;
use crate::tui::keyboard::{KeyHandler, KeyContext};

// ── Time window ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TimeWindow {
    Today,
    CurrentWeek,
    Last7Days,
}

// ── Message overlay ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct MessageOverlay {
    pub text: String,
    pub kind: MessageKind,
    pub shown_at: Instant,
    pub auto_dismiss_secs: u64,
}

impl MessageOverlay {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: MessageKind::Success,
            shown_at: Instant::now(),
            auto_dismiss_secs: 2,
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: MessageKind::Warning,
            shown_at: Instant::now(),
            auto_dismiss_secs: 2,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: MessageKind::Error,
            shown_at: Instant::now(),
            auto_dismiss_secs: 3,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.shown_at.elapsed().as_secs() >= self.auto_dismiss_secs
    }
}

// ── Tab ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Log,
    Report,
    Settings,
    Pomodoro,
}

// ── PromptAction ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PromptAction {
    StartSession,
    /// Second step: user already entered task name, now entering optional tag.
    StartSessionTag {
        task: String,
    },
    RenameSession {
        id: i64,
    },
    /// First step for Pomodoro: gathering task name.
    StartPomodoroName,
    /// Second step for Pomodoro: entering optional tag.
    StartPomodoroTag {
        task: String,
    },
}

// ── Overlay ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Overlay {
    None,
    Prompt {
        label: String,
        value: String,
        action: PromptAction,
    },
    ConfirmDelete {
        session_id: i64,
        session_name: String,
    },
    Help,
    /// Mode selector shown when user presses 'n' on Dashboard.
    ModeSelector {
        /// 0 = Freeform, 1 = Pomodoro
        cursor: usize,
    },
    /// Confirm stopping an in-progress Pomodoro work phase.
    PomodoroConfirmStop,
}

impl Overlay {
    pub fn is_active(&self) -> bool {
        !matches!(self, Overlay::None)
    }
}

// ── TimeWindow state ───────────────────────────────────────────────────────────

pub fn window_to_idx(w: &TimeWindow) -> usize {
    match w {
        TimeWindow::Today => 0,
        TimeWindow::CurrentWeek => 1,
        TimeWindow::Last7Days => 2,
    }
}

pub fn idx_to_window(idx: usize) -> TimeWindow {
    match idx {
        0 => TimeWindow::Today,
        1 => TimeWindow::CurrentWeek,
        _ => TimeWindow::Last7Days,
    }
}

// ── App ────────────────────────────────────────────────────────────────────────

pub struct App {
    pub active_tab: Tab,
    pub overlay: Overlay,
    pub log_selected: usize,
    pub config: AppConfig,
    // Report state
    pub report_window: TimeWindow,
    pub report_selected_window: usize,
    // Log pagination (page stored in App now)
    pub log_page: usize,
    // Data fields
    pub active_session: Option<Session>,
    pub today_summary: Vec<(Option<String>, i64)>,
    pub today_sessions: Vec<Session>,
    pub log_entries: Vec<Session>,
    pub log_total_pages: usize,
    pub report_rows: Vec<(Option<String>, i64)>,
    pub message: Option<MessageOverlay>,
    pub quit_pending: bool,
    pub terminal_too_small: bool,
    pub no_color: bool,
    /// Active Pomodoro timer (Some while a session is running in Pomodoro mode).
    pub pomodoro_timer: Option<PomodoroTimer>,
    /// Currently loaded Pomodoro default config (for the Settings tab).
    pub pomo_config: PomodoroConfig,
    /// Selected row index in the Settings tab (0=vim, 1=work, 2=break, 3=long_break, 4=long_break_after).
    pub settings_selected: usize,
    // TODO fields for 007-ui-refresh feature
    pub todos: Vec<Todo>,
    pub selected_todo_idx: Option<usize>,
    pub todo_input_mode: bool,
    pub todo_input_buffer: String,
    // Keyboard handler for context-aware input routing
    pub keyboard_handler: KeyHandler,
}

pub const LOG_PAGE_SIZE: usize = 10;

impl App {
    pub fn new(no_color: bool, config: AppConfig) -> Self {
        let vim_mode = config.vim_mode;
        Self {
            active_tab: Tab::Dashboard,
            overlay: Overlay::None,
            log_selected: 0,
            config,
            report_window: TimeWindow::Today,
            report_selected_window: 0,
            log_page: 0,
            active_session: None,
            today_summary: Vec::new(),
            today_sessions: Vec::new(),
            log_entries: Vec::new(),
            log_total_pages: 1,
            report_rows: Vec::new(),
            message: None,
            quit_pending: false,
            terminal_too_small: false,
            no_color,
            pomodoro_timer: None,
            pomo_config: PomodoroConfig::default(),
            settings_selected: 0,
            todos: Vec::new(),
            selected_todo_idx: None,
            todo_input_mode: false,
            todo_input_buffer: String::new(),
            keyboard_handler: KeyHandler::new(vim_mode),
        }
    }

    /// Load/refresh dashboard data from the database.
    pub fn load_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::commands::report::today_start;
        use crate::db::session_store;

        self.active_session = session_store::get_active_session(conn)?;
        self.today_summary = session_store::aggregate_by_tag(conn, today_start())?;
        self.today_sessions = session_store::list_completed_since(conn, today_start())?;
        self.load_todos(conn)?;
        Ok(())
    }

    /// Load TODOs from the database into the app state.
    pub fn load_todos(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::models::todo;
        self.todos = todo::list_all(conn)?;
        Ok(())
    }

    /// Load log entries from the database (all completed, newest first).
    pub fn load_log(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::db::session_store;

        self.log_entries = session_store::list_all_completed(conn)?;
        self.log_entries.reverse();
        let total = self.log_entries.len();
        self.log_total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(LOG_PAGE_SIZE)
        };
        self.log_selected = 0;
        self.log_page = 0;
        Ok(())
    }

    /// Clamp log_selected to valid range after a page change or reload.
    pub fn clamp_log_selected(&mut self) {
        let page_entries = self.log_page_entries(self.log_page).len();
        if page_entries == 0 {
            self.log_selected = 0;
        } else if self.log_selected >= page_entries {
            self.log_selected = page_entries - 1;
        }
    }

    /// Load report rows for the given time window.
    pub fn load_report(
        &mut self,
        conn: &rusqlite::Connection,
        window: &TimeWindow,
    ) -> anyhow::Result<()> {
        use crate::commands::report::{current_week_start, rolling_7d_start, today_start};
        use crate::db::session_store;

        let since = match window {
            TimeWindow::Today => today_start(),
            TimeWindow::CurrentWeek => current_week_start(),
            TimeWindow::Last7Days => rolling_7d_start(),
        };
        self.report_rows = session_store::aggregate_by_tag(conn, since)?;
        self.report_window = window.clone();
        self.report_selected_window = window_to_idx(window);
        Ok(())
    }

    /// Tick update for Dashboard tab (refreshes active session timer).
    pub fn tick_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::commands::report::today_start;
        use crate::db::session_store;
        self.active_session = session_store::get_active_session(conn)?;
        self.today_sessions = session_store::list_completed_since(conn, today_start())?;
        if let Some(ref msg) = self.message {
            if msg.is_expired() {
                self.message = None;
            }
        }
        Ok(())
    }

    /// Returns the log page slice for the given page index (0-based).
    pub fn log_page_entries(&self, page: usize) -> &[Session] {
        let start = page * LOG_PAGE_SIZE;
        let end = (start + LOG_PAGE_SIZE).min(self.log_entries.len());
        if start >= self.log_entries.len() {
            &[]
        } else {
            &self.log_entries[start..end]
        }
    }

    /// Count of completed sessions.
    pub fn count_completed(conn: &rusqlite::Connection) -> anyhow::Result<usize> {
        use crate::db::session_store;
        session_store::count_completed(conn)
    }

    /// Enter TODO input mode: set keyboard context to Input and clear buffer for new entry
    pub fn enter_todo_input_mode(&mut self) {
        self.todo_input_mode = true;
        self.todo_input_buffer.clear();
        self.keyboard_handler.set_context(KeyContext::Input);
    }

    /// Exit TODO input mode: set keyboard context to Viewing and clear buffer
    pub fn exit_todo_input_mode(&mut self) {
        self.todo_input_mode = false;
        self.todo_input_buffer.clear();
        self.keyboard_handler.set_context(KeyContext::Viewing);
    }
}

/// Truncate a string to `max_chars` Unicode scalar values.
pub fn truncate_to(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    }
}
