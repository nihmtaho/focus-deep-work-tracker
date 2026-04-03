# Data Model: Focus TUI Dashboard

**Branch**: `003-focus-tui-dashboard` | **Date**: 2026-04-02

## Existing Persisted Entities (unchanged)

These live in `src/models/session.rs` and `src/db/session_store.rs`. No modifications.

### Session

| Field | Type | Notes |
|-------|------|-------|
| `id` | `i64` | SQLite AUTOINCREMENT primary key |
| `task` | `String` | Non-empty, required |
| `tag` | `Option<String>` | Optional label |
| `start_time` | `DateTime<Utc>` | Stored as Unix epoch integer |
| `end_time` | `Option<DateTime<Utc>>` | `None` = active session |

**Active invariant**: At most one session has `end_time = NULL` at any time (enforced by `FocusError::AlreadyRunning` check in `commands::start`).

---

## New TUI-Only State (in-memory, not persisted)

All types live in `src/tui/app.rs`.

### `TimeWindow` (enum)

Represents the selected time range in the Report view.

```
Today         — from today_start() (local midnight)
CurrentWeek   — from current_week_start() (local Monday midnight)
Last7Days     — from rolling_7d_start() (Utc::now() - 7 days)
```

Mapped to `commands::report::{today_start, current_week_start, rolling_7d_start}` — no duplication.

### `InputField` (enum)

Tracks which field has focus in the Start Form view.

```
Task    — cursor is in the task description input
Tag     — cursor is in the optional tag input
```

### `MessageKind` (enum)

Visual style for transient overlay messages.

```
Success  — displayed in green, auto-dismissed after 2 seconds
Warning  — displayed in yellow, auto-dismissed after 2 seconds
Error    — displayed in red, auto-dismissed after 3 seconds or on keypress
```

### `MessageOverlay` (struct)

A transient notification shown at the bottom of the screen.

| Field | Type | Notes |
|-------|------|-------|
| `text` | `String` | User-facing message text |
| `kind` | `MessageKind` | Determines color and dismiss timing |
| `shown_at` | `std::time::Instant` | Used to compute auto-dismiss |

### `View` (enum)

Represents the currently active screen. Only one `View` is active at a time.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Dashboard` | — | Live session status + today's tag summary |
| `Menu` | `selected: usize` | Main menu with highlighted item (0–5) |
| `StartForm` | `task: String`, `tag: String`, `active_field: InputField` | Multi-field text entry for new session |
| `Log` | `page: usize` | Paginated session log (10 rows/page) |
| `Report` | `window: TimeWindow`, `selected_window: usize` | Tag aggregation with selectable time window |

**Transitions**:
```
Dashboard  ←→  Menu (Tab / M / D keys)
Menu       →   StartForm  (S key or Enter on "Start Session")
Menu       →   Log        (L key or Enter on "Session Log")
Menu       →   Report     (R key or Enter on "Generate Report")
Menu       →   Dashboard  (D key or Enter on "Back to Dashboard", or Esc)
StartForm  →   Dashboard  (on success or Esc)
Log        →   Menu       (Esc or B)
Report     →   Menu       (Esc or B)
```

### `App` (struct)

Root application state. Passed to both `events.rs` (mutated) and `ui.rs` (read-only during render).

| Field | Type | Notes |
|-------|------|-------|
| `view` | `View` | Current active screen |
| `active_session` | `Option<Session>` | Re-queried from DB on every dashboard tick |
| `today_summary` | `Vec<(Option<String>, i64)>` | Tag → total seconds for today; re-queried on every dashboard tick |
| `log_entries` | `Vec<Session>` | Current page of sessions; fetched on Log view entry and page change |
| `log_total_pages` | `usize` | Computed from total completed session count and page size (10) |
| `report_rows` | `Vec<(Option<String>, i64)>` | Tag → total seconds for selected window; re-queried on window change |
| `message` | `Option<MessageOverlay>` | Transient notification; `None` when no message is active |
| `quit_pending` | `bool` | `true` after first Q press with active session; resets on any non-Q key |
| `terminal_too_small` | `bool` | Set when terminal width < 60 or height < 12 |

---

## Reused Business Logic (no duplication)

| Concern | Existing symbol | Module |
|---------|----------------|--------|
| Open/connect DB | `db::open_db()` | `src/db/mod.rs` |
| Get active session | `db::session_store::get_active_session` | `src/db/session_store.rs` |
| Insert session | `db::session_store::insert_session` | `src/db/session_store.rs` |
| Stop session | `db::session_store::stop_session` | `src/db/session_store.rs` |
| List sessions (paged) | `db::session_store::list_sessions(conn, limit)` | `src/db/session_store.rs` |
| Aggregate by tag | `db::session_store::aggregate_by_tag(conn, since)` | `src/db/session_store.rs` |
| Today start epoch | `commands::report::today_start()` | `src/commands/report.rs` |
| Week start epoch | `commands::report::current_week_start()` | `src/commands/report.rs` |
| Rolling 7d start | `commands::report::rolling_7d_start()` | `src/commands/report.rs` |
| Format duration | `display::format::format_duration` | `src/display/format.rs` |
| DB error type | `error::FocusError::DataFileCorrupted` | `src/error.rs` |
