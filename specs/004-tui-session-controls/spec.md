# Feature Specification: TUI Session Controls with Vim Mode and Tab Views

**Feature Branch**: `004-tui-session-controls`  
**Created**: 2026-04-03  
**Status**: Draft  
**Input**: User description: "thêm tính năng tạo nhanh session, quản lý session, stop/start ngay trên dashboard thay vì phải mở menu. Hỗ trợ vim mode. hiển thị view session log, view report dưới dạng tab trên main view. control các task session bằng các phím nhanh."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Session Start/Stop from Dashboard (Priority: P1)

A user is working in the TUI dashboard and wants to start a new deep work session or stop the current one without leaving the main view to navigate through menus. They press a single key to start or stop a session immediately.

**Why this priority**: This is the core workflow improvement — eliminating menu navigation for the most frequent actions (start/stop) directly reduces friction for every single session a user runs.

**Independent Test**: Can be fully tested by pressing the start/stop key on the dashboard and verifying a session is created/ended without any menu interaction, delivering immediate workflow value.

**Acceptance Scenarios**:

1. **Given** the dashboard is open with no active session, **When** the user presses the designated "start session" key, **Then** a new session begins immediately with the current timestamp and the dashboard reflects the active state.
2. **Given** an active session is running, **When** the user presses the designated "stop session" key, **Then** the session is ended, its duration is recorded, and the dashboard reflects the stopped state.
3. **Given** the dashboard is open, **When** the user presses the designated "start session" key, **Then** the action completes in under 1 second with visible confirmation on screen.

---

### User Story 2 - Tab Navigation: Session Log and Report Views (Priority: P2)

A user wants to review their session history (log) and productivity report without switching to a different screen or running a separate command. These views are available as tabs on the main dashboard, navigable with a single key.

**Why this priority**: Inline access to historical data and reports enriches the dashboard as a single-pane-of-glass workspace, making it the primary interface rather than a supplement to CLI commands.

**Independent Test**: Can be tested by switching between tabs and verifying the session log and report content are displayed correctly within the main view, with no CLI commands needed.

**Acceptance Scenarios**:

1. **Given** the main dashboard is open, **When** the user presses the tab-switch key (or `1`/`2`/`3`), **Then** the active tab changes and the corresponding content (dashboard / session log / report) is displayed.
2. **Given** the Session Log tab is active, **When** the view renders, **Then** a scrollable list of past sessions is shown with each session's name, date, and duration.
3. **Given** the Report tab is active, **When** the view renders, **Then** a summary of productivity metrics is shown (total time, session count, averages) covering the user's history.
4. **Given** any tab is active, **When** the user navigates between tabs, **Then** the previously active tab's scroll position and state are preserved.

---

### User Story 3 - Vim Mode Navigation (Priority: P3)

A user familiar with vim-style key bindings wants to navigate all lists and interactive elements in the TUI using `j`/`k` (down/up), `g`/`G` (top/bottom), and related keys instead of arrow keys. Vim mode applies to all list views throughout the dashboard.

**Why this priority**: Vim mode is a quality-of-life enhancement for keyboard-centric users. It does not block core functionality but significantly improves usability for a key segment of the audience.

**Independent Test**: Can be tested by navigating session lists using only `j`/`k`/`g`/`G` keys and confirming selection moves correctly, independently of other features.

**Acceptance Scenarios**:

1. **Given** any list view is focused, **When** the user presses `j`, **Then** the selection moves down one item.
2. **Given** any list view is focused, **When** the user presses `k`, **Then** the selection moves up one item.
3. **Given** any list view is focused, **When** the user presses `g`, **Then** the selection jumps to the first item.
4. **Given** any list view is focused, **When** the user presses `G`, **Then** the selection jumps to the last item.
5. **Given** vim mode is enabled in settings, **When** the user presses arrow keys, **Then** arrow keys continue to work as an alternative navigation method.
6. **Given** vim mode is disabled, **When** the user presses `j` or `k`, **Then** those keys have no navigation effect.

