# Feature Specification: Integrated Pomodoro Timer Mode

**Feature Branch**: `005-pomodoro-timer-mode`  
**Created**: 2026-04-04  
**Status**: Draft  

## Overview

Focus CLI currently supports freeform deep-work session tracking. This feature adds an integrated Pomodoro Timer mode that runs alongside freeform sessions. Users can choose between timed Pomodoro cycles (work → break → long break) or their existing freeform workflow when starting any session. All Pomodoro activity persists to the same data store, blending naturally with freeform history across all existing reports and commands.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Start a Pomodoro Session via CLI (Priority: P1)

A developer wants to focus on a task using the Pomodoro technique directly from the terminal, without opening a TUI. They run a single command and the timer begins immediately, cycling through work and break phases automatically.

**Why this priority**: This is the minimum viable feature — CLI-first users need a working Pomodoro timer before anything else delivers value.

**Independent Test**: Run `focus start --pomodoro "task name"` and observe that a 25-minute work phase begins, transitions to a 5-minute break at the correct time, and a completed session record appears in `focus log` afterward.

**Acceptance Scenarios**:

1. **Given** no active session, **When** the user runs `focus start --pomodoro "refactor auth module"`, **Then** a 25-minute work phase begins, displaying the current phase, remaining time, and pomodoro count in the terminal.
2. **Given** a work phase completes, **When** 25 minutes elapse, **Then** the system transitions automatically to a 5-minute break phase, displaying a clear phase-change notification.
3. **Given** 4 consecutive completed work phases, **When** the 4th work phase completes, **Then** the system offers a 15-minute long break instead of the standard 5-minute break.
4. **Given** a running Pomodoro session, **When** the user runs `focus stop`, **Then** only fully completed work phases are saved; partial phases are discarded; a confirmation message shows the count saved vs. abandoned.
5. **Given** the user provides `--work 30 --break 10` flags, **When** the session starts, **Then** each work phase is 30 minutes and each break is 10 minutes.

---

### User Story 2 — Pomodoro Mode in the TUI Dashboard (Priority: P2)

A developer who prefers the TUI interface wants to start a Pomodoro session from the menu, see a dedicated timer view with live countdown and controls, and switch between Pomodoro and freeform modes without leaving the application.

**Why this priority**: The TUI is the primary interactive surface of Focus. Pomodoro support there dramatically increases the feature's usefulness for users who keep Focus open while working.

**Independent Test**: Launch the TUI, select "Start Pomodoro Session" from the menu, enter a task name, and verify the dedicated pomodoro view shows a live countdown, phase indicator, progress bar, and responds to [P] pause and [S] skip-break keyboard shortcuts.

**Acceptance Scenarios**:

1. **Given** the TUI is open on the main menu, **When** the user selects "Start Pomodoro Session", **Then** a mode-selector dialog appears offering "Pomodoro", "Freeform", and "Cancel" options.
2. **Given** the user selects Pomodoro mode, **When** prompted for task and optional tag input, **Then** an optional customization dialog offers work/break/long-break duration fields (pre-filled with defaults) before the session starts.
3. **Given** a Pomodoro session is active in the TUI, **When** the dedicated Pomodoro view is displayed, **Then** it shows: phase name (🍅 WORK / ☕ BREAK / 🌿 LONG BREAK), remaining time in MM:SS format, completed pomodoro count (e.g., "3/4"), total elapsed time, a progress bar, and keyboard shortcut hints.
4. **Given** a Pomodoro session is active, **When** the user presses [P], **Then** the timer pauses and "PAUSED" is displayed; pressing [P] again resumes the countdown from where it stopped.
5. **Given** the user is in a break phase, **When** the user presses [S], **Then** the break ends immediately and the next work phase begins.
6. **Given** the user presses [Q] mid-session, **Then** a confirmation dialog shows completed pomodoro count and asks whether to stop or continue.

---

### User Story 3 — View Pomodoro Statistics (Priority: P3)

A developer wants to review how many Pomodoros they completed today and this week, see their current streak, and understand their productivity patterns over time.

**Why this priority**: Statistics close the feedback loop for Pomodoro practitioners; without them the technique loses its motivational structure. Delivers independently of TUI integration.

**Independent Test**: Complete at least one Pomodoro session, then run `focus pomo-stats --today` and verify it shows completed count, abandoned count, total work minutes, and current streak.

**Acceptance Scenarios**:

