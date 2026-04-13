# Tasks: TUI Polish & Fixes (009)

**Input**: Design documents from `specs/009-tui-polish-fixes/`
**Branch**: `009-tui-polish-fixes`
**Issues**: #10, #11, #12, #13, #14, #15, #16

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared dependencies)
- **[Story]**: User story this task belongs to (US1–US7)

---

## Phase 1: Setup

**Purpose**: Confirm branch, toolchain, and verify baseline tests pass.

- [x] T001 Verify branch is `009-tui-polish-fixes` and `cargo test` baseline passes (all existing tests green)
- [x] T002 Run `cargo clippy -- -D warnings` and `cargo fmt --check` to confirm clean baseline

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Infrastructure needed by multiple user stories — `KeyAction::DeleteItem` variant, config persistence helper, and `App::has_active_pomodoro()` guard.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Add `KeyAction::DeleteItem` variant to the `KeyAction` enum in `src/tui/keyboard.rs`
- [x] T004 [P] Add `pub fn has_active_pomodoro(&self) -> bool` to `App` in `src/tui/app.rs` (returns `self.pomodoro_timer.is_some()`)
- [x] T005 [P] Add `pub fn save_config_now(&self) -> anyhow::Result<()>` to `App` in `src/tui/app.rs` that calls `crate::config::save_config(&crate::config::config_file_path(), &self.config)` and surfaces errors — no silent discards

**Checkpoint**: Foundation ready — all user story phases may begin.

---

## Phase 3: User Story 1 — Delete Key Conflict Resolution (Priority: P1) 🎯 MVP

**Goal**: `Delete`/`Backspace` deletes the selected todo in Log tab and Dashboard; `d` navigates to Dashboard tab unconditionally.

**Independent Test**: Press `d` in Log tab → Dashboard tab activates. Press `Backspace` in Log tab with todo selected → todo is removed. Session history records cannot be deleted.

### Tests (write first — must FAIL before implementation)

- [x] T006 [P] [US1] Write failing tests in `tests/integration/test_keyboard_bindings.rs`:
  - `test_delete_key_emits_delete_item_action` — verify `KeyCode::Delete` in `Viewing` context returns `KeyAction::DeleteItem`
  - `test_backspace_key_emits_delete_item_action` — verify `KeyCode::Backspace` in `Viewing` context returns `KeyAction::DeleteItem`
  - `test_d_key_always_navigates_to_dashboard` — verify `KeyCode::Char('d')` in `Viewing` context always returns `KeyAction::NavigateTab(TabTarget::Dashboard)`

### Implementation

- [x] T007 [US1] In `src/tui/keyboard.rs` `handle_key()`: add `KeyCode::Delete | KeyCode::Backspace` arm (in `Viewing` context) that returns `KeyAction::DeleteItem`; this arm must appear BEFORE any existing `Backspace` handling (ensure `d` arm remains unchanged — `NavigateTab(Dashboard)`)
- [x] T008 [US1] In `src/tui/events.rs`: route `KeyAction::DeleteItem` to `handle_todo_key(app, conn, KeyCode::Delete)` when the active tab is `Tab::Log` OR `Tab::Dashboard`; ignore when active tab is anything else or when in Input context
- [x] T009 [US1] Confirm `tests/integration/test_keyboard_bindings.rs` tests from T006 now pass; run `cargo test test_delete_key` to verify

**Checkpoint**: Delete/Backspace routes correctly. `d` still navigates. No conflict.

---

## Phase 4: User Story 2 — Todo Deletion from Dashboard (Priority: P1)

**Goal**: Pressing `Delete`/`Backspace` on a selected todo in the Dashboard panel removes it permanently.

**Independent Test**: Select a todo on Dashboard → press `Backspace` → todo gone from list → reopen app → todo still gone.

### Tests (write first — must FAIL before implementation)

