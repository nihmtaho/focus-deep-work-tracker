# Tasks: Integrated Pomodoro Timer Mode

**Input**: Design documents from `/specs/005-pomodoro-timer-mode/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓

**TDD**: All implementation tasks are preceded by their test tasks (Principle II — non-negotiable).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1–US10)
- Exact file paths are included in every task description

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies, scaffold new module structure, create stub files. No behavioural logic — just compiling scaffolding.

- [X] T001 Add `toml = "0.8"` dependency to `Cargo.toml`
- [X] T002 Create `src/pomodoro/mod.rs`, `src/pomodoro/config.rs`, `src/pomodoro/timer.rs`, `src/pomodoro/notify.rs` as empty modules; declare `pub mod pomodoro;` in `src/lib.rs`
- [X] T003 [P] Create `src/models/pomodoro.rs` with stub types: `PomodoroPhase` enum (Work, Break, LongBreak) and `PomodoroStats` struct; declare in `src/models/mod.rs`
- [X] T004 [P] Create `src/db/pomodoro_store.rs` as empty module with stub fns returning `Ok(())`; declare `pub mod pomodoro_store;` in `src/db/mod.rs`
- [X] T005 [P] Create `src/commands/pomo_stats.rs` with stub `pub fn run(...) -> Result<()>` returning `Ok(())`; declare in `src/commands/mod.rs`
- [X] T006 [P] Add `mod test_pomodoro_schema;` to `tests/integration.rs`; create empty `tests/integration/test_pomodoro_schema.rs`
- [X] T007 Run `cargo build` — must compile with zero errors before proceeding

**Checkpoint**: All new files compile; `cargo build` green ✓

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: DB schema, core domain types, timer state machine, config resolver, and stats store. Everything US1–US10 depends on. **No user story work begins until this phase passes `cargo test`.**

### 2a — DB Schema Migration

- [X] T008 Write failing tests in `tests/integration/test_pomodoro_schema.rs`: verify `sessions` table has `mode` column with default `'freeform'`; verify `pomodoro_stats` table exists with correct columns (date TEXT PK, completed INT, abandoned INT, work_minutes INT, break_minutes INT); verify existing rows created before migration have `mode='freeform'`
- [X] T009 Implement DB migration in `src/db/mod.rs` inside `open_db_at`: extend the `execute_batch` string to add `ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform'` (guard with `PRAGMA table_info(sessions)` check to make the migration idempotent) and `CREATE TABLE IF NOT EXISTS pomodoro_stats (date TEXT PRIMARY KEY, completed INTEGER NOT NULL DEFAULT 0, abandoned INTEGER NOT NULL DEFAULT 0, work_minutes INTEGER NOT NULL DEFAULT 0, break_minutes INTEGER NOT NULL DEFAULT 0)` and `CREATE INDEX IF NOT EXISTS idx_pomodoro_stats_date ON pomodoro_stats(date)`
- [X] T010 Run `cargo test test_pomodoro_schema` — all schema tests must pass

### 2b — Session Model + Store Update

- [X] T011 [P] Write failing unit tests inline in `src/models/session.rs` `#[cfg(test)]`: verify `Session` struct has `mode: String` field; verify `row_to_session` returns correct mode from row data
- [X] T012 [P] Update `Session` struct in `src/models/session.rs` to add `pub mode: String`; update `row_to_session` helper signature in `src/db/session_store.rs` to accept `mode: String` as 6th parameter
- [X] T013 [P] Update all SQL SELECT queries in `src/db/session_store.rs` to include `, mode` at column index 5; update `query_map` closures to `row.get::<_, String>(5)?`; update `insert_session` to accept `mode: &str` parameter (default `"freeform"` for all callers); update all callers in `src/commands/start.rs` and `src/tui/events.rs` to pass `"freeform"`
- [X] T014 [P] Write failing tests in `src/db/session_store.rs` `#[cfg(test)]`: verify `insert_session` with mode `"pomodoro"` stores and retrieves correctly; verify `list_sessions` returns mode for each row; verify `list_all_completed` returns mode field
- [X] T015 Run `cargo test` — all existing session_store tests plus new mode tests must pass; no regressions

