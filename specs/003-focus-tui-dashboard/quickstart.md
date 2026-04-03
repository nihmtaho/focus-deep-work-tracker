# Developer Quickstart: Focus TUI Dashboard

**Branch**: `003-focus-tui-dashboard`

## Prerequisites

- Rust stable 1.77+ (`rustup update stable`)
- macOS or Linux terminal emulator (≥ 60×12)

## Build

```bash
# Standard build (TUI always included, no feature flag needed)
cargo build

# Release build
cargo build --release
```

No additional system dependencies. SQLite is statically linked via `rusqlite bundled`.

## New Dependencies to Add to Cargo.toml

```toml
ratatui = "0.29"
crossterm = "0.28"
```

Add these under `[dependencies]` before starting implementation. Run `cargo build` to confirm they resolve.

## Run the TUI

```bash
# Launch interactive dashboard
cargo run -- ui

# Release binary
./target/release/focus ui
```

Requires an interactive terminal. Does not work when piped:
```bash
focus ui | cat   # → Error: focus ui requires an interactive terminal.
```

## Run Tests

```bash
# All tests (TUI unit tests are in tests/unit/tui/)
cargo test

# TUI-specific unit tests only
cargo test --test unit -- tui::

# Lint (must pass with no warnings)
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## TUI Module Layout

```
src/tui/
├── mod.rs          ← Entry point: tui::run(conn) sets up terminal and starts event loop
├── app.rs          ← App struct + View enum + all in-memory state
├── events.rs       ← Keyboard dispatch: handle_key_event(&mut app, key)
├── ui.rs           ← Render dispatch: render(frame, &app) called on every tick
└── views/
    ├── dashboard.rs
    ├── menu.rs
    ├── start_form.rs
    ├── log.rs
    └── report.rs
```

## Key Design Constraints

1. **All DB access uses existing `db::session_store::*` functions** — do not write raw SQL in `src/tui/`.
2. **All duration formatting uses `display::format::format_duration`** — do not write a new formatter.
3. **All time window calculations use `commands::report::{today_start, current_week_start, rolling_7d_start}`** — do not duplicate.
4. **Panic hook**: Install before entering alternate screen; always restore terminal state on panic.
5. **TDD**: Write tests in `tests/unit/tui/` before implementing each module. Run `cargo test` to confirm red, then implement to green.

## Phase Checkpoints

Each checkpoint requires: `cargo build` + `cargo clippy -- -D warnings` + `cargo test` all passing.

| Checkpoint | What must pass |
|------------|---------------|
| After adding dependencies | `cargo build` with ratatui + crossterm |
| After `tui::mod` skeleton | Terminal opens/closes cleanly, Q exits |
| After dashboard | Live elapsed time updates, today summary renders |
| After start/stop | Sessions can be created and stopped from TUI |
| After log view | Paginated log renders correctly |
| After report view | All three time windows produce correct totals |
| Final | Full `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` |
