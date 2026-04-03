# Tasks: TUI Session Controls with Vim Mode and Tab Views

**Input**: Design documents from `specs/004-tui-session-controls/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓, quickstart.md ✓

**TDD**: All test tasks MUST be written and confirmed FAILING before their implementation tasks begin (Constitution Principle II — NON-NEGOTIABLE).

**Organization**: Grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependency conflicts)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Exact file paths in all descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependency and new error variant needed by all subsequent phases.

- [X] T001 Add `ctrlc = "3"` to `[dependencies]` in `Cargo.toml`; run `cargo build` to verify it resolves
- [X] T002 Add `FocusError::SessionNotFound { id: i64 }` variant with message `"Session #{id} not found."` to `src/error.rs`

**Checkpoint**: `cargo build` passes. No logic changes yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: New DB operations, config persistence, and complete App state model refactor. No user story feature logic — only the skeleton that all stories build on.

**⚠️ CRITICAL**: No user story work begins until `cargo build` passes at end of this phase.

### DB Operations

- [X] T003 [P] Write failing unit tests for `delete_session` in `src/db/session_store.rs` `#[cfg(test)]`: (a) deletes a completed session by id and returns `Ok(())`; (b) returns `FocusError::SessionNotFound` when id does not exist; (c) does NOT delete an active session (end_time IS NULL guard) — confirm tests FAIL
- [X] T004 [P] Write failing unit tests for `rename_session` in `src/db/session_store.rs` `#[cfg(test)]`: (a) updates task name for given id; (b) returns `FocusError::SessionNotFound` for unknown id; (c) returns `FocusError::EmptyTask` when new name is blank — confirm tests FAIL
- [X] T005 [P] Implement `pub fn delete_session(conn: &Connection, id: i64) -> Result<()>` in `src/db/session_store.rs` — `DELETE FROM sessions WHERE id = ?1 AND end_time IS NOT NULL`; error on 0 rows affected — confirm T003 tests PASS
- [X] T006 [P] Implement `pub fn rename_session(conn: &Connection, id: i64, new_task: &str) -> Result<()>` in `src/db/session_store.rs` — validate non-empty, `UPDATE sessions SET task = ?1 WHERE id = ?2`; error on 0 rows affected — confirm T004 tests PASS

### Config Persistence

- [X] T007 Write failing unit tests inline in `src/config.rs` `#[cfg(test)]`: (a) `AppConfig::default()` returns `vim_mode: false`; (b) `save_config` writes valid JSON to a `tempfile` path; (c) `load_config` reads it back correctly; (d) `load_config` on missing file returns default; (e) `load_config` on malformed JSON returns default — confirm tests FAIL
- [X] T008 Create `src/config.rs` — define `AppConfig { vim_mode: bool }` with `#[derive(Serialize, Deserialize, Default)]`; implement `load_config(path: &Path) -> AppConfig` and `save_config(path: &Path, cfg: &AppConfig) -> Result<()>`; implement `config_file_path() -> PathBuf` using `dirs::config_dir()` — confirm T007 tests PASS
- [X] T009 Expose `pub mod config` in `src/lib.rs`

### App State Refactor