### 2c — PomodoroConfig

- [X] T016 [P] Write failing unit tests inline in `src/pomodoro/config.rs` `#[cfg(test)]`: test default values (work=25, break=5, long_break=15, long_break_after=4); test TOML file parsing with custom `work_duration_mins = 45`; test `FOCUS_POMODORO_WORK=30` env var overrides file value; test CLI flag values override env vars; test `validate()` rejects work=0, work=121, break=0, break=61, long_break=0, long_break=61
- [X] T017 [P] Implement `PomodoroConfig` struct and builder in `src/pomodoro/config.rs`: fields `work_duration_mins: u32`, `break_duration_mins: u32`, `long_break_duration_mins: u32`, `long_break_after: u32`; `impl Default` with (25, 5, 15, 4); `load_from_file(path: &Path) -> Self` (TOML via `toml` crate, silently returns default on missing/bad file); `apply_env(&mut self)` reads `FOCUS_POMODORO_{WORK,BREAK,LONG_BREAK,LONG_BREAK_AFTER}` env vars; `apply_cli_flags(&mut self, work: Option<u32>, break_: Option<u32>, long_break: Option<u32>, after: Option<u32>)` overrides fields; `validate(&self) -> Result<()>` checks ranges with clear error messages; `pomodoro_config_path() -> PathBuf` returns `~/.config/focus/pomodoro.toml`
- [X] T018 Run `cargo test` — all config tests pass

### 2d — PomodoroTimer State Machine

- [X] T019 [P] Write failing unit tests inline in `src/pomodoro/timer.rs` `#[cfg(test)]`: test initial state is Work phase with configured duration remaining; test `tick_secs(1)` decrements `remaining_secs` by 1; test `tick_secs(remaining)` emits `PhaseComplete { from: Work, to: Break, .. }` and transitions to break; test 4th consecutive work completion transitions to `LongBreak` instead of `Break`; test `pause()` sets `paused = true`; test ticking while paused does NOT decrement remaining_secs; test `resume()` continues from frozen remaining_secs; test `tick_secs(3601)` while paused emits `AutoAbandoned { completed: 0 }`; test `skip_break()` in Break phase immediately transitions to Work; test `skip_break()` in Work phase is a no-op; test `extend()` adds 300 to remaining_secs; test elapsed real-time > phase duration on wake triggers correct phase advances
- [X] T020 [P] Define `TimerEvent` enum in `src/pomodoro/timer.rs`: variants `Tick`, `PhaseComplete { from: PomodoroPhase, to: PomodoroPhase, work_saved: bool }`, `AutoAbandoned { completed: u32 }`, `Paused`, `Resumed`; derive `Debug, Clone`
- [X] T021 [P] Implement `PomodoroTimer` struct in `src/pomodoro/timer.rs`: fields `phase: PomodoroPhase`, `completed_work: u32`, `remaining_secs: u64`, `paused: bool`, `pause_accumulated_secs: u64`, `pause_started_secs: Option<u64>` (monotonic secs via `std::time::Instant` snapshot), `config: PomodoroConfig`, `task: String`, `tag: Option<String>`; methods `new(task, tag, config) -> Self`; `tick_secs(&mut self, delta_secs: u64, conn: &Connection) -> Result<Vec<TimerEvent>>` — handles phase transitions, saves completed work-phase session to DB via `session_store::insert_session_with_times`, increments pomodoro_store stats; `pause(&mut self)`; `resume(&mut self)`; `skip_break(&mut self)`; `extend(&mut self)`; `is_in_work_phase(&self) -> bool`; `format_remaining(&self) -> String` returns "MM:SS"; `phase_label(&self) -> &str` returns emoji + name
- [X] T022 Add `pub fn insert_session_with_times(conn: &Connection, task: &str, tag: Option<&str>, mode: &str, start_epoch: i64, end_epoch: i64) -> Result<i64>` to `src/db/session_store.rs` — inserts a completed session with explicit start/end times and returns the inserted row id; write inline test verifying the returned id is correct
- [X] T023 Run `cargo test` — all timer state machine tests pass