- [x] T010 [P] [US2] Write failing tests in `tests/integration/test_keyboard_bindings.rs`:
  - `test_backspace_on_dashboard_with_todo_selected_deletes_todo` — App on Dashboard tab with todo list populated; dispatch `KeyAction::DeleteItem` → verify `app.todos` no longer contains the item after reload
  - `test_backspace_on_dashboard_empty_list_is_noop` — App on Dashboard with no todos; dispatch `KeyAction::DeleteItem` → no error, no panic

### Implementation

- [x] T011 [US2] In `src/tui/handlers_todo.rs` `handle_todo_key()`: ensure `KeyCode::Delete` triggers `todo::delete(db, todo_id)?` when a todo is selected; check `todo::can_delete(db, todo_id)` first; on failure show `MessageOverlay::warning` (already exists for linked sessions); on success show `MessageOverlay::success("Todo deleted")`; reload todo list via `app.load_todos(conn)?`
- [x] T012 [US2] In `src/tui/events.rs` routing (from T008): pass `KeyCode::Delete` to `handle_todo_key` when active tab is `Tab::Dashboard` and `app.focused_panel_idx` is the todos panel index (panel 1)
- [x] T013 [US2] Confirm tests from T010 now pass; run `cargo test test_backspace_on_dashboard` to verify

**Checkpoint**: Todos can be deleted from Dashboard. Deletion is permanent.

---

## Phase 5: User Story 4 — Pomodoro Timer Freezes on Session End (Priority: P1)

**Goal**: Timer display stops updating immediately when a Pomodoro/work session ends.

**Independent Test**: Start session → wait 30s → press ESC → timer shows ~0:30 and does not increment.

### Tests (write first — must FAIL before implementation)

- [x] T014 [P] [US4] Write failing tests in `tests/integration/test_timer_display.rs`:
  - `test_timer_does_not_tick_when_no_active_pomodoro` — create `App` with `pomodoro_timer = None`; call the tick/elapsed logic; assert timer value does not change
  - `test_timer_ticks_when_pomodoro_active` — create `App` with `pomodoro_timer = Some(...)` and `has_active_pomodoro()` = true; tick once; assert timer value increments

### Implementation

- [x] T015 [US4] In `src/tui/mod.rs` event loop: wrap the section that advances the Pomodoro timer elapsed counter inside `if app.has_active_pomodoro() { ... }`; the display render path must still run unconditionally (to show the frozen value)
- [x] T016 [US4] In `src/tui/app.rs` (or wherever `PomodoroEvent::SessionEnded` is handled): ensure `app.pomodoro_timer = None` is set when a session ends via ESC or natural completion; confirm `has_active_pomodoro()` returns `false` after this point
- [x] T017 [US4] Confirm tests from T014 now pass; run `cargo test test_timer` to verify

**Checkpoint**: Timer is frozen after session end. New session resets to zero.

---

## Phase 6: User Story 5 — Settings Persistence Across Restarts (Priority: P1)

**Goal**: `vim_mode` and `theme` settings survive app restarts by being written to disk immediately on change.

**Independent Test**: Toggle vim mode ON in TUI → quit → reopen → vim mode still ON. Set theme → quit → reopen → same theme.

### Tests (write first — must FAIL before implementation)

- [x] T018 [P] [US5] Write failing tests in `tests/integration/test_config_persistence.rs` (NEW FILE):
  - `test_save_config_now_writes_vim_mode` — create temp config path; create `App` with `config.vim_mode = true`; call `app.save_config_now()`; load config from temp path; assert `vim_mode == true`
  - `test_save_config_now_writes_theme` — same pattern for `config.theme = Some("material".into())`
  - `test_load_config_returns_defaults_when_missing` — call `load_config` on non-existent path; assert defaults are returned without panic

### Implementation

- [x] T019 [US5] In `src/tui/app.rs`: at every mutation site for `app.config.vim_mode` (the settings panel toggle), call `self.save_config_now()` immediately after; surface `Err` via `app.set_message(MessageOverlay::error(...))` — no `let _ = ...` discards
- [x] T020 [US5] In `src/tui/app.rs`: at every mutation site for `app.config.theme` (the settings panel toggle), call `self.save_config_now()` immediately after; surface errors the same way
- [x] T021 [US5] Confirm tests from T018 now pass; run `cargo test test_save_config test_load_config` to verify

