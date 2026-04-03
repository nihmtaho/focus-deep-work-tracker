# Feature Specification: Focus TUI Dashboard

**Feature Branch**: `003-focus-tui-dashboard`  
**Created**: 2026-04-02  
**Status**: Draft  
**Input**: User description: "Build a Terminal User Interface (TUI) for the Focus CLI deep work tracker that provides real-time session monitoring and interactive menu-driven control, while maintaining full backward compatibility with the existing command-line interface."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real-Time Dashboard View (Priority: P1)

A user launches the TUI and sees a live dashboard showing whether a session is active, how long it has been running, and a summary of today's completed sessions grouped by tag — all updating in real time without manual refresh.

**Why this priority**: This is the core value proposition of the TUI: replacing the need to repeatedly run `focus status` by providing a persistent, always-current view. Every other feature builds on this foundation.

**Independent Test**: Launch `focus ui` with no active session, verify the dashboard shows "No active session" and today's summary. Start a session via CLI, verify elapsed time ticks live on the dashboard.

**Acceptance Scenarios**:

1. **Given** no session is running, **When** the user launches `focus ui`, **Then** the dashboard shows "No active session" and today's completed session summary grouped by tag.
2. **Given** a session is running, **When** the dashboard is visible, **Then** the active session's task name, tag, and elapsed time are displayed and the elapsed counter updates at least once per second.
3. **Given** the terminal is resized, **When** the dashboard is displayed, **Then** the layout reflows without crashing and remains readable at a minimum of 60 columns × 12 rows.
4. **Given** `NO_COLOR=1` is set in the environment, **When** the dashboard renders, **Then** all content is readable without color formatting.

---

### User Story 2 - Start and Stop Sessions via TUI (Priority: P2)

A user navigates to the menu from the dashboard, chooses to start a session, enters a task description and optional tag, and the session begins. Later, they stop it from the same menu and see a confirmation with the recorded duration.

**Why this priority**: Without session control in the TUI, users must switch to another terminal window to run CLI commands, breaking the TUI's utility as a self-contained interface.

**Independent Test**: Open TUI, start a session with a task and tag, verify dashboard reflects the active session, then stop it and verify the confirmation shows the correct task, tag, and a non-zero duration.

**Acceptance Scenarios**:

1. **Given** no active session, **When** the user selects "Start Session" and provides a task description, **Then** the session starts and the dashboard reflects it within one refresh cycle.
2. **Given** no active session, **When** the user attempts to start a session without providing a task description, **Then** the input is rejected with a clear inline error.
3. **Given** an active session already exists, **When** the user navigates to "Start Session", **Then** a warning is displayed instead of the input prompt.
4. **Given** an active session, **When** the user selects "Stop Current Session", **Then** the session ends and a confirmation showing task name, tag, and duration is displayed for 2 seconds before returning to the prior view.
5. **Given** no active session, **When** the user selects "Stop Current Session", **Then** a message stating "No active session" is shown.
6. **Given** an active session, **When** the user presses Q, **Then** a warning "Session still running — press Q again to quit" is shown; pressing Q a second time exits the TUI while the session continues running; pressing any other key dismisses the warning.
7. **Given** no active session, **When** the user presses Q, **Then** the TUI exits immediately.

---

### User Story 3 - Session Log View (Priority: P3)

A user opens the session log from the menu and browses recent sessions in reverse chronological order, with columns for date, time, task, tag, and duration. They can scroll through entries and paginate if there are more than fit on screen.

**Why this priority**: Log browsing is a read-only reference feature. It delivers real value but does not block the core tracking workflow.

**Independent Test**: With at least 11 completed sessions in the database, open the log view and verify the first page shows 10 entries newest-first, that the page indicator shows "1/N pages", and that pressing N advances to page 2.

**Acceptance Scenarios**:

1. **Given** completed sessions exist, **When** the user opens the session log, **Then** the most recent 10 sessions are shown with Date, Time, Task, Tag, and Duration columns.
2. **Given** more sessions than fit on one page, **When** the user presses N, **Then** the next page of sessions is displayed and the page indicator updates.
3. **Given** the user is on a page beyond the first, **When** they press P, **Then** the previous page is shown.
4. **Given** no completed sessions, **When** the log view is opened, **Then** a message indicating no sessions are recorded is shown.