1. **Given** completed Pomodoro sessions today, **When** the user runs `focus pomo-stats --today`, **Then** output shows: date, completed pomodoros, abandoned pomodoros, total work minutes, total break minutes, and current streak in days.
2. **Given** sessions spanning multiple days, **When** the user runs `focus pomo-stats --week`, **Then** output shows a daily breakdown for the past 7 days plus an overall weekly summary and best streak.
3. **Given** no Pomodoro sessions today, **When** the user runs `focus pomo-stats --today`, **Then** the command exits gracefully with a "No Pomodoro sessions today" message rather than erroring.
4. **Given** Pomodoro sessions exist, **When** the user runs `focus log`, **Then** Pomodoro work-phase sessions appear alongside freeform sessions, each labeled with their mode.

---

### User Story 4 — Configure Default Durations (Priority: P4)

A developer who prefers longer focus blocks wants to set their default work duration to 45 minutes and their long break to 20 minutes once, without specifying flags on every command.

**Why this priority**: Personalisation is essential for adoption; users who cannot set their own defaults will abandon the feature. Delivers independently as configuration only.

**Independent Test**: Write a `~/.config/focus/pomodoro.toml` with `work_duration_minutes = 45`, then run `focus start --pomodoro "task"` without any duration flags and verify the work phase runs for 45 minutes.

**Acceptance Scenarios**:

1. **Given** `~/.config/focus/pomodoro.toml` sets `work_duration_minutes = 45`, **When** the user runs `focus start --pomodoro "task"` without duration flags, **Then** each work phase lasts 45 minutes.
2. **Given** the config file sets defaults, **When** the user also provides `--work 30` on the CLI, **Then** the CLI flag overrides the config file for that session only.
3. **Given** the environment variable `FOCUS_POMODORO_WORK=50` is set, **When** the user starts a Pomodoro session, **Then** the 50-minute value takes precedence over both config file and CLI defaults.
4. **Given** the user provides `--work 0` or `--work 200`, **When** the session attempts to start, **Then** the system rejects the value with a clear validation error: work must be between 1 and 120 minutes, break between 1 and 60 minutes.

---

### User Story 5 — Pause and Resume a Timer (Priority: P5)

A developer is mid-Pomodoro when an urgent interruption occurs. They want to pause the timer, handle the interruption, then resume without losing their progress or corrupting the session record.

**Why this priority**: Interruptions are inevitable; without pause/resume, users must abandon sessions unnecessarily, reducing data accuracy and adoption.

**Independent Test**: Start a Pomodoro in the TUI, press [P] to pause after 3 minutes, wait 1 minute, press [P] to resume, and verify the countdown continues from the paused time (not from a reduced amount).

**Acceptance Scenarios**:

1. **Given** a running work phase with 20 minutes remaining, **When** the user pauses, **Then** the countdown freezes, "PAUSED" is displayed, and time continues to show 20:00 unchanged.
2. **Given** the timer is paused, **When** the user resumes, **Then** the countdown continues from 20:00 and does not include the pause duration in the remaining time.
3. **Given** the timer has been paused for more than 60 minutes, **When** the 60-minute limit is reached, **Then** the session is automatically abandoned with a notification explaining the reason; partial completed pomodoros are saved.
4. **Given** the timer is in a break phase, **When** the user pauses and resumes, **Then** the break countdown behaves identically to the work-phase pause/resume behaviour.

---

### User Story 6 — Receive Phase-Transition Notifications (Priority: P6)

A developer running a Pomodoro session wants to be clearly notified when a work phase ends or a break ends, so they do not need to watch the timer constantly.

**Why this priority**: Notification is central to the Pomodoro method; without it users must monitor the timer manually, defeating the purpose of the technique.

**Independent Test**: Allow a work phase to expire while the terminal is not in focus; verify a desktop notification appears (if supported by OS) and that the TUI shows a visible phase-change flash and on-screen message.

**Acceptance Scenarios**:

1. **Given** a work phase ends, **When** the transition occurs, **Then** a two-second on-screen message appears (e.g., "Work phase complete! Take a 5-minute break.") and the TUI flashes the phase indicator.
2. **Given** a break phase ends, **When** the transition occurs, **Then** a two-second message appears (e.g., "Break over! Time to focus.") and the display updates to show the new work phase.
3. **Given** the OS supports desktop notifications, **When** a phase transition occurs, **Then** a native desktop notification is sent with the phase name and next action.
4. **Given** desktop notifications are unavailable (e.g., `notify-send` not installed), **When** a phase transition occurs, **Then** the system falls back gracefully to terminal-only notification without crashing.
5. **Given** `FOCUS_POMODORO_SOUND=false` is set, **When** a phase transition occurs, **Then** no audio is produced but all visual notifications still fire.

