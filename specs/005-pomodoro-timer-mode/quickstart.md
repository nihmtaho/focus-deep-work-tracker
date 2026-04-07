# Quickstart: Pomodoro Timer Mode

## CLI Quick Start

### Start a Pomodoro session

```bash
focus start --pomodoro "refactor auth module"
focus start --pomodoro "refactor auth module" --tag backend
focus start --pomodoro "write docs" --work 30 --break 10
```

### Stop a running Pomodoro session

```bash
focus stop
# Output: Stopped. 2 pomodoros completed, 1 abandoned.
```

### View today's stats

```bash
focus pomo-stats --today
focus pomo-stats --week
```

### See all sessions (including pomodoro)

```bash
focus log
```

## TUI Quick Start

```bash
focus ui
# In the TUI:
# Press [S] on the dashboard to open the start dialog
# Select "Pomodoro" mode
# Enter task name → Enter tag (optional) → Configure durations → Confirm
# The timer view opens automatically
```

### TUI Keyboard Shortcuts (Pomodoro view)

| Key | Action |
|-----|--------|
| `P` | Pause / Resume timer |
| `S` | Skip current break phase |
| `+` | Extend current phase by 5 minutes |
| `Q` | Stop session (with confirmation) |
| `?` | Toggle help overlay |

## Configuration

Create `~/.config/focus/pomodoro.toml`:

```toml
work_duration_mins = 45
break_duration_mins = 10
long_break_duration_mins = 20
long_break_after = 4
```

### Environment variable overrides

```bash
FOCUS_POMODORO_WORK=30 focus start --pomodoro "task"
```

## Precedence

CLI flags > `FOCUS_POMODORO_*` env vars > `pomodoro.toml` config > built-in defaults