---

### User Story 4 - Quick Session Creation with Name Input (Priority: P2)

A user wants to quickly create a named session from the dashboard by pressing a key, entering a task name in an inline prompt, and starting the session — all without leaving the TUI.

**Why this priority**: Named sessions are essential for meaningful reporting. Quick creation with inline naming keeps the user in flow compared to exiting and re-entering the CLI.

**Independent Test**: Can be tested by pressing the "new session" key, entering a name in the inline prompt, confirming, and verifying a named session appears in the session log.

**Acceptance Scenarios**:

1. **Given** the dashboard is open, **When** the user presses the "new session" key, **Then** an inline input prompt appears for the session name without switching screens.
2. **Given** the inline prompt is open, **When** the user types a name and presses Enter, **Then** the named session starts and the prompt closes.
3. **Given** the inline prompt is open (TUI is in input mode), **When** the user presses Escape, **Then** input mode exits, the prompt closes, and no session is created; the TUI returns to normal mode.
4. **Given** the inline prompt is open and the user submits an empty name, **Then** the session starts with a default name (e.g., "Untitled Session").

---

### User Story 5 - Quick Keys for Task Session Control (Priority: P3)

A user managing multiple tasks in a session list can use single-key shortcuts to perform common actions on the selected session: delete, rename, or mark as complete — without navigating to a context menu.

**Why this priority**: Quick keys for task-level operations speed up session management workflows. Dependent on session list being present (P2 tab view), so this is lower priority but high value once the list exists.

**Independent Test**: Can be tested on the session log tab by selecting a session and pressing the designated action key, verifying the action is performed immediately.

**Acceptance Scenarios**:

1. **Given** a session is selected in the session log, **When** the user presses the "delete" key, **Then** a confirmation prompt appears, and upon confirmation the session is removed from the list.
2. **Given** a session is selected in the session log, **When** the user presses the "rename" key, **Then** an inline input prompt appears pre-filled with the current name.
3. **Given** any focused view, **When** the user presses `?`, **Then** a key binding reference overlay is displayed listing all available shortcuts.

---

### Edge Cases

- What happens when the user presses "start session" while a session is already active? The key is ignored or a notification is shown; no duplicate session is created. Stopped sessions are immutable — they cannot be resumed; each start always creates a new session.
- What happens when the session log tab is opened with no sessions recorded? An empty-state message is displayed ("No sessions yet").
- What happens when the report tab is opened with no data? An empty-state message is shown rather than blank or error output.
- How does the system handle pressing quick keys when no item is selected in a list? The action is ignored gracefully with no error.
- What happens if the user's terminal is too small to render tabs or panels? The dashboard degrades gracefully, hiding or stacking panels rather than crashing.
- What happens if the TUI is force-closed (Ctrl+C, terminal kill, system crash) while a session is active? The active session is automatically saved with elapsed time up to the moment of exit; no session data is lost.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to start a new deep work session from the main dashboard view using a single key press, without navigating any menu.
- **FR-002**: Users MUST be able to stop the currently active session from the main dashboard view using a single key press.
- **FR-003**: The dashboard MUST provide a two-step inline text input for starting a new session: step 1 collects the session name; step 2 collects an optional tag (pressing Enter with an empty tag skips it). Both steps are accessible via a single initial key press, without leaving the TUI.
- **FR-004**: The main view MUST display at least three named tabs: Dashboard (current view), Session Log, and Report.
- **FR-005**: Users MUST be able to switch between tabs using a single key press (e.g., Tab key or number keys `1`/`2`/`3`).
- **FR-006**: The Session Log tab MUST display a scrollable list of all past sessions, showing at minimum: session name, start time, and duration.
- **FR-007**: The Report tab MUST display aggregated productivity metrics including total focused time, session count, and average session duration.
- **FR-008**: When vim mode is enabled, all list-based views MUST support vim-style navigation keys: `j` (down), `k` (up), `g` (top), `G` (bottom).
- **FR-009**: Arrow key navigation MUST remain functional regardless of whether vim mode is enabled.
- **FR-010**: Users MUST be able to delete a selected session from the session log using a dedicated quick key, with a confirmation step.
- **FR-011**: Users MUST be able to rename a selected session from the session log using a dedicated quick key with an inline prompt.
- **FR-012**: Pressing `?` from any view MUST display a reference overlay listing all available keyboard shortcuts.
- **FR-013**: All quick key actions MUST complete and reflect in the UI in under 1 second on standard hardware.
- **FR-014**: The dashboard MUST display an empty-state message when the session log or report has no data.
- **FR-015**: When the TUI exits for any reason (clean quit, Ctrl+C, SIGTERM, or terminal close) while a session is active, the system MUST automatically save the session with the elapsed time up to the moment of exit.
- **FR-016**: The TUI MUST provide a settings screen where the user can toggle vim mode on or off. This preference MUST persist across sessions.
- **FR-018**: The dashboard's "Today's Summary" panel MUST display each completed session for the current day by its individual **name** and duration, followed by a total row. If a session has a tag, the tag MUST be shown alongside the name (e.g., `"deep work [dev]"`). This replaces the previous tag-based aggregation view.