- [X] T010 Refactor `src/tui/app.rs`: remove `View` enum; add `Tab` enum (`Dashboard | Log | Report | Settings`); add `Overlay` enum (`None | Prompt { label: String, value: String, action: PromptAction } | ConfirmDelete { session_id: i64, session_name: String } | Help`); add `PromptAction` enum (`StartSession | RenameSession { id: i64 }`); add `active_tab: Tab`, `overlay: Overlay`, `log_selected: usize`, `config: AppConfig` fields to `App`; update `App::new(no_color, config)` signature; adapt `load_dashboard`, `load_log`, `load_report`, `tick_dashboard` to compile (no logic change); keep all existing fields
- [X] T011 Skeleton-rewrite `src/tui/events.rs`: replace `match &app.view.clone()` dispatch with two-level dispatch — overlay active → `handle_overlay()`; else → `match app.active_tab` calling stub functions `handle_dashboard_tab`, `handle_log_tab`, `handle_report_tab`, `handle_settings_tab`; preserve global `Ctrl+C` quit; stubs return `Ok(false)`; app compiles
- [X] T012 Rewrite `src/tui/ui.rs`: replace `match &app.view` with (1) render tab bar across top showing `[1]Dashboard  [2]Log  [3]Report  [4]Settings` with active tab highlighted, (2) dispatch content area to existing view render functions keyed by `app.active_tab`, (3) render overlay layer on top if `app.overlay != Overlay::None` (stubs for now); file must compile
- [X] T013 Create `src/tui/views/settings.rs` with `pub fn render(frame: &mut Frame, app: &App)` stub — renders "Settings" title block and placeholder text
- [X] T014 Update `src/tui/views/mod.rs` to `pub mod settings`
- [X] T015 [P] Update `src/tui/views/dashboard.rs` — update `render()` signature if needed to compile with new `App` fields; no logic change yet
- [X] T016 [P] Update `src/tui/views/log.rs` — update `render()` to accept `log_selected: usize` parameter and highlight selected row; compile against new App fields
- [X] T017 Update `src/tui/views/report.rs` — compile against new App (no logic change yet)

### Startup + Signal Handler

- [X] T018 Update `src/tui/mod.rs` — in `run()`: load `AppConfig` via `load_config(config_file_path())`; in `run_app()`: add `Arc<AtomicBool>` signal flag; install `ctrlc::set_handler` that sets the flag to `true`; check flag at top of event loop and `break` if set; after loop ends (any exit path), if `session_store::get_active_session` returns `Some(_)`, call `session_store::stop_session` (auto-save); pass `config` to `App::new`

**Checkpoint**: `cargo build` passes. `cargo run -- ui` shows tab bar skeleton. DB functions tested.

---

## Phase 3: User Stories 1 & 4 — Dashboard Session Controls (P1/P2)

**Goal**: Users can stop the active session and start a new named session directly from the Dashboard tab using single keys, with an inline name prompt. Auto-save on any quit path.

**Independent Test**: Launch `cargo run -- ui`. Press `n`, type "deep work", press Enter — session starts. Press `s` — session stops with elapsed time. Press `q` — no session lost. Verify `focus log` shows both sessions.

### Tests for US1 + US4 (write and confirm FAILING first)

- [X] T019 [P] [US1] Write failing unit tests for `handle_dashboard_tab` in `src/tui/events.rs` `#[cfg(test)]`: (a) `'s'` with active session → calls stop, app message set to success, overlay remains None; (b) `'s'` with no active session → error message shown, no crash; (c) active session present → pressing `'s'` does not open any overlay
- [X] T020 [P] [US4] Write failing unit tests for `handle_overlay_prompt` in `src/tui/events.rs` `#[cfg(test)]`: (a) printable char appends to `overlay.value`; (b) Backspace removes last char; (c) Esc clears overlay to `Overlay::None`; (d) Enter with empty value triggers `StartSession` with name "Untitled Session"; (e) Enter with non-empty value triggers `StartSession` with that name

### Implementation for US1 + US4

