# focus Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-02

## Active Technologies
- Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new) (003-focus-tui-dashboard)
- SQLite WAL mode via existing `db::open_db()`; no schema changes (003-focus-tui-dashboard)

- Rust stable (1.77+) + clap 4 (derive API), rusqlite 0.31 (`bundled` feature), chrono 0.4, colored 2.x, dirs 5.x, thiserror 1.x, anyhow 1.x (001-focus-cli-tracker)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust stable (1.77+): Follow standard conventions

## Recent Changes
- 003-focus-tui-dashboard: Added Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new)

- 001-focus-cli-tracker: Added Rust stable (1.77+) + clap 4 (derive API), rusqlite 0.31 (`bundled` feature), chrono 0.4, colored 2.x, dirs 5.x, thiserror 1.x, anyhow 1.x

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
