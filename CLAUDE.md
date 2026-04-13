# focus Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-13

## Active Technologies
- Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new) (003-focus-tui-dashboard)
- SQLite WAL mode via existing `db::open_db()`; no schema changes (003-focus-tui-dashboard)
- Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, dirs 5, chrono 0.4, colored 2, anyhow 1, thiserror 1; **new**: ctrlc 3 (004-tui-session-controls)
- SQLite WAL mode via existing `db::open_db()` — no schema changes (004-tui-session-controls)
- Rust stable 1.77+ + clap 4, rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3. New: `toml = "0.8"`. (005-pomodoro-timer-mode)
- SQLite WAL mode via existing `db::open_db()`. Two additive changes: `ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform'`; `CREATE TABLE IF NOT EXISTS pomodoro_stats (...)`. (005-pomodoro-timer-mode)
- Rust 1.77+ (stable) + Existing: clap 4, rusqlite 0.31 (bundled), chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ratatui 0.29, crossterm 0.28, ctrlc 3, toml 0.8. New: `npm` CLI tooling, GitHub Actions (CI/CD) (006-npm-package-publish)
- SQLite (no schema changes) (006-npm-package-publish)
- Rust stable 1.77+ (existing project standard) + clap 4 (CLI), rusqlite 0.31 (`bundled` feature), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3, toml 0.8, serde_json 1 (007-ui-refresh)
- SQLite with WAL mode (existing db::open_db() wrapper); new `todos` table and `mode` column on sessions (007-ui-refresh)
- Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, toml 0.8, dirs 5, anyhow 1, thiserror 1 (009-tui-polish-fixes)
- SQLite WAL at `~/.local/share/focus/focus.db`; settings JSON at `~/.config/focus/config.json` (already wired via `src/config.rs`) (009-tui-polish-fixes)

- Rust stable (1.77+) + clap 4 (derive API), rusqlite 0.31 (`bundled` feature), chrono 0.4, colored 2.x, dirs 5.x, thiserror 1.x, anyhow 1.x (001-focus-cli-tracker)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable (1.77+): Follow standard conventions

## Recent Changes
- 009-tui-polish-fixes: 7 TUI polish fixes (US1–US7); `focus config get/set` CLI; vim dd/gg/G; Delete/Backspace for todo delete; config persistence tests; Theme::from_name/resolve helpers
- 009-tui-polish-fixes: Added Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, toml 0.8, dirs 5, anyhow 1, thiserror 1
- 007-ui-refresh: Added Rust stable 1.77+ (existing project standard) + clap 4 (CLI), rusqlite 0.31 (`bundled` feature), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3, toml 0.8, serde_json 1
- 006-npm-package-publish: Added Rust 1.77+ (stable) + Existing: clap 4, rusqlite 0.31 (bundled), chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ratatui 0.29, crossterm 0.28, ctrlc 3, toml 0.8. New: `npm` CLI tooling, GitHub Actions (CI/CD)


<!-- MANUAL ADDITIONS START -->

## 009: TUI Polish Fixes (2026-04)

### What Changed

**Keyboard conflict resolved (US1 + US2)**:
- `d` in Log tab no longer conflicts with Dashboard navigation
- `Delete`/`Backspace` now trigger deletion (todos in Dashboard, no-op in Log)
- In vim mode, single `d` starts a `dd` command window instead of navigating

**Vim mode multi-key commands (US7)**:
- `dd` within 1s window → `DeleteItem` (delete selected todo)
- `gg` within 1s window → `JumpTop` (jump to first item)
- `G` → `JumpBottom` (jump to last item)
- `KeyHandler` now uses `&mut self` with `pending_d`/`pending_g` Instant fields

**Config CLI (US6)**:
- `focus config get theme` — print current theme
- `focus config set theme onedark|material|light|dark|auto`
- `focus config get vim-mode` / `focus config set vim-mode on|off`
- Theme config is read by TUI at startup via `app.config.theme.as_deref()`

**Theme helpers (US6)**:
- `Theme::from_name(s)` — parse theme name string to enum variant
- `Theme::name()` — canonical name string for a variant
- `Theme::resolve(Option<&str>)` — resolve config value to Theme (with auto-detect fallback)

**Settings persistence (US5)**:
- Vim mode toggle already called `save_config` immediately — verified and tested

### Key Files Added/Changed in 009