- [X] T021 [US1] Implement `handle_dashboard_tab` in `src/tui/events.rs` — `'s'`/`Enter`: if active session exists call `stop_session`, set success message, reload dashboard; if no session set error message; `'n'`: if active session exists set warning "Session already running"; else open `Overlay::Prompt { label: "Session name:".to_string(), value: String::new(), action: PromptAction::StartSession }` — confirm T019 PASS
- [X] T022 [US4] Implement `handle_overlay_prompt` in `src/tui/events.rs` — `Char(c)`: push to value; `Backspace`: pop from value; `Esc`: set overlay to None; `Enter`: dispatch `PromptAction`; extract `PromptAction::StartSession` handler: use value or "Untitled Session", call `insert_session`, set overlay to None, reload dashboard, show success — confirm T020 PASS
- [X] T023 [US1] Update `src/tui/views/dashboard.rs` help bar to `[N] New  [S] Stop  [Q] Quit` (remove old `[M] Menu` reference)
- [X] T024 [US4] Render `Overlay::Prompt` in `src/tui/ui.rs` — centered floating block (50% width, ~5 lines tall) with label on first line, current value with blinking-style cursor marker on second line, `[Enter] Confirm  [Esc] Cancel` hint on last line

**Checkpoint**: Dashboard controls work. 'n' opens inline prompt. 's' stops session. Auto-save triggers on Ctrl+C/SIGTERM. Confirm with `focus log`.

---

## Phase 4: User Story 2 — Tab Navigation (P2)

**Goal**: Dashboard, Session Log, and Report are always-accessible tabs on the main view, reachable by number keys or Tab key. Settings tab also accessible.

**Independent Test**: Press `2` → Session Log tab shows all past sessions. Press `3` → Report shows productivity metrics. Press `4` → Settings tab visible. Press `Tab` → cycles through all 4 tabs. Press `1` → back to Dashboard.

### Tests for US2 (write and confirm FAILING first)

- [X] T025 [P] [US2] Write failing unit tests for global tab switching in `src/tui/events.rs` `#[cfg(test)]`: (a) `'1'` sets `active_tab = Tab::Dashboard`; (b) `'2'` sets `Tab::Log` and calls `load_log`; (c) `'3'` sets `Tab::Report` and calls `load_report`; (d) `'4'` sets `Tab::Settings`; (e) `Tab` key cycles `Dashboard→Log→Report→Settings→Dashboard`
- [X] T026 [P] [US2] Write failing unit tests for Log tab navigation in `src/tui/events.rs` `#[cfg(test)]`: (a) `Down` increments `log_selected` up to `log_entries.len()-1`; (b) `Up` decrements `log_selected` down to 0; (c) `Right` advances page; (d) `Left` goes back page

### Implementation for US2

- [X] T027 [US2] Implement global tab switching in `handle_key_event` in `src/tui/events.rs` — `'1'`/`'2'`/`'3'`/`'4'` and `Tab`/`BackTab` set `app.active_tab`; switching to Log calls `load_log`; switching to Report calls `load_report` with current window; reset `log_selected = 0` on tab switch to Log — confirm T025 PASS
- [X] T028 [US2] Implement `handle_log_tab` in `src/tui/events.rs` — `Down`/`Up` keys move `log_selected` (clamped to list bounds); `Right`/`PgDn` next page; `Left`/`PgUp` prev page; `'q'`/`Esc` go to Dashboard tab — confirm T026 PASS
- [X] T029 [US2] Update `src/tui/app.rs` `load_log` to reset `self.log_selected = 0`; add bounds-check helper `fn clamp_log_selected(&mut self)` used after page change
- [X] T030 [US2] Update `src/tui/views/log.rs` — highlight selected row with reversed style; show `(empty)` message if no sessions; update help bar to `[↑↓] Select  [←→] Page  [D] Delete  [R] Rename  [Q] Back`
- [X] T031 [US2] Implement `handle_report_tab` in `src/tui/events.rs` — `'h'`/`Left` and `'l'`/`Right` cycle time window; remove `'1'`/`'2'`/`'3'` window shortcuts (now global tab keys); `'q'`/`Esc` go to Dashboard — confirm report tab compiles and `h`/`l` works
- [X] T032 [US2] Update `src/tui/views/report.rs` — remove `1:Today 2:This Week 3:Last7Days` legend; update help bar to `[H/L] Change Period  [Q] Back`
- [X] T033 [US2] Update `src/tui/ui.rs` tab bar — active tab shown with bold/highlight style; inactive tabs shown in dim style; tab bar renders above content area
- [X] T034 [US2] Update `src/tui/views/settings.rs` `render()` — show `Vim Mode: [ OFF ]` or `Vim Mode: [ ON ]` based on `app.config.vim_mode`; show `[V] Toggle` hint