---

### User Story 7 — Skip a Break or Extend a Phase (Priority: P7)

A developer who is deeply in flow at the end of a work phase wants to skip the upcoming break and continue immediately, or extend their current work phase by 5 minutes without restarting.

**Why this priority**: Flexibility in cycle management is a key differentiator of a good Pomodoro tool; it lets users respect their cognitive state rather than blindly following the clock.

**Independent Test**: While in a break phase in the TUI, press [S]; verify the timer immediately shows the next work phase starting from its full duration.

**Acceptance Scenarios**:

1. **Given** a break phase is active, **When** the user presses [S] (Skip), **Then** the break ends immediately and the next work phase begins from its full configured duration.
2. **Given** any active phase, **When** the user presses [+5] (Extend), **Then** 5 minutes are added to the current phase's remaining time and the updated countdown is shown.
3. **Given** the user extends a phase multiple times, **When** each [+5] is pressed, **Then** each press adds a further 5 minutes with no enforced limit on extensions.

---

### User Story 8 — Session Integration with Existing History and Reports (Priority: P8)

A developer who uses both Pomodoro and freeform sessions wants their historical data to remain unified so that `focus log` and `focus report` show all sessions in one place, with the mode visible but not disruptive.

**Why this priority**: Data integrity is foundational; users must not lose or fragment historical records when adopting a new mode.

**Independent Test**: Run a freeform session and a Pomodoro session in sequence, then run `focus log`; verify both appear, each showing their respective mode label.

**Acceptance Scenarios**:

1. **Given** a mix of freeform and Pomodoro sessions in history, **When** the user runs `focus log`, **Then** all sessions appear in chronological order, each with a "mode" indicator ("freeform" or "pomodoro").
2. **Given** a Pomodoro session completes 3 work phases, **When** the user views `focus log`, **Then** 3 separate session records appear (one per completed work phase), each tagged with the task name and mode.
3. **Given** existing freeform session records, **When** the Pomodoro feature is activated for the first time, **Then** all existing records remain intact with no data loss or format change.
4. **Given** the user runs `focus report`, **When** Pomodoro sessions exist, **Then** the report includes a mode breakdown showing total time in Pomodoro mode vs freeform mode.

---

### User Story 9 — Abandonment Handling (Priority: P9)

A developer who must stop a session before completing a work phase wants clear confirmation of what was saved and what was lost.

**Why this priority**: Honest abandonment handling builds trust in the data; users need to know what counts toward their record.

**Independent Test**: Start a Pomodoro session, complete 2 full work phases, stop mid-way through the 3rd, and verify `focus log` shows exactly 2 sessions and `focus pomo-stats --today` shows 1 abandoned.

**Acceptance Scenarios**:

1. **Given** 2.5 completed pomodoros, **When** the user stops the session, **Then** exactly 2 session records are saved and 1 is recorded as abandoned in statistics.
2. **Given** a stop is requested via TUI [Q], **When** the confirmation dialog appears, **Then** it shows "2 pomodoros completed, 1 incomplete — stop anyway?" before acting.
3. **Given** a stop is requested via CLI `focus stop`, **When** the command runs, **Then** the session ends immediately, saved records are reported in the terminal output, and the abandonment is logged to statistics.
4. **Given** the user stops during a break phase (not a work phase), **When** the session ends, **Then** no additional abandonment is recorded; only the break is discarded.

---

### User Story 10 — TUI Settings Adjustment at Runtime (Priority: P10)

A developer in the TUI customization dialog wants to adjust work/break/long-break durations before starting a Pomodoro session without restarting the application or editing config files.

**Why this priority**: Runtime adjustment removes friction for users who want different durations for different tasks without modifying global configuration.

**Independent Test**: Open the TUI, start a Pomodoro session, set work duration to 35 in the customization dialog, and verify the timer counts from 35:00 at the start of the first work phase.

**Acceptance Scenarios**:

1. **Given** the customization dialog is shown, **When** the user changes work duration to 35 minutes and confirms, **Then** the session begins with 35-minute work phases.
2. **Given** the customization dialog, **When** the user enters an invalid value (e.g., 0 or 200 for work), **Then** an inline validation error is shown and the dialog does not close until valid values are provided.
3. **Given** the user does not change any values in the dialog, **When** they confirm, **Then** sessions use the configured or default durations.

