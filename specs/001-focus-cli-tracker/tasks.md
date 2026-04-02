# Tasks: Focus CLI — Deep Work Session Tracker

**Input**: Design documents from `/specs/001-focus-cli-tracker/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/cli-schema.md ✓  
**Tests**: Added retroactively (constitution v1.0.0 Principle II compliance). Integration tests in `tests/integration/`, unit tests in `tests/unit/`.

**Organization**: Tasks grouped by user story. Each story is independently buildable and testable after its phase completes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no blocking dependency)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Paths follow single-project layout from plan.md

---

## Phase 1: Setup

**Purpose**: Initialize the Rust project with full dependency stack and module skeleton.

- [X] T001 Initialize `Cargo.toml` with all dependencies: `clap = { version = "4", features = ["derive"] }`, `rusqlite = { version = "0.31", features = ["bundled"] }`, `chrono = { version = "0.4", features = ["serde"] }`, `colored = "2"`, `dirs = "5"`, `thiserror = "1"`, `anyhow = "1"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"` in `Cargo.toml`
- [X] T002 Create full module skeleton: empty files for `src/main.rs`, `src/error.rs`, `src/commands/mod.rs`, `src/commands/start.rs`, `src/commands/stop.rs`, `src/commands/status.rs`, `src/commands/log.rs`, `src/commands/report.rs`, `src/commands/export.rs`, `src/db/mod.rs`, `src/db/session_store.rs`, `src/models/session.rs`, `src/display/format.rs`; confirm `cargo build` compiles without errors
- [X] T003 [P] Add `rustfmt.toml` at repo root with `edition = "2021"`; add `.cargo/config.toml` with `[build] rustflags = ["-D", "warnings"]`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared infrastructure that every command depends on — must complete before any user story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Implement `FocusError` enum in `src/error.rs` using `thiserror`: variants `NoActiveSession`, `AlreadyRunning { task: String, elapsed: String }`, `EmptyTask`, `InvalidLimit`, `InvalidFormat`, `DataFileCorrupted { path: String }`, `Db(#[from] rusqlite::Error)`, `Io(#[from] std::io::Error)`
- [X] T005 Implement data directory bootstrap and DB connection in `src/db/mod.rs`: resolve `dirs::home_dir()/.local/share/focus/`, call `std::fs::create_dir_all`, open rusqlite connection, run `CREATE TABLE IF NOT EXISTS sessions` + both indexes from `data-model.md`; map any open/parse error to `FocusError::DataFileCorrupted` with the full DB path; expose `pub fn open_db() -> anyhow::Result<Connection>`
- [X] T006 Implement `Session` struct in `src/models/session.rs`: fields `id: i64`, `task: String`, `tag: Option<String>`, `start_time: DateTime<Utc>`, `end_time: Option<DateTime<Utc>>`; methods `is_active() -> bool`, `duration() -> Option<chrono::Duration>`, `elapsed() -> chrono::Duration`; derive `serde::Serialize`
- [X] T007 [P] Implement duration formatter in `src/display/format.rs`: `pub fn format_duration(d: chrono::Duration) -> String` producing `"Xh Ym Zs"` / `"Ym Zs"` / `"Zs"` per contract; add `pub fn format_elapsed(start: DateTime<Utc>) -> String` that calls `format_duration(Utc::now() - start)`
- [X] T008 Implement clap 4 CLI struct in `src/main.rs`: `#[derive(Parser)] struct Cli`, `#[derive(Subcommand)] enum Commands` with all six subcommands (`Start`, `Stop`, `Status`, `Log`, `Report`, `Export`) and their flags per `contracts/cli-schema.md`; wire `anyhow::Result<()>` main that opens DB, matches `Commands`, and calls the corresponding command module; print errors to stderr on `Err`

**Checkpoint**: `cargo build` succeeds, `focus --help` shows all six subcommands, DB file is created at `~/.local/share/focus/focus.db` on first run.

---

## Phase 3: User Story 1 — Start and Stop a Work Session (Priority: P1) 🎯 MVP

