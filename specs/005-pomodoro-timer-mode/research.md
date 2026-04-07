# Research: Integrated Pomodoro Timer Mode

## Decision 1: Timer Implementation Strategy

**Decision**: Use `std::time::Instant` for elapsed-time tracking (not wall-clock drift).  
**Rationale**: `Instant` is monotonic and not affected by system clock adjustments (NTP, DST). Sleep drift is tolerable for a 25-minute timer. When the machine sleeps and wakes, `elapsed()` will report the full wall time — we use this to detect skipped phases (if elapsed > phase duration, advance as many phases as needed).  
**Alternatives considered**:
- `chrono::Utc::now()` — affected by NTP/clock skew adjustments; less reliable.  
- `tokio` async timers — adds async runtime dependency; violates YAGNI given no other async need.

## Decision 2: TOML Config Crate

**Decision**: Add `toml = "0.8"` to Cargo.toml for `~/.config/focus/pomodoro.toml` parsing.  
**Rationale**: Spec mandates `.toml` format. The `toml` crate is stable, minimal, and pairs with serde derive which is already a project dependency.  
**Alternatives considered**:
- Re-use JSON (`serde_json`) with renamed file — contradicts spec requirement.  
- INI-style hand-parser — brittle and non-standard.

## Decision 3: CLI Pomodoro Rendering

**Decision**: For `focus start --pomodoro`, use crossterm raw mode + inline countdown display with `\r` carriage return to refresh the status line. Single-key input via `crossterm::event::poll` handles P/S/Q in-loop.  
**Rationale**: crossterm is already a project dependency. Raw mode gives immediate key response without Enter. Clearing + redrawing is cheap for a single status line.  
**Alternatives considered**:
- Full TUI (ratatui) for CLI — overkill; TUI mode is already the P2 story.  
- print + sleep loop without raw mode — no live key handling; user must Ctrl+C.

## Decision 4: Desktop Notifications

**Decision**: Implement via `std::process::Command` subprocess:
- macOS: `osascript -e 'display notification "..." with title "Focus"'`
- Linux: `notify-send "Focus" "..."`  
- Windows: skipped for now (no common CLI tool available cross-distro; out of scope per spec Assumptions).  
Wrap in a fire-and-forget fn that logs errors to stderr but never panics.  
**Rationale**: No new Rust crate needed; approach matches spec's "fall back gracefully if unavailable" requirement.  
**Alternatives considered**:
- `notify-rust` crate — adds D-Bus dependency on Linux; breaks Principle I (static binary).
- `tauri-plugin-notification` — not applicable for CLI.

## Decision 5: Pomodoro State Machine Placement

**Decision**: `src/pomodoro/timer.rs` holds a `PomodoroTimer` struct with a pure state machine (no I/O) so it can be tested without a TUI or crossterm.  
**Rationale**: Separates timing logic from rendering and DB persistence; enables unit testing of phase transitions, pause/resume, abandonment detection.  
**Alternatives considered**:
- Embedding state in `App` struct — couples timer logic to TUI; not testable in isolation.

## Decision 6: Schema Migration Strategy

**Decision**: Add `mode TEXT NOT NULL DEFAULT 'freeform'` column to `sessions` via `ALTER TABLE` in `open_db_at`. Add new `pomodoro_stats` table as `CREATE TABLE IF NOT EXISTS`.  
**Rationale**: Both are additive, backward-compatible SQLite migrations. `ALTER TABLE ADD COLUMN` with a literal DEFAULT is handled atomically by SQLite; no data migration script needed. Existing records get `mode = 'freeform'` automatically.  
**Alternatives considered**:
- Versioned migration table — adds complexity; unnecessary for two additive changes.
- Separate database file — contradicts FR-027 (sessions must appear in `focus log`).

## Decision 7: Pomodoro Stats Tracking

**Decision**: Per-day stats (`pomodoro_stats`) keyed on `date TEXT` (ISO 8601 local date, e.g., "2026-04-06"). Incremented atomically via `INSERT OR REPLACE` upsert pattern.  
**Rationale**: Simple, queryable, avoids complex aggregate queries at stats display time. Date is local (not UTC) so stats match the user's calendar day.  
**Alternatives considered**:
- Derive stats purely from `sessions` table on read — requires complex GROUP BY with timezone conversion; harder to count abandonments (abandoned phases are never written to sessions).

## Decision 8: Pause Timeout Auto-Abandon

**Decision**: Track `pause_started_at: Option<Instant>` in `PomodoroTimer`. On every tick, if paused and elapsed-since-pause > 60 minutes, emit a `TimerEvent::AutoAbandoned` to the caller.  
**Rationale**: Spec FR-010 requires this; keeping it in the state machine makes it testable.

## Decision 9: `+5` Extend Key

**Decision**: Use `KeyCode::Char('+')` for extend (no modifier). Document as `[+]` in shortcuts (not `[+5]` as a single key). The action adds 300 seconds to the current phase's `remaining_secs`.  
**Rationale**: `[+5]` as a key combo is not a standard crossterm keycode; `+` is simpler and common in TUI tools. The help text says "press + to extend 5 min".

## Decision 10: TUI PomodoroView Integration

**Decision**: Add a new `Tab::Pomodoro` variant that the TUI switches to when a Pomodoro session is active. The existing dashboard hides the "Start Session" shortcut and shows "Start Pomodoro [P]" when no session is active.  
**Rationale**: Keeps Pomodoro timer isolated from dashboard clutter; allows full-screen timer view with progress bar. Tab label updates to "🍅 Pomodoro" when active.  
**Alternatives considered**:
- Embed timer in Dashboard — tight coupling; hard to show progress bar and all required fields in dashboard's existing layout.
