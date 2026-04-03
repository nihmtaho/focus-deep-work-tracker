# Tasks: Focus TUI Dashboard

**Input**: Design documents from `specs/003-focus-tui-dashboard/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/cli-schema.md ✓

**Organization**: Grouped by user story for independent implementation and testing.  
**TDD**: Per Constitution Principle II — test tasks appear before their implementation tasks. Tests MUST be confirmed failing before implementation begins.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no inter-task dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4)

---

## Phase 1: Setup

**Purpose**: Wire new dependencies and create empty module scaffolding so `cargo build` passes before any logic is written.

- [X] T001 Add `ratatui = "0.29"` and `crossterm = "0.28"` to `[dependencies]` in `Cargo.toml`
- [X] T002 Create `src/tui/mod.rs` with a stub `pub fn run(_conn: rusqlite::Connection) -> anyhow::Result<()> { Ok(()) }`
- [X] T003 [P] Create `src/tui/app.rs` as an empty file (`// placeholder`)
- [X] T004 [P] Create `src/tui/events.rs` as an empty file (`// placeholder`)
- [X] T005 [P] Create `src/tui/ui.rs` as an empty file (`// placeholder`)
- [X] T006 [P] Create `src/tui/views/mod.rs`, `src/tui/views/dashboard.rs`, `src/tui/views/menu.rs`, `src/tui/views/start_form.rs`, `src/tui/views/log.rs`, `src/tui/views/report.rs` each as empty files
- [X] T007 Add `pub mod tui;` to `src/lib.rs`
- [X] T008 Add `Commands::Ui` variant to `src/main.rs` with dispatch `tui::run(db::open_db()?)?` and help text `"Launch interactive TUI dashboard"`
- [X] T009 [P] Create `tests/unit/tui/mod.rs` as an empty `#[cfg(test)]` module and wire it into the test harness

**Checkpoint**: `cargo build` passes. `cargo test` passes (no new failures). `focus ui` runs and exits cleanly.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core state types and terminal lifecycle that every user story builds on. All user story phases depend on this phase completing first.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete and `cargo build` + `cargo test` pass.

- [X] T010 Write unit tests for `truncate_to` helper in `tests/unit/tui/mod.rs`: test truncation at boundary, truncation with `…` appended, no truncation when text fits, empty string, single-char string
- [X] T011 Implement `pub fn truncate_to(s: &str, max_chars: usize) -> String` in `src/tui/app.rs`: truncate Unicode string to `max_chars`, appending `…` when truncated; used by all table views
- [X] T012 [P] Implement all core state types in `src/tui/app.rs`: `TimeWindow` enum (Today/CurrentWeek/Last7Days), `InputField` enum (Task/Tag), `MessageKind` enum (Success/Warning/Error), `MessageOverlay` struct (text, kind, shown_at: Instant), `View` enum (Dashboard/Menu/StartForm/Log/Report with fields per data-model.md), `App` struct (all fields per data-model.md)
- [X] T013 Implement terminal lifecycle in `src/tui/mod.rs`: TTY check via `std::io::IsTerminal` (exit code 1 with error message if not TTY), `NO_COLOR` env detection stored in `App`, raw mode enable/disable, alternate screen enter/leave, panic hook that restores terminal before printing panic, minimum 60×12 size guard loop, outer event loop calling `crossterm::event::poll(Duration::from_millis(100))`
- [X] T014 [P] Implement top-level render dispatch in `src/tui/ui.rs`: `pub fn render(frame: &mut ratatui::Frame, app: &App)` that delegates to the correct view renderer based on `app.view`, renders `MessageOverlay` at screen bottom if `app.message.is_some()`, renders quit-confirm warning bar if `app.quit_pending`
- [X] T015 [P] Implement keyboard dispatch skeleton in `src/tui/events.rs`: `pub fn handle_key_event(app: &mut App, conn: &rusqlite::Connection, key: crossterm::event::KeyEvent) -> anyhow::Result<bool>` returning `true` to signal quit; global `Q` handling with `quit_pending` logic; `Tab` toggle between Dashboard and Menu

