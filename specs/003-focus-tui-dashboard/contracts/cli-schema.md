# CLI Contract: `focus ui`

**Branch**: `003-focus-tui-dashboard` | **Date**: 2026-04-02

## Subcommand

```
focus ui
```

**Aliases**: none  
**Flags**: none  
**Arguments**: none

## Behaviour Contract

### When stdout IS an interactive terminal

Launches the interactive TUI dashboard. Enters alternate screen (contents restored on exit), enables raw mode. Returns exit code 0 on normal quit.

### When stdout is NOT an interactive terminal (piped/redirected)

Prints to stderr:
```
Error: focus ui requires an interactive terminal.
```
Exits with code 1. No TUI state is rendered. Stdout is not written.

### When the database is missing or unreadable

Prints to stderr:
```
Error: Data file is corrupted or unreadable: /home/<user>/.local/share/focus/focus.db
```
Exits with code 1. No TUI state is rendered.

### When the terminal is smaller than 60×12

Renders a full-screen message (no alternate state entered until size is met):
```
Terminal too small. Minimum size: 60×12.
Current: <W>×<H>
Resize to continue or press Q to quit.
```
Responds to resize events and transitions to the dashboard once the minimum is met.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Normal quit (Q pressed, session either not running or second Q confirmed) |
| 1 | Fatal error (DB unavailable, not a TTY) |

## Backward Compatibility

All existing subcommands are unaffected:

```
focus start <TASK> [--tag <TAG>]
focus stop
focus status
focus log [--limit <N>]
focus report [--today | --week]
focus export --format <FORMAT>
```

The `ui` subcommand is purely additive. No existing subcommand changes behaviour, flags, or exit codes.

## Keyboard Interface (within TUI)

### Global keys (all views)

| Key | Action |
|-----|--------|
| `Q` | Quit (immediate if no active session; confirmation required if session active) |
| `Tab` | Toggle between Dashboard and Menu |

### Dashboard view

| Key | Action |
|-----|--------|
| `M` | Go to Menu |
| `D` | Stay on Dashboard (no-op) |

### Menu view

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Activate selected item |
| `S` | Start Session |
| `T` | Stop Current Session |
| `L` | Session Log |
| `R` | Generate Report |
| `D` / `Esc` | Back to Dashboard |

### Start Form view

| Key | Action |
|-----|--------|
| `Tab` | Switch between Task and Tag fields |
| `Enter` | Submit form (requires non-empty Task) |
| `Esc` | Cancel, return to Dashboard |
| Printable chars | Append to active field |
| `Backspace` | Delete last character in active field |

### Log view

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `N` | Next page |
| `P` | Previous page |
| `Esc` / `B` | Back to Menu |

### Report view

| Key | Action |
|-----|--------|
| `↑` / `k` | Previous time window |
| `↓` / `j` | Next time window |
| `Enter` | Select highlighted time window |
| `Esc` / `B` | Back to Menu |
