# Key Binding Contract — 009 Polish Fixes

## Deletion Keys

| Panel | Key | Action | Guard |
|---|---|---|---|
| Dashboard (Todo list) | `Delete` | Delete selected todo | Only if `can_delete(id)` = true (no active session linked) |
| Dashboard (Todo list) | `Backspace` | Delete selected todo | Same guard |
| Log tab (Todo entries) | `Delete` | Delete selected todo | Same guard |
| Log tab (Todo entries) | `Backspace` | Delete selected todo | Same guard |
| Any input field (`TodoInput` context) | `Backspace` | Delete preceding character | No guard |

## Tab Navigation

| Key | Action | Notes |
|---|---|---|
| `d` | Navigate to Dashboard tab | Unchanged; no conflict with deletion |
| `l` | Navigate to Log tab | Unchanged |
| `s` | Navigate to Session tab | Unchanged |
| `t` | Navigate to Timer/Pomodoro tab | Unchanged |

## Vim Mode Keys

| Key | Action | Condition |
|---|---|---|
| `h` | Move left / previous panel | `vim_mode = true`, `Viewing` context |
| `j` | Move down | `vim_mode = true`, `Viewing` context |
| `k` | Move up | `vim_mode = true`, `Viewing` context |
| `l` | Move right / next panel | `vim_mode = true`, `Viewing` context |
| `d d` (within 1s) | Delete selected item | `vim_mode = true`, `Viewing` context |
| `g g` | Jump to top of list | `vim_mode = true`, `Viewing` context (future) |
| `G` | Jump to bottom of list | `vim_mode = true`, `Viewing` context (future) |

## Conflict Resolution

The former conflict (`d` = Dashboard tab AND `d` = delete in Log tab) is resolved by:
- `d` ALWAYS navigates to Dashboard tab (single press, any context)
- Deletion uses `Delete` / `Backspace` keys (single press) or `d d` (double press in vim mode)
- No ambiguity remains

## Timer Display Characters

All timer digit cells MUST use only:
- `█` (U+2588 FULL BLOCK) for "on" segments
- ` ` (SPACE) for "off" segments
- Box-drawing characters (`┌`, `┐`, `└`, `┘`, `│`, `─`) are PROHIBITED