---

### Edge Cases

- **What happens when the system clock is adjusted while a session is running?** Timer must tolerate clock skew without producing negative or wildly incorrect remaining times; it should clamp to zero and advance to the next phase.
- **What happens when the terminal window is resized mid-session?** The TUI must redraw without corrupting or freezing the timer.
- **What happens if the data store is read-only or full when a phase completes?** The system must attempt to save, display a clear error if it fails, and continue the timer so data loss is minimised.
- **What happens if a Pomodoro session is running and the machine sleeps?** On wakeup, the timer should detect elapsed real time and either advance phases appropriately or notify the user that a phase was missed.
- **What happens when `--work` and `--break` are equal (e.g., both 5 minutes)?** The system accepts this as valid; no special behaviour required.
- **What happens if the user has both `FOCUS_POMODORO_WORK` and `--work` specified?** Environment variable takes precedence over config file; CLI flag takes precedence over environment variable.
- **What if `focus pomo-stats` is run with no previous Pomodoro sessions ever?** Graceful empty-state message, not an error.

---

## Requirements *(mandatory)*

### Functional Requirements

**Session Management**

- **FR-001**: Users MUST be able to start a Pomodoro session from the CLI using `focus start --pomodoro "<task>"`.
- **FR-002**: Users MUST be able to start a Pomodoro session from the TUI via a "Start Pomodoro Session" menu option.
- **FR-003**: The system MUST allow users to specify a task name and optional tag when starting any Pomodoro session.
- **FR-004**: The system MUST execute the standard Pomodoro cycle: 25-minute work → 5-minute break, repeating, with a 15-minute long break after every 4 completed work phases (all durations configurable).
- **FR-005**: The system MUST save a session record for each fully completed work phase only; incomplete phases MUST NOT be saved.
- **FR-006**: The system MUST record an abandonment entry in Pomodoro statistics when a session is stopped with an incomplete work phase.

**Timer Display & Controls**

- **FR-007**: The system MUST display the current phase (Work / Break / Long Break), remaining time in MM:SS format, completed pomodoro count (e.g., "3/4"), and total elapsed session time.
- **FR-008**: The TUI MUST update the timer display every second with no perceptible lag.
- **FR-009**: Users MUST be able to pause and resume the timer via the [P] key in the TUI; pause duration MUST NOT count toward the phase timer.
- **FR-010**: The system MUST automatically cancel a session if the timer has been paused for more than 60 minutes, saving any completed phases and notifying the user.
- **FR-011**: Users MUST be able to skip the current break phase via the [S] key in the TUI, transitioning immediately to the next work phase.
- **FR-012**: Users MUST be able to extend the current phase by 5 minutes via the [+5] control in the TUI.

**Phase Transitions & Notifications**

- **FR-013**: The system MUST display a 2-second on-screen transition message when any phase changes (e.g., "Work phase complete! Take a 5-minute break.").
- **FR-014**: The system MUST visually flash or highlight the phase indicator in the TUI at every phase transition.
- **FR-015**: The system MUST send a native desktop notification on phase transitions when the host OS supports it (Linux via notify-send, macOS via AppleScript/osascript, Windows via Windows Notification API), and fall back gracefully if unavailable.
- **FR-016**: The system MUST respect the `FOCUS_POMODORO_SOUND=false` environment variable to suppress audio output while preserving all visual notifications.

**Configuration**

- **FR-017**: Users MUST be able to configure default durations via `~/.config/focus/pomodoro.toml`.
- **FR-018**: Users MUST be able to override durations per-session via CLI flags `--work <min>`, `--break <min>`, `--long-break <min>`, `--long-break-after <count>`.
- **FR-019**: The system MUST support environment variable overrides: `FOCUS_POMODORO_WORK`, `FOCUS_POMODORO_BREAK`, `FOCUS_POMODORO_LONG_BREAK`, `FOCUS_POMODORO_LONG_BREAK_AFTER`.
- **FR-020**: Precedence order MUST be: CLI flags > environment variables > config file > built-in defaults.
- **FR-021**: The system MUST validate that work duration is between 1 and 120 minutes and break duration is between 1 and 60 minutes, rejecting invalid values with a clear error message.
- **FR-022**: The TUI MUST provide an optional customization dialog for adjusting durations before a Pomodoro session begins, with inline validation.

**Statistics**

