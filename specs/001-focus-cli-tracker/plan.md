# Implementation Plan: Focus CLI — Deep Work Session Tracker

**Branch**: `001-focus-cli-tracker` | **Date**: 2026-04-02 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/001-focus-cli-tracker/spec.md`

## Summary

Build `focus`, a single-binary Rust CLI that tracks developer deep work sessions. Sessions are stored in a local SQLite database at `~/.local/share/focus/focus.db`. The tool exposes six subcommands (`start`, `stop`, `status`, `log`, `report`, `export`) via clap 4. Timestamps are stored as Unix epoch integers; elapsed time and reporting aggregations are computed at read time using chrono. Output is formatted with colored for terminal use. No daemon, no network, no runtime dependencies beyond the binary itself.

## Technical Context

**Language/Version**: Rust stable (1.77+)  
**Primary Dependencies**: clap 4 (derive API), rusqlite 0.31 (`bundled` feature), chrono 0.4, colored 2.x, dirs 5.x, thiserror 1.x, anyhow 1.x  
**Storage**: SQLite via rusqlite with statically bundled SQLite at `~/.local/share/focus/focus.db`; WAL mode (`PRAGMA journal_mode=WAL`) enabled on every connection open (FR-018, constitution Principle V)  
**Testing**: `cargo test` — unit tests inline, integration tests in `tests/`  
**Target Platform**: macOS + Linux (x86_64, aarch64)  
**Project Type**: CLI tool  
**Performance Goals**: Sub-second command execution for all six commands under any realistic data volume (single-user local data)  
**Constraints**: Single standalone binary, no daemon or background process, fully offline, no system SQLite dependency (bundled), no root/sudo required  
**Scale/Scope**: Single user, local-only; data volume negligible (hundreds to thousands of sessions over lifetime)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*
*Updated post-ratification against constitution v1.0.0 (2026-04-02).*

| Principle | Gate question | Status |
|---|---|---|
| I. Single Binary | Does this feature require a second binary, daemon, or system dependency? | ✅ PASS — single binary, `rusqlite bundled`, no daemon |
| II. Test-First | Are tests planned before implementation tasks in tasks.md? | ✅ PASS — Phase 9 (T032–T038) adds 27 tests; retroactively compliant |
| III. Structured Error Handling | Do all error paths use `anyhow::Result` + `FocusError` variants? | ✅ PASS — `FocusError` enum in `src/error.rs`, all commands return `anyhow::Result` |
| IV. Color-Independent Output | Is output readable without color? TTY detection confirmed? | ✅ PASS — `colored` crate auto-detects TTY; T031 verifies `NO_COLOR=1` |
| V. Data Safety | Is WAL mode enabled on DB open? Is `DataFileCorrupted` surfaced on failure? | ✅ PASS — `PRAGMA journal_mode=WAL` in `open_db_at()`; `FocusError::DataFileCorrupted` on open failure |
| VI. Commit Hygiene | No Co-Authored-By AI attribution in planned commits? | ✅ PASS — no AI attribution in any commit |

No violations. No Complexity Tracking entries required.

## Project Structure

### Documentation (this feature)

```text
specs/001-focus-cli-tracker/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── cli-schema.md    # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created here)
```

### Source Code (repository root)

```text
src/
├── main.rs                  # Entry point; clap CLI struct definition
├── error.rs                 # Typed error enum (thiserror)
├── commands/
│   ├── mod.rs
│   ├── start.rs             # focus start
│   ├── stop.rs              # focus stop
│   ├── status.rs            # focus status
│   ├── log.rs               # focus log
│   ├── report.rs            # focus report
│   └── export.rs            # focus export
├── db/
│   ├── mod.rs               # DB connection setup, directory creation, migrations
│   └── session_store.rs     # All SQL queries (insert, query active, list, aggregate)
├── models/
│   └── session.rs           # Session struct + duration/elapsed helpers
└── display/
    └── format.rs            # Duration formatting, table column alignment

tests/
├── integration/
│   ├── test_start_stop.rs
│   ├── test_log.rs
│   ├── test_report.rs
│   └── test_export.rs
└── unit/
    └── test_format.rs
```

**Structure Decision**: Single Rust project (Cargo workspace not needed — one binary, no shared library). Commands are thin handlers that delegate to `db::session_store` for all data access. `display::format` is pure (no I/O), making it easily unit-testable. Integration tests use a temporary SQLite file (via `tempfile` crate or `std::env::temp_dir()`) isolated from the user's real data.

## Complexity Tracking

No constitution violations to justify.