### 2e — PomodoroStore

- [X] T024 [P] Write failing integration tests in `tests/integration/test_pomodoro_schema.rs` (extend file): test `increment_completed(conn, "2026-04-06", 25, 5)` upserts row and `get_stats_for_date` returns completed=1; test calling it twice gives completed=2; test `increment_abandoned(conn, "2026-04-06")` increments abandoned field; test `get_stats_range` with 3-date dataset returns all 3 rows in ascending order; test `calculate_streak` with 3 consecutive days of completed>0 returns 3; test gap in days resets streak; test empty range returns streak=0 without error
- [X] T025 [P] Implement `src/db/pomodoro_store.rs`: `increment_completed(conn: &Connection, date: &str, work_mins: u32, break_mins: u32) -> Result<()>` using `INSERT INTO pomodoro_stats(date, completed, work_minutes, break_minutes) VALUES(?,1,?,?) ON CONFLICT(date) DO UPDATE SET completed=completed+1, work_minutes=work_minutes+excluded.work_minutes, break_minutes=break_minutes+excluded.break_minutes`; `increment_abandoned(conn: &Connection, date: &str) -> Result<()>` analogous upsert; `get_stats_for_date(conn: &Connection, date: &str) -> Result<PomodoroStats>` returns zeroed struct if no row; `get_stats_range(conn: &Connection, from_date: &str, to_date: &str) -> Result<Vec<PomodoroStats>>` ordered ASC; `calculate_streak(stats: &[PomodoroStats]) -> u32` counts consecutive tail days with completed>0; `calculate_best_streak(stats: &[PomodoroStats]) -> u32`; `today_local_date() -> String` returns ISO 8601 local date
- [X] T026 Run `cargo test` — all pomodoro_store tests pass; `cargo build` green

**Checkpoint**: All foundational tests green; `cargo test` clean ✓

---

## Phase 3: User Story 1 — CLI Pomodoro Session (Priority: P1) 🎯 MVP

**Goal**: `focus start --pomodoro "task"` starts a live CLI countdown, cycles through work/break phases, saves each completed phase, and stops cleanly on Q or Ctrl+C.

**Independent Test**: Run `focus start --pomodoro "refactor auth module" --work 1 --break 1`; observe live countdown; after 1 minute a break phase starts automatically; press Q; output shows "1 pomodoro completed, 0 abandoned"; `focus log` shows 1 record with mode=pomodoro.

- [X] T027 [US1] Write failing integration tests in `tests/integration/test_pomo_cli.rs`: test validation error when `--work 0`; test validation error when `--work 121`; test validation error when `--break 61`; test `--work 1 --break 1` session saves completed phase to `sessions` table with `mode='pomodoro'`; test `focus stop` while pomodoro-mode session is active prints count message and increments abandoned if mid-phase; test `focus log` output includes mode column after a pomodoro session; add `mod test_pomo_cli;` to `tests/integration.rs`
- [X] T028 [US1] Add to `Start` variant in `src/main.rs`: `#[arg(long)] pomodoro: bool`, `#[arg(long)] work: Option<u32>`, `#[arg(long, name = "break")] break_mins: Option<u32>`, `#[arg(long)] long_break: Option<u32>`, `#[arg(long)] long_break_after: Option<u32>`; dispatch to `commands::start::run_pomodoro` when `pomodoro` flag is set
- [X] T029 [US1] Implement `run_pomodoro(conn: &Connection, task: String, tag: Option<String>, work: Option<u32>, break_mins: Option<u32>, long_break: Option<u32>, long_break_after: Option<u32>) -> Result<()>` in `src/commands/start.rs`: load PomodoroConfig from file; apply env vars; apply CLI flags; call `config.validate()`; check no active session; enter crossterm raw mode; instantiate `PomodoroTimer`; run event loop: poll 100ms for key events (P=pause/resume, S=skip_break, Q=stop, Ctrl+C=stop); call `timer.tick_secs(delta, conn)` each iteration; print status line with `\r\x1b[K` clear-line prefix; on `PhaseComplete` call `pomodoro::notify::send_notification`; on Q/Ctrl+C call `pomodoro_store::increment_abandoned` if mid-work-phase; restore terminal; print summary
- [X] T030 [US1] Implement `src/pomodoro/notify.rs`: `pub fn send_notification(title: &str, body: &str)` detects OS via `cfg!(target_os = "macos")` / `cfg!(target_os = "linux")`; spawns `osascript -e 'display notification "BODY" with title "TITLE"'` or `notify-send "TITLE" "BODY"` with `std::process::Command::new(...).spawn()` as fire-and-forget; if spawn fails, logs `eprintln!("notification error: {e}")` but never returns `Err`; reads `FOCUS_POMODORO_SOUND` env var (currently unused for audio, but reads it without error)
- [X] T031 [US1] Write unit tests for `src/pomodoro/notify.rs` `#[cfg(test)]`: test `send_notification("t", "b")` does not panic; test with deliberately bad command path (override `cfg` with a test helper flag) — no panic; these are smoke tests only since subprocess behavior is OS-dependent
- [X] T032 [US1] Run `cargo test test_pomo_cli` + `cargo test` full suite — all pass; run `cargo clippy -- -D warnings` on modified files

