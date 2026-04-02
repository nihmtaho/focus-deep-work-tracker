# CLI Contract: focus

**Version**: 1.0  
**Date**: 2026-04-02  
**Type**: Command-line interface schema

This document defines the public contract for the `focus` binary — all subcommands, flags, arguments, exit codes, and output conventions. Implementations must conform to this contract. Tests validate against it.

---

## Global Conventions

| Convention | Specification |
|---|---|
| Human-readable output | stdout |
| Error messages | stderr |
| Success exit code | `0` |
| Error exit code | non-zero (typically `1`) |
| Color output | Enabled when stdout is a TTY and `NO_COLOR` is not set |
| Color disabled | When stdout is piped, redirected, or `NO_COLOR` env var is set |

---

## Elapsed Time Format

Elapsed/duration values are formatted as:

```
[Xh ][Ym ]Zs
```

- Hours shown only if duration ≥ 1 hour
- Minutes shown only if duration ≥ 1 minute
- Seconds always shown

**Examples**:
- 45 seconds → `45s`
- 3 minutes 12 seconds → `3m 12s`
- 1 hour 23 minutes 45 seconds → `1h 23m 45s`

---

## Subcommands

### `focus start`

Start a new work session.

```
focus start <TASK> [--tag <TAG>]
```

**Arguments**:

| Name | Type | Required | Description |
|---|---|---|---|
| `TASK` | String (positional) | Yes | Description of the work to be done |
| `--tag <TAG>` / `-t <TAG>` | String | No | Category label for the session |

**Behaviour**:
- `TASK` must be non-empty and not purely whitespace
- If a session is already active: print warning with current task name and elapsed time; exit `1`; do NOT create a new session
- If `TASK` is empty/whitespace: print error; exit `1`

**Stdout (success)**:
```
Session started: <task>  [tag: <tag>]
```

**Stdout (session already active)**:
```
Session already running: "<task>" [tag: <tag>] — elapsed: <elapsed>
```

**Exit codes**: `0` = started, `1` = error

---

### `focus stop`

Stop the current active session.

```
focus stop
```

**Behaviour**:
- Records current UTC time as `end_time`
- Prints summary to stdout
- If no session is active: print error to stderr; exit `1`

**Stdout (success)**:
```
Stopped: "<task>"  [tag: <tag>]
Duration: <duration>
```

**Exit codes**: `0` = stopped, `1` = no active session

---

### `focus status`

Show the current session at a glance.

```
focus status
```

**Behaviour**:
- Always exits `0`
- If active session: print task, tag, elapsed time
- If no session: print idle message

**Stdout (active)**:
```
Working on: "<task>"  [tag: <tag>]
Elapsed: <elapsed>
```

**Stdout (idle)**:
```
No active session.
```

**Exit codes**: `0` always

---

### `focus log`

List completed sessions in reverse chronological order.

```
focus log [--limit <N>]
```

**Flags**:

| Flag | Type | Default | Description |
|---|---|---|---|
| `--limit <N>` / `-n <N>` | positive integer | `10` | Maximum number of sessions to show |

**Behaviour**:
- Lists only completed sessions (not the active session if one exists)
- `--limit N`: must be a positive integer; if `0`, negative, or non-numeric: print error "Error: --limit must be a positive integer" and exit `1`
- If no completed sessions exist: print "No sessions recorded yet."

**Stdout (table)**:
```
DATE                 TASK                              TAG          DURATION
2026-04-01 14:30     refactor auth module              rust         1h 12m 03s
2026-04-01 09:00     write unit tests                  —            45m 22s
```

**Exit codes**: `0` = success, `1` = invalid flag value

---

### `focus report`

Show time aggregated by tag for a time window.

```
focus report [--today | --week]
```

**Flags** (mutually exclusive):

| Flag | Description |
|---|---|
| *(default)* | Current calendar week: Monday 00:00:00 local time through now |
| `--today` | Today only: 00:00:00 local time through now |
| `--week` | Last 7 rolling days: 7 × 24 hours ago through now |

**Behaviour**:
- Groups completed sessions by tag; sessions with no tag grouped as `untagged`
- Columns are aligned; grand total row at the bottom
- If no sessions in the selected window: print "No sessions recorded for this period."

**Stdout**:
```
Tag              Total
──────────────────────
rust             3h 42m 00s
client-a         1h 15m 30s
untagged           28m 07s
──────────────────────
TOTAL            5h 25m 37s
```

**Exit codes**: `0` always (no data is not an error)

---

### `focus export`

Export all completed session history to stdout.

```
focus export --format <FORMAT>
```

**Flags**:

| Flag | Type | Required | Values |
|---|---|---|---|
| `--format <FORMAT>` / `-f <FORMAT>` | String | Yes | `json`, `markdown` |

**Behaviour**:
- Exports all completed sessions, ordered by `start_time ASC`
- Output goes to stdout; user can redirect to a file
- Invalid format value: print "Error: --format must be one of: json, markdown" to stderr; exit `1`
- If no sessions: output an empty but valid structure (`[]` for JSON; empty table with headers for Markdown)
- Active session is NOT included in export

**JSON output schema**:
```json
[
  {
    "id": 1,
    "task": "refactor auth module",
    "tag": "rust",
    "start_time": "2026-04-01T14:30:00Z",
    "end_time": "2026-04-01T15:42:03Z",
    "duration_seconds": 4323
  }
]
```

**Markdown output**:
```markdown
| Date | Task | Tag | Start | End | Duration |
|------|------|-----|-------|-----|----------|
| 2026-04-01 | refactor auth module | rust | 14:30 | 15:42 | 1h 12m 03s |
```

**Exit codes**: `0` = success, `1` = invalid format

---

## Error Message Conventions

All error messages are written to stderr and are human-readable. Format:

```
Error: <message>
```

Examples:
- `Error: No active session to stop.`
- `Error: Session already running: "write tests" — elapsed: 23m 14s`
- `Error: --limit must be a positive integer.`
- `Error: --format must be one of: json, markdown.`
- `Error: Task description cannot be empty.`
- `Error: Data file is corrupted or unreadable: /Users/you/.local/share/focus/focus.db`
