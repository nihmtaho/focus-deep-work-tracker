# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Stack

- **Rust stable 1.77+** — clap 4 (derive API), rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3, toml 0.8, serde_json 1
- **SQLite WAL** at `~/.local/share/focus/focus.db` via `db::open_db()`
- **Config JSON** at `~/.config/focus/config.json` via `src/config.rs`

## Commands

```bash
cargo test                   # run all tests (~302: lib + integration + unit)
cargo test <name>            # run a single test by name
cargo clippy -- -D warnings  # zero warnings policy
cargo fmt                    # required before every commit
```

## Architecture

```
src/
  main.rs          — CLI entry point (clap dispatch)
  lib.rs           — public re-exports
  config.rs        — FocusConfig load/save (JSON, ~/.config/focus/config.json)
  db/              — SQLite helpers: open_db(), migrations, queries
  models/          — domain types (Session, Todo, PomodoroStat, …)
  commands/        — one file per CLI subcommand (start, stop, log, status, config, …)
  tui/
    mod.rs         — event loop: poll(250ms), tick, crossterm setup/teardown
    app.rs         — App state: active tab, keyboard_context, config, theme, no_color
    keyboard.rs    — KeyHandler (pending multi-key state: dd/gg), KeyboardContext enum
    handlers_todo.rs — todo CRUD key handlers
    ui.rs          — top-level render dispatch
    views/         — per-tab render (dashboard, log, pomodoro, report)
    timer_display.rs — flip-clock 3×5 block-char renderer
    report.rs      — ReportMetrics aggregation + BarChart/Table rendering
    themes.rs      — load_theme(&name) → FocusTheme
    text_input.rs  — shared TextInput widget (vim Normal/Insert mode)
  theme/
    mod.rs         — Theme enum, from_name/name/resolve helpers, FocusTheme struct
    dark.rs / light.rs / material.rs / onedark.rs — theme implementations
  display/         — colored CLI output helpers
  pomodoro/        — pomodoro timer logic
  error.rs         — FocusError via thiserror
tests/
  integration/     — integration tests (DB, config, keyboard, timer, theme)
```

## Key Patterns

**Theme system**: `Theme::resolve(Option<&str>)` reads config, falls back to dark. `NO_COLOR` env var is checked once at startup and stored in `App.no_color`; all render paths guard on it.

**Keyboard context**: `App.keyboard_context` is `Normal | TodoInput | ConfirmQuit`. `KeyHandler::handle` dispatches on context first. Multi-key sequences (`dd`, `gg`) use `pending_d`/`pending_g` `Instant` fields with a 1s window.

**TUI event loop**: `event::poll(250ms)` — keys return immediately; timeout drives timer ticks. Pomodoro timer calls `tick_secs(elapsed, conn)`.

**Config CLI**: `focus config get/set theme` and `focus config get/set vim-mode` persist via `save_config()`.
