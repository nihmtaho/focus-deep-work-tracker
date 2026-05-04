# Focus CLI — Surface Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove dead code, eliminate duplication, and reduce UX friction across TUI, CLI commands, Pomodoro, and TODOs.

**Architecture:** Three phases — dead code removal first (safest), then deduplication, then UX behaviour changes. Each task produces a green `cargo test` and a commit. Breaking CLI changes allowed (user confirmed).

**Tech Stack:** Rust stable 1.77+, rusqlite 0.31, ratatui 0.29, crossterm 0.28, clap 4, chrono 0.4

---

## File Map

| File | Change |
|---|---|
| `src/tui/views/menu.rs` | **Delete** — dead, never called, references removed `InputField` |
| `src/tui/views/start_form.rs` | **Delete** — dead, references removed `InputField` |
| `src/tui/views/report.rs` | **Delete** — `render()` never called, no `Tab::Report` exists |
| `src/tui/views/mod.rs` | Remove three dead `pub mod` declarations |
| `src/tui/app.rs` | Remove 6 dead fields, `load_report()`, `window_to_idx()`, `idx_to_window()`; merge clock anim |
| `src/config.rs` | Remove `KeyboardConfig` struct and `keyboard` field from `AppConfig` |
| `src/tui/themes.rs` | Remove dead `get_current_colors()` function |
| `src/db/mod.rs` | Add `pub mod time` |
| `src/db/time.rs` | **New** — `today_start()`, `current_week_start()`, `rolling_7d_start()` |
| `src/commands/report.rs` | Remove time util re-exports; add `--pomo` flag integration |
| `src/commands/pomo_stats.rs` | Remove from CLI surface (kept as internal helper) |
| `src/tui/ui.rs` | Resolve theme once per frame; pass `&ThemeColors` down; remove `focused: bool` from panel renders; update panel titles |
| `src/tui/views/dashboard.rs` | Remove `panel_focused` calls |
| `src/tui/views/pomodoro.rs` | Accept `&ThemeColors` param |
| `src/tui/views/log.rs` | Accept `&ThemeColors` param |
| `src/tui/views/settings.rs` | Accept `&ThemeColors` param |
| `src/tui/events.rs` | Remove `d/l/s` tab shortcuts; simplify `1/2/3`; remove `focused_panel_idx` mutations |
| `src/tui/handlers_todo.rs` | Replace `→` handler (skip ModeSelector); remove dead `'s'` handler |
| `src/main.rs` | Add `--pomo` to `Report`; remove `PomoStats` command |

---

## Phase 1: Dead Code Removal

### Task 1: Remove orphaned view files

**Files:**
- Delete: `src/tui/views/menu.rs`
- Delete: `src/tui/views/start_form.rs`
- Delete: `src/tui/views/report.rs`
- Modify: `src/tui/views/mod.rs`

- [ ] **Step 1: Verify baseline passes**

```bash
cargo test 2>&1 | tail -5
```
Expected: all tests pass.

- [ ] **Step 2: Delete the three dead view files**

```bash
rm src/tui/views/menu.rs src/tui/views/start_form.rs src/tui/views/report.rs
```

- [ ] **Step 3: Remove their pub mod declarations from views/mod.rs**

Current content of `src/tui/views/mod.rs`:
```rust
pub mod dashboard;
pub mod log;
pub mod menu;
pub mod pomodoro;
pub mod report;
pub mod settings;
pub mod start_form;
```

New content:
```rust
pub mod dashboard;
pub mod log;
pub mod pomodoro;
pub mod settings;
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo check 2>&1
```
Expected: `Finished dev profile` with no errors.

- [ ] **Step 5: Run tests**