**Checkpoint**: MVP complete — CLI Pomodoro timer works end-to-end ✓

---

## Phase 4: User Story 2+5+7+10 — TUI Pomodoro View (Priority: P2/P5/P7/P10)

**Goal**: TUI shows a dedicated Pomodoro view with live countdown, phase indicator, progress bar, and responds to P/S/+/Q keys. Mode selector dialog and customization dialog appear before session starts. Pause suspends countdown. Skip ends breaks immediately. Extend adds 5 min. Customization dialog validates inputs.

**Independent Test**: Launch `focus ui`, press S, select "Pomodoro", enter task name, confirm durations dialog; Pomodoro view shows `🍅 WORK 25:00`; press P → PAUSED shown; press P → countdown resumes from same seconds; in break phase press S → next work phase starts; press + → 5 minutes added to remaining time; press Q → confirmation dialog shows count; press Y → returns to dashboard.

- [X] T033 [P] [US2] Write failing unit tests for `src/tui/views/pomodoro.rs` `#[cfg(test)]`: test `format_remaining(1499)` returns `"24:59"`; test `format_remaining(0)` returns `"00:00"`; test `phase_label(Work)` returns string containing "WORK"; test `phase_label(LongBreak)` returns string containing "LONG BREAK"; test progress ratio is `(total - remaining) / total` clamped to 0.0..=1.0
- [X] T034 [P] [US2] Create `src/tui/views/pomodoro.rs`: `pub fn render_pomodoro(frame: &mut Frame, area: Rect, timer: &PomodoroTimer, no_color: bool)` — layout: top block shows phase emoji + label + "PAUSED" badge when paused; middle shows large MM:SS countdown + "X/4 pomodoros" line + total elapsed; below shows progress bar (Gauge widget); bottom shows keyboard shortcut hints `[P] pause  [S] skip  [+] extend  [Q] stop`; export `pub mod pomodoro;` from `src/tui/views/mod.rs`
- [X] T035 [P] [US2] Add to `src/tui/app.rs`: `Tab::Pomodoro` variant in `Tab` enum; `pomodoro_timer: Option<PomodoroTimer>` field in `App`; update `App::new` to init as `None`; new `Overlay` variants: `ModeSelector { cursor: usize }`, `PomodoroCustomize { work: String, break_: String, long_break: String, focused_field: usize, error: Option<String> }`, `PomodoroConfirmStop { completed: u32 }`; update `Overlay::is_active` match to include new variants
- [X] T036 [P] [US2] Update `src/tui/ui.rs`: add `Tab::Pomodoro` arm in tab-bar rendering (label "🍅 Pomodoro"); add `Tab::Pomodoro` arm in main content dispatch calling `views::pomodoro::render_pomodoro`; add rendering for `Overlay::ModeSelector` (centered box with 3 options, cursor indicator), `Overlay::PomodoroCustomize` (3 labeled input fields with error line), `Overlay::PomodoroConfirmStop` (message with Y/N prompt)
- [X] T037 [US2] Update `src/tui/events.rs` for Pomodoro tab key handling: when `app.active_tab == Tab::Pomodoro` and no overlay active: `Char('p') | Char('P')` → `timer.pause()` or `timer.resume()`; `Char('s') | Char('S')` → `timer.skip_break()`; `Char('+')` → `timer.extend()`; `Char('q') | Char('Q')` → open `Overlay::PomodoroConfirmStop { completed: timer.completed_work }`; add `Overlay::PomodoroConfirmStop` handler: `Char('y')|Enter` → call `pomodoro_store::increment_abandoned(conn, today_date)` if mid-work-phase, set `app.pomodoro_timer = None`, switch to `Tab::Dashboard`, reload dashboard; `Char('n')|Esc` → close overlay
- [X] T038 [US2] Update `src/tui/events.rs` session start flow: intercept existing StartSession prompt trigger to instead open `Overlay::ModeSelector { cursor: 0 }`; handle `ModeSelector`: `Up/Down` moves cursor (0=Pomodoro, 1=Freeform, 2=Cancel); `Enter`: cursor=0 → open `Overlay::PomodoroCustomize` pre-filled with defaults, cursor=1 → open existing `Overlay::Prompt { action: PromptAction::StartSession, .. }`, cursor=2 → close; handle `PomodoroCustomize`: `Tab/Down` advances focused_field (0→1→2→0); `Backspace/Char` edits focused field string; `Enter` validates all 3 fields via `PomodoroConfig::validate()` — on error set `error: Some(msg)` and keep open — on success build `PomodoroConfig`, create `PomodoroTimer`, set `app.pomodoro_timer = Some(timer)`, switch to `Tab::Pomodoro`, close overlay; `Esc` → close overlay
- [X] T039 [US2] Update `src/tui/mod.rs` `run_app` tick else-branch: when `app.pomodoro_timer.is_some()`, compute delta since last tick (track `last_tick: Instant` in loop), call `timer.tick_secs(delta_secs, conn)`, process each `TimerEvent`: on `PhaseComplete` → call `pomodoro::notify::send_notification(...)` + set `app.message = Some(MessageOverlay::success("Work phase complete! Take a break."))` + (if `Tab::Pomodoro`) the view auto-updates via re-render; on `AutoAbandoned` → set error message overlay + set `app.pomodoro_timer = None` + switch to `Tab::Dashboard`
- [X] T040 [US10] Write failing tests in `src/tui/app.rs` `#[cfg(test)]`: test `Overlay::PomodoroCustomize` stores correct field strings; test `Tab` enum has `Pomodoro` variant; test `Overlay::is_active()` returns true for `ModeSelector`, `PomodoroCustomize`, `PomodoroConfirmStop`
- [X] T041 [US10] Write failing tests for PomodoroCustomize inline validation (can be unit tests in `src/tui/events.rs` or via `PomodoroConfig::validate`): test work="0" fails validation; test work="200" fails validation; test break="abc" fails with parse error; test valid "25"/"5"/"15" passes and config is created correctly
- [X] T042 [US2] Run `cargo test` — all TUI Pomodoro tests pass; `cargo clippy -- -D warnings`