**Checkpoint**: `cargo build` passes. `cargo test` passes. `truncate_to` tests are green.

---

## Phase 3: User Story 1 — Real-Time Dashboard View (Priority: P1) 🎯 MVP

**Goal**: `focus ui` opens and shows a live dashboard: active session with ticking elapsed time + today's tag summary, both polling the database on every tick.

**Independent Test**: Launch `focus ui` with no active session — dashboard shows "No active session" and today's summary. In a second terminal run `focus start "writing" --tag work`; within 1 second the dashboard shows the task name, tag, and ticking elapsed time. Press Q to exit.

### Tests for User Story 1 ⚠️ Write first — confirm FAILING before T019

- [X] T016 [P] [US1] Write unit tests for `App::tick_dashboard` in `tests/unit/tui/app_test.rs`: given a mock active session, `active_session` field is updated; given no session, field is `None`; `today_summary` contains correct tag rows; `terminal_too_small` is set when dimensions below minimum
- [X] T017 [P] [US1] Write unit tests for `render_dashboard` output in `tests/unit/tui/app_test.rs`: "No active session" text present when `active_session` is None; task name and tag appear when session is Some; elapsed string is non-empty; today summary section present

### Implementation for User Story 1

- [X] T018 [US1] Implement `App::tick_dashboard(&mut self, conn: &Connection)` in `src/tui/app.rs`: calls `session_store::get_active_session`, calls `session_store::aggregate_by_tag(conn, today_start())`, stores results in `self.active_session` and `self.today_summary`; updates `self.terminal_too_small` from current terminal size
- [X] T019 [US1] Implement `render_dashboard(frame, app)` in `src/tui/views/dashboard.rs`: header bar with "FOCUS" title + current date/time; active session panel showing task (truncated to 40 chars), tag, elapsed time formatted via `display::format::format_duration`; "No active session" when `active_session` is None; today summary table with tag and total columns; footer with key hint bar (`[M]enu  [Q]uit`)
- [X] T020 [US1] Implement `render_too_small(frame)` in `src/tui/views/dashboard.rs`: full-frame message "Terminal too small. Minimum: 60×12. Current: W×H. Resize to continue or press Q to quit."
- [X] T021 [US1] Wire `App::tick_dashboard` into the event loop in `src/tui/mod.rs`: call on every tick before `terminal.draw`; wire `render_too_small` when `app.terminal_too_small`; handle `crossterm::event::Event::Resize` to update `terminal_too_small`

**Checkpoint**: `cargo test` green. `focus ui` shows live dashboard. Elapsed time ticks. `NO_COLOR=1 focus ui` renders without ANSI codes. Terminal resize is handled. Q exits cleanly.

---

## Phase 4: User Story 2 — Start and Stop Sessions via TUI (Priority: P2)

**Goal**: Full session lifecycle from within the TUI — open menu, start a session with task/tag inputs, stop it, see 2-second confirmation. Quit with active session requires double-Q.

**Independent Test**: Open TUI, press M for menu, press S, type "deep work", Tab to tag field, type "coding", press Enter — dashboard shows session running. Press M → T (stop) — confirmation shows task, tag, and non-zero duration for 2 seconds. Then press Q (active session warning), then Q again — TUI exits, session still recorded in DB.

### Tests for User Story 2 ⚠️ Write first — confirm FAILING before T026

- [X] T022 [P] [US2] Write unit tests for menu navigation in `tests/unit/tui/app_test.rs`: arrow-down advances `selected`; arrow-up wraps; j/k equivalents; S/T/L/R/D shortcuts trigger correct View transitions
- [X] T023 [P] [US2] Write unit tests for start form validation in `tests/unit/tui/app_test.rs`: empty task string is rejected with Error overlay; non-empty task + empty tag starts session; `FocusError::AlreadyRunning` produces Warning overlay instead of form
- [X] T024 [P] [US2] Write unit tests for quit-confirm state machine in `tests/unit/tui/app_test.rs`: first Q with active session sets `quit_pending = true`; second Q returns `should_quit = true`; any non-Q key clears `quit_pending`; Q with no active session returns `should_quit = true` immediately