**Checkpoint**: Settings persist across restarts. First-run creates default config.

---

## Phase 7: User Story 6 — Custom Theme Selection via CLI (Priority: P2)

**Goal**: `focus config set theme <name>` saves theme override; TUI loads it instead of OS auto-detect.

**Independent Test**: `focus config set theme material` → `focus ui` → material palette shown. `focus config set theme auto` → OS detection resumes.

### Tests (write first — must FAIL before implementation)

- [x] T022 [P] [US6] Write failing tests in `tests/integration/test_theme_loading.rs`:
  - `test_config_theme_overrides_auto_detect` — create `AppConfig { theme: Some("material".into()), .. }` and call `resolve_theme_from_config(&config)` (new function); assert `Theme::Material` returned
  - `test_config_theme_none_falls_back_to_auto` — `AppConfig { theme: None, .. }` → assert auto-detect path is followed
  - `test_config_theme_invalid_name_falls_back_to_auto` — `AppConfig { theme: Some("neon".into()), .. }` → assert fallback

- [x] T023 [P] [US6] Write failing integration test in `tests/integration/test_config_persistence.rs`:
  - `test_focus_config_set_theme_saves_to_file` — invoke `commands::config::run_set("theme", "onedark", &tmp_path)`; load config from `tmp_path`; assert `theme == Some("onedark")`
  - `test_focus_config_set_invalid_theme_errors` — invoke `run_set("theme", "neon", &tmp_path)`; assert `Err` is returned

### Implementation

- [x] T024 [US6] Create `src/commands/config.rs` with:
  - `pub fn run_set(key: &str, value: &str, config_path: &std::path::Path) -> anyhow::Result<()>`
  - Validate `key ∈ {"theme", "vim-mode"}`; for `theme` validate `value ∈ {"dark","light","material","onedark","auto"}` (or return error listing valid names); for `vim-mode` validate `value ∈ {"true","false"}`
  - Mutate `AppConfig`, call `save_config(config_path, &cfg)`; print `"Saved: {key} = {value}"` to stdout
  - `pub fn run_get(key: &str, config_path: &std::path::Path) -> anyhow::Result<()>` — prints current value

- [x] T025 [US6] In `src/main.rs`: add `Config { #[command(subcommand)] cmd: ConfigCmd }` variant to `Commands` enum; add `ConfigCmd { Set { key: String, value: String }, Get { key: String } }` enum; dispatch to `commands::config::run_set` / `run_get` using `config::config_file_path()`; add `mod config;` to `src/commands/mod.rs`

- [x] T026 [US6] Add `pub fn resolve_theme_from_config(config: &AppConfig) -> Option<Theme>` to `src/tui/themes.rs`: maps `Some("dark")` → `Theme::Dark`, `Some("light")` → `Theme::Light`, etc.; `None` or unknown → `None` (caller falls back to auto-detect); call this in the existing theme loading path at TUI startup so config override takes precedence

- [x] T027 [US6] Confirm tests from T022–T023 now pass; run `cargo test test_config_theme test_focus_config` to verify

**Checkpoint**: `focus config set theme material` works end-to-end. TUI uses custom theme on next start.

---

## Phase 8: User Story 3 — Sharp Block-Style Timer Digits (Priority: P2)

**Goal**: All digit segments in the flip-clock timer use `█` (U+2588); no box-drawing characters.

**Independent Test**: Run app with active session. All timer digit cells contain only `█` and space. No `┌`, `┐`, `└`, `┘`, `│`, `─` characters.

### Tests (write first — must FAIL before implementation)

- [x] T028 [P] [US3] Write failing tests in `tests/integration/test_timer_display.rs`:
  - `test_all_digits_use_block_chars_only` — for each digit 0–9, call the digit renderer; assert the output string contains no characters from the set `{┌,┐,└,┘,│,─,╭,╮,╰,╯}`
  - `test_colon_separator_uses_block_chars_only` — same for the colon separator

