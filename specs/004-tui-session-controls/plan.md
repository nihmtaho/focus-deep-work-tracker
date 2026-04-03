# Implementation Plan: TUI Session Controls with Vim Mode and Tab Views

**Branch**: `004-tui-session-controls` | **Date**: 2026-04-03 | **Spec**: [spec.md](spec.md)  
**Input**: Feature specification from `specs/004-tui-session-controls/spec.md`

## Summary

Refactor the `focus ui` TUI to replace menu-driven navigation with a persistent tab bar (Dashboard / Log / Report / Settings), add inline session start/stop controls with a modal prompt overlay, support vim-style list navigation (configurable per-user), enable quick keys for session delete and rename in the Log tab, and guarantee active-session auto-save on any process exit. The architecture change is: `View` enum (full-screen switching) → `Tab` enum (active tab) + `Overlay` enum (modal layer). No schema changes required; two new DB functions and one new crate (`ctrlc`) are added.

## Technical Context

**Language/Version**: Rust stable 1.77+  
**Primary Dependencies**: ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, dirs 5, chrono 0.4, colored 2, anyhow 1, thiserror 1; **new**: ctrlc 3  
**Storage**: SQLite WAL mode via existing `db::open_db()` — no schema changes  
**Testing**: `cargo test`; unit tests inline (`#[cfg(test)]`); integration tests in `tests/integration/` with `tempfile` DB  
**Target Platform**: macOS and Linux (x86_64, aarch64) — single binary  
**Project Type**: CLI + TUI (single binary)  
**Performance Goals**: All key actions reflect in UI in < 1 second (SC-001); tab switch is instant (no DB I/O required on switch)  
**Constraints**: No new runtime dependencies beyond `ctrlc`; no schema migrations; single binary (Principle I)  
**Scale/Scope**: Single user; local SQLite; session list performance is not a concern at typical scale (< 10,000 rows)

## Constitution Check

| Principle | Gate question | Status |
|---|---|---|
| I. Single Binary | Does this feature require a second binary, daemon, or system dependency? | PASS — ctrlc is statically linked; no daemon |
| II. Test-First | Are tests planned before implementation tasks in tasks.md? | PASS — TDD cycle mandated per phase |
| III. Structured Error Handling | Do all error paths use `anyhow::Result` + `FocusError` variants? | PASS — new `SessionNotFound` variant added; all new paths use `anyhow::Result` |
| IV. Color-Independent Output | Is output readable without color? TTY detection confirmed? | PASS — ratatui renders structure via borders/text; color is additive only; `no_color` flag passed through |
| V. Data Safety | Is WAL mode enabled on DB open? Is `DataFileCorrupted` surfaced on failure? | PASS — existing `db::open_db()` with WAL unchanged; auto-save uses existing `stop_session` |
| VI. Commit Hygiene | No Co-Authored-By AI attribution in planned commits? | PASS |
| VII. Pull Request Standards | Will PRs follow title format, include spec/task links, and have test plans? | PASS |

## Project Structure

### Documentation (this feature)

```text
specs/004-tui-session-controls/
├── plan.md              ← this file
├── research.md          ← Phase 0 decisions
├── data-model.md        ← Phase 1 entities and state
├── quickstart.md        ← Phase 1 developer guide
├── contracts/
│   ├── key-bindings.md  ← keyboard interface contract
│   └── config-format.md ← config file schema
└── tasks.md             ← Phase 2 output (/speckit.tasks)
```

### Source Code