```bash
cargo test 2>&1 | tail -5
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "refactor: remove dead view files (menu, start_form, report tabs)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 2: Remove dead App fields and panel focus

**Files:**
- Modify: `src/tui/app.rs`
- Modify: `src/tui/events.rs`
- Modify: `src/tui/ui.rs`
- Modify: `src/tui/views/dashboard.rs`

The following are unused and being removed:
- `App::today_summary` — populated but never read
- `App::terminal_too_small` — never set/read
- `App::report_rows` — only fed the deleted `views/report.rs`
- `App::report_window` — same
- `App::report_selected_window` — same
- `App::focused_panel_idx` — panel focus concept removed
- `App::load_report()` method
- `window_to_idx()` and `idx_to_window()` free functions (only used by `load_report`)

- [ ] **Step 1: Remove fields from the App struct definition in `src/tui/app.rs`**

Remove these lines from the `pub struct App { ... }` block:

```rust
// REMOVE these lines:
    pub today_summary: Vec<(Option<String>, i64)>,
    pub terminal_too_small: bool,
    pub report_rows: Vec<(Option<String>, i64)>,
    // Report state (legacy tab window — kept for backward compat with load_report)
    pub report_window: TimeWindow,
    pub report_selected_window: usize,
    /// Currently focused dashboard panel index (0=Timer/Pomodoro, 1=TODOs, 2=Report).
    /// None means no panel is focused.
    pub focused_panel_idx: Option<usize>,
```

Also remove the `TimeWindow` enum and its helpers. Confirmed: `TimeWindow` is only used in `app.rs` and the already-deleted `views/report.rs`, so removal is safe and unconditional:
- The `TimeWindow` enum (`Today`, `CurrentWeek`, `Last7Days`)
- `window_to_idx()` function
- `idx_to_window()` function

- [ ] **Step 2: Remove field initializers from `App::new()` in `src/tui/app.rs`**

Remove from `Self { ... }`:
```rust
// REMOVE:
            report_window: TimeWindow::Today,
            report_selected_window: 0,
            today_summary: Vec::new(),
            report_rows: Vec::new(),
            terminal_too_small: false,
            focused_panel_idx: None,
```

- [ ] **Step 3: Remove dead methods from App in `src/tui/app.rs`**

Delete the entire `pub fn load_report(...)` method (approximately 15 lines starting with `pub fn load_report`).

Also delete from `load_dashboard()` the line:
```rust
        self.today_summary = session_store::aggregate_by_tag(conn, today_start())?;
```

- [ ] **Step 4: Fix events.rs — remove all `focused_panel_idx` mutations**

In `src/tui/events.rs`, remove all lines that set `focused_panel_idx`:

```bash
grep -n "focused_panel_idx" src/tui/events.rs
```

For each occurrence, remove the line entirely. These are:
- `app.focused_panel_idx = Some(0);` in the `'1'` handler
- `app.focused_panel_idx = Some(1);` in the `'2'` handler
- `app.focused_panel_idx = Some(2);` in the `'3'` handler
- All `app.focused_panel_idx = None;` lines scattered through Tab/BackTab/letter shortcuts

After the fix, the `'1'/'2'/'3'` handlers in `handle_key_event` should read:

```rust
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
            app.active_tab = Tab::Settings;
            return Ok(false);
        }
