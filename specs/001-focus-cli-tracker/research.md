# Research: Focus CLI

**Phase**: 0 — Pre-design research  
**Date**: 2026-04-02  
**Branch**: `001-focus-cli-tracker`

## Decisions

---

### 1. SQLite Storage via rusqlite (bundled)

**Decision**: Use `rusqlite` with the `bundled` feature flag.

**Rationale**: Statically compiles SQLite 3.49.x into the binary, producing a truly standalone executable with no system SQLite dependency. Eliminates version mismatch across macOS and Linux distributions. Build-time overhead is acceptable for a developer tool compiled once.

**Alternatives considered**:
- System-linked rusqlite: lighter binary but runtime dependency; breaks on systems with old/missing SQLite
- `sled` (pure-Rust embedded KV store): no SQL aggregation; `report` grouping by tag would require manual computation in Rust
- `redb` (pure-Rust embedded): same limitation as sled for aggregation queries

---

### 2. Timestamp Storage as INTEGER (Unix Epoch Seconds)

**Decision**: Store `start_time` and `end_time` as `INTEGER` (Unix epoch seconds, UTC) in SQLite.

**Rationale**: Enables fast range queries (`WHERE start_time >= ?`) without parsing overhead; duration is a simple subtraction (`end_time - start_time`); timezone-neutral at storage layer, converted to local time only at display. `chrono::DateTime<Utc>::from_timestamp(secs, 0)` is direct.

**Alternatives considered**:
- TEXT (RFC3339): human-readable in raw DB inspection, but slower for range comparisons and slightly larger footprint
- REAL (fractional seconds): sub-second precision unnecessary per spec assumption

---

### 3. Data Directory via `dirs` Crate

**Decision**: Use `dirs::home_dir()` and manually append `.local/share/focus/` to resolve `~/.local/share/focus/`.

**Rationale**: The spec explicitly requires `~/.local/share/focus/` on both macOS and Linux. The `dirs` crate provides `home_dir()` with XDG compliance on Linux. Using `dirs` (not `directories`) avoids the `ProjectDirs` abstraction, which on macOS would resolve to `~/Library/Application Support/focus/` — contrary to spec.

**Alternatives considered**:
- `directories` crate with `ProjectDirs`: wrong macOS path for this spec's requirements
- Hardcoded `$HOME` env var: not portable; `dirs::home_dir()` handles edge cases (no HOME set)
- XDG_DATA_HOME env var with fallback: valid but adds complexity; `dirs` handles this already

---

### 4. clap 4 Derive API

**Decision**: Use `#[derive(Parser)]` + `#[derive(Subcommand)]` from clap 4.

**Rationale**: Type-safe, declarative, generates `--help` automatically, minimal boilerplate for the 6-subcommand structure. Mutually exclusive flags (`--today` vs `--week` on `report`) handled via `#[arg(conflicts_with = "week")]`.

**Alternatives considered**:
- clap builder API: more verbose, no ergonomic benefit for this scope
- `argh`: less feature-rich help output
- `pico-args`: no subcommand support built-in

---

### 5. `colored` 2.x for Terminal Output

**Decision**: Use `colored` 2.x.

**Rationale**: Respects `NO_COLOR` env var (per no-color.org standard); auto-detects non-TTY (piped output) and strips ANSI codes. Simple `"text".green()` API. No known macOS issues.

**Alternatives considered**:
- `owo-colors`: zero-alloc, slightly more ergonomic generic API, but less adoption for simple CLI use
- `termcolor`: more verbose, designed for multi-stream use (less needed here)
- `nu-ansi-term`: heavier, overkill for this use

---

### 6. Error Handling: thiserror + anyhow

**Decision**: `thiserror` for typed error enums in `error.rs` and `db/`; `anyhow` for `main.rs` and command handler propagation.

**Rationale**: Idiomatic Rust CLI pattern as of 2025. `thiserror` enables matchable, typed errors at the data layer (e.g., `FocusError::NoActiveSession`, `FocusError::DataFileCorrupted`). `anyhow` adds context strings and `?`-operator ergonomics in the CLI entry points.

**Alternatives considered**:
- Plain `Box<dyn Error>`: less informative for user-facing messages
- `eyre`: similar to anyhow, smaller ecosystem adoption

---

### 7. Integration Test Isolation

**Decision**: Integration tests use a per-test temporary SQLite file via `tempfile::NamedTempFile` (or `std::env::temp_dir()` + unique filename). DB path injected via a constructor argument on the store.

**Rationale**: Tests must never touch the user's real `~/.local/share/focus/focus.db`. Temporary file is cleaned up automatically on drop.

**Alternatives considered**:
- In-memory SQLite (`:memory:`): faster but can't test file creation/error behavior
- Environment variable override: works but couples test setup to env state
