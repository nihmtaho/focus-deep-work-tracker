# Key Binding Contract

**Feature**: TUI Session Controls with Vim Mode and Tab Views  
**Version**: 1.0  
**Date**: 2026-04-03

This document defines the complete keyboard interface for the `focus ui` TUI after this feature is implemented.

---

## Global (all tabs, no overlay active)

| Key | Action | Notes |
|-----|--------|-------|
| `q` / `Q` | Quit TUI | Auto-saves active session if running |
| `Ctrl+C` | Quit TUI | Auto-saves active session if running |
| `?` | Show help overlay | Dismissed by any key |
| `Tab` | Next tab (cycles Dashboard → Log → Report → Settings → Dashboard) | — |
| `Shift+Tab` | Previous tab | — |
| `1` | Switch to Dashboard tab | — |
| `2` | Switch to Log tab | — |
| `3` | Switch to Report tab | — |
| `4` | Switch to Settings tab | — |
| `Esc` | Clear message notification | Only when no overlay is active |

---

## Dashboard Tab (normal mode)

| Key | Action | Notes |
|-----|--------|-------|
| `n` | New session — opens inline name prompt | Opens `Overlay::Prompt` |
| `s` / `Enter` | Stop active session | Ignored if no active session; shows notification |

---

## Log Tab (normal mode)

| Key | Action | Notes |
|-----|--------|-------|
| `Down` | Move selection down one item | Always active |
| `Up` | Move selection up one item | Always active |
| `j` | Move selection down (vim) | Only when vim mode enabled |
| `k` | Move selection up (vim) | Only when vim mode enabled |
| `g` | Jump to first item (vim) | Only when vim mode enabled |
| `G` | Jump to last item (vim) | Only when vim mode enabled |
| `Right` / `PgDn` | Next page | — |
| `Left` / `PgUp` | Previous page | — |
| `d` | Delete selected session | Opens `Overlay::ConfirmDelete`; ignored if no selection |
| `r` | Rename selected session | Opens `Overlay::Prompt` pre-filled with current name; ignored if no selection |

---

## Report Tab (normal mode)

| Key | Action | Notes |
|-----|--------|-------|
| `h` / `Left` | Previous time window (Today ← This Week ← Last 7 Days) | — |
| `l` / `Right` | Next time window | — |
| `j` / `Down` | Scroll report list down (vim) | Only when vim mode enabled and list is scrollable |
| `k` / `Up` | Scroll report list up (vim) | Only when vim mode enabled |

---

## Settings Tab (normal mode)

| Key | Action | Notes |
|-----|--------|-------|
| `v` | Toggle vim mode on/off | Saves to config file immediately |

---

## Overlay: Inline Prompt (input mode)

Active when `Overlay::Prompt { .. }` is set. **All printable keys are text input.**

| Key | Action |
|-----|--------|
| Any printable char | Appends to input value |
| `Backspace` | Deletes last character |
| `Enter` | Confirms prompt; executes `PromptAction` |
| `Esc` | Cancels prompt; returns to normal mode with no side effects |

---

## Overlay: Confirm Delete

| Key | Action |
|-----|--------|
| `y` / `Enter` | Confirms deletion |
| `n` / `Esc` | Cancels; selection preserved |

---

## Overlay: Help

| Key | Action |
|-----|--------|
| Any key | Dismisses overlay |

---

## Key Conflict Notes

- Number keys `1`–`4` are reserved for tab switching globally. Within the Report tab, the time window is controlled by `h`/`l` only — not number keys — to avoid conflicts.
- `n` on the Dashboard means "New session". In the Log tab, `n` (previously "next page") is superseded by `Right`/`PgDn` for pagination. The `n` key is not active in Log tab to avoid conflicting with vim muscle memory (`n` = search next in vim).
- `s` on the Dashboard means "Stop session". In all other contexts `s` has no binding.
- Vim keys (`j`/`k`/`g`/`G`) are only active when vim mode is enabled. Arrow keys are always active.