### Implementation

- [x] T029 [US3] In `src/tui/timer_display.rs`: audit all glyph/segment arrays for digits 0–9 and the colon; replace any occurrence of `┌`, `┐`, `└`, `┘`, `│`, `─`, `╭`, `╮`, `╰`, `╯` with `█` for "on" cells or ` ` for "off" cells; if a `GlyphStyle::Unicode` enum variant exists with box-drawing chars, replace its patterns or remove the variant and default to `GlyphStyle::Block`
- [x] T030 [US3] Confirm tests from T028 now pass; run `cargo test test_all_digits test_colon_separator` to verify

**Checkpoint**: Timer digits use only block characters. Renders cleanly in all terminal emulators.

---

## Phase 9: User Story 7 — Full Vim Mode Navigation (Priority: P3)

**Goal**: Vim mode supports `j`/`k` (already exists), `gg` (jump top), `G` (jump bottom), `dd` (delete) with 1-second partial-command window.

**Independent Test**: Enable vim mode. `j`/`k` navigate. `gg` jumps to top. `G` jumps to bottom. `dd` deletes selected todo. Single `d` does NOT navigate to Dashboard (waits for command).

### Tests (write first — must FAIL before implementation)

- [x] T031 [P] [US7] Write failing tests in `tests/integration/test_keyboard_bindings.rs`:
  - `test_vim_dd_within_timeout_emits_delete` — send `d` then `d` within 500ms; assert `KeyAction::DeleteItem` is returned on second `d`
  - `test_vim_dd_after_timeout_discards` — send `d`, sleep 1100ms, send `d` again; assert second `d` does NOT emit `DeleteItem` (first `d` starts new pending)
  - `test_vim_d_single_does_not_navigate_when_vim_mode` — in vim mode, send single `d`; assert action is `KeyAction::None` (not `NavigateTab`)
  - `test_vim_gg_emits_jump_top` — send `g` then `g` within 1s; assert `KeyAction::JumpTop`
  - `test_vim_G_emits_jump_bottom` — send `G`; assert `KeyAction::JumpBottom`

### Implementation

- [x] T032 [US7] Add `KeyAction::JumpTop` and `KeyAction::JumpBottom` variants to `KeyAction` enum in `src/tui/keyboard.rs`
- [x] T033 [US7] Add `pending_d: Option<std::time::Instant>` and `pending_g: Option<std::time::Instant>` fields to `KeyHandler` struct in `src/tui/keyboard.rs`; update `KeyHandler::new()` to initialise both as `None`; make `handle_key` take `&mut self` (currently `&self`) to allow mutation of pending state
- [x] T034 [US7] In `src/tui/keyboard.rs` `handle_key()` (now `&mut self`): implement vim `d` command composition — when `vim_mode` and `Viewing` context and `key == 'd'`: if `pending_d` is `None` set `pending_d = Some(Instant::now())` and return `KeyAction::None`; if `pending_d` is `Some(t)` and `t.elapsed() < 1s` clear `pending_d` and return `KeyAction::DeleteItem`; if elapsed ≥ 1s reset `pending_d = Some(Instant::now())` and return `KeyAction::None`
- [x] T035 [US7] In `src/tui/keyboard.rs`: implement vim `g`/`G` — `G` alone in vim mode → `KeyAction::JumpBottom`; first `g` sets `pending_g`, second `g` within 1s → `KeyAction::JumpTop`
- [x] T036 [US7] In `src/tui/events.rs`: handle `KeyAction::JumpTop` by setting `app.todo_selected = Some(0)` (or equivalent first-item index) and scrolling list to top; handle `KeyAction::JumpBottom` by setting selection to last item index
- [x] T037 [US7] Update all call sites of `keyboard_handler.handle_key(...)` in `src/tui/events.rs` (and anywhere else) from `&self` to `&mut self` now that `handle_key` is `&mut self`
- [x] T038 [US7] Confirm tests from T031 now pass; run `cargo test test_vim` to verify

