# focus Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-06

## Active Technologies
- Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new) (003-focus-tui-dashboard)
- SQLite WAL mode via existing `db::open_db()`; no schema changes (003-focus-tui-dashboard)
- Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, dirs 5, chrono 0.4, colored 2, anyhow 1, thiserror 1; **new**: ctrlc 3 (004-tui-session-controls)
- SQLite WAL mode via existing `db::open_db()` — no schema changes (004-tui-session-controls)
- Rust stable 1.77+ + clap 4, rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3. New: `toml = "0.8"`. (005-pomodoro-timer-mode)
- SQLite WAL mode via existing `db::open_db()`. Two additive changes: `ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform'`; `CREATE TABLE IF NOT EXISTS pomodoro_stats (...)`. (005-pomodoro-timer-mode)

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
- 005-pomodoro-timer-mode: Added Rust stable 1.77+ + clap 4, rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3. New: `toml = "0.8"`.
- 004-tui-session-controls: Added Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, dirs 5, chrono 0.4, colored 2, anyhow 1, thiserror 1; **new**: ctrlc 3
- 003-focus-tui-dashboard: Added Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
