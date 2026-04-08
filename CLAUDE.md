# focus Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-08

## Active Technologies
- Rust stable (1.77+) + clap 4 (existing), rusqlite 0.31 bundled (existing), chrono 0.4 (existing), colored 2 (existing), anyhow 1 (existing), thiserror 1 (existing), dirs 5 (existing), **ratatui 0.29** (new), **crossterm 0.28** (new) (003-focus-tui-dashboard)
- SQLite WAL mode via existing `db::open_db()`; no schema changes (003-focus-tui-dashboard)
- Rust stable 1.77+ + ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, dirs 5, chrono 0.4, colored 2, anyhow 1, thiserror 1; **new**: ctrlc 3 (004-tui-session-controls)
- SQLite WAL mode via existing `db::open_db()` — no schema changes (004-tui-session-controls)
- Rust stable 1.77+ + clap 4, rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3. New: `toml = "0.8"`. (005-pomodoro-timer-mode)
- SQLite WAL mode via existing `db::open_db()`. Two additive changes: `ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform'`; `CREATE TABLE IF NOT EXISTS pomodoro_stats (...)`. (005-pomodoro-timer-mode)
- Rust 1.77+ (stable) + Existing: clap 4, rusqlite 0.31 (bundled), chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ratatui 0.29, crossterm 0.28, ctrlc 3, toml 0.8. New: `npm` CLI tooling, GitHub Actions (CI/CD) (006-npm-package-publish)
- SQLite (no schema changes) (006-npm-package-publish)
- Rust stable 1.77+ (existing project standard) + clap 4 (CLI), rusqlite 0.31 (`bundled` feature), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3, toml 0.8, serde_json 1 (007-ui-refresh)
- SQLite with WAL mode (existing db::open_db() wrapper); new `todos` table and `mode` column on sessions (007-ui-refresh)

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
- 007-ui-refresh: Added Rust stable 1.77+ (existing project standard) + clap 4 (CLI), rusqlite 0.31 (`bundled` feature), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3, toml 0.8, serde_json 1
- 006-npm-package-publish: Added Rust 1.77+ (stable) + Existing: clap 4, rusqlite 0.31 (bundled), chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ratatui 0.29, crossterm 0.28, ctrlc 3, toml 0.8. New: `npm` CLI tooling, GitHub Actions (CI/CD)
- 005-pomodoro-timer-mode: Added Rust stable 1.77+ + clap 4, rusqlite 0.31 (bundled), ratatui 0.29, crossterm 0.28, chrono 0.4, colored 2, anyhow 1, thiserror 1, dirs 5, ctrlc 3. New: `toml = "0.8"`.


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