**Checkpoint**: TUI Pomodoro view complete with all controls ✓

---

## Phase 5: User Story 3 — Pomodoro Statistics CLI (Priority: P3)

**Goal**: `focus pomo-stats --today` and `focus pomo-stats --week` display accurate per-day aggregates and streaks.

**Independent Test**: Manually insert pomodoro_stats rows; run `focus pomo-stats --today` — shows all fields; run `focus pomo-stats --week` — shows 7-day table; run with empty DB — prints graceful empty-state message.

- [X] T043 [US3] Write failing integration tests in `tests/integration/test_pomo_stats_cmd.rs`: test `run(conn, true, false)` output contains "Completed", "Abandoned", "Work time", "Break time", "streak" when stats exist; test `run(conn, false, true)` output contains date column and "Total" row; test `run(conn, true, false)` with no stats prints "No Pomodoro sessions today" and returns `Ok(())`; test `run(conn, false, false)` defaults to today view; add `mod test_pomo_stats_cmd;` to `tests/integration.rs`
- [X] T044 [US3] Implement `src/commands/pomo_stats.rs` `pub fn run(conn: &Connection, today: bool, week: bool) -> Result<()>`: if `today || (!today && !week)`: call `pomodoro_store::get_stats_for_date` for today's local date, call `pomodoro_store::get_stats_range` for last 30 days to calculate streak via `calculate_streak`, print formatted table per contracts/cli-commands.md; if `week`: collect last 7 local dates, call `get_stats_range`, compute totals + `calculate_best_streak`, print day-by-day table + totals row; empty state branches print single descriptive message
- [X] T045 [US3] Add `PomoStats { #[arg(long, conflicts_with="week")] today: bool, #[arg(long, conflicts_with="today")] week: bool }` subcommand to `src/main.rs`; add dispatch in `fn run()` calling `commands::pomo_stats::run`
- [X] T046 [US3] Run `cargo test test_pomo_stats_cmd` — all pass