**Checkpoint**: All 4 tabs accessible. Log tab scrollable. Report windows via h/l. Tab bar visible with active highlight.

---

## Phase 5: User Story 3 — Vim Mode Navigation (P3)

**Goal**: Users can enable vim mode in Settings; when enabled, `j`/`k`/`g`/`G` navigate all list views. Preference persists across restarts.

**Independent Test**: Go to Settings (`4`), press `v` — "Vim Mode: [ ON ]" appears. Press `2` (Log tab), press `j` repeatedly — selection moves down. Press `g` — jumps to top. Press `G` — jumps to bottom. Press `q`, relaunch — vim mode still ON. Press `4`, press `v` — vim mode OFF. Restart — vim mode OFF.

### Tests for US3 (write and confirm FAILING first)

- [X] T035 [P] [US3] Write failing unit tests for vim keys in `handle_log_tab` `#[cfg(test)]`: (a) `'j'` moves selection down when `config.vim_mode = true`; (b) `'j'` has no effect when `config.vim_mode = false`; (c) `'k'` moves up when vim enabled; (d) `'g'` sets `log_selected = 0` when vim enabled; (e) `'G'` sets `log_selected = len-1` when vim enabled
- [X] T036 [P] [US3] Write failing unit tests for settings toggle: `'v'` flips `config.vim_mode` and returns action to save config in `handle_settings_tab` `#[cfg(test)]`

### Implementation for US3

- [X] T037 [US3] Add vim key guards to `handle_log_tab` in `src/tui/events.rs` — `'j'`/`'k'`/`'g'`/`'G'` only active when `app.config.vim_mode == true`; `Down`/`Up` always active — confirm T035 PASS
- [X] T038 [US3] Implement `handle_settings_tab` in `src/tui/events.rs` — `'v'`: toggle `app.config.vim_mode`, call `save_config(config_file_path(), &app.config)`, show success message "Vim mode enabled" or "Vim mode disabled" — confirm T036 PASS
- [X] T039 [US3] Add vim key scroll to `handle_report_tab` in `src/tui/events.rs` — `'j'`/`'k'` scroll report rows list when `app.config.vim_mode == true` (add `report_scroll: usize` to App if report list is longer than screen area)

**Checkpoint**: Vim mode toggle persists. `j`/`k`/`g`/`G` navigate Log list when enabled. Arrow keys always work.

---

## Phase 6: User Story 5 — Quick Keys for Session Management (P3)

**Goal**: Users can delete and rename sessions in the Log tab with single keys and inline prompts. Help overlay (`?`) lists all shortcuts.

**Independent Test**: In Log tab, select a session, press `d`, confirm with `y` — session gone. Select another, press `r`, edit name, press Enter — name updated in list. Press `?` from any view — key binding overlay appears. Any key dismisses it.

### Tests for US5 (write and confirm FAILING first)

- [X] T040 [P] [US5] Write failing unit tests for delete flow in `src/tui/events.rs` `#[cfg(test)]`: (a) `'d'` with selected session opens `Overlay::ConfirmDelete { session_id, session_name }`; (b) `'d'` with no selection (empty log) does nothing; (c) in confirm overlay, `'y'` calls `delete_session` and resets overlay; (d) `'n'`/`Esc` in confirm overlay resets to `Overlay::None`
- [X] T041 [P] [US5] Write failing unit tests for rename flow in `src/tui/events.rs` `#[cfg(test)]`: (a) `'r'` with selected session opens `Overlay::Prompt` pre-filled with session task name; (b) `'r'` with no selection does nothing; (c) Enter in prompt with `PromptAction::RenameSession` calls `rename_session`
- [X] T042 [P] [US5] Write failing unit test for `'?'` key: global handler sets `app.overlay = Overlay::Help`; any subsequent key resets to `Overlay::None`