### Implementation for User Story 2

- [X] T025 [US2] Implement `render_menu(frame, app)` in `src/tui/views/menu.rs`: list of 6 items (Start Session, Stop Current Session, View Status, Session Log, Generate Report, Back to Dashboard); highlighted row for `selected` index; footer key hints (`[↑↓/jk] navigate  [Enter] select  [D] dashboard  [Q]uit`)
- [X] T026 [US2] Implement `render_start_form(frame, app)` in `src/tui/views/start_form.rs`: two labelled text fields (Task — required, Tag — optional); active field highlighted with cursor; inline error text below Task field when validation fails; footer hints (`[Tab] switch field  [Enter] start  [Esc] cancel`)
- [X] T027 [US2] Implement `App::start_session(&mut self, conn: &Connection)` in `src/tui/app.rs`: calls `session_store::insert_session`; on `FocusError::AlreadyRunning` sets `self.message` to Warning overlay; on success sets Success overlay "Session started" and transitions to Dashboard
- [X] T028 [US2] Implement `App::stop_session(&mut self, conn: &Connection)` in `src/tui/app.rs`: calls `session_store::stop_session`; on `FocusError::NoActiveSession` sets Warning overlay "No active session"; on success sets Success overlay with task name, tag, and formatted duration; transitions to Dashboard
- [X] T029 [US2] Implement keyboard handlers for Menu view in `src/tui/events.rs`: ↑/↓/j/k navigation, Enter activates item, S/T/L/R/D/Q shortcuts, Esc returns to Dashboard
- [X] T030 [US2] Implement keyboard handlers for StartForm view in `src/tui/events.rs`: printable chars append to active field, Backspace deletes last char, Tab switches between Task/Tag fields, Enter calls `app.start_session()`, Esc cancels to Dashboard
- [X] T031 [US2] Implement quit-confirm logic in `src/tui/events.rs`: first Q with active session sets `app.quit_pending = true`, second Q returns should-quit, non-Q clears `quit_pending`; wire quit warning bar rendering in `src/tui/ui.rs`

**Checkpoint**: `cargo test` green. Full start→stop session cycle works in TUI. Double-Q confirm works. Menu navigation all keys respond correctly.

---

## Phase 5: User Story 3 — Session Log View (Priority: P3)

**Goal**: Paginated reverse-chronological session log with Date/Time/Task/Tag/Duration columns, keyboard scrolling, and N/P pagination.

**Independent Test**: With ≥11 completed sessions in DB, open TUI → M → L. First page shows 10 sessions newest-first; page indicator shows "Page 1 / N". Press N → page 2 loads. Press P → back to page 1. Press Esc → Menu.

### Tests for User Story 3 ⚠️ Write first — confirm FAILING before T034

- [X] T032 [P] [US3] Write unit tests for `App::load_log_page` in `tests/unit/tui/app_test.rs`: page 0 returns most recent 10 sessions; advancing page returns next 10; `log_total_pages` computed correctly from session count; page 0 with 0 sessions returns empty vec with 0 pages

### Implementation for User Story 3

- [X] T033 [US3] Implement `App::load_log_page(&mut self, conn: &Connection)` in `src/tui/app.rs`: queries `session_store::list_sessions(conn, limit)` with `limit = (page + 1) * 10` and slices to current page; stores in `self.log_entries`; computes `self.log_total_pages` (requires a `COUNT(*)` query — add `session_store::count_completed(conn) -> Result<usize>` to `src/db/session_store.rs`)
- [X] T034 [US3] Implement `render_log(frame, app)` in `src/tui/views/log.rs`: header "Session Log — Page N / M"; table with columns Date (10), Time (8), Task (truncated to 30), Tag (truncated to 12), Duration (9); empty-state message when no sessions; footer hints (`[↑↓/jk] scroll  [N]ext page  [P]rev page  [Esc] back`)
- [X] T035 [US3] Implement keyboard handlers for Log view in `src/tui/events.rs`: N calls `app.load_log_page` with incremented page (clamped to last page); P decrements (clamped to 0); ↑/↓/j/k scroll within page (update `scroll` field in `View::Log`); Esc/B returns to Menu