**Checkpoint**: `focus pomo-stats --today/--week` works correctly ✓

---

## Phase 6: User Story 8 — Integration with Log and Report (Priority: P8)

**Goal**: `focus log` shows Mode column; `focus report` shows mode breakdown when both modes present; existing freeform records unaffected.

**Independent Test**: Insert one freeform + one pomodoro session; `focus log` shows Mode column with correct labels; `focus report` shows "Mode Breakdown" section.

- [X] T047 [P] [US8] Write failing integration tests in `tests/integration/test_pomodoro_schema.rs` (extend): test that `list_sessions` returns sessions with `mode='freeform'` for old rows and `mode='pomodoro'` for new pomodoro rows; test `print_log_table` output string contains "Mode" column header
- [X] T048 [P] [US8] Update `src/display/format.rs` `print_log_table`: add `Mode` column header; include `session.mode` in each row (right-padded to 8 chars); adjust column widths to accommodate new column
- [X] T049 [P] [US8] Write failing integration tests for report mode breakdown: setup conn with one freeform (48 min) + one pomodoro (25 min) completed session; call `commands::report::run(conn, true, false)`; verify output contains "Mode Breakdown" and both mode names with durations; verify if only one mode present, no breakdown section printed
- [X] T050 [P] [US8] Update `src/commands/report.rs`: after the tag breakdown table, query `SELECT mode, SUM(end_time - start_time) as secs FROM sessions WHERE end_time IS NOT NULL AND start_time >= ?1 GROUP BY mode` for the same time window; if result has >1 row: print "Mode Breakdown" section with percentage; if 0 or 1 mode: omit section silently
- [X] T051 [US8] Run `cargo test` — log + report tests pass; verify no regressions in `test_log`, `test_report`, `test_export` integration tests

---

## Phase 7: User Story 9 — Abandonment Handling (Priority: P9)

**Goal**: Stopping mid-work-phase saves exactly the completed count; abandoned count increments; summary message is accurate; stopping during a break does NOT record an abandonment.

**Independent Test**: Start pomodoro with 1-min work/break durations; let 2 work phases complete; stop mid-3rd work phase; `focus log` shows exactly 2 records with mode=pomodoro; `focus pomo-stats --today` shows abandoned=1.