### Implementation for US5

- [X] T043 [US5] Wire `'d'` key in `handle_log_tab` in `src/tui/events.rs` — if `log_selected < log_entries.len()`, get session at index, open `Overlay::ConfirmDelete { session_id: session.id, session_name: session.task.clone() }`; else ignore — confirm T040a/T040b PASS
- [X] T044 [US5] Implement `handle_overlay_confirm` in `src/tui/events.rs` — `'y'`/`Enter`: call `delete_session(conn, session_id)`, reload log, clamp `log_selected`, show success message, reset overlay; `'n'`/`Esc`: reset overlay only — confirm T040c/T040d PASS
- [X] T045 [US5] Render `Overlay::ConfirmDelete` in `src/tui/ui.rs` — centered block (60% width, 5 lines): title "Confirm Delete", body `Delete "{name}"?`, buttons `[Y]es  [N]o`
- [X] T046 [US5] Wire `'r'` key in `handle_log_tab` in `src/tui/events.rs` — if session selected, open `Overlay::Prompt { label: "Rename session:".to_string(), value: session.task.clone(), action: PromptAction::RenameSession { id: session.id } }`; else ignore — confirm T041a/T041b PASS
- [X] T047 [US5] Implement `PromptAction::RenameSession` in `handle_overlay_prompt` in `src/tui/events.rs` — call `rename_session(conn, id, &value)`, reload log, preserve selection index, show success, clear overlay — confirm T041c PASS
- [X] T048 [US5] Wire `'?'` key in global handler in `src/tui/events.rs` — set `app.overlay = Overlay::Help`; in `handle_overlay_help`: any key clears overlay — confirm T042 PASS
- [X] T049 [US5] Render `Overlay::Help` in `src/tui/ui.rs` — centered scrollable block listing key bindings from contracts/key-bindings.md content (hard-coded as `const HELP_TEXT: &str = "..."` in ui.rs); dismiss with any key hint at bottom

**Checkpoint**: Delete/rename/help all functional. `cargo test` passes. All US5 acceptance scenarios verified manually.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T050 [P] Run `cargo clippy -- -D warnings` across entire project; fix all warnings in modified files
- [X] T051 [P] Run `cargo fmt` and verify no formatting diffs remain
- [X] T052 Verify `NO_COLOR=1 cargo run -- ui` produces no ANSI escape codes in session notification messages (success/error overlays render as plain text)
- [X] T053 Run all manual test scenarios from `specs/004-tui-session-controls/quickstart.md` — vim mode persist, force-close auto-save via `kill -TERM`, delete/rename flow, tab navigation, `?` help overlay
- [X] T054 Run full `cargo test` suite and confirm all tests pass including existing tests from prior features

