# Data Model: TUI Session Controls with Vim Mode and Tab Views

**Branch**: `004-tui-session-controls` | **Phase**: 1 | **Date**: 2026-04-03

---

## Existing Entities (unchanged)

### Session (SQLite: `sessions` table)

No schema changes. The existing schema covers all requirements.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Stable identifier for delete/rename operations |
| `task` | TEXT | NOT NULL | Mutable — rename updates this field |
| `tag` | TEXT | NULL | Optional |
| `start_time` | INTEGER | NOT NULL | Unix epoch (UTC) |
| `end_time` | INTEGER | NULL | NULL = active session; NOT NULL = completed |

**State transitions:**
```
[created]  →  end_time IS NULL   (active)
[stopped]  →  end_time IS NOT NULL  (completed, immutable history)
```
Completed sessions are immutable except for `task` (rename) and deletion.

**Constraints enforced at the store layer:**
- Only completed sessions (`end_time IS NOT NULL`) may be deleted.
- `task` may not be set to empty string on rename.
- There is at most one active session at any time (enforced by `get_active_session` + app logic).

---

## New Entities

### AppConfig (file: `{config_dir}/focus/config.json`)

Persisted as JSON. Loaded on TUI start; saved on any mutation.

```json
{
  "vim_mode": false
}
```

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `vim_mode` | bool | `false` | When true, `j`/`k`/`g`/`G` navigate lists |

**Rust representation:**
```
AppConfig {
    vim_mode: bool,
}
```

**File path resolution:** `dirs::config_dir() / "focus" / "config.json"`  
**Fallback:** If the file does not exist or fails to parse, `AppConfig::default()` (`vim_mode: false`) is used silently; no error surfaced to the user.

---

## TUI State Model (in-memory, `src/tui/app.rs`)

### Tab Enum

Replaces the navigation role of the old `View::Menu` variant.

```
Tab::Dashboard
Tab::Log
Tab::Report
Tab::Settings
```

### Overlay Enum

Layered on top of the active tab. At most one overlay is active at a time.

```
Overlay::None
Overlay::Prompt { label: String, value: String, action: PromptAction }
Overlay::ConfirmDelete { session_id: i64, session_name: String }
Overlay::Help
```

### PromptAction Enum

Drives what happens when a `Prompt` overlay is confirmed.

```
PromptAction::StartSession
PromptAction::RenameSession { id: i64 }
```

### InputMode Enum

Derived from `Overlay` — no separate field needed; the event handler checks `app.overlay` to determine mode:
- `Overlay::None` → normal mode (navigation + quick keys active)
- `Overlay::Prompt { .. }` → input mode (all printable keys → value)
- `Overlay::ConfirmDelete { .. }` → confirm mode (`y`/Enter = confirm, `n`/Esc = cancel)
- `Overlay::Help` → any key dismisses

### Updated App Struct (key fields added/changed)

| Field | Type | Change | Notes |
|-------|------|--------|-------|
| `active_tab` | `Tab` | NEW | Replaces `view` |
| `overlay` | `Overlay` | NEW | Replaces StartForm/Menu in View |
| `log_selected` | `usize` | NEW | Selected row index in Log tab |
| `config` | `AppConfig` | NEW | Loaded from file on startup |
| `view` | `View` | REMOVED | Superseded by `active_tab` + `overlay` |

Existing fields retained: `active_session`, `today_summary`, `log_entries`, `log_total_pages`, `report_rows`, `message`, `quit_pending`, `terminal_too_small`, `no_color`.

---

## New DB Store Functions

### `delete_session(conn, id: i64) -> Result<()>`

```sql
DELETE FROM sessions WHERE id = ?1 AND end_time IS NOT NULL
```

Returns `FocusError::SessionNotFound { id }` if rows affected = 0.

### `rename_session(conn, id: i64, new_task: &str) -> Result<()>`

```sql
UPDATE sessions SET task = ?1 WHERE id = ?2
```

Returns `FocusError::SessionNotFound { id }` if rows affected = 0.  
Returns `FocusError::EmptyTask` if `new_task.trim().is_empty()`.

---

## New Error Variants (`src/error.rs`)

```
FocusError::SessionNotFound { id: i64 }
  → "Session #{id} not found."
```

---

## New Dependency

| Crate | Version | Justification |
|-------|---------|---------------|
| `ctrlc` | `3` | SIGTERM + SIGINT handler for auto-save on process termination. No overlap with existing stack. |
