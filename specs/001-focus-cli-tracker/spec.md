# Feature Specification: Focus CLI — Deep Work Session Tracker

**Feature Branch**: `001-focus-cli-tracker`  
**Created**: 2026-04-02  
**Status**: Draft  
**Input**: User description: "Build a CLI tool called focus that helps developers track deep work sessions."

## Clarifications

### Session 2026-04-02

- Q: When the tool detects an active session that was never stopped (e.g., after a crash or reboot), what should it do? → A: Leave it open — treat it as still-running, show it in `focus status`, and let the user stop it manually. No data is discarded; elapsed time accumulates from the original start timestamp.
- Q: Should a session support more than one tag? → A: Single tag only — one optional `--tag` value per session, keeping the data model and report grouping simple.
- Q: Should the tool support any form of data export? → A: Yes — both JSON and Markdown formats via a `focus export` command.
- Q: When the data file is corrupted or unreadable, what should the tool do? → A: Fail with a clear error message that includes the file path — no silent data discard; the user retains control to inspect or repair the file.
- Q: How should `focus log` handle an invalid `--limit` value (0, negative, non-numeric)? → A: Reject with a clear error ("--limit must be a positive integer") and exit without output.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start and Stop a Work Session (Priority: P1)

A developer begins a focused work session by running `focus start "refactor auth module"`. The timer starts immediately. When done, they run `focus stop` and receive a summary showing what they worked on and how long.

**Why this priority**: This is the core loop — without start/stop, nothing else works. Every other feature depends on session data existing.

**Independent Test**: Can be fully tested by running `focus start`, waiting, then `focus stop` — delivers the ability to record a single completed session.

**Acceptance Scenarios**:

1. **Given** no session is running, **When** `focus start "task name"` is run, **Then** a session is created with the current timestamp and the message "Session started: task name" is displayed
2. **Given** no session is running, **When** `focus start "task name" --tag rust` is run, **Then** a session with tag "rust" is created
3. **Given** a session is already running, **When** `focus start "another task"` is run, **Then** the user sees a warning with the current task name and elapsed time, and no new session is started
4. **Given** a session is running, **When** `focus stop` is run, **Then** the session ends, duration is saved, and a summary (task name + total duration) is printed
5. **Given** no session is running, **When** `focus stop` is run, **Then** a friendly error is shown ("No active session to stop")

---

### User Story 2 - Check Current Session Status (Priority: P2)

A developer glances at their terminal and runs `focus status` to confirm what they're supposed to be working on and for how long they've been at it.

**Why this priority**: Developers context-switch frequently. Status provides instant grounding without interrupting flow. It's read-only and depends only on P1 data.

**Independent Test**: Can be tested after starting a session — displays the current task, tag, and elapsed time in a readable format.

**Acceptance Scenarios**:

1. **Given** a session is running, **When** `focus status` is run, **Then** task name, tag (if set), and elapsed time are shown (format: `1h 23m 45s`)
2. **Given** no session is running, **When** `focus status` is run, **Then** a clear message is shown ("No active session")

---

### User Story 3 - View Session History (Priority: P3)

A developer runs `focus log` at the end of the day to review what sessions they completed.

**Why this priority**: History review is useful but not critical — it only makes sense once multiple sessions exist. Comes after core session tracking.

**Independent Test**: Can be tested once 2+ completed sessions exist — displays a reverse-chronological list with date, task, tag, and duration.

**Acceptance Scenarios**:

1. **Given** completed sessions exist, **When** `focus log` is run, **Then** the last 10 sessions are shown in reverse chronological order with date, task name, tag, and duration
2. **Given** `focus log --limit 3` is run, **Then** only the 3 most recent sessions are shown
3. **Given** no completed sessions exist, **When** `focus log` is run, **Then** a message indicates no sessions recorded yet

---

### User Story 4 - Weekly Productivity Report (Priority: P4)

A developer runs `focus report` on Friday afternoon to understand where their focused time went during the week, grouped by tag.

**Why this priority**: Reporting is a summary layer on top of existing data. It provides the highest-level value but depends on P1–P3 being in place.

**Independent Test**: Can be tested once 1+ completed tagged sessions exist — shows grouped totals by tag and a grand total.

**Acceptance Scenarios**:

1. **Given** sessions exist for the current week, **When** `focus report` is run, **Then** time is grouped by tag with aligned columns and a grand total at the bottom
2. **Given** `focus report --today` is run, **Then** only today's sessions are included in the summary
3. **Given** `focus report --week` is run, **Then** sessions from the last 7 days (rolling window) are included
4. **Given** sessions have no tags, **When** `focus report` is run, **Then** untagged sessions are grouped under an "untagged" label
5. **Given** no sessions exist for the period, **When** `focus report` is run, **Then** a message indicates no data for the selected period

---

### User Story 5 - Export Session Data (Priority: P5)

A developer wants to back up their session history or analyze it in another tool. They run `focus export --format json` or `focus export --format markdown` and redirect the output to a file.

**Why this priority**: Portability is a nice-to-have that adds long-term value. It depends on all session data being in place and is purely additive.

**Independent Test**: Can be tested with any completed sessions present — produces valid JSON or Markdown output on stdout.

**Acceptance Scenarios**:

