use std::time::Instant;

use crate::models::session::Session;

// ── Time window ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TimeWindow {
    Today,
    CurrentWeek,
    Last7Days,
}

// ── Input field ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Task,
    Tag,
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

// ── View ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum View {
    Dashboard,
    Menu {
        selected: usize,
    },
    StartForm {
        task: String,
        tag: String,
        active_field: InputField,
    },
    Log {
        page: usize,
    },
    Report {
        window: TimeWindow,
        selected_window: usize,
    },
}

// ── App ────────────────────────────────────────────────────────────────────────

pub struct App {
    pub view: View,
    pub active_session: Option<Session>,
    pub today_summary: Vec<(Option<String>, i64)>,
    pub log_entries: Vec<Session>,
    pub log_total_pages: usize,
    pub report_rows: Vec<(Option<String>, i64)>,
    pub message: Option<MessageOverlay>,
    pub quit_pending: bool,
    pub terminal_too_small: bool,
    pub no_color: bool,
}

pub const LOG_PAGE_SIZE: usize = 10;

impl App {
    pub fn new(no_color: bool) -> Self {
        Self {
            view: View::Dashboard,
            active_session: None,
            today_summary: Vec::new(),
            log_entries: Vec::new(),
            log_total_pages: 1,
            report_rows: Vec::new(),
            message: None,
            quit_pending: false,
            terminal_too_small: false,
            no_color,
        }
    }

    /// Load/refresh dashboard data from the database.
    pub fn load_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::commands::report::today_start;
        use crate::db::session_store;

        self.active_session = session_store::get_active_session(conn)?;
        self.today_summary = session_store::aggregate_by_tag(conn, today_start())?;
        Ok(())
    }

    /// Load log entries from the database (all completed, for pagination).
    pub fn load_log(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::db::session_store;

        self.log_entries = session_store::list_all_completed(conn)?;
        // Reverse so newest first
        self.log_entries.reverse();
        let total = self.log_entries.len();
        self.log_total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(LOG_PAGE_SIZE)
        };
        Ok(())
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
        Ok(())
    }

    /// Tick update for Dashboard view (refreshes active session timer).
    pub fn tick_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::db::session_store;
        self.active_session = session_store::get_active_session(conn)?;
        // Auto-dismiss expired messages
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
}

/// Truncate a string to `max_chars` Unicode scalar values.
/// If truncation occurs, the last character is replaced with `…` (ellipsis, counts as 1).
pub fn truncate_to(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        // Take max_chars - 1 chars, then append ellipsis
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    }
}
