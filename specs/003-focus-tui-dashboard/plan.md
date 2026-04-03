# Implementation Plan: Focus TUI Dashboard

**Branch**: `003-focus-tui-dashboard` | **Date**: 2026-04-02 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `specs/003-focus-tui-dashboard/spec.md`

## Summary

Add an interactive terminal dashboard (`focus ui`) that displays real-time session status and provides keyboard-driven control over sessions, log browsing, and tag reporting — all sharing the existing SQLite database, session store, and business logic with no schema changes. The TUI is built on Ratatui + Crossterm with a 100ms poll-based event loop, is always included in the binary (no feature flag), and degrades gracefully to a CLI-style error message when the terminal is not interactive.

## Technical Context

**Language/Version**: Rust stable (1.77+)  
**Primary Dependencies**: clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new)  
**Storage**: SQLite WAL mode via existing `db::open_db()`; no schema changes  
**Testing**: `cargo test`; unit tests inline or in `tests/unit/tui/`; integration tests use `tempfile` (existing pattern)  
**Target Platform**: macOS Terminal, iTerm2, Linux terminal emulators; minimum 60×12 terminal  
**Project Type**: CLI tool — TUI as interactive subcommand  
**Performance Goals**: <500ms launch, <50ms render per frame, 100ms event poll interval  
**Constraints**: Single binary, no runtime deps, fully offline, no schema migrations  
**Scale/Scope**: Single user, local only

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Gate question | Status |
|---|---|---|
| I. Single Binary | Does this feature require a second binary, daemon, or system dependency? | PASS — ratatui and crossterm are statically linked; `cargo build --release` produces one binary |
| II. Test-First | Are tests planned before implementation tasks in tasks.md? | PASS — test tasks will appear before implementation tasks in each phase |
| III. Structured Error Handling | Do all error paths use `anyhow::Result` + `FocusError` variants? | PASS — DB failure at TUI launch uses existing `FocusError::DataFileCorrupted`; all fallible TUI paths propagate `anyhow::Result` |
| IV. Color-Independent Output | Is output readable without color? TTY detection confirmed? | PASS — FR-012 exits gracefully when not interactive; FR-013 disables color on `NO_COLOR`; Ratatui renders to raw terminal cells, not ANSI strings |
| V. Data Safety | Is WAL mode enabled on DB open? Is `DataFileCorrupted` surfaced on failure? | PASS — TUI reuses `db::open_db()` which already sets WAL mode and maps failures to `DataFileCorrupted` |
| VI. Commit Hygiene | No Co-Authored-By AI attribution in planned commits? | PASS |
| VII. Pull Request Standards | Will PRs follow title format, include spec/task links, and have test plans? | PASS |

No violations. Complexity Tracking table omitted.

## Project Structure

### Documentation (this feature)

```text
specs/003-focus-tui-dashboard/
├── plan.md              # This file
├── research.md          # Phase 0 — library evaluation & event loop design
├── data-model.md        # Phase 1 — TUI state entities
├── quickstart.md        # Phase 1 — developer onboarding
├── contracts/
│   └── cli-schema.md    # Phase 1 — focus ui subcommand contract
└── tasks.md             # Phase 2 output (speckit.tasks)
```

### Source Code

```text
src/
├── commands/            (existing — unchanged)
├── db/                  (existing — unchanged)
├── display/             (existing — unchanged)
├── error.rs             (existing — unchanged)
├── models/              (existing — unchanged)
├── lib.rs               (modify: add pub mod tui)
├── main.rs              (modify: add Commands::Ui variant, dispatch to tui::run)
└── tui/                 (new module)
    ├── mod.rs           (pub fn run() — terminal setup/teardown, event loop entry)
    ├── app.rs           (App struct, View enum, transient state, DB refresh)
    ├── events.rs        (handle_key_event — per-view dispatch table)
    ├── ui.rs            (render() — top-level frame dispatch)
    └── views/
        ├── mod.rs
        ├── dashboard.rs (render_dashboard — active session + today summary)
        ├── menu.rs      (render_menu — navigable option list)
        ├── start_form.rs(render_start_form — task/tag text inputs)
        ├── log.rs       (render_log — paginated session table)
        └── report.rs    (render_report — tag aggregation table)

tests/
├── integration/         (existing — CLI regression tests unchanged)
└── unit/
    └── tui/
        ├── app_test.rs       (View transitions, App state mutations, quit-confirm logic)
        └── truncate_test.rs  (text truncation helper with ellipsis)
```

**Structure Decision**: Option 1 (single project). The TUI is a new `src/tui/` module within the existing crate. All existing modules are reused without modification (except `lib.rs` and `main.rs` for wiring). No new crate, no workspace split.
