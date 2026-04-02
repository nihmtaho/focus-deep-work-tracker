# Data Model: Focus CLI

**Phase**: 1 — Design  
**Date**: 2026-04-02  
**Branch**: `001-focus-cli-tracker`

## Database

**Engine**: SQLite (statically bundled via rusqlite)  
**Location**: `~/.local/share/focus/focus.db`  
**Created**: Automatically on first command invocation

---

## Schema

```sql
-- Applied once on DB open (CREATE IF NOT EXISTS)

CREATE TABLE IF NOT EXISTS sessions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    task        TEXT    NOT NULL,
    tag         TEXT,                    -- NULL = no tag
    start_time  INTEGER NOT NULL,        -- Unix epoch seconds, UTC
    end_time    INTEGER                  -- NULL = active session
);

-- Optimise active-session lookup (WHERE end_time IS NULL)
CREATE INDEX IF NOT EXISTS idx_sessions_end_time
    ON sessions(end_time);

-- Optimise date-range queries for report + log
CREATE INDEX IF NOT EXISTS idx_sessions_start_time
    ON sessions(start_time DESC);
```

---

## Entities

### Session

Represents a single tracked work interval.

| Field        | Type            | Nullable | Description                                      |
|--------------|-----------------|----------|--------------------------------------------------|
| `id`         | `i64`           | No       | Auto-assigned unique identifier                  |
| `task`       | `String`        | No       | Developer-provided description of the work       |
| `tag`        | `Option<String>`| Yes      | Short label for categorisation (e.g., "rust")    |
| `start_time` | `DateTime<Utc>` | No       | When the session started (stored as Unix seconds)|
| `end_time`   | `Option<DateTime<Utc>>` | Yes | When the session ended; `None` = active    |

**Rust struct**:

```rust
pub struct Session {
    pub id: i64,
    pub task: String,
    pub tag: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Session {
    /// True if end_time is None (session not yet stopped)
    pub fn is_active(&self) -> bool;

    /// Duration of a completed session; None if still active
    pub fn duration(&self) -> Option<chrono::Duration>;

    /// Elapsed time from start to now (for active sessions)
    pub fn elapsed(&self) -> chrono::Duration;
}
```

---

## State Transitions

```
[No session]
    │
    │  focus start <task> [--tag]
    ▼
[Active]  ──────────────────────────────────────────────────────────┐
    │                                                               │
    │  focus stop                          System crash / reboot   │
    ▼                                      (end_time stays NULL,   │
[Completed]                                session remains active) ─┘
```

- **Active**: `end_time IS NULL`. At most one row in this state at any time.
- **Completed**: `end_time IS NOT NULL`. Immutable after creation.
- **Orphaned** (crash recovery): remains in Active state on next startup; `focus status` surfaces it as still-running; user resolves via `focus stop`.

---

## Constraints & Validation

| Constraint | Where enforced |
|---|---|
| `task` must not be empty or whitespace-only | CLI layer (clap validator or command handler) |
| Only one active session at a time | DB layer: check `SELECT COUNT(*) WHERE end_time IS NULL` before INSERT |
| `--tag` is a single value per session | CLI layer (single `--tag` flag, not repeatable) |
| `start_time` always in UTC | Application layer: always use `Utc::now()` at session creation |
| `end_time > start_time` | Application layer: stop records `Utc::now()` which is always after start |
| `--limit N` must be a positive integer | CLI layer: clap value_parser + custom validator |

---

## Query Patterns

| Operation | SQL pattern |
|---|---|
| Get active session | `SELECT * FROM sessions WHERE end_time IS NULL LIMIT 1` |
| Stop session | `UPDATE sessions SET end_time = ? WHERE end_time IS NULL` |
| List recent (log) | `SELECT * FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time DESC LIMIT ?` |
| Report (by tag, date range) | `SELECT tag, SUM(end_time - start_time) FROM sessions WHERE end_time IS NOT NULL AND start_time >= ? GROUP BY tag` |
| Export all | `SELECT * FROM sessions WHERE end_time IS NOT NULL ORDER BY start_time ASC` |

---

## Data Directory Bootstrap

On every command invocation, before opening the DB:

1. Resolve `$HOME` via `dirs::home_dir()`
2. Construct path: `$HOME/.local/share/focus/`
3. `std::fs::create_dir_all(path)` — no-op if already exists
4. Open SQLite connection to `$HOME/.local/share/focus/focus.db`
5. Run `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` DDL