**Checkpoint**: All tests green. Clippy clean. Manual scenarios pass.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Setup): No dependencies — start immediately
- **Phase 2** (Foundational): Depends on Phase 1 — **BLOCKS all user story phases**
- **Phase 3** (US1+US4): Depends on Phase 2 completion
- **Phase 4** (US2): Depends on Phase 2 completion; may start in parallel with Phase 3 if separately staffed
- **Phase 5** (US3): Depends on Phase 4 (vim keys apply to tabs established in US2)
- **Phase 6** (US5): Depends on Phase 4 (delete/rename operate on the Log tab's selection from US2)
- **Phase 7** (Polish): Depends on all story phases complete

### User Story Dependencies

| Story | Depends on | Notes |
|-------|-----------|-------|
| US1+US4 (Phase 3) | Phase 2 only | Independent of tab navigation — Dashboard actions work in any tab model |
| US2 (Phase 4) | Phase 2 only | Can start once App skeleton compiles |
| US3 (Phase 5) | US2 (Phase 4) | Vim keys apply to list views introduced in US2 |
| US5 (Phase 6) | US2 (Phase 4) | Delete/rename operate on Log tab selection from US2 |

### Within Each Phase

- Test tasks MUST be confirmed FAILING before implementation begins (TDD)
- Tests before models/services before UI
- Each phase ends with `cargo build` + `cargo test` green

### Parallel Opportunities

Within Phase 2:
- T003 + T004 (write DB tests) can run in parallel
- T005 + T006 (implement DB functions) can run after T003+T004 individually
- T015 + T016 + T017 (update views to compile) can run in parallel

Within Phase 3:
- T019 + T020 (write US1 and US4 tests) can run in parallel

Within Phase 6:
- T040 + T041 + T042 (write US5 tests) can all run in parallel

---

## Parallel Execution Example: Phase 2 Foundation

```
# These can run in parallel (different files):
Task T003: Write failing tests for delete_session in src/db/session_store.rs
Task T004: Write failing tests for rename_session in src/db/session_store.rs
Task T007: Write failing tests for AppConfig in src/config.rs

# After T003+T004 fail confirmed, these can run in parallel:
Task T005: Implement delete_session in src/db/session_store.rs
Task T006: Implement rename_session in src/db/session_store.rs
```

## Parallel Execution Example: Phase 6 US5

```
# All three test-writing tasks can run in parallel:
Task T040: Write failing delete flow tests in src/tui/events.rs
Task T041: Write failing rename flow tests in src/tui/events.rs
Task T042: Write failing '?' help key test in src/tui/events.rs
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 4 Only)

1. Complete Phase 1 (Setup)
2. Complete Phase 2 (Foundational — CRITICAL, blocks everything)
3. Complete Phase 3 (US1+US4 — Dashboard session controls)
4. **STOP and VALIDATE**: Start/stop sessions from Dashboard; auto-save on exit
5. This MVP delivers the most-impactful user story (P1) with minimal scope

### Incremental Delivery

1. Setup + Foundational → skeleton compiles, tab bar visible
2. Phase 3 (US1+US4) → session controls on Dashboard → demo-able MVP
3. Phase 4 (US2) → full tab navigation → Log and Report always accessible
4. Phase 5 (US3) → vim mode toggle → power user quality-of-life
5. Phase 6 (US5) → delete/rename/help → session management complete
6. Each phase is independently verifiable before proceeding

---

## Notes

- **[P]** tasks operate on different files — safe to parallelize
- **[Story]** label maps each task to its user story for traceability
- **TDD (NON-NEGOTIABLE, Principle II)**: Test tasks MUST appear before implementation tasks. Confirm tests FAIL before writing any implementation. No task is complete if its test was written after or skipped.
- **WAL mode (Principle V)**: `delete_session` and `rename_session` use the existing `conn` which already has WAL enabled via `db::open_db()` — no additional PRAGMA needed in new functions.
- **No AI attribution (Principle VI)**: Commit messages must not include Co-Authored-By lines.
- **PR Standards (Principle VII)**: Use `gh pr create` with `.github/PULL_REQUEST_TEMPLATE.md`. Title: `feat: add tui session controls with vim mode and tab views`. Link `specs/004-tui-session-controls/spec.md` and `tasks.md`. Include ≥2 manual test steps in test plan.
- Commit after each phase checkpoint: run `cargo clippy -- -D warnings` + `cargo fmt` before committing.
- `App::new()` signature change (adding `config: AppConfig` param) will require updating the call site in `src/tui/mod.rs` — handle in T018.
- The old `View::Menu` and `View::StartForm` variants are fully removed in T010 — delete their corresponding `src/tui/views/menu.rs` and `src/tui/views/start_form.rs` files in T010 as well (and remove from `mod.rs`).