---

### User Story 4 - Report View (Priority: P4)

A user opens the report view and sees their time aggregated by tag for a chosen time window (Today, Current Week, Last 7 Days). The report includes an "untagged" row for sessions without tags and a TOTAL row.

**Why this priority**: Reporting is analytical and supplementary to active tracking. It adds value but does not affect core session management.

**Independent Test**: With sessions tagged across at least two tags in the current week, open the report, select "Current Week", and verify each tag appears as a row with accurate total time and session count, plus a TOTAL row.

**Acceptance Scenarios**:

1. **Given** the report view is open, **When** a time window is selected (Today / Current Week / Last 7 Days), **Then** the report updates to show tag rows with total time and session count for that period.
2. **Given** sessions exist without a tag, **When** the report is shown, **Then** those sessions appear under an "untagged" row.
3. **Given** any time window, **When** the report is displayed, **Then** a TOTAL row summarising all time is shown at the bottom.
4. **Given** no sessions in the selected period, **When** the report is displayed, **Then** a message indicating no data for the period is shown (not a blank screen).

---

### Edge Cases

- If the database file is missing or unreadable at launch, the TUI prints a clear error message (e.g., "Cannot open database at ~/.local/share/focus/focus.db: <reason>") and exits with a non-zero status code — consistent with existing CLI failure behaviour.
- When the user presses Q while a session is active, a one-line warning is displayed ("Session still running — press Q again to quit") and a second Q keypress is required to exit; pressing any other key dismisses the warning and resumes normal operation. If no session is active, Q exits immediately.
- What happens when the terminal is smaller than 60 × 12?
- What happens when the output is piped or redirected (non-interactive terminal)?
- If a session is started or stopped via CLI while the TUI is open, the dashboard reflects the change automatically on the next refresh tick (~100ms) via polling — no manual reload required.
- Task names and tags that exceed their allocated column width are truncated with a trailing `…`; column alignment is always preserved and row height is always one line.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The TUI MUST be launchable via a dedicated subcommand (`focus ui`) that is additive to the existing CLI; all existing CLI commands MUST remain fully functional.
- **FR-002**: The dashboard MUST display the active session's task, tag, and live elapsed time, refreshing at least once per second. All displayed state (active session, today's summary) MUST be read from the database on each refresh cycle so that changes made via external CLI commands are reflected automatically.
- **FR-003**: The dashboard MUST display today's completed sessions aggregated by tag, showing tag name, total duration, and session count.
- **FR-004**: Users MUST be able to navigate the main menu using arrow keys (↑/↓), vim-style keys (j/k), and single-letter shortcuts (S/T/L/R/D/Q).
- **FR-005**: Users MUST be able to start a new session from the TUI by entering a task description (required) and an optional tag.
- **FR-006**: The TUI MUST prevent starting a duplicate session when one is already active, displaying a warning instead of the input form.
- **FR-007**: Users MUST be able to stop the active session from the TUI; a confirmation with task name, tag, and duration MUST be shown for 2 seconds.
- **FR-008**: The session log MUST display completed sessions in reverse chronological order with columns: Date, Time, Task, Tag, Duration. Text that exceeds column width MUST be truncated with a trailing `…`; each row occupies exactly one line.
- **FR-009**: The session log MUST support keyboard scrolling (↑/↓, j/k) and pagination (N/P for next/previous page) with a page indicator.
- **FR-010**: The report view MUST support at minimum three selectable time windows: Today, Current Calendar Week (Monday to present), and Last 7 Rolling Days.
- **FR-011**: The report view MUST include an "untagged" group for sessions without a tag and a TOTAL row.
- **FR-012**: The TUI MUST switch to a plain CLI-compatible mode (or refuse gracefully with a clear message) when stdout is not an interactive terminal (piped/redirected).
- **FR-013**: The TUI MUST disable color output when the `NO_COLOR` environment variable is set.
- **FR-014**: If the database is missing or unreadable at launch, the TUI MUST print a descriptive error message to stderr and exit with a non-zero status code; no partial TUI state should be rendered.
- **FR-015**: Error messages MUST be displayed at the bottom of the screen in a distinct style and auto-clear after 3 seconds or on the next keypress.
- **FR-016**: Success and warning messages MUST be displayed for 2 seconds then automatically dismissed.
- **FR-017**: The TUI MUST handle terminal resize events gracefully without crashing.
- **FR-018**: The TUI MUST enforce a minimum terminal size of 60 columns × 12 rows; if the terminal is smaller, a clear message must be shown.
- **FR-019**: When the user presses Q while a session is active, the TUI MUST display a one-line warning ("Session still running — press Q again to quit") and require a second Q keypress to exit; any other key MUST dismiss the warning. If no session is active, Q MUST exit immediately.
- **FR-020**: The TUI MUST reuse the existing session data store and business logic rather than duplicating database access patterns.

