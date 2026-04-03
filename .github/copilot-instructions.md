# focus — Copilot Instructions

## Project Overview
`focus` is a Rust CLI productivity tracker with a TUI dashboard. It uses SQLite for persistence and ratatui for the terminal UI.

## Tech Stack
| Crate | Version | Notes |
|---|---|---|
| clap | 4 | derive API |
| rusqlite | 0.31 | `bundled` feature, WAL mode via `db::open_db()` |
| chrono | 0.4 | `serde` feature |
| colored | 2 | terminal coloring |
| dirs | 5 | XDG-style paths |
| thiserror | 1 | error types |
| anyhow | 1 | error propagation |
| serde / serde_json | 1 | serialization |
| ratatui | 0.29 | TUI dashboard |
| crossterm | 0.28 | terminal backend for ratatui |

## Project Structure
```
src/
  main.rs        # entry point, CLI dispatch
  lib.rs         # library root
  error.rs       # error types (thiserror)
  commands/      # one module per CLI subcommand
  db/            # database layer (open_db, migrations, queries)
  display/       # colored terminal output helpers
  models/        # domain types
  tui/           # ratatui TUI dashboard
tests/           # integration tests
```

## Commands
```bash
cargo test      # run all tests
cargo clippy    # lint
```

## Code Style
- Rust stable (1.77+), standard idioms
- Use `thiserror` for library errors, `anyhow` for command/bin errors
- Keep database logic in `db/`, display logic in `display/` and `tui/`
- No schema changes without updating migrations in `db/`
- Prefer `?` over `.unwrap()` / `.expect()` in non-test code