**Goal**: Developers can begin and end a tracked work session from the terminal. Session data is persisted to SQLite.

**Independent Test**: Run `focus start "test task" --tag rust`, wait a few seconds, run `focus stop`. Verify summary prints task name and non-zero duration. Run `sqlite3 ~/.local/share/focus/focus.db "SELECT * FROM sessions;"` to confirm a completed row exists.

- [X] T009 Implement `insert_session`, `get_active_session`, and `stop_session` in `src/db/session_store.rs`: `insert_session(conn, task, tag)` inserts row with `start_time = Utc::now()` as Unix epoch; `get_active_session(conn)` returns `Option<Session>` for row where `end_time IS NULL`; `stop_session(conn)` sets `end_time = Utc::now()` for active row and returns the completed `Session`
- [X] T010 [P] [US1] Implement `focus start` in `src/commands/start.rs`: validate task is non-empty/non-whitespace (return `FocusError::EmptyTask` if not); call `get_active_session` — if found, print warning with current task and elapsed time and return `FocusError::AlreadyRunning`; else call `insert_session` and print "Session started: \<task\>  [tag: \<tag\>]" using `colored`
- [X] T011 [P] [US1] Implement `focus stop` in `src/commands/stop.rs`: call `get_active_session` — if none return `FocusError::NoActiveSession`; call `stop_session`, print "Stopped: \"\<task\>\"  [tag: \<tag\>]\nDuration: \<duration\>" using `format_duration`
- [X] T012 [US1] Wire `Commands::Start` and `Commands::Stop` arms in `src/main.rs` dispatch to call `commands::start::run` and `commands::stop::run`

**Checkpoint**: `focus start` and `focus stop` are fully functional. Double-start shows warning. Stop with no session shows error.

---

## Phase 4: User Story 2 — Check Current Session Status (Priority: P2)

**Goal**: Developers can glance at their active session and elapsed time with a single command.

**Independent Test**: Start a session, run `focus status`, verify task name and elapsed time in `Xh Ym Zs` format appear. Run `focus stop`, then `focus status` again; verify "No active session." is printed and exit code is `0`.

- [X] T013 [US2] Implement `focus status` in `src/commands/status.rs`: call `get_active_session` (reuses T009); if found, print "Working on: \"\<task\>\"  [tag: \<tag\>]\nElapsed: \<elapsed\>" using `format_elapsed`; if none print "No active session." — always exits `0`
- [X] T014 [US2] Wire `Commands::Status` arm in `src/main.rs` dispatch to call `commands::status::run`

**Checkpoint**: `focus status` shows running session or idle message correctly.

---

## Phase 5: User Story 3 — View Session History (Priority: P3)

**Goal**: Developers can review completed sessions in a reverse-chronological table with optional limit.

**Independent Test**: Complete 3+ sessions. Run `focus log --limit 2`. Verify exactly 2 rows shown with DATE, TASK, TAG, DURATION columns; most recent first. Run `focus log --limit 0` and verify "Error: --limit must be a positive integer." on stderr.

- [X] T015 Implement `list_sessions(conn, limit: u32) -> Vec<Session>` in `src/db/session_store.rs`: `SELECT * FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time DESC LIMIT ?`
- [X] T016 [P] [US3] Add `format_table_row` and column-alignment helpers to `src/display/format.rs`: given a list of `Session`, compute max widths for TASK and TAG columns and return padded rows for the log table (headers: DATE, TASK, TAG, DURATION)
- [X] T017 [US3] Implement `focus log` in `src/commands/log.rs`: validate `--limit` is a positive integer, return `FocusError::InvalidLimit` with error message if not; call `list_sessions`; if empty print "No sessions recorded yet."; otherwise print aligned table using `format_table_row`
- [X] T018 [US3] Wire `Commands::Log` arm in `src/main.rs` dispatch to call `commands::log::run`

**Checkpoint**: `focus log` shows aligned session table; `--limit` works; invalid limit prints correct error.

---

## Phase 6: User Story 4 — Weekly Productivity Report (Priority: P4)

**Goal**: Developers can see time grouped by tag for a selected time window, with a grand total.