### Key Entities

- **Session**: A timed work unit with an id, task description, optional tag, start timestamp, and optional end timestamp. Active when end timestamp is absent.
- **Tag**: An optional label attached to a session used to group and aggregate time in reports.
- **Time Window**: A named date range (Today, Current Week, Last 7 Days, Custom) used to filter sessions in the report view.
- **View State**: The current active screen (Dashboard, Menu, Start Form, Log, Report) and associated UI state (scroll position, selected menu item, current page) that persists when switching between views.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The TUI is ready for user interaction within 500 milliseconds of launching `focus ui`.
- **SC-002**: The active session elapsed time updates at least once per second with no perceptible lag; each screen render completes within 50 milliseconds.
- **SC-003**: All existing CLI commands (`start`, `stop`, `status`, `log`, `report`, `export`) continue to work identically after the TUI feature is added — verified by running the existing test suite with no regressions.
- **SC-004**: A user can start a session, browse the session log, and stop the session entirely within the TUI without switching to the CLI — completing the full workflow in under 60 seconds.
- **SC-005**: The TUI runs without crashes on macOS Terminal, iTerm2, and a standard Linux terminal emulator at the minimum supported size of 60 × 12.
- **SC-006**: When `NO_COLOR=1` is set, all information remains fully readable in a monochrome terminal — verified by screenshot or automated render check.

## Assumptions

- The existing SQLite database schema (`sessions` table with `id`, `task`, `tag`, `start_time`, `end_time`) is stable; no schema migrations are required for the TUI.
- The TUI reads from and writes to the same database path (`~/.local/share/focus/focus.db`) as the CLI.
- Users are running focus on a desktop machine with a proper terminal emulator; mobile or embedded terminal environments are out of scope.
- The TUI does not need to support simultaneous multi-user access or locking beyond what SQLite provides natively.
- The `export` command is CLI-only and is intentionally excluded from the TUI.
- Custom date range selection in the report view is a stretch goal and is acceptable to defer to a follow-up; the three named time windows (Today, Week, Last 7 Days) are the required minimum.
- Unicode box-drawing characters are used by default; ASCII fallback is provided for terminals that report no unicode support.
- The TUI is always included in every binary build; no compile-time feature flag is used. Both CLI and TUI modes ship in a single unified binary.

## Clarifications

### Session 2026-04-02

- Q: What should happen when the database file is missing or unreadable at launch? → A: Print a clear error message to stderr and exit with a non-zero status code (consistent with existing CLI failure behaviour).
- Q: Should the dashboard auto-detect sessions started or stopped via CLI while the TUI is open, or require manual refresh? → A: Poll the database on every refresh tick (~100ms); external changes appear automatically within one cycle — no manual reload needed.
- Q: Should the TUI be gated behind an optional compile-time feature flag or always included? → A: Always included; CLI and TUI ship together in a single unified binary with no feature flag.
- Q: How should text that overflows a column width be handled in tabular views (log, dashboard, report)? → A: Truncate at column boundary with a trailing `…`; rows are always single-line, alignment is always preserved.
- Q: Should pressing Q while a session is active exit immediately or require confirmation? → A: Show a one-line warning and require a second Q keypress to confirm; any other key dismisses the warning.
