# CLI Command Contracts: Pomodoro Timer Mode

## Modified: `focus start`

```
focus start [--pomodoro] <task> [--tag <tag>]
            [--work <mins>] [--break <mins>]
            [--long-break <mins>] [--long-break-after <count>]
```

### Flags

| Flag | Type | Valid Range | Description |
|------|------|-------------|-------------|
| `--pomodoro` | bool | — | Activate Pomodoro mode |
| `--work` | u32 | 1–120 | Work phase duration in minutes |
| `--break` | u32 | 1–60 | Break phase duration in minutes |
| `--long-break` | u32 | 1–60 | Long break duration in minutes |
| `--long-break-after` | u32 | — | Work phases before a long break |

### Behaviour (--pomodoro mode)

- Validates all duration flags before starting.
- Checks no active session exists (same guard as freeform start).
- Enters crossterm raw mode and begins the interactive CLI Pomodoro runner.
- Prints live countdown: `🍅 WORK | 24:58 | Pomodoro 1/4 | [P] pause  [S] skip  [Q] stop`
- Each completed work phase is persisted immediately to the sessions table.
- On exit (Q / Ctrl+C): prints summary of completed vs abandoned phases.

### Validation Errors

- `work must be between 1 and 120 minutes`
- `break must be between 1 and 60 minutes`
- `long-break must be between 1 and 60 minutes`
- Same "Session already running" guard as freeform start.

---

## New: `focus pomo-stats`

```
focus pomo-stats [--today | --week]
```

### Flags

| Flag | Description |
|------|-------------|
| `--today` | Show today's statistics (default if neither flag given) |
| `--week` | Show past 7 days as a daily breakdown + overall summary |

### `--today` Output

```
Pomodoro Statistics — Monday, 6 April 2026
──────────────────────────────────────────
Completed pomodoros :  4
Abandoned           :  1
Work time           :  1h 40m
Break time          :  25m
Current streak      :  3 days
```

### `--week` Output

```
Pomodoro Statistics — Last 7 Days
────────────────────────────────────────────────────────
Date        Completed  Abandoned  Work time  Break time
────────────────────────────────────────────────────────
2026-04-06      4          1        1h 40m     25m
2026-04-05      6          0        2h 30m     37m
...
────────────────────────────────────────────────────────
Total          10          1        4h 10m     62m
Best streak : 3 days
```

### Empty State

```
No Pomodoro sessions today.
```

---

## Modified: `focus log`

Each row gains a `Mode` column:

```
 #   Task                    Tag        Duration    Mode       Started
 1   refactor auth module    backend    25m 02s     pomodoro   Apr 06 10:00
 2   write tests             —          48m 12s     freeform   Apr 06 11:30
```

---

## Modified: `focus report`

Appends a mode-breakdown section when both modes exist:

```
Mode Breakdown
──────────────
Pomodoro   : 1h 40m  (67%)
Freeform   : 48m     (32%)
```