**Checkpoint**: All vim mode commands (`j`, `k`, `gg`, `G`, `dd`) work correctly. `d` alone does not navigate in vim mode.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T039 Run `cargo clippy -- -D warnings` and fix any new lints introduced by this feature
- [x] T040 [P] Run `cargo fmt` across all modified files
- [x] T041 [P] Run full `cargo test` suite; confirm all 142+ tests pass (no regressions)
- [x] T042 [P] Manually validate quickstart.md acceptance scenarios 1–7 (see `specs/009-tui-polish-fixes/quickstart.md`)
- [x] T043 Update `CLAUDE.md` "Recent Changes" section with 009 implementation summary
- [x] T044 Commit polish changes; push `009-tui-polish-fixes` branch

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Setup): No dependencies — start immediately
- **Phase 2** (Foundational): Depends on Phase 1 — **BLOCKS all user stories**
- **Phase 3** (US1): Depends on Phase 2
- **Phase 4** (US2): Depends on Phase 3 (routes `KeyAction::DeleteItem` to the handler)
- **Phase 5** (US4): Depends on Phase 2 only — independent of US1/US2
- **Phase 6** (US5): Depends on Phase 2 only — independent of US1/US2/US4
- **Phase 7** (US6): Depends on Phase 6 (US5 persistence must be wired first)
- **Phase 8** (US3): Depends on Phase 2 only — fully independent
- **Phase 9** (US7): Depends on Phase 2; also depends on Phase 3 (reuses `KeyAction::DeleteItem`)
- **Phase 10** (Polish): Depends on all story phases

### User Story Dependencies Summary

```
Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2)
                  → Phase 5 (US4)
                  → Phase 6 (US5) → Phase 7 (US6)
                  → Phase 8 (US3)
                  → Phase 3 (US1) → Phase 9 (US7)
```

### Parallel Opportunities per Story

```
# After Phase 2 completes, these can run in parallel:
Phase 5 (US4 — timer freeze)
Phase 6 (US5 — settings persistence)
Phase 8 (US3 — block chars)
Phase 3 (US1 — delete key) → then Phase 4 and Phase 9 sequentially

# Within Phase 7 (US6): T022 and T023 test writing can run in parallel
# Within Phase 9 (US7): T031–T032 can start in parallel
```

---

## Implementation Strategy

### MVP First (P1 stories only)

1. Phase 1: Setup
2. Phase 2: Foundational
3. Phase 3: US1 — Delete conflict
4. Phase 4: US2 — Dashboard deletion
5. Phase 5: US4 — Timer freeze
6. Phase 6: US5 — Settings persistence
7. **STOP AND VALIDATE**: All P1 fixes working, push for review

### Incremental Delivery

1. Setup + Foundational → clean baseline
2. US1 + US2 → delete conflict resolved, todo deletion works
3. US4 → timer correctness fixed
4. US5 + US6 → settings persisted, theme CLI added
5. US3 → visual polish (timer block chars)
6. US7 → vim mode completed

---

## Notes

- **TDD (NON-NEGOTIABLE, Principle II)**: Test tasks must be written and confirmed FAILING before implementation begins. Mark tests `[x]` only after they fail, then implement.
- **Clippy gate**: Run `cargo clippy -- -D warnings` before each checkpoint commit; CI runs Rust 1.94 which may catch lints not visible locally.
- **No AI attribution (Principle VI)**: No `Co-Authored-By` in any commit message.
- **PR Standards (Principle VII)**: Final PR uses `gh pr create` with `.github/PULL_REQUEST_TEMPLATE.md`; title format `fix: TUI polish — delete conflict, timer freeze, settings persistence, block chars, vim mode`; link `specs/009-tui-polish-fixes/spec.md` and this tasks.md.
- `handle_key` signature change (`&self` → `&mut self`) in T033 requires updating every call site — check `events.rs` and any test helper that calls `handle_key` directly.
- Config file path: `~/.config/focus/config.json` (from `config::config_file_path()`). Integration tests MUST use a temp path — never the real user config.
