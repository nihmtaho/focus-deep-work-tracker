# Config File Contract

**Feature**: TUI Session Controls with Vim Mode and Tab Views  
**Version**: 1.0  
**Date**: 2026-04-03

---

## File Location

```
{config_dir}/focus/config.json
```

Where `{config_dir}` is:
- macOS: `~/Library/Application Support`
- Linux: `~/.config` (XDG_CONFIG_HOME or fallback)

Full path examples:
- macOS: `~/Library/Application Support/focus/config.json`
- Linux: `~/.config/focus/config.json`

The directory `{config_dir}/focus/` is created automatically on first write if it does not exist.

---

## Schema

```json
{
  "vim_mode": false
}
```

| Field | Type | Default | Valid values |
|-------|------|---------|-------------|
| `vim_mode` | boolean | `false` | `true` or `false` |

---

## Behavior

| Scenario | Behavior |
|----------|----------|
| File does not exist | Use defaults (`vim_mode: false`); file is created on first settings change |
| File exists but is malformed JSON | Use defaults silently; overwrite with valid config on next settings change |
| File has unknown fields | Unknown fields are ignored (forward-compatible) |
| Config directory does not exist | Created automatically via `std::fs::create_dir_all` |
| Write fails | Error surfaced as `MessageOverlay::error` in the TUI; in-memory config change still applies for session |

---

## Lifecycle

1. TUI starts → reads config file → loads `AppConfig` into `App.config`
2. User toggles vim mode in Settings tab → `App.config.vim_mode` toggled in memory → config file written immediately
3. TUI quits → no config write needed (already written on change)