**Independent Test**: Complete 3+ sessions with different tags. Run `focus report`. Verify each tag appears as a row with total duration; an "untagged" row for tag-less sessions; separator lines; and TOTAL row at the bottom. Run `focus report --today` and `--week` and verify correct time windows filter correctly.

- [X] T019 Implement `aggregate_by_tag(conn, since: i64) -> Vec<(Option<String>, i64)>` in `src/db/session_store.rs`: `SELECT tag, SUM(end_time - start_time) FROM sessions WHERE end_time IS NOT NULL AND start_time >= ? GROUP BY tag ORDER BY SUM(end_time - start_time) DESC`; returns `(tag, total_seconds)` pairs
- [X] T020 [P] [US4] Implement time window helpers in `src/commands/report.rs`: `current_week_start() -> i64` (Monday 00:00:00 local → UTC epoch), `today_start() -> i64` (today 00:00:00 local → UTC epoch), `rolling_7d_start() -> i64` (now minus 7 × 86400 seconds) using `chrono`
- [X] T021 [US4] Implement `focus report` in `src/commands/report.rs`: parse `--today`/`--week` flags (mutually exclusive via clap `conflicts_with`); compute `since` epoch from appropriate helper; call `aggregate_by_tag`; if empty print "No sessions recorded for this period."; otherwise print aligned two-column table (Tag / Total) with "untagged" label for `None`, separator line `──────────────────────`, and TOTAL row using `format_duration`
- [X] T022 [US4] Wire `Commands::Report` arm in `src/main.rs` dispatch to call `commands::report::run`

**Checkpoint**: `focus report` shows tag aggregation table with all three time modes.

---

## Phase 7: User Story 5 — Export Session Data (Priority: P5)

**Goal**: Developers can export all session history to JSON or Markdown via stdout for backup or analysis.

**Independent Test**: Complete 2+ sessions. Run `focus export --format json | python3 -m json.tool` and verify valid JSON array. Run `focus export --format markdown` and verify a valid Markdown table. Run `focus export --format csv` and verify stderr error message and non-zero exit code.

- [X] T023 Implement `list_all_completed(conn) -> Vec<Session>` in `src/db/session_store.rs`: `SELECT * FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time ASC`
- [X] T024 [P] [US5] Implement `export_json(sessions: &[Session]) -> String` in `src/commands/export.rs`: serialize to JSON array per contract schema (`id`, `task`, `tag`, `start_time` as RFC3339 UTC, `end_time` as RFC3339 UTC, `duration_seconds`); use `serde_json::to_string_pretty`; empty input → `"[]"`
- [X] T025 [P] [US5] Implement `export_markdown(sessions: &[Session]) -> String` in `src/commands/export.rs`: produce Markdown table with columns Date, Task, Tag, Start, End, Duration per contract; header + separator + one row per session; empty input → table with headers and no rows
- [X] T026 [US5] Implement `focus export` command in `src/commands/export.rs`: validate `--format` is `json` or `markdown`, return `FocusError::InvalidFormat` with error "Error: --format must be one of: json, markdown" if not; call `list_all_completed`; dispatch to `export_json` or `export_markdown`; print result to stdout
- [X] T027 [US5] Wire `Commands::Export` arm in `src/main.rs` dispatch to call `commands::export::run`

**Checkpoint**: All six commands are fully functional. The binary is complete.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Error hardening, code quality, and final validation.

- [X] T028 [P] Run `cargo clippy -- -D warnings` and fix all reported issues across all source files
- [X] T029 [P] Run `cargo fmt` across all source files; ensure formatting is consistent
- [X] T030 Run `cargo build --release` and validate the binary at `./target/release/focus` works end-to-end per all scenarios in `specs/001-focus-cli-tracker/quickstart.md`
- [X] T031 [P] Verify `NO_COLOR=1 focus status` produces plain text output (no ANSI codes) and piped output (`focus log | cat`) also strips color

## Phase 9: Test Suite (Constitution v1.0.0 Principle II Compliance)