- **FR-023**: The system MUST track per-day Pomodoro statistics: completed count, abandoned count, total work minutes, total break minutes.
- **FR-024**: Users MUST be able to query today's Pomodoro statistics via `focus pomo-stats --today`.
- **FR-025**: Users MUST be able to query a 7-day breakdown via `focus pomo-stats --week`, including current streak and best streak.
- **FR-026**: The system MUST display a streak in days: a streak is only maintained if at least one Pomodoro was completed on each consecutive day.

**Integration with Existing Features**

- **FR-027**: Pomodoro work-phase session records MUST appear in `focus log` output alongside freeform sessions, each labeled with a "mode" field.
- **FR-028**: The `focus report` command MUST include a breakdown of time spent in Pomodoro mode vs freeform mode when both exist.
- **FR-029**: All existing freeform session records MUST remain intact and unmodified after the Pomodoro feature is introduced; no data migration is required.
- **FR-030**: The existing `focus start` command (without `--pomodoro`) MUST continue to behave exactly as before.

### Key Entities

- **Pomodoro Session**: A single work phase within a Pomodoro cycle. Has a task name, optional tag, start time, end time, configured duration, mode ("pomodoro"), and completion status. Stored in the existing sessions table.
- **Pomodoro Cycle**: A grouping of work phases and their associated breaks for a single user-initiated session. Not directly persisted; reconstructed from session records.
- **Pomodoro Daily Statistics**: A per-date record of aggregate Pomodoro activity. Holds completed count, abandoned count, total work minutes, and total break minutes. Stored in a new `pomodoro_stats` table.
- **Pomodoro Configuration**: User-defined defaults for work/break/long-break durations and notification preferences. Sourced from config file, environment variables, and CLI flags (in increasing priority order).

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can start, run, and complete a full 4-pomodoro cycle (approx. 2 hours) from the CLI without any manual intervention beyond the initial command.
- **SC-002**: Timer accuracy is within ±2 seconds over a full 4-pomodoro cycle (~2 hours of continuous operation).
- **SC-003**: The TUI timer display updates visibly every second; phase transitions appear within 1 second of the scheduled time.
- **SC-004**: Pomodoro sessions appear in `focus log` alongside freeform sessions with no additional flags or configuration required.
- **SC-005**: `focus pomo-stats --today` returns a result in under 1 second, including accurate completed count, abandoned count, and streak.
- **SC-006**: After an unclean shutdown (process kill) mid-session, previously completed work phases are recoverable from the data store; no data is silently lost.
- **SC-007**: All existing `focus` commands (start, stop, log, report, status) continue to work identically for freeform sessions after the Pomodoro feature is introduced; zero regressions.
- **SC-008**: Changing a duration via CLI flag (`--work 30`) produces a work phase of exactly 30 minutes; the config file and environment variable resolution chain is testable and deterministic.
- **SC-009**: Desktop notifications fire within 2 seconds of a phase transition on supported operating systems; no crash or hang occurs on unsupported systems.
- **SC-010**: The TUI pause feature correctly suspends the countdown; resuming after a 5-minute pause leaves the remaining time identical to the paused value (pause duration not consumed).

---

## Assumptions

- Users run Focus on macOS, Linux, or Windows; all three platforms are in scope for the CLI; TUI is in scope for all three.
- Desktop notification commands (`notify-send`, `osascript`) are available on typical installations of those OSes; absence is handled gracefully rather than treated as a fatal error.
- A single user runs Focus; there is no multi-user or concurrent-session scenario.
- The existing sessions data store schema supports adding an optional `mode` column without requiring a destructive migration; a default of `'freeform'` is applied to all existing records.
- A new `pomodoro_stats` table is introduced additively; it does not affect any existing query or command that does not explicitly reference it.
- Audio notification defaults to the system terminal bell character; configurable sound file playback is explicitly out of scope for this release.
- The TUI keyboard shortcut [S] is context-sensitive: it starts a session when no session is active, and skips the current break when a Pomodoro session is in a break phase.
- Gamification (badges, streaks beyond day-count), team sharing, cloud sync, mobile, calendar integration, and AI-suggested durations are explicitly out of scope.
- The maximum pomodoro count before a long break (default: 4) is configurable but is expected to be between 2 and 8; values outside this range are not validated (user's responsibility).
- If a machine sleeps and wakes mid-phase, elapsed real time advances the timer; the system does not attempt to "replay" missed phase audio or desktop notifications for the elapsed period.