- [X] T052 [US9] Write failing integration tests in `tests/integration/test_abandonment.rs`: test stop after 2 complete phases + 1 partial → sessions table has exactly 2 pomodoro records; test pomodoro_stats row shows abandoned=1; test stop during a break phase → pomodoro_stats.abandoned unchanged (0); test CLI output message format "N pomodoros completed, M abandoned"; add `mod test_abandonment;` to `tests/integration.rs`
- [X] T053 [US9] Verify `run_pomodoro` in `src/commands/start.rs` (T029): on stop, if `timer.is_in_work_phase()` is true call `pomodoro_store::increment_abandoned(conn, today_date)`; if in break phase, do NOT call increment_abandoned — adjust if not already correct
- [X] T054 [US9] Verify TUI `PomodoroConfirmStop` handler in `src/tui/events.rs` (T037): same work-vs-break distinction for `increment_abandoned` call — adjust if needed; add `timer.is_in_work_phase()` helper to `PomodoroTimer` if not already present (T021 should have it)
- [X] T055 [US9] Run `cargo test test_abandonment` — all pass

---

## Phase 8: Polish and Cross-Cutting Concerns

**Purpose**: Ensure correctness, clean build, no regressions, backward compatibility.

- [X] T056 Consolidate `tests/integration.rs` mod declarations: add any missing `mod` lines for `test_pomo_cli`, `test_pomo_stats_cmd`, `test_abandonment`, `test_pomodoro_schema`
- [X] T057 Verify backward compatibility: run `cargo test test_start_stop test_log test_report test_export` — all must pass without any change; manually verify `focus start "task"` (no `--pomodoro`) still inserts a freeform session with `mode='freeform'`
- [X] T058 Verify NO_COLOR compliance: test that `NO_COLOR=1 cargo run -- pomo-stats --today` produces no ANSI escape sequences; test `NO_COLOR=1 cargo run -- log` with pomodoro sessions produces no ANSI; add inline assertion in `test_pomo_stats_cmd.rs` that output with `NO_COLOR=1` contains no ESC character `\x1b`
- [X] T059 Run `cargo clippy -- -D warnings` across the full project — fix all warnings in new and modified files; zero warnings allowed
- [X] T060 Run `cargo fmt` across all new and modified source files
- [X] T061 Run `cargo test` — full test suite must pass; zero failures; zero test regressions
- [X] T062 Run `cargo build --release` — single executable binary produced successfully

**Final Checkpoint**: All tests green; clippy clean; fmt clean; release build succeeds ✓

---

## Dependency Graph

```
T001-T007 (Setup)
    ↓
T008-T026 (Foundational: schema, model, timer, config, store)
    ↓
    ├── T027-T032 (US1: CLI Pomodoro)        ← MVP deliverable
    ├── T033-T042 (US2+5+7+10: TUI)          ← depends on timer (T019-T023)
    ├── T043-T046 (US3: pomo-stats)           ← depends on store (T024-T026)
    ├── T047-T051 (US8: log+report)           ← depends on mode column (T011-T015)
    └── T052-T055 (US9: abandonment)          ← depends on timer + store
    ↓
T056-T062 (Polish)
```

## Parallel Execution Opportunities

Within Phase 2, independent tracks can run simultaneously:

| Track | Tasks | Dependency |
|-------|-------|------------|
| A — Schema | T008 → T009 → T010 | None |
| B — Session model | T011 → T012 → T013 → T014 → T015 | After Track A (needs mode column) |
| C — Config | T016 → T017 → T018 | None |
| D — Timer | T019 → T020 → T021 → T022 → T023 | After Track C (needs PomodoroConfig) |
| E — Store | T024 → T025 → T026 | After Track A (needs pomodoro_stats table) |

Within Phase 4, T033–T036 can run in parallel (different files); T037–T039 must be sequential (all touch events.rs/mod.rs).

## Implementation Strategy (MVP-first)

1. **MVP**: Phases 1–3 only — CLI Pomodoro fully working with stats and abandonment handling.
2. **Increment 2**: Phase 4 — TUI Pomodoro view (no schema changes needed).
3. **Increment 3**: Phase 5 — `focus pomo-stats` command.
4. **Increment 4**: Phase 6 — `focus log` mode column + report breakdown.
5. **Increment 5**: Phase 7 — Abandonment stats refinement.
6. **Final**: Phase 8 — Polish, full test suite green.

**Total tasks**: 62