**Purpose**: Retroactive TDD compliance — integration and unit tests using isolated temporary SQLite files.

- [X] T032 Add `tempfile = "3"` dev-dependency and expose `src/lib.rs` for test access; add `open_db_at(path)` to `src/db/mod.rs` for test isolation
- [X] T033 [P] Write `tests/integration/test_start_stop.rs`: FR-001, FR-002, FR-004, FR-010, FR-013 (6 tests)
- [X] T034 [P] Write `tests/integration/test_log.rs`: FR-005, FR-006, FR-017 (5 tests)
- [X] T035 [P] Write `tests/integration/test_report.rs`: FR-007, FR-008, FR-009 (5 tests)
- [X] T036 [P] Write `tests/integration/test_export.rs`: FR-015 (6 tests)
- [X] T037 [P] Write `tests/unit/test_format.rs`: duration format contract from contracts/cli-schema.md (5 tests)
- [X] T038 Run `cargo test` — all 27 tests pass; fix sort tiebreaker (`ORDER BY start_time, id`) for same-second inserts

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Requires Phase 1 — **blocks all user stories**
- **US1 (Phase 3)**: Requires Phase 2 — no dependency on other stories
- **US2 (Phase 4)**: Requires Phase 2 — reuses `get_active_session` from T009 (US1); start after T009 is done
- **US3 (Phase 5)**: Requires Phase 2 — independent from US1/US2
- **US4 (Phase 6)**: Requires Phase 2 — independent from US1/US2/US3
- **US5 (Phase 7)**: Requires Phase 2 — independent from all other stories
- **Polish (Phase 8)**: Requires all desired stories to be complete

### User Story Dependencies

- **US1 (P1)**: Standalone after foundation — defines `session_store` CRUD (T009) used by US2
- **US2 (P2)**: Can start after T009 (get_active_session) — read-only, no new DB writes
- **US3 (P3)**: Can start after foundation — adds `list_sessions` to session_store independently
- **US4 (P4)**: Can start after foundation — adds `aggregate_by_tag` to session_store independently
- **US5 (P5)**: Can start after foundation — adds `list_all_completed` to session_store independently

### Within Each User Story

- DB store methods before command handlers
- [P]-marked tasks within a story can run in parallel
- Wire-up to `main.rs` is always last in each story's phase

### Parallel Opportunities

All tasks within a story marked `[P]` can execute simultaneously once their story's store task is done.

Between stories (after Phase 2 completes): US3, US4, and US5 can be built fully in parallel — they each add a new command and a new `session_store` method independently.

---

## Parallel Example: User Story 5 (Export)

```text
After T023 (list_all_completed) is done:
  → T024 [P] export_json in src/commands/export.rs
  → T025 [P] export_markdown in src/commands/export.rs
  (both in parallel, different functions in same file — write separately, merge)
Then:
  → T026 focus export command dispatcher
  → T027 wire into main.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundation (T004–T008) — **must complete before any story**
3. Complete Phase 3: US1 (T009–T012)
4. **STOP and VALIDATE**: `focus start` + `focus stop` work; data persists in SQLite
5. Usable as a basic time tracker immediately

### Incremental Delivery

1. Setup + Foundational → project compiles, DB bootstraps
2. US1 → start/stop works → **MVP** (record sessions)
3. US2 → status works → check what you're doing
4. US3 → log works → review history
5. US4 → report works → weekly insight
6. US5 → export works → portability
7. Polish → production quality

### Parallel Team Strategy

With 2+ developers after Phase 2 completes:
- Dev A: US1 + US2 (share session_store CRUD)
- Dev B: US3 (list_sessions)
- Dev C: US4 (aggregate_by_tag) + US5 (list_all_completed)

---

## Notes

- `[P]` tasks operate on different files or distinct functions within a file with no ordering constraint
- `session_store.rs` accumulates new functions per story — within a story, implement store methods before command handlers
- No external test framework added; validate via `cargo test` built-ins and manual quickstart checks
- `colored` output is automatically disabled when stdout is not a TTY; verify this explicitly in T031
- Commit after each phase checkpoint for clean incremental history