1. **Given** completed sessions exist, **When** `focus export --format json` is run, **Then** all session history is printed to stdout as valid JSON
2. **Given** completed sessions exist, **When** `focus export --format markdown` is run, **Then** all session history is printed to stdout as a Markdown table or document
3. **Given** an invalid format is passed (e.g., `--format csv`), **Then** a clear error lists the accepted values (`json`, `markdown`)
4. **Given** no completed sessions exist, **When** `focus export` is run, **Then** an empty but valid structure is output (e.g., `[]` for JSON, empty table for Markdown)

---

### Edge Cases

- **Resolved**: When the user's system clock changes mid-session (e.g., DST transition): `start_time` is stored as a UTC Unix epoch integer, so DST shifts only affect local-time display formatting — stored values are unaffected. No special handling required.
- **Resolved**: If the data file is corrupted or unreadable, the tool MUST exit with a clear error message that includes the file path. No automatic recovery or data deletion is attempted.
- **Resolved**: If `focus start` is called with an empty or whitespace-only task name, the system MUST reject the input with a clear error ("Task description cannot be empty") and exit with code 1.
- **Resolved**: If `--limit` is 0, negative, or non-numeric, the tool MUST reject the input with a clear error ("--limit must be a positive integer") and exit without producing any output.
- **Resolved**: If the data directory doesn't exist on first run, the system creates it automatically via `std::fs::create_dir_all` (see FR-012).
- **Resolved**: If a session was active when the machine crashed or was shut down, it remains open on the next run — `focus status` shows it as still-running and the user can stop it manually. Elapsed time is calculated from the original start timestamp.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `start` command that accepts a required task description and an optional `--tag` flag to begin a work session
- **FR-002**: System MUST reject a new `start` command if a session is already active, displaying the current task name and elapsed time
- **FR-003**: System MUST provide a `status` command that displays the active session's task name, tag, and elapsed time; if no session is active, it MUST display a clear idle message
- **FR-004**: System MUST provide a `stop` command that ends the active session, persists its duration, and prints a summary (task name + total time); if no session is active, it MUST show a friendly error
- **FR-005**: System MUST provide a `log` command that lists completed sessions in reverse chronological order, showing date, task name, tag, and duration
- **FR-006**: The `log` command MUST default to 10 sessions and MUST support a `--limit N` flag to override the default
- **FR-007**: System MUST provide a `report` command that aggregates completed session time grouped by tag for a configurable time window
- **FR-008**: The `report` command MUST default to the current calendar week (Monday through today); MUST support `--today` (today only) and `--week` (last 7 rolling days) flags
- **FR-009**: The `report` output MUST include total time per tag in aligned columns and a grand total row
- **FR-010**: System MUST allow only one active session at a time; concurrent sessions are not permitted
- **FR-011**: System MUST store all data locally in the standard user data directory (`~/.local/share/focus/` on macOS/Linux)
- **FR-012**: System MUST create the data directory on first run if it does not already exist
- **FR-013**: Task description MUST NOT be empty or whitespace-only; the system MUST reject such input with a clear error
- **FR-014**: If an active session is detected on startup after a crash or reboot, the system MUST treat it as still-running; `focus status` MUST display it with elapsed time calculated from the original start timestamp, and the user MUST be able to stop it normally via `focus stop`
- **FR-015**: System MUST provide an `export` command that outputs all completed session history in either JSON or Markdown format, selected via a `--format` flag (accepted values: `json`, `markdown`); output MUST be printed to stdout so users can redirect to a file
- **FR-016**: If the data file is corrupted or cannot be parsed, the system MUST exit with a non-zero status code and display a human-readable error message including the full path to the affected file; the system MUST NOT automatically modify or delete any data
- **FR-017**: The `log` command MUST reject a `--limit` value that is 0, negative, or non-numeric with a clear error message ("--limit must be a positive integer") and MUST exit without producing session output
- **FR-018**: The SQLite database MUST be opened with WAL (Write-Ahead Logging) mode enabled (`PRAGMA journal_mode=WAL`) on every connection open to ensure crash safety and data consistency

### Key Entities

- **Session**: A single work interval with a task description, optional tag, start timestamp, and end timestamp (null if active). A session is "active" when end timestamp is absent.
- **Tag**: A short string label used to categorize sessions (e.g., "rust", "client-a"). A session may have zero or one tag. Multiple tags per session are explicitly out of scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can start a new tracking session in under 5 seconds from the terminal (no prompts, no configuration required on first use)
- **SC-002**: `focus status` displays the current task and elapsed time in a single command invocation with no additional input
- **SC-003**: Session data persists correctly across terminal restarts and system reboots — a stopped session is always recoverable from local storage
- **SC-004**: `focus report` groups and totals time by tag accurately across any number of completed sessions for the selected period
- **SC-005**: Every subcommand's `--help` output includes a description and at least one usage example; a developer can determine correct invocation syntax without external documentation
- **SC-006**: The tool operates entirely offline with no network requests at any point

## Assumptions

- Users are running macOS or Linux; Windows support is out of scope
- A single user per machine — no multi-user or profile separation needed
- Session data is stored in a local SQLite database (`focus.db`); the `sqlite3` CLI can be used to inspect it directly
- The tool is distributed as a single installable binary or script — no daemon or background process required
- Elapsed time display in `focus status` reflects wall-clock time since session start; sub-second precision is not required
- "Current week" in `focus report` defaults to Monday as the start of the week (ISO week convention)
- Tags are case-sensitive strings; no tag normalization is applied
- Sessions shorter than 1 second are valid and should be recorded
- The `--week` flag means the last 7 calendar days (rolling window), distinct from the default current-week behavior
