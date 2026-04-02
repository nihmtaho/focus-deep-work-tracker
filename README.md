# focus

A minimal CLI for tracking deep work sessions. Start a timer, stop it when you're done, and review where your focused time went — all from the terminal, with no network, no accounts, and no background processes.

Data is stored locally in a SQLite database at `~/.local/share/focus/focus.db`.

**Platforms**: macOS, Linux (x86_64, aarch64)

---

## Prerequisites

- **Rust** stable 1.77+ — install via [rustup.rs](https://rustup.rs/)
- **C compiler** (required to compile bundled SQLite):
  - macOS: `xcode-select --install`
  - Ubuntu/Debian: `sudo apt install build-essential`
  - Fedora/RHEL: `sudo dnf install gcc`

---

## Install

**From source (recommended):**

```bash
git clone https://github.com/nihmtaho/focus-deep-work-tracker.git
cd focus-deep-work-tracker
cargo install --path .
```

**Development build:**

```bash
cargo build --release
# Binary at ./target/release/focus
```

Verify:

```bash
focus --help
```

---

## Quick Start

```bash
# 1. Start a session
focus start "refactor auth module" --tag rust

# 2. Check what you're working on
focus status

# 3. Stop when done
focus stop
```

Output:

```
Session started: refactor auth module  [tag: rust]

Working on: "refactor auth module"  [tag: rust]
Elapsed: 4m 22s

Stopped: "refactor auth module"  [tag: rust]
Duration: 4m 23s
```

---

## Commands

### `focus start <TASK> [--tag <TAG>]`

Start a new work session.

```bash
focus start "write unit tests"
focus start "review PR #42" --tag client-a
focus start "deep work block" -t research
```

- `TASK` is required and must not be empty
- `--tag` / `-t` is optional — used to group sessions in reports
- If a session is already running, prints a warning and exits without creating a new one

---

### `focus stop`

Stop the active session and print a summary.

```bash
focus stop
```

```
Stopped: "write unit tests"
Duration: 47m 12s
```

---

### `focus status`

Show the current session at a glance. Always exits `0` — safe to use in scripts.

```bash
focus status
```

```
Working on: "write unit tests"  [tag: rust]
Elapsed: 12m 05s
```

If no session is running:

```
No active session.
```

---

### `focus log [--limit <N>]`

List completed sessions in reverse chronological order.

```bash
focus log            # last 10 sessions (default)
focus log --limit 25
focus log -n 5
```

```
DATE                 TASK                              TAG          DURATION
2026-04-02 14:30     refactor auth module              rust         1h 12m 03s
2026-04-02 09:00     write unit tests                  —            45m 22s
2026-04-01 16:45     review PR #42                     client-a     28m 07s
```

`--limit` must be a positive integer. `--limit 0` returns an error.

---

### `focus report [--today | --week]`

Show time grouped by tag for a time window.

```bash
focus report           # current calendar week (Monday through now)
focus report --today   # today only
focus report --week    # last 7 rolling days
```

```
Tag              Total
──────────────────────
rust             3h 42m 00s
client-a         1h 15m 30s
untagged           28m 07s
──────────────────────
TOTAL            5h 25m 37s
```

Sessions with no tag appear as `untagged`. `--today` and `--week` are mutually exclusive.

---

### `focus export --format <FORMAT>`

Export all completed sessions to stdout. Redirect to save to a file.

```bash
focus export --format json > sessions.json
focus export --format markdown > sessions.md
```

**JSON output:**

```json
[
  {
    "id": 1,
    "task": "refactor auth module",
    "tag": "rust",
    "start_time": "2026-04-02T14:30:00Z",
    "end_time": "2026-04-02T15:42:03Z",
    "duration_seconds": 4323
  }
]
```

**Markdown output:**

```
| Date       | Task                 | Tag  | Start | End   | Duration   |
|------------|----------------------|------|-------|-------|------------|
| 2026-04-02 | refactor auth module | rust | 14:30 | 15:42 | 1h 12m 03s |
```

Accepted formats: `json`, `markdown`. Any other value prints an error and exits `1`.

---

## Data Storage

Sessions are stored at:

```
~/.local/share/focus/
└── focus.db    # SQLite database, created automatically on first run
```

To inspect raw data directly:

```bash
sqlite3 ~/.local/share/focus/focus.db "SELECT * FROM sessions ORDER BY start_time DESC LIMIT 10;"
```

To back up your data:

```bash
focus export --format json > ~/focus-backup-$(date +%Y%m%d).json
```

---

## Crash Recovery

If a session was running when your machine crashed or was shut down, it remains open. `focus status` will show it as still-running with elapsed time from the original start. Run `focus stop` to close it normally.

---

## Color Output

Color is automatically disabled when:

- stdout is piped or redirected (`focus log | grep rust`)
- The `NO_COLOR` environment variable is set (`NO_COLOR=1 focus status`)

---

## Running Tests

```bash
cargo test
```

27 tests covering all six commands, duration formatting, and edge cases. Integration tests use a temporary isolated database — your real data is never touched.

---

## Uninstall

Remove the binary:

```bash
cargo uninstall focus
```

Remove session data (irreversible):

```bash
rm -rf ~/.local/share/focus/
```