```

(Remove the `if app.active_tab == Tab::Dashboard { ... } else { ... }` branching entirely.)

- [ ] **Step 5: Fix ui.rs — remove `focused: bool` param from panel renders**

In `src/tui/ui.rs`, change the four affected function signatures by removing the `focused: bool` parameter and removing the `border_style` branch that used it:

**`render_pomodoro_panel`** (around line 388):
```rust
// Before:
pub fn render_pomodoro_panel(frame: &mut Frame, area: Rect, _app: &App, focused: bool) {
    ...
    let border_style = if focused {
        Style::default().fg(tc.panel_focus_border).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(tc.panel_border)
    };

// After:
pub fn render_pomodoro_panel(frame: &mut Frame, area: Rect, app: &App) {
    ...
    let border_style = Style::default().fg(tc.panel_border);
```

**`render_report_panel`** (around line 451):
```rust
// Before:
pub fn render_report_panel(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    ...
    let border_style = if focused { ... } else { ... };

// After:
pub fn render_report_panel(frame: &mut Frame, area: Rect, app: &App) {
    ...
    let border_style = Style::default().fg(tc.panel_border);
```

**`render_timer_zone`** (around line 526):
```rust
// Before:
pub fn render_timer_zone(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    ...
    let border_style = if focused { ... } else { ... };

// After:
pub fn render_timer_zone(frame: &mut Frame, area: Rect, app: &App) {
    ...
    let border_style = Style::default().fg(tc.panel_border);
```

**`render_todo_zone`** (around line 615):
```rust
// Before:
pub fn render_todo_zone(frame: &mut Frame, area: Rect, app: &App, focused: bool) {
    ...
    let border_style = if focused { ... } else { ... };

// After:
pub fn render_todo_zone(frame: &mut Frame, area: Rect, app: &App) {
    ...
    let border_style = Style::default().fg(tc.panel_border);
```

Also update the panel **titles** to remove the now-meaningless numeric shortcuts:
- `" [1] Pomodoro "` → `" Pomodoro "`
- `" [2] TODOs "` → `" TODOs "`
- `" [3] Report "` → `" Report "`

- [ ] **Step 6: Fix dashboard.rs — remove panel_focused calls**

In `src/tui/views/dashboard.rs`, remove the `panel_focused` closure and update all three render calls:

```rust
// REMOVE these lines:
    let panel_focused = |idx: usize| app.focused_panel_idx == Some(idx);

// CHANGE:
    crate::tui::ui::render_timer_zone(frame, main_chunks[0], app, panel_focused(0));
    // to:
    crate::tui::ui::render_timer_zone(frame, main_chunks[0], app);

    crate::tui::ui::render_todo_zone(frame, right_chunks[0], app, panel_focused(1));
    // to:
    crate::tui::ui::render_todo_zone(frame, right_chunks[0], app);

    crate::tui::ui::render_report_panel(frame, right_chunks[1], app, panel_focused(2));
    // to:
    crate::tui::ui::render_report_panel(frame, right_chunks[1], app);
```

Also fix the idle Pomodoro panel call:
```rust
    crate::tui::ui::render_pomodoro_panel(frame, main_chunks[0], app, panel_focused(0));
    // to:
    crate::tui::ui::render_pomodoro_panel(frame, main_chunks[0], app);
```

- [ ] **Step 7: Verify**

```bash
cargo check 2>&1
cargo test 2>&1 | tail -5
```
Expected: no errors, all tests pass.

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "refactor: remove dead App fields (today_summary, report_rows, focused_panel_idx, terminal_too_small)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 3: Remove KeyboardConfig from AppConfig

**Files:**
- Modify: `src/config.rs`

`KeyboardConfig` and its fields (`enable_number_shortcuts`, `enable_letter_shortcuts`) are serialized to config.json but never read by the keyboard handler.

- [ ] **Step 1: Remove from `src/config.rs`**

Delete the `KeyboardConfig` struct entirely:
```rust
// REMOVE:
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardConfig {
    #[serde(default = "default_true")]
    pub enable_number_shortcuts: bool,
    #[serde(default = "default_true")]
    pub enable_letter_shortcuts: bool,
}

fn default_true() -> bool {
    true
}
```

Remove `keyboard` field from `AppConfig`:
```rust
// Before:
pub struct AppConfig {
    pub vim_mode: bool,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub keyboard: KeyboardConfig,
}

// After:
pub struct AppConfig {
    pub vim_mode: bool,
    #[serde(default)]
    pub theme: Option<String>,
}
```

- [ ] **Step 2: Update tests in config.rs that referenced `keyboard`**

In the test block, update the `AppConfig { ... }` literals by removing the `keyboard: KeyboardConfig::default()` field. Example:

```rust
// Before:
let cfg = AppConfig { vim_mode: true, theme: None, keyboard: KeyboardConfig::default() };
// After:
let cfg = AppConfig { vim_mode: true, theme: None };
```

Apply to all test instantiations.

- [ ] **Step 3: Verify and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "refactor: remove unused KeyboardConfig from AppConfig

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 4: Remove dead `get_current_colors` function

**Files:**
- Modify: `src/tui/themes.rs`

- [ ] **Step 1: Remove the function and its test from `src/tui/themes.rs`**

Delete this function:
```rust
// REMOVE:
/// Get the current theme colors (cached)
pub fn get_current_colors() -> ThemeColors {
    // TODO: Integrate with config system to read saved theme
    // For now, use auto-detection
    Theme::auto_detect().colors()
}
```

In the `#[cfg(test)]` block, remove the test `test_get_current_colors_returns_valid_colors`:
```rust
// REMOVE:
    #[test]
    fn test_get_current_colors_returns_valid_colors() {
        let colors = get_current_colors();
        assert_eq!(colors.validate(), Ok(()));
    }
```

- [ ] **Step 2: Verify and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "refactor: remove dead get_current_colors() from tui/themes

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Phase 2: Duplication Elimination

### Task 5: Move time utilities to `db/time.rs`

**Files:**
- New: `src/db/time.rs`
- Modify: `src/db/mod.rs`
- Modify: `src/commands/report.rs`
- Modify: `src/tui/app.rs`
- Modify: `src/tui/report.rs`

Currently `today_start()`, `current_week_start()`, `rolling_7d_start()` live in `commands/report.rs` but are imported by the TUI layer — a cross-layer dependency smell.

- [ ] **Step 1: Write the test for db/time.rs**

Create `src/db/time.rs`:

```rust
use chrono::{Datelike, Duration, Local, TimeZone};

pub fn today_start() -> i64 {
    let now = Local::now();
    let today = now.date_naive().and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&today)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn current_week_start() -> i64 {
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday() as i64;
    let monday = now.date_naive() - Duration::days(days_since_monday);
    let midnight = monday.and_hms_opt(0, 0, 0).expect("valid time");
    Local
        .from_local_datetime(&midnight)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

pub fn rolling_7d_start() -> i64 {
    (chrono::Utc::now() - Duration::seconds(7 * 86400)).timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_start_is_before_now() {
        let ts = today_start();
        assert!(ts > 0);
        assert!(ts <= chrono::Utc::now().timestamp());
    }

    #[test]
    fn today_start_is_midnight_local() {
        let ts = today_start();
        let dt = chrono::Local.timestamp_opt(ts, 0).single().unwrap();
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn current_week_start_is_monday() {
        let ts = current_week_start();
        let dt = chrono::Local.timestamp_opt(ts, 0).single().unwrap();
        assert_eq!(dt.weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn rolling_7d_start_is_7_days_ago() {
        let ts = rolling_7d_start();
        let now = chrono::Utc::now().timestamp();
        let diff = now - ts;
        // Should be approximately 7 days (604800 seconds), within 5s tolerance
        assert!(diff >= 604795 && diff <= 604805, "diff={diff}");
    }
}
```

- [ ] **Step 2: Run the new tests to verify they pass**

```bash
cargo test db::time 2>&1
```
Expected: 4 tests pass.

- [ ] **Step 3: Add `pub mod time` to `src/db/mod.rs`**

```rust
// Add at the top of src/db/mod.rs, with existing pub mods:
pub mod time;
pub mod pomodoro_store;
pub mod session_store;
```

- [ ] **Step 4: Update `src/commands/report.rs` — replace function bodies with re-exports**

Replace the three function definitions with re-exports:

```rust
// Replace the three standalone functions:
pub use crate::db::time::{current_week_start, rolling_7d_start, today_start};
```

(Keep the rest of the file — the `run()` function — unchanged.)

- [ ] **Step 5: Update `src/tui/app.rs` import sites**

Find all uses:
```bash
grep -n "commands::report::" src/tui/app.rs
```

Change every `use crate::commands::report::{...}` import to `use crate::db::time::{...}`. Example:

```rust
// Before:
use crate::commands::report::today_start;
// After:
use crate::db::time::today_start;

// Before:
use crate::commands::report::{current_week_start, rolling_7d_start, today_start};
// After:
use crate::db::time::{current_week_start, rolling_7d_start, today_start};
```

- [ ] **Step 6: Update `src/tui/report.rs` import sites**

`src/tui/report.rs` has two private wrapper functions (around lines 187-194) that delegate to `commands::report`:

```rust
fn today_start_ts() -> i64 {
    crate::commands::report::today_start()
}

fn week_start_ts() -> i64 {
    crate::commands::report::current_week_start()
}
```

Change these to call `db::time` directly:

```rust
fn today_start_ts() -> i64 {
    crate::db::time::today_start()
}

fn week_start_ts() -> i64 {
    crate::db::time::current_week_start()
}
```

- [ ] **Step 7: Verify and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "refactor: move time utilities to db/time.rs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 6: Merge duplicate clock animation methods

**Files:**
- Modify: `src/tui/app.rs`

`advance_clock_anim` and `advance_pomo_clock_anim` are ~50 lines each with identical logic.

- [ ] **Step 1: Add a private helper function in `src/tui/app.rs`**

Add this private function **before** the `impl App` block (or inside it as a private method):

```rust
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
```

- [ ] **Step 2: Replace `advance_clock_anim` body**

```rust
pub fn advance_clock_anim(&mut self, new_str: &str) {
    advance_anim(
        &mut self.clock_curr_str,
        &mut self.clock_prev_str,
        &mut self.clock_anim_frame,
        new_str,
    );
}
```

- [ ] **Step 3: Replace `advance_pomo_clock_anim` body**

```rust
pub fn advance_pomo_clock_anim(&mut self, new_str: &str) {
    advance_anim(
        &mut self.pomo_clock_curr_str,
        &mut self.pomo_clock_prev_str,
        &mut self.pomo_clock_anim_frame,
        new_str,
    );
}
```

- [ ] **Step 4: Verify and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "refactor: extract shared advance_anim helper, removing ~50 lines of duplication

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 7: Cache theme resolution per render frame

**Files:**
- Modify: `src/tui/ui.rs`
- Modify: `src/tui/views/dashboard.rs`
- Modify: `src/tui/views/pomodoro.rs`
- Modify: `src/tui/views/log.rs`
- Modify: `src/tui/views/settings.rs`

Currently `get_colors_for_theme(app.config.theme.as_deref())` is called 9+ times per frame. Resolve once in `render()` and thread the result down.

- [ ] **Step 1: Update `render()` signature and propagate `tc` in `src/tui/ui.rs`**

The top-level `pub fn render(frame, app)` already resolves `tc` at line 53. Now pass it to every sub-function instead of re-resolving. Internal function signatures change:

```rust
// Before:
fn render_tab_bar(frame: &mut Frame, app: &App, area: Rect)
// After:
fn render_tab_bar(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, area: Rect)

// Before:
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect)
// After:
fn render_status_bar(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, area: Rect)

// Before:
fn render_prompt_overlay(frame: &mut Frame, app: &App, label: &str)
// After:
fn render_prompt_overlay(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, label: &str)

// Before (public):
pub fn render_pomodoro_panel(frame: &mut Frame, area: Rect, app: &App)
// After:
pub fn render_pomodoro_panel(frame: &mut Frame, area: Rect, app: &App, tc: &crate::theme::ThemeColors)

// Same pattern for: render_report_panel, render_timer_zone, render_todo_zone, render_controls_zone
```

Remove the `let tc = crate::tui::themes::get_colors_for_theme(...)` line from every internal function body that gets `tc` as a parameter. The line stays only in `pub fn render()` at the top.

Update `render_message_overlay` in ui.rs to stop using `app` for colors — it already has access to `tc` if we pass it, or thread it as a param. Add `tc: &ThemeColors` to it.

- [ ] **Step 2: Update callers in `render()` to pass `&tc`**

```rust
pub fn render(frame: &mut Frame, app: &App) {
    let tc = crate::tui::themes::get_colors_for_theme(app.config.theme.as_deref());

    // Fill background
    frame.render_widget(
        Block::default().style(Style::default().bg(tc.background).fg(tc.foreground)),
        area,
    );

    render_tab_bar(frame, app, &tc, chunks[0]);

    match &app.active_tab {
        Tab::Dashboard => views::dashboard::render(frame, app, &tc, chunks[1]),
        Tab::Log => views::log::render(frame, app, &tc, app.log_page, app.log_selected, chunks[1]),
        Tab::Settings => views::settings::render(frame, app, &tc, chunks[1]),
    }

    render_status_bar(frame, app, &tc, chunks[2]);

    if app.overlay.is_active() {
        render_overlay(frame, app, &tc, area);
    }
    // ... etc
}
```

- [ ] **Step 3: Update view function signatures to accept `tc`**

In `src/tui/views/dashboard.rs`:
```rust
// Before:
pub fn render(frame: &mut Frame, app: &App, area: Rect)
// After:
pub fn render(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, area: Rect)
```

Remove the `let tc = ...` line inside the function. Pass `tc` to all nested render calls that need it. The `render_full_pomodoro_panel` function:
```rust
// Before:
pub fn render_full_pomodoro_panel(frame: &mut Frame, app: &App, area: Rect)
// After:
pub fn render_full_pomodoro_panel(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, area: Rect)
```

In `src/tui/views/log.rs`:
```rust
// Before:
pub fn render(frame: &mut Frame, app: &App, page: usize, selected: usize, area: Rect)
// After:
pub fn render(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, page: usize, selected: usize, area: Rect)
```

In `src/tui/views/settings.rs`:
```rust
// Before:
pub fn render(frame: &mut Frame, app: &App, area: Rect)
// After:
pub fn render(frame: &mut Frame, app: &App, tc: &crate::theme::ThemeColors, area: Rect)
```

In `src/tui/views/pomodoro.rs`:
```rust
// Before:
pub fn render(frame: &mut Frame, timer: &PomodoroTimer, app: &App, area: Rect)
// After:
pub fn render(frame: &mut Frame, timer: &PomodoroTimer, app: &App, tc: &crate::theme::ThemeColors, area: Rect)
```

Update all internal `let tc = ...` removals within each of these files.

Note: `render_message_overlay_pub` in `dashboard.rs` also calls `get_colors_for_theme` — thread `tc` to it as well.

- [ ] **Step 4: Fix the `dashboard.rs` call to `pomodoro::render`**

```rust
// Before:
crate::tui::views::pomodoro::render(frame, timer, app, main_chunks[0]);
// After:
crate::tui::views::pomodoro::render(frame, timer, app, tc, main_chunks[0]);
```

- [ ] **Step 5: Verify and commit**

```bash
cargo check 2>&1
cargo test 2>&1 | tail -5
git add -A && git commit -m "perf: resolve theme colors once per frame, pass &ThemeColors down render stack

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Phase 3: UX Improvements

### Task 8: Simplify keyboard navigation — remove `d/l/s` tab shortcuts

**Files:**
- Modify: `src/tui/events.rs`
- Modify: `src/tui/handlers_todo.rs`
- Modify: `src/tui/ui.rs` (help text + status bar)

**Rationale:** `d/l/s` duplicate `1/2/3` and `Tab`, while `d` additionally conflicts with vim's `dd` delete — requiring fragile conditional logic.

- [ ] **Step 1: Remove `d/l/s` global key handlers from `src/tui/events.rs`**

Find the block in `handle_key_event` that handles letter shortcuts (around lines 72–92) and delete it:

```rust
// REMOVE entirely:
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // In vim mode on Dashboard, 'd' starts 'dd' (delete) — let tab handler process it
            if !(app.config.vim_mode && app.active_tab == Tab::Dashboard) {
                app.active_tab = Tab::Dashboard;
                app.focused_panel_idx = None;
                return Ok(false);
            }
            // Fall through to tab handler for vim 'dd'
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.active_tab = Tab::Log;
            app.focused_panel_idx = None;
            app.load_log(conn)?;
            return Ok(false);
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.active_tab = Tab::Settings;
            app.focused_panel_idx = None;
            return Ok(false);
        }
```

(After Task 2, the `app.focused_panel_idx = None` lines were already gone — remove any remaining `None` assignments in these blocks.)

- [ ] **Step 2: Remove dead `'s'` handler from `src/tui/handlers_todo.rs`**

In `handle_todo_key`, remove this arm:

```rust
// REMOVE:
        KeyCode::Char('s') => {
            app.overlay = crate::tui::app::Overlay::ModeSelector { cursor: 0 };
        }
```

This was the old "start session" shortcut replaced by `'n'` in the Dashboard handler. With global `'s'` removed, this would have been reached on Dashboard, causing `'s'` to open a session instead of going to Settings. Removing it here prevents that regression.

- [ ] **Step 3: Update the help text constant in `src/tui/ui.rs`**

Find the `HELP_TEXT` constant and update the global navigation section:

```rust
const HELP_TEXT: &str = "\
Global
  1/2/3     Dashboard/Log/Settings
  Tab       Next tab
  ?         Show this help
  q         Quit
  Esc       Clear message

// ... rest stays the same
";
```

(Remove `d/l/s` line, replace with `1/2/3`.)

- [ ] **Step 4: Update status bar hints in `src/tui/ui.rs` if they mention `d/l/s`**

In `render_status_bar`, check the hint strings for Tab::Dashboard, Tab::Log, Tab::Settings — remove any references to `[d]`, `[l]`, `[s]` as tab navigation shortcuts.

- [ ] **Step 5: Update tab bar labels in `render_tab_bar`**

Currently shows `[d]Dashboard`, `[l]Log`, `[s]Settings`. Change to:

```rust
let tabs = [
    (Tab::Dashboard, "Dashboard"),
    (Tab::Log, "Log"),
    (Tab::Settings, "Settings"),
];
```

- [ ] **Step 6: Write a test verifying `'d'` now reaches the Dashboard todo handler**

In `src/tui/events.rs` test block (already has test helpers), add:

```rust
#[test]
fn d_key_with_vim_mode_does_not_switch_tab() {
    use crate::config::AppConfig;
    use crate::tui::app::App;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut app = App::new(false, AppConfig { vim_mode: true, theme: None });
    app.active_tab = Tab::Dashboard;
    // Pressing 'd' should NOT switch to Dashboard (already there) or do anything global
    // It should fall through to the dashboard handler for vim dd
    // We can't fully test dd without a db, but we verify the tab doesn't change
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    let conn = crate::db::open_db_at(&std::path::PathBuf::from(":memory:")).unwrap();
    let result = handle_key_event(&mut app, &conn, key);
    assert!(result.is_ok());
    assert_eq!(app.active_tab, Tab::Dashboard);
}
```

- [ ] **Step 7: Verify and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "feat: remove d/l/s tab shortcuts; 1/2/3 always navigate tabs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 9: Streamline TODO → Session flow (skip task name prompt)

**Files:**
- Modify: `src/tui/app.rs`
- Modify: `src/tui/handlers_todo.rs`
- Modify: `src/tui/events.rs`

Current: `→` on a TODO → ModeSelector → task name prompt → tag prompt (5+ interactions).
New: `→` on a TODO → tag prompt only (1 prompt, Enter to skip).

- [ ] **Step 1: Add the new `StartSessionFromTodo` PromptAction variant in `src/tui/app.rs`**

In the `PromptAction` enum, add:

```rust
pub enum PromptAction {
    AddTodo,
    StartSession,
    StartSessionTag { task: String },
    RenameSession { id: i64 },
    StartPomodoroName,
    StartPomodoroTag { task: String },
    /// Start a freeform session using the selected TODO's title, skipping name prompt.
    /// Only prompts for an optional tag.
    StartSessionFromTodo { task: String, todo_id: u64 },
}
```

- [ ] **Step 2: Write the failing test for the new flow**

Add to the test block in `src/tui/events.rs`:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn right_arrow_on_selected_todo_opens_tag_prompt_not_mode_selector() {
        use crate::config::AppConfig;
        use crate::models::todo::Todo;
        use crate::tui::app::{App, Overlay, PromptAction};
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new(false, AppConfig::default());
        app.todos = vec![Todo {
            id: 1,
            title: "Write tests".to_string(),
            status: "active".to_string(),
            created_at: 0,
            completed_at: None,
        }];
        app.selected_todo_idx = Some(0);
        app.active_tab = super::Tab::Dashboard;

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let conn = crate::db::open_db_at(&std::path::PathBuf::from(":memory:")).unwrap();
        let _ = handle_key_event(&mut app, &conn, key);

        // Should be a Prompt overlay (tag prompt), NOT a ModeSelector
        assert!(
            matches!(
                &app.overlay,
                Overlay::Prompt { action: PromptAction::StartSessionFromTodo { .. }, .. }
            ),
            "Expected StartSessionFromTodo prompt, got {:?}",
            app.overlay
        );
    }
}
```

- [ ] **Step 3: Run test to confirm it fails**

```bash
cargo test right_arrow_on_selected_todo 2>&1 | tail -20
```
Expected: FAIL (StartSessionFromTodo variant doesn't exist yet, or overlay is ModeSelector).

- [ ] **Step 4: Update `src/tui/handlers_todo.rs` — change the `→` handler**

```rust
// Before:
        KeyCode::Right if app.selected_todo_idx.is_some() => {
            app.overlay = crate::tui::app::Overlay::ModeSelector { cursor: 0 };
        }

// After:
        KeyCode::Right if app.selected_todo_idx.is_some() => {
            if let Some(idx) = app.selected_todo_idx {
                if let Some(todo) = app.todos.get(idx) {
                    let task = todo.title.clone();
                    let todo_id = todo.id;
                    app.open_prompt(
                        &format!("Tag for «{}» (optional, Enter to skip):", task),
                        "",
                        crate::tui::app::PromptAction::StartSessionFromTodo { task, todo_id },
                    );
                }
            }
        }
```

- [ ] **Step 5: Handle `StartSessionFromTodo` in `src/tui/events.rs` overlay handler**

In `handle_overlay_prompt`, add the new match arm inside the `TextInputEvent::Submit(value)` block:

```rust
                PromptAction::StartSessionFromTodo { task, todo_id } => {
                    let tag_opt = if value.is_empty() { None } else { Some(value) };
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
                            session_store::insert_session_with_todo(
                                conn,
                                &task,
                                tag_opt.as_deref(),
                                Some(todo_id),
                            )?;
                            app.message = Some(MessageOverlay::success(format!(
                                "Session started: \"{}\"",
                                task
                            )));
                            let _ = app.load_dashboard(conn);
                        }
                    }
                    app.overlay = Overlay::None;
                }
```

- [ ] **Step 6: Run the test to confirm it passes**

```bash
cargo test right_arrow_on_selected_todo 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 7: Run all tests and commit**

```bash
cargo test 2>&1 | tail -5
git add -A && git commit -m "feat: streamline TODO→session flow — right arrow opens tag prompt directly

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task 10: Unify `report` and `pomo-stats` CLI commands

**Files:**
- Modify: `src/main.rs`
- Modify: `src/commands/report.rs`

**Goal:** `focus report --pomo [--today|--week]` replaces `focus pomo-stats`. The `PomoStats` command variant is removed from the CLI.

- [ ] **Step 1: Write a test validating the pomo flag is wired up**

Add to an integration test or to `src/commands/report.rs` test block:

```rust
// In src/commands/report.rs tests or a new integration test:
#[test]
fn report_pomo_flag_calls_pomo_run() {
    // This is an integration test: create a temp DB and ensure
    // run_with_pomo(conn, false, false, true) calls pomo_stats output path.
    // We test it doesn't panic and returns Ok.
    use tempfile::NamedTempFile;
    let f = NamedTempFile::new().unwrap();
    let conn = crate::db::open_db_at(f.path()).unwrap();
    // No pomodoro sessions — should print "No Pomodoro sessions today." and return Ok
    let result = super::run(&conn, false, false, true);
    assert!(result.is_ok());
}
```

(This test will fail until we add the `pomo` parameter to `run()`.)

- [ ] **Step 2: Update `src/main.rs` — add `--pomo` to Report, remove PomoStats**

In the `Commands` enum:

```rust
    /// Show time aggregated by tag or Pomodoro statistics
    Report {
        /// Show today's sessions only
        #[arg(long, conflicts_with = "week")]
        today: bool,
        /// Show last 7 rolling days
        #[arg(long, conflicts_with = "today")]
        week: bool,
        /// Show Pomodoro statistics instead of session time report
        #[arg(long)]
        pomo: bool,
    },
```

Remove `PomoStats` variant entirely.

In the `run()` dispatch:

```rust
        Commands::Report { today, week, pomo } => commands::report::run(&conn, today, week, pomo)?,
```

Remove the `Commands::PomoStats { today, week } => ...` arm.

- [ ] **Step 3: Update `src/commands/report.rs` — add `pomo` parameter to `run()`**

Change the function signature:

```rust
pub fn run(conn: &Connection, today: bool, week: bool, pomo: bool) -> Result<()> {
    if pomo {
        return crate::commands::pomo_stats::run(conn, today, week);
    }
    // ... existing report logic unchanged ...
```

- [ ] **Step 4: Run the failing test to confirm it now passes**

```bash
cargo test report_pomo_flag 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 5: Verify and commit**

```bash
cargo test 2>&1 | tail -5
cargo clippy -- -D warnings 2>&1 | tail -10
git add -A && git commit -m "feat: consolidate pomo-stats into report --pomo; remove PomoStats command

BREAKING CHANGE: 'focus pomo-stats' removed. Use 'focus report --pomo' instead.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Final Verification

- [ ] **Run full test suite**

```bash
cargo test 2>&1
```
Expected: All tests pass (baseline was ~302 tests).

- [ ] **Run clippy**

```bash
cargo clippy -- -D warnings 2>&1
```
Expected: No warnings.

- [ ] **Run fmt check**

```bash
cargo fmt --check 2>&1
```
Expected: No formatting issues. If any, run `cargo fmt` then re-commit.

- [ ] **Spot-check dead code is gone**

```bash
grep -rn "today_summary\|terminal_too_small\|focused_panel_idx\|report_rows\|report_window\|KeyboardConfig\|get_current_colors\|pomo-stats" src/ --include="*.rs"
```
Expected: 0 matches.

- [ ] **Verify 1/2/3 tab navigation works in built binary**

```bash
cargo build --release 2>&1 | tail -3
```
Expected: `Finished release profile`.
