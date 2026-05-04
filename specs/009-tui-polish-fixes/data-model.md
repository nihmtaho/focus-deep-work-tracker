# Data Model — 009 TUI Polish Fixes

No schema changes to SQLite. All entities already exist; changes are to Rust struct fields and function call sites.

---

## Entity: `AppConfig`

**Location**: `src/config.rs`
**Storage**: `~/.config/focus/config.json` (JSON via `serde_json`)

**Current fields** (unchanged):
| Field | Type | Default | Notes |
|---|---|---|---|
| `vim_mode` | `bool` | `false` | Whether hjkl/dd navigation is active |
| `theme` | `Option<String>` | `None` | `None` = OS auto-detect; `Some(name)` = manual override |
| `keyboard` | `KeyboardConfig` | default | Number/letter shortcut toggles |

**Validation rules**:
- `theme`: if `Some`, must be one of `["dark", "light", "material", "onedark"]`. CLI rejects unknown names. TUI falls back to auto-detect on unknown.
- `vim_mode`: boolean; no additional validation.

**State transitions**:
```
None (file missing) ──load──► AppConfig::default()
AppConfig ──set vim-mode──► AppConfig { vim_mode: true/false } ──save──► config.json
AppConfig ──set theme──► AppConfig { theme: Some("material") } ──save──► config.json
```

**Invariant**: `save_config` errors are NEVER silently discarded. TUI shows `MessageOverlay::error`; CLI prints to stderr and exits non-zero.

---

## Entity: `KeyHandler` (extended)

**Location**: `src/tui/keyboard.rs`

**New field**:
| Field | Type | Notes |
|---|---|---|
| `pending_d` | `Option<std::time::Instant>` | Set on first `d` keypress in vim mode; cleared after 1s or on second key |

**State machine for vim `d`**:
```
Idle ──d keypress──► PendingD { ts }
PendingD { ts } ──d within 1s──► Idle + emit DeleteItem
PendingD { ts } ──d after 1s──► Idle (discard)
PendingD { ts } ──other key within 1s──► Idle + re-dispatch other key
PendingD { ts } ──other key after 1s──► Idle + re-dispatch other key
```

---

## Entity: `PomodoroTimer` (no change)

**Location**: `src/tui/app.rs` — `app.pomodoro_timer: Option<PomodoroTimer>`

**Freeze behaviour** (new):
- `None` → timer display is static (shows last elapsed value); no tick increments.
- `Some(_)` → timer ticks normally.

---

## Key Binding Registry (updated)

| Context | Key | Before | After |
|---|---|---|---|
| Viewing | `d` | Navigate to Dashboard | Navigate to Dashboard (unchanged) |
| Viewing / Log tab | `Delete` | — (unbound) | Initiate todo delete |
| Viewing / Log tab | `Backspace` | — (unbound) | Initiate todo delete |
| Viewing / Dashboard | `Delete` | — (unbound) | Initiate todo delete |
| Viewing / Dashboard | `Backspace` | — (unbound) | Initiate todo delete |
| Viewing, vim mode | `d d` (within 1s) | — (unbound) | Initiate todo delete |
| TodoInput | `Backspace` | Delete char in field | Delete char in field (unchanged) |