### Key Entities

- **Session**: A discrete, immutable deep work interval with a name, start timestamp, end timestamp, and computed duration. Once stopped, a session cannot be resumed; a new session must be started.
- **Tab**: A named view panel within the main dashboard. Each tab has an identity, display content, and independent scroll state.
- **Key Binding**: A mapping from a keyboard key or key combination to a specific action within a given context (global, list, input prompt).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can start or stop a session from the dashboard in 1 key press and under 1 second, compared to the current multi-step menu navigation.
- **SC-002**: Users can create a named session (including entering the name) without leaving the TUI, completing the action in under 10 seconds.
- **SC-003**: All three tabs (Dashboard, Session Log, Report) are accessible from any view using at most 1 key press.
- **SC-004**: Vim navigation keys (`j`, `k`, `g`, `G`) work correctly in 100% of list views in the TUI.
- **SC-005**: All destructive actions (delete session) require confirmation before executing, preventing accidental data loss in 100% of cases.
- **SC-006**: The key binding reference overlay (`?`) lists all available shortcuts and is accessible from any view within the application.

## Clarifications

### Session 2026-04-03

- Q: Can a previously stopped session be resumed, or does each start always create a new session? → A: Each start always creates a new session; stopped sessions are immutable history.
- Q: What happens to an active session when the TUI is force-closed or the terminal is killed? → A: On any exit (clean or forced), the active session is automatically saved with elapsed time up to that moment.
- Q: Is vim mode always-on or a user-configurable preference? → A: Vim mode is a persistent user preference stored in config, toggled via a settings screen within the TUI.
- Q: How does the TUI distinguish navigation keys from text input when a prompt is open? → A: Explicit modal boundary — when a text prompt is open the TUI enters input mode (all keys treated as text); Escape exits input mode and returns to normal mode.

## Assumptions

- Users are running the TUI in a standard terminal emulator with at minimum 80×24 character dimensions.
- The existing `focus ui` command launches the TUI dashboard; this feature enhances that existing dashboard rather than introducing a new entry point.
- Session data (names, timestamps, durations) is already persisted in the local SQLite database; this feature reads from and writes to that existing store.
- Vim mode is a persistent user preference (default: off). Users enable it once via a settings screen within the TUI; the preference is saved to the local config file and restored on next launch.
- The Report tab displays lifetime or rolling aggregated data; time-range filtering is out of scope for this feature.
- Session "tasks" referenced in the description are equivalent to the existing session concept in the app (not a separate sub-entity); no new data model is introduced.
- The quick-create flow defaults to starting the session immediately upon name confirmation; scheduling future sessions is out of scope.