| File | Purpose |
|---|---|
| `src/commands/config.rs` | `focus config get/set` CLI subcommand |
| `src/tui/keyboard.rs` | `KeyHandler` with pending multi-key state; `DeleteItem`/`JumpTop`/`JumpBottom` |
| `src/tui/handlers_todo.rs` | Delete/Backspace → todo deletion |
| `src/tui/app.rs` | `has_active_pomodoro()`, `save_config_now()` helpers |
| `src/theme/mod.rs` | `from_name()`, `name()`, `resolve()` on `Theme` |
| `tests/integration/test_config_persistence.rs` | Config round-trip tests (7 tests) |
| `tests/integration/test_keyboard_bindings.rs` | US1 + US7 keyboard binding tests |
| `tests/integration/test_timer_display.rs` | US3 block-char + US4 freeze guard tests |

### Commands

```bash
cargo test                        # 302 tests (140 lib + 135 integration + 27 unit)
cargo clippy -- -D warnings       # zero warnings policy
cargo fmt                         # required before every commit
focus config set theme onedark    # set theme via CLI
focus config get theme            # read current theme
focus config set vim-mode on      # enable vim mode via CLI
```

<!-- MANUAL ADDITIONS END -->

## 008: Dashboard UI/UX Enhancements (2026-04)

### Architecture Patterns

**Theme system** (`src/theme/`):
- Each theme is a `FocusTheme` struct in its own file (`dark.rs`, `light.rs`, `material.rs`, `onedark.rs`)
- Themes expose typed color fields (`primary`, `secondary`, `success`, `warning`, `error`, `timer_digit`, `timer_separator`)
- `src/theme/mod.rs` resolves theme name → struct; falls back to `dark` on unknown name
- `app.theme_name: String` stores the chosen theme; `tui::themes::load_theme(&name)` returns the struct
- `NO_COLOR` is checked once at startup and stored in `App.no_color`; all render paths guard on it

**Flip-clock timer display** (`src/tui/timer_display.rs`):
- Renders digits as 3×5 block characters using Unicode box-drawing / block elements
- `TimerDisplay::render_into(frame, area, secs, theme, no_color)` is the single entry point
- Two usage sites: `views/dashboard.rs` (running session) and `views/pomodoro.rs` (pomodoro timer)
- When `no_color = true`, all Color fields are ignored and default `Style::default()` is used

**Keyboard context system** (`src/tui/keyboard.rs`):
- `KeyboardContext` enum: `Normal`, `TodoInput`, `ConfirmQuit`
- Stored in `App.keyboard_context`; `handle_key_event` dispatches on context first, then tab
- Letter shortcuts `d/l/p/r` switch tabs from `Normal` context only
- ESC in `TodoInput` cancels input without committing; ESC in `Normal` shows quit confirm

**Report panel** (`src/tui/report.rs`):
- Queries sessions, pomodoro_stats, and todos tables
- `ReportMetrics` struct holds all aggregated values; built once per tab-switch + periodic refresh
- Bar chart uses ratatui `BarChart` widget; summary section uses `Table`

**TUI event loop** (`src/tui/mod.rs`):
- `event::poll(Duration::from_millis(250))` — key events return immediately on press (<1ms latency)
- 250ms timeout drives timer tick and UI refresh; well under perceptible lag threshold
- `last_tick` advances by whole elapsed seconds; pomodoro timer uses `tick_secs(elapsed, conn)`

### Key Files Added in 008

| File | Purpose |
|---|---|
| `src/theme/mod.rs` | Theme loader + `FocusTheme` struct |
| `src/theme/{dark,light,material,onedark}.rs` | Theme implementations |
| `src/tui/timer_display.rs` | Flip-clock digit renderer |
| `src/tui/report.rs` | Report panel metrics + rendering |
| `src/tui/keyboard.rs` | Keyboard context switching |
| `src/tui/themes.rs` | Theme integration helpers |
| `tests/integration/test_theme_loading.rs` | Theme + NO_COLOR tests |
| `tests/integration/test_pomodoro_panel.rs` | Pomodoro panel state tests |

### Commands

```bash
cargo test                        # 142 tests (115 unit + 27 integration)
cargo clippy -- -D warnings       # zero warnings policy
cargo fmt                         # required before every commit
NO_COLOR=1 focus ui               # verify ANSI-free output (Principle IV)
```

<!-- MANUAL ADDITIONS END -->