**Checkpoint**: `cargo test` green. Log view opens, paginates correctly, empty state shown when no sessions.

---

## Phase 6: User Story 4 — Report View (Priority: P4)

**Goal**: Tag-aggregated time report for three selectable time windows (Today / Current Week / Last 7 Days) with "untagged" row and TOTAL row.

**Independent Test**: With sessions tagged across ≥2 tags in the current week, open TUI → M → R. "Current Week" is pre-selected; rows show each tag with correct total time and session count. Press ↓ to select "Today"; report updates. "untagged" row is present if untagged sessions exist. TOTAL row always present.

### Tests for User Story 4 ⚠️ Write first — confirm FAILING before T038

- [X] T036 [P] [US4] Write unit tests for `App::load_report` in `tests/unit/tui/app_test.rs`: switching `TimeWindow` triggers different `since` epoch; untagged group present when sessions have `tag = None`; TOTAL row value equals sum of all tag row values

### Implementation for User Story 4

- [X] T037 [US4] Implement `App::load_report(&mut self, conn: &Connection)` in `src/tui/app.rs`: maps `self.view` (Report { window, .. }) to the correct `since` epoch using `commands::report::{today_start, current_week_start, rolling_7d_start}`; calls `session_store::aggregate_by_tag(conn, since)`; stores in `self.report_rows`
- [X] T038 [US4] Implement `render_report(frame, app)` in `src/tui/views/report.rs`: time-window selector at top (Today / Current Week / Last 7 Days) with active window highlighted; table with Tag (truncated to 20), Total Time, Sessions columns; "untagged" label for `None` tag rows; TOTAL row at bottom; empty-period message when `report_rows` is empty; footer hints (`[↑↓/jk] select window  [Enter] apply  [Esc] back`)
- [X] T039 [US4] Implement keyboard handlers for Report view in `src/tui/events.rs`: ↑/↓/j/k cycle `selected_window` (0–2); Enter/↵ applies selected window by calling `app.load_report(conn)`; Esc/B returns to Menu; pre-load report on view entry (in the Menu→Report transition in events.rs)

**Checkpoint**: `cargo test` green. All three time windows produce correct data. TOTAL row is always present. Empty state shown for periods with no data.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Reliability, compatibility, and final quality gates across all stories.

- [X] T040 [P] Implement message overlay auto-dismiss in `src/tui/app.rs`: in `App::tick()` (called each event loop iteration), check `self.message.as_ref().map(|m| m.shown_at.elapsed().as_secs() >= m.auto_dismiss_secs)`; clear `self.message` when elapsed
- [X] T041 [P] Implement `NO_COLOR` propagation in `src/tui/mod.rs` and `src/tui/app.rs`: if `std::env::var("NO_COLOR").is_ok()`, store `app.no_color = true`; pass to all `render_*` functions so `Style::default()` (no colors) is used unconditionally
- [X] T042 [P] Verify resize handling in `src/tui/mod.rs`: `Event::Resize(w, h)` must update `app.terminal_too_small = w < 60 || h < 12` and force an immediate re-render
- [X] T043 Implement `session_store::count_completed(conn: &Connection) -> Result<usize>` in `src/db/session_store.rs` (needed by T033; add here if not already done)
- [X] T044 Run `cargo test` — all existing CLI integration tests pass with no regressions; all new unit tests pass
- [X] T045 Run `cargo clippy -- -D warnings` — fix all warnings
- [X] T046 Run `cargo fmt --check` — apply `cargo fmt` and commit

