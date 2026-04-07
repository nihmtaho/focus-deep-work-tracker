# Data Model: Integrated Pomodoro Timer Mode

## Database Schema Changes

### Modified: `sessions` table

Add a new optional column to record session mode.

```sql
ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'freeform';
```

Existing rows automatically get `mode = 'freeform'`. Pomodoro work-phase sessions
get `mode = 'pomodoro'`. No other columns change.

Updated full schema:

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task       TEXT    NOT NULL,
    tag        TEXT,
    start_time INTEGER NOT NULL,
    end_time   INTEGER,
    mode       TEXT    NOT NULL DEFAULT 'freeform'
);
```

### New: `pomodoro_stats` table

Per-day aggregate statistics for Pomodoro activity.

```sql
CREATE TABLE IF NOT EXISTS pomodoro_stats (
    date              TEXT    PRIMARY KEY,   -- ISO 8601 local date: "2026-04-06"
    completed         INTEGER NOT NULL DEFAULT 0,
    abandoned         INTEGER NOT NULL DEFAULT 0,
    work_minutes      INTEGER NOT NULL DEFAULT 0,
    break_minutes     INTEGER NOT NULL DEFAULT 0
);
```

- `date`: Local calendar date (not UTC). Streak calculation uses consecutive local dates.
- `completed`: Number of fully completed work phases.
- `abandoned`: Number of incomplete work phases discarded on session stop.
- `work_minutes`: Total accumulated work minutes (from completed phases only).
- `break_minutes`: Total accumulated break minutes (from completed break phases).

## Domain Types

### `PomodoroConfig`

```rust
pub struct PomodoroConfig {
    pub work_duration_mins: u32,         // default: 25, valid: 1–120
    pub break_duration_mins: u32,        // default: 5,  valid: 1–60
    pub long_break_duration_mins: u32,   // default: 15, valid: 1–60
    pub long_break_after: u32,           // default: 4,  expected: 2–8
}
```

Config file: `~/.config/focus/pomodoro.toml` (TOML format via `toml` crate).  
Resolution precedence: CLI flags > env vars > config file > built-in defaults.

Environment variables:
- `FOCUS_POMODORO_WORK` → `work_duration_mins`
- `FOCUS_POMODORO_BREAK` → `break_duration_mins`
- `FOCUS_POMODORO_LONG_BREAK` → `long_break_duration_mins`
- `FOCUS_POMODORO_LONG_BREAK_AFTER` → `long_break_after`

### `PomodoroPhase`

```rust
pub enum PomodoroPhase {
    Work,
    Break,
    LongBreak,
}
```

### `PomodoroTimer` (in-memory state machine)

```rust
pub struct PomodoroTimer {
    pub phase: PomodoroPhase,
    pub completed_phases: u32,           // count of fully completed WORK phases
    pub remaining_secs: u64,             // seconds left in current phase
    pub paused: bool,
    pub pause_started_at: Option<Instant>,
    pub total_pause_secs: u64,           // accumulated pause time this phase
    pub config: PomodoroConfig,
    pub task: String,
    pub tag: Option<String>,
    pub session_start: Instant,          // when the overall Pomodoro session started
}
```

State transitions:

```
Work(remaining > 0)  --tick--> Work(remaining - 1)
Work(remaining == 0) --tick--> Break | LongBreak  (save session record)
Break/LongBreak(remaining > 0)  --tick--> same(remaining - 1)
Break/LongBreak(remaining == 0) --tick--> Work(full duration)
Any --pause--> paused=true
Any(paused) --resume--> paused=false, pause_secs accumulated
Any(paused > 60min) --tick--> AutoAbandoned event emitted
```

### `TimerEvent`

Events emitted by `PomodoroTimer::tick()` to callers:

```rust
pub enum TimerEvent {
    Tick,                        // Normal tick, display update only
    PhaseComplete {              // Current phase ended; next phase started
        completed_phase: PomodoroPhase,
        next_phase: PomodoroPhase,
        saved_session_id: Option<i64>,  // Some if a work phase was saved
    },
    AutoAbandoned {             // Paused > 60 minutes; session force-ended
        completed_count: u32,
    },
    SessionComplete,             // Full cycle done (optional: no enforced max)
}
```

### `PomodoroStats`

```rust
pub struct PomodoroStats {
    pub date: String,           // "2026-04-06"
    pub completed: u32,
    pub abandoned: u32,
    pub work_minutes: u32,
    pub break_minutes: u32,
}
```

## Updated `Session` Model

Add `mode` field:

```rust
pub struct Session {
    pub id: i64,
    pub task: String,
    pub tag: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub mode: String,           // "freeform" | "pomodoro"
}
```

## Source File Layout

```
src/
├── pomodoro/
│   ├── mod.rs
│   ├── config.rs          # PomodoroConfig load/validate/resolve precedence
│   ├── timer.rs           # PomodoroTimer state machine + TimerEvent
│   └── notify.rs          # Desktop notification helpers (fire-and-forget)
├── db/
│   ├── mod.rs             # Migration: ADD COLUMN mode + CREATE pomodoro_stats
│   ├── session_store.rs   # Updated: mode field in all queries
│   └── pomodoro_store.rs  # CRUD for pomodoro_stats table
├── models/
│   ├── session.rs         # Add mode field
│   └── pomodoro.rs        # PomodoroStats, PomodoroPhase (re-exported)
├── commands/
│   ├── start.rs           # Add --pomodoro flag; dispatch to pomodoro CLI runner
│   ├── pomo_stats.rs      # `focus pomo-stats --today | --week`
│   └── report.rs          # Add mode breakdown section
├── tui/
│   ├── app.rs             # Add Tab::Pomodoro, PomodoroState field, pomodoro overlay types
│   ├── events.rs          # Add P/S/+/Q handlers for Pomodoro tab
│   └── views/
│       └── pomodoro.rs    # PomodoroView: countdown, progress bar, phase indicator
└── main.rs                # Add PomoStats subcommand; add --pomodoro to Start
```