```text
src/
├── config.rs                      [NEW] AppConfig struct; load/save to {config_dir}/focus/config.json
├── error.rs                       [MODIFY] Add FocusError::SessionNotFound { id: i64 }
├── lib.rs                         [MODIFY] expose config module
├── tui/
│   ├── app.rs                     [MODIFY — major] Replace View enum with Tab + Overlay enums;
│   │                                add log_selected, config fields; retain existing data fields
│   ├── events.rs                  [MODIFY — major] Rewrite dispatch: tab routing, overlay/input mode,
│   │                                vim keys, dashboard quick keys, log delete/rename
│   ├── mod.rs                     [MODIFY] Add ctrlc signal handler; auto-save in run_app cleanup path
│   ├── ui.rs                      [MODIFY] Add tab bar render; dispatch to tab view renders;
│   │                                render overlays (Prompt, ConfirmDelete, Help) on top
│   └── views/
│       ├── dashboard.rs           [MODIFY] Update help hint to [N] New  [S] Stop; remove [M] Menu
│       ├── log.rs                 [MODIFY] Add selection highlight row; vim nav; delete/rename hint
│       ├── report.rs              [MODIFY] Remove 1/2/3 window shortcuts (replaced by h/l only)
│       ├── settings.rs            [NEW] Render vim mode toggle (ON/OFF indicator + [V] Toggle hint)
│       └── mod.rs                 [MODIFY] Add pub mod settings

db/
└── session_store.rs               [MODIFY] Add delete_session(conn, id) and rename_session(conn, id, task)

tests/
├── unit/
│   ├── config_test.rs             [NEW] AppConfig load/save/defaults/malformed
│   └── session_store_delete_rename_test.rs  [NEW] delete_session, rename_session happy + error paths
└── integration/
    └── tui_controls_test.rs       [NEW] Integration tests for inline prompt, tab state, auto-save path
```

**Structure Decision**: Single project layout (Option 1). All changes are within the existing `src/` and `tests/` trees. No new top-level directories.

## Complexity Tracking

> No constitution violations. Table omitted per template instructions.

---

## Phase 0: Research (Complete)

See [research.md](research.md) for all decisions with rationale and alternatives.

**Key resolved decisions:**
1. Tab navigation model: `Tab` enum + `Overlay` enum replaces `View` enum
2. Vim mode persistence: JSON config via existing `serde_json` + `dirs`
3. Signal handling: `ctrlc = "3"` crate with `Arc<AtomicBool>` flag pattern
4. Key conflict resolution: Number keys 1–4 = tab switch globally; Report uses h/l for windows
5. Inline prompt: `Overlay::Prompt { action: PromptAction }` modal layer
6. DB operations: `delete_session` + `rename_session` with `FocusError::SessionNotFound` guard

---

## Phase 1: Design & Contracts (Complete)

See [data-model.md](data-model.md) for entity definitions and state model.  
See [contracts/key-bindings.md](contracts/key-bindings.md) for the complete keyboard interface.  
See [contracts/config-format.md](contracts/config-format.md) for the config file schema.  
See [quickstart.md](quickstart.md) for developer workflow and manual test scenarios.

**Design summary:**

### App State Changes

```
// BEFORE
App {
    view: View,       // Dashboard | Menu | StartForm | Log | Report
    ...
}

// AFTER  
App {
    active_tab: Tab,  // Dashboard | Log | Report | Settings
    overlay: Overlay, // None | Prompt | ConfirmDelete | Help
    log_selected: usize,
    config: AppConfig,
    ...               // all existing data fields retained
}
```

### Event Handler Structure

```
handle_key_event(app, conn, key):
  1. If overlay active → dispatch to overlay handler (input/confirm/help mode)
  2. Else → dispatch to tab handler by app.active_tab
     - Tab::Dashboard → handle_dashboard_tab (n=new, s=stop, 1-4=switch)
     - Tab::Log       → handle_log_tab (nav, d=delete, r=rename, 1-4=switch)
     - Tab::Report    → handle_report_tab (h/l=window, 1-4=switch)
     - Tab::Settings  → handle_settings_tab (v=toggle vim)
  3. Global keys always checked first: q=quit, ?=help, Tab=next tab, 1-4=switch
```

### Auto-Save Pattern

```
// In run_app (tui/mod.rs):
let signal_flag = Arc::new(AtomicBool::new(false));
ctrlc::set_handler(clone signal_flag → set true)?;

loop {
    if signal_flag.load(Relaxed) { break; }
    // ... existing event loop
}

// After loop exits (any path):
if let Ok(Some(_)) = session_store::get_active_session(&conn) {
    let _ = session_store::stop_session(&conn);
}
```

### Agent Context Update

Run after artifacts are complete:
`.specify/scripts/bash/update-agent-context.sh claude`