**Checkpoint**: `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` all pass clean. Manual walkthrough of all 4 user stories succeeds.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Requires Phase 1 — **BLOCKS all user stories**
- **US1 Dashboard (Phase 3)**: Requires Phase 2 — no dependency on US2/US3/US4
- **US2 Start/Stop (Phase 4)**: Requires Phase 2 — no dependency on US3/US4; builds on App and event loop from Phase 2
- **US3 Log (Phase 5)**: Requires Phase 2 — no dependency on US2/US4; needs `count_completed` (add in T033 or T043)
- **US4 Report (Phase 6)**: Requires Phase 2 — no dependency on US2/US3
- **Polish (Phase 7)**: Requires all desired stories complete

### User Story Dependencies

- **US1 (P1)**: Independent after Phase 2 — the MVP
- **US2 (P2)**: Independent after Phase 2 — adds menu and session control on top of the running App struct
- **US3 (P3)**: Independent after Phase 2 — read-only view, no dependency on US2
- **US4 (P4)**: Independent after Phase 2 — read-only view, no dependency on US2/US3

### Within Each Phase

1. **TDD order**: test tasks (T016/T017, T022/T023/T024, T032, T036) MUST be written and confirmed failing before their implementation tasks run
2. State types (app.rs) before renderers (views/*.rs) before event handlers (events.rs)
3. `count_completed` (T033/T043) required before log pagination

### Parallel Opportunities

Within Phase 2: T012, T014, T015 can run in parallel after T011 (truncate tests), T013  
Within Phase 3: T016 and T017 can run in parallel; T018–T021 sequential  
Within Phase 4: T022, T023, T024 can run in parallel; T025–T031 sequential  
Within Phase 7: T040, T041, T042, T043 can run in parallel

---

## Parallel Example: Phase 3 (US1)

```text
# Run in parallel — no shared files:
Task T016: Unit tests for App::tick_dashboard in tests/unit/tui/app_test.rs
Task T017: Unit tests for render_dashboard output in tests/unit/tui/app_test.rs

# Then sequential:
Task T018: App::tick_dashboard() in src/tui/app.rs
Task T019: render_dashboard() in src/tui/views/dashboard.rs
Task T020: render_too_small() in src/tui/views/dashboard.rs
Task T021: Wire into event loop in src/tui/mod.rs + src/tui/ui.rs
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational — `cargo build` + `cargo test` must pass
3. Complete Phase 3: US1 Real-Time Dashboard
4. **STOP and VALIDATE**: `focus ui` shows live dashboard, `NO_COLOR` works, resize handled, Q exits
5. Commit checkpoint — MVP is shippable

### Incremental Delivery

1. Phase 1 + 2 → skeleton builds
2. + Phase 3 → live dashboard (MVP)
3. + Phase 4 → full session control from TUI
4. + Phase 5 → log browsing
5. + Phase 6 → tag reporting
6. + Phase 7 → polish and final quality gate

---

## Notes

- **[P]** = different files, no inter-task dependency — safe to run in parallel
- **[USN]** label maps each task to its user story for traceability
- **TDD (NON-NEGOTIABLE, Principle II)**: Test tasks MUST appear before implementation tasks. Confirm tests fail before writing implementation.
- **WAL mode (Principle V)**: TUI uses `db::open_db()` which already sets `PRAGMA journal_mode=WAL` — no additional work needed. Any future `Connection::open` in tests MUST use `db::open_db_at` which also sets WAL.
- **No AI attribution (Principle VI)**: Commit messages MUST NOT include `Co-Authored-By` lines.
- **PR Standards (Principle VII)**: Use `gh pr create` with `.github/PULL_REQUEST_TEMPLATE.md`. PR title format: `feat: add interactive TUI dashboard (focus ui)`. Link `specs/003-focus-tui-dashboard/spec.md` and `specs/003-focus-tui-dashboard/tasks.md` in PR body. Include ≥2 manual test steps.
- `commands::report::{today_start, current_week_start, rolling_7d_start}` are `pub` — reuse directly, do not duplicate.
- All duration formatting via `display::format::format_duration` — do not write a new formatter.
- Run `cargo fmt` before every checkpoint commit.
