use std::time::Instant;

use crate::config::AppConfig;
use crate::models::session::Session;
use crate::models::todo::Todo;
use crate::pomodoro::config::PomodoroConfig;
use crate::pomodoro::timer::PomodoroTimer;
use crate::tui::keyboard::{KeyHandler};
use crate::tui::report::ReportMetrics;
use crate::tui::text_input::TextInput;

// Re-export so existing callers (ui.rs) can use `app::VimInputMode`.
pub use crate::tui::text_input::VimInputMode;

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
    Settings,
}

// ── PromptAction ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PromptAction {
    AddTodo,
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

// ── App ────────────────────────────────────────────────────────────────────────

pub struct App {
    pub active_tab: Tab,
    pub overlay: Overlay,
    pub log_selected: usize,
    pub config: AppConfig,
    // Log pagination (page stored in App now)
    pub log_page: usize,
    // Data fields
    pub active_session: Option<Session>,
    pub today_sessions: Vec<Session>,
    pub log_entries: Vec<Session>,
    pub log_total_pages: usize,
    pub message: Option<MessageOverlay>,
    pub quit_pending: bool,
    pub no_color: bool,
    /// Active Pomodoro timer (Some while a session is running in Pomodoro mode).
    pub pomodoro_timer: Option<PomodoroTimer>,
    /// Currently loaded Pomodoro default config (for the Settings tab).
    pub pomo_config: PomodoroConfig,
    /// Selected row index in the Settings tab (0=vim, 1=theme, 2=work, 3=break, 4=long_break, 5=long_break_after).
    pub settings_selected: usize,
    // TODO fields for 007-ui-refresh feature
    pub todos: Vec<Todo>,
    pub selected_todo_idx: Option<usize>,
    /// Vim-aware text input state for all prompt overlays (todo add, session name, tag, rename).
    pub prompt_input: TextInput,
    // Keyboard handler for context-aware input routing
    pub keyboard_handler: KeyHandler,
    // Report panel metrics shown in Dashboard (replaces Today's Summary)
    pub report_metrics: ReportMetrics,
    /// Time the report_metrics were last computed (for 5-second cache).
    pub report_metrics_cached_at: Option<Instant>,
    /// When true, the dashboard shows an expanded full-screen Pomodoro panel.
    pub full_pomodoro_panel: bool,
    /// Previous clock string (what we're animating FROM).
    pub clock_prev_str: String,
    /// Current clock string (what we're animating TO), kept in sync each frame.
    pub clock_curr_str: String,
    /// Per-character animation frame: 0 = static, 1–6 = transition in progress (300 ms total).
    pub clock_anim_frame: [u8; 8],
    /// Previous Pomodoro MM:SS string (what we're animating FROM).
    pub pomo_clock_prev_str: String,
    /// Current Pomodoro MM:SS string, kept in sync each frame.
    pub pomo_clock_curr_str: String,
    /// Per-character animation frame for the Pomodoro MM:SS clock (5 positions).
    pub pomo_clock_anim_frame: [u8; 5],
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
            log_page: 0,
            active_session: None,
            today_sessions: Vec::new(),
            log_entries: Vec::new(),
            log_total_pages: 1,
            message: None,
            quit_pending: false,
            no_color,
            pomodoro_timer: None,
            pomo_config: PomodoroConfig::default(),
            settings_selected: 0,
            todos: Vec::new(),
            selected_todo_idx: None,
            prompt_input: TextInput::new(vim_mode),
            keyboard_handler: KeyHandler::new(vim_mode),
            report_metrics: ReportMetrics::default(),
            report_metrics_cached_at: None,
            full_pomodoro_panel: false,
            clock_prev_str: "--:--:--".to_string(),
            clock_curr_str: "--:--:--".to_string(),
            clock_anim_frame: [0u8; 8],
            pomo_clock_prev_str: "--:--".to_string(),
            pomo_clock_curr_str: "--:--".to_string(),
            pomo_clock_anim_frame: [0u8; 5],
        }
    }

    /// Refresh Report panel metrics from the database, with a 5-second cache.
    ///
    /// If the cache is still fresh (< 5 s old) the stored metrics are returned
    /// without hitting the database.  Call `invalidate_report_metrics_cache()`
    /// to force an immediate refresh (e.g. after a session ends).
    pub fn load_report_metrics(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        const CACHE_TTL_SECS: u64 = 5;
        let stale = self
            .report_metrics_cached_at
            .map(|t| t.elapsed().as_secs() >= CACHE_TTL_SECS)
            .unwrap_or(true);

        if stale {
            self.report_metrics = ReportMetrics::compute(conn)?;
            self.report_metrics_cached_at = Some(Instant::now());
        }
        Ok(())
    }

    /// Force the next `load_report_metrics` call to re-query the database.
    pub fn invalidate_report_metrics_cache(&mut self) {
        self.report_metrics_cached_at = None;
    }

    /// Load/refresh dashboard data from the database.
    pub fn load_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::db::time::today_start;
        use crate::db::session_store;

        self.active_session = session_store::get_active_session(conn)?;
        self.today_sessions = session_store::list_completed_since(conn, today_start())?;
        self.load_todos(conn)?;
        self.load_report_metrics(conn)?;
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

    /// Tick update for Dashboard tab (refreshes active session timer).
    pub fn tick_dashboard(&mut self, conn: &rusqlite::Connection) -> anyhow::Result<()> {
        use crate::db::time::today_start;
        use crate::db::session_store;
        self.active_session = session_store::get_active_session(conn)?;
        self.today_sessions = session_store::list_completed_since(conn, today_start())?;
        if let Some(ref msg) = self.message {
            if msg.is_expired() {
                self.message = None;
            }
        }
        self.load_report_metrics(conn)?;
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

    /// Open a Prompt overlay and prepare prompt_input (reset or pre-fill).
    pub fn open_prompt(&mut self, label: impl Into<String>, initial: &str, action: PromptAction) {
        self.prompt_input.set_vim_enabled(self.config.vim_mode);
        if initial.is_empty() {
            self.prompt_input.reset();
        } else {
            self.prompt_input.set_value(initial);
        }
        self.overlay = Overlay::Prompt {
            label: label.into(),
            action,
        };
    }

    /// Advance one character-fade animation step.
    ///
    /// `curr` and `prev` track the current and previous display strings.
    /// `frames` is per-character animation progress (0=idle, 1–6=fading).
    /// Only digit-to-digit transitions trigger animation.
    fn advance_anim(curr: &mut String, prev: &mut String, frames: &mut [u8], new_str: &str) {
        if new_str != curr.as_str() {
            let prev_chars: Vec<char> = curr.chars().collect();
            let new_chars: Vec<char> = new_str.chars().collect();
            *prev = std::mem::replace(curr, new_str.to_string());
            let len = frames.len().min(new_chars.len());
            for (i, &nch) in new_chars.iter().enumerate().take(len) {
                let pch = prev_chars.get(i).copied().unwrap_or(' ');
                if pch != nch && pch.is_ascii_digit() && nch.is_ascii_digit() {
                    frames[i] = 1;
                }
            }
        } else {
            for f in frames.iter_mut() {
                if *f > 0 {
                    *f += 1;
                    if *f > 6 {
                        *f = 0;
                    }
                }
            }
        }
    }

    /// Advance clock digit-fade animation.  Call once per render frame (~50 ms).
    ///
    /// If `new_str` differs from the tracked current string a digit changed:
    /// any position whose character changed gets its animation frame set to 1
    /// (start of a 6-frame / 300 ms fade).  On subsequent calls with the same
    /// string, in-progress frames are incremented; at frame 7 they reset to 0
    /// (animation complete, digit shown at full opacity).
    ///
    /// Only digit↔digit transitions (both chars are ASCII digits) trigger the
    /// animation — switching from `--:--:--` to digits or back does not fade.
    pub fn advance_clock_anim(&mut self, new_str: &str) {
        Self::advance_anim(
            &mut self.clock_curr_str,
            &mut self.clock_prev_str,
            &mut self.clock_anim_frame,
            new_str,
        );
    }

    /// Same fade-animation logic as [`advance_clock_anim`] but for the Pomodoro
    /// MM:SS countdown.  Call once per render frame when a Pomodoro is running.
    pub fn advance_pomo_clock_anim(&mut self, new_str: &str) {
        Self::advance_anim(
            &mut self.pomo_clock_curr_str,
            &mut self.pomo_clock_prev_str,
            &mut self.pomo_clock_anim_frame,
            new_str,
        );
    }

    /// Returns true when a Pomodoro session is currently active (timer is running).
    pub fn has_active_pomodoro(&self) -> bool {
        self.pomodoro_timer.is_some()
    }

    /// Persist the current config to disk immediately.
    ///
    /// Errors are returned to the caller for surfacing via `MessageOverlay::error`.
    /// Never silently discarded.
    pub fn save_config_now(&self) -> anyhow::Result<()> {
        crate::config::save_config(&crate::config::config_file_path(), &self.config)
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
