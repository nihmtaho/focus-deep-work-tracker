# Feature Specification: TUI Polish & Fixes

**Feature Branch**: `009-tui-polish-fixes`
**Created**: 2026-04-13
**Status**: Draft
**Issues**: #10, #11, #12, #13, #14, #15, #16

## Overview

This feature addresses seven usability issues discovered after the initial TUI dashboard (feature 008) was shipped. The fixes span keyboard conflicts, todo management, timer display accuracy, settings persistence, theme customization, and vim-mode completeness. Together they bring the TUI to a production-quality experience.

---

## Clarifications

### Session 2026-04-13

- Q: How does the user toggle vim mode? → A: CLI command `focus config set vim-mode true/false`, mirroring the theme command pattern
- Q: Does the Log tab delete session history records, or only todo items? → A: Only todo items; session history records are never deleted via the TUI in this feature
- Q: What is the timeout for vim's partial `d` command before it is discarded? → A: 1 second — after 1s without a second `d`, the partial command is discarded and input resets
- Q: When are settings written to disk — immediately on change or only on clean exit? → A: Immediately on every change, preventing data loss on crash or force-quit

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Delete Key Conflict Resolution (Priority: P1)

A user in the **Log tab** tries to delete a todo item by pressing `d`. Instead of deleting the item, the app switches to the Dashboard tab because `d` is a global tab shortcut.

The user needs a dedicated, unambiguous key for todo item deletion that does not conflict with tab navigation shortcuts.

**Why this priority**: Keyboard conflicts make core functionality (deletion) completely inaccessible. This is a blocker for day-to-day use of the Log tab.

**Independent Test**: Open the Log tab with a todo item selected, press `Delete` (macOS) or `Backspace` (Linux/Windows). The todo item should be removed and the list updates. Press `d` — the Dashboard tab should activate. Both actions work without conflict. Session history records in the Log tab are read-only and cannot be deleted.

**Acceptance Scenarios**:

1. **Given** user is on the Log tab with a todo item selected, **When** they press `Delete` (macOS) or `Backspace` (Linux/Windows), **Then** the selected todo item is removed and the list updates
2. **Given** user is on any tab, **When** they press `d`, **Then** the Dashboard tab activates and no delete action occurs
3. **Given** user is on the Dashboard tab with a todo selected, **When** they press `Delete`/`Backspace`, **Then** the todo is deleted (not a tab switch)
4. **Given** user is viewing session history records in the Log tab, **Then** those records have no delete action — they are read-only in this feature

---

### User Story 2 — Todo Deletion from Dashboard (Priority: P1)

A user viewing their todo list in the Dashboard panel wants to remove a completed or irrelevant todo. Currently there is no way to delete a todo from the TUI — the user must use the CLI.

**Why this priority**: Without deletion, the todo list grows unboundedly and cannot be managed within the TUI, breaking the single-tool workflow.

**Independent Test**: In the Dashboard, navigate to a todo item using arrow keys, press `Delete`/`Backspace`. The todo disappears from the list. Reopen the app — the todo is gone from storage.

**Acceptance Scenarios**:

1. **Given** user has at least one todo in Dashboard, **When** they press `Delete`/`Backspace` on a selected todo, **Then** the todo is permanently removed and the list refreshes
2. **Given** the user presses `Delete` when no todo is selected or the list is empty, **Then** nothing happens and no error is shown
3. **Given** a todo is deleted, **When** the user reopens the app, **Then** the deleted todo does not reappear

---

### User Story 3 — Sharp Block-Style Timer Digits (Priority: P2)

A user glancing at the flip-clock timer display notices the digit corners appear rounded (using box-drawing characters like `┌ ┐ └ ┘`), which looks inconsistent with the rest of the block-based UI aesthetic.

**Why this priority**: Visual polish improves perceived quality. Timer is a primary element users watch throughout work sessions.

**Independent Test**: Start a session and open the TUI. The timer digits should render using solid block characters (█) with no rounded corners. All digits 0–9 and the colon separator render cleanly across all supported terminal emulators.

**Acceptance Scenarios**:

1. **Given** a session is running, **When** the timer is visible in Dashboard or Pomodoro tab, **Then** all digit characters use solid block elements with sharp edges
2. **Given** `NO_COLOR=1` is set, **When** the timer renders, **Then** digits are still readable in monochrome using block characters
3. **Given** any terminal width above the minimum, **When** the timer renders, **Then** no character artifacts or alignment issues appear

---

### User Story 4 — Pomodoro Timer Stops When Session Ends (Priority: P1)

A user ends their Pomodoro session (presses ESC or uses stop). The timer display continues counting up, creating confusion about whether the session truly ended.

**Why this priority**: A running timer after session end is a critical correctness bug that undermines user trust in the app's state.

**Independent Test**: Start a Pomodoro session. Let it run for 30 seconds. Press ESC to end it. The timer display freezes at the last elapsed time (e.g., 0:30). It does not increment further.

**Acceptance Scenarios**:

1. **Given** a Pomodoro session is active, **When** the user ends the session (ESC or stop command), **Then** the timer display immediately freezes at the final elapsed time
2. **Given** the session has ended, **When** the user switches between tabs, **Then** the frozen timer value remains consistent
3. **Given** a new Pomodoro session is started after the previous one ended, **Then** the timer resets to 0:00 and begins counting from zero

---

### User Story 5 — Settings Persistence Across Restarts (Priority: P1)

A user enables vim mode and selects a theme during a session. On closing and reopening the app, both settings have reset to their defaults. The user must reconfigure every time.

**Why this priority**: Non-persistent settings are a fundamental UX failure — users expect their preferences to be remembered.

**Independent Test**: Enable vim mode and set theme to `material`. Quit the app. Reopen the app. Vim mode is enabled and theme is `material` without any additional configuration.

**Acceptance Scenarios**:

1. **Given** user enables vim mode in settings, **When** they quit and reopen the app, **Then** vim mode is still enabled
2. **Given** user selects a theme, **When** they quit and reopen the app, **Then** the same theme is applied at startup
3. **Given** no settings file exists (first run), **When** the app starts, **Then** a default settings file is created automatically with sensible defaults
4. **Given** the settings file is corrupted or unreadable, **When** the app starts, **Then** defaults are used and no crash occurs

---

### User Story 6 — Custom Theme Selection (Priority: P2)

A user wants to use the `onedark` theme, but the app auto-detects their OS dark mode and always applies the built-in `dark` theme. There is no way to override the auto-detected theme.

**Why this priority**: OS theme detection is imprecise and users have personal preferences. Without manual override, theme customization is inaccessible.

**Independent Test**: Run `focus config set theme onedark`. Reopen the TUI. The onedark theme is applied. Run `focus config set theme light`. Reopen the TUI. The light theme is applied.

**Acceptance Scenarios**:

1. **Given** available themes are `dark`, `light`, `material`, `onedark`, **When** user runs `focus config set theme <name>`, **Then** the theme is saved and applied on next start
2. **Given** user provides an invalid theme name, **When** they run `focus config set theme invalid`, **Then** an error lists available theme names
3. **Given** a custom theme is configured, **When** the user opens the TUI, **Then** the custom theme overrides OS auto-detection
4. **Given** no theme is configured, **When** the user opens the TUI, **Then** the OS-detected theme is used as fallback

---

### User Story 7 — Full Vim Mode Navigation (Priority: P3)

A user enables vim mode expecting standard vim keybindings (`hjkl` navigation, `dd` delete, `gg`/`G` jump) across all panels. Currently only partial vim keys are recognised and many commands do nothing.

**Why this priority**: Vim mode was advertised but is incomplete. Users who enabled it have a broken experience. Full implementation makes it a first-class input mode.

**Independent Test**: Enable vim mode. In any panel: press `j`/`k` to move down/up, `gg` to jump to top, `G` to jump to bottom, `dd` to delete selected item. All commands work as expected.

**Acceptance Scenarios**:

1. **Given** vim mode is enabled, **When** user presses `j`/`k`, **Then** selection moves down/up by one item
2. **Given** vim mode is enabled, **When** user presses `gg`, **Then** selection jumps to the first item in the list
3. **Given** vim mode is enabled, **When** user presses `G`, **Then** selection jumps to the last item in the list
4. **Given** vim mode is enabled, **When** user presses `dd` on a selected todo, **Then** the todo is deleted (same as Delete/Backspace)
5. **Given** vim mode is disabled, **When** user presses `j`/`k`/`gg`/`G`, **Then** these keys are either ignored or handled by normal mode bindings
6. **Given** vim mode is enabled, **When** user presses `d` (single), **Then** the Dashboard tab shortcut is not triggered — `d` is captured for vim command composition

---

### Edge Cases

- What happens when the user presses `Delete`/`Backspace` while in an input field (todo text entry)? The character before the cursor should be deleted from the input, not the whole todo.
- What happens when vim mode is enabled and the user types `d` — the app must not switch tabs. It must wait up to **1 second** for the second character to form a command (`dd`). If no second `d` arrives within 1 second, the partial command is silently discarded and input resets to normal.
- What if the settings file exists but is missing specific keys (partial config)? Missing keys should fall back to defaults, not crash.
- What if the user resizes the terminal while the timer is frozen after session end? The frozen time value should remain correct.
- What if two sessions end simultaneously (edge case in timer)? Only one timer tick should be applied per second regardless.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `Delete` key (macOS) and `Backspace` key (Linux/Windows) MUST trigger todo item deletion in the Log tab (todo sub-panel) and Dashboard todo list; session history records in the Log tab are read-only and not affected
- **FR-002**: The `d` key MUST be reserved exclusively for tab navigation to Dashboard and MUST NOT trigger deletion in any context
- **FR-003**: Users MUST be able to delete todos from the Dashboard panel using `Delete`/`Backspace`
- **FR-004**: Todo deletion MUST be permanent — the todo MUST NOT reappear after app restart
- **FR-005**: The flip-clock timer digits MUST use solid block characters (no box-drawing rounded corners)
- **FR-006**: The timer display MUST remain unchanged (frozen) after a Pomodoro or work session ends
- **FR-007**: The timer MUST reset to zero only when a new session is explicitly started
- **FR-008**: All user settings (vim mode, theme) MUST be persisted to a local configuration file **immediately on change** (not deferred to app exit)
- **FR-009**: Settings MUST be loaded from the configuration file on every app startup
- **FR-010**: If no configuration file exists, the app MUST create one with default values silently
- **FR-011**: Users MUST be able to set a custom theme via `focus config set theme <name>` and toggle vim mode via `focus config set vim-mode true/false`
- **FR-012**: The manually selected theme MUST take precedence over OS auto-detection
- **FR-013**: Attempting to set an invalid theme name MUST display an error listing valid theme names
- **FR-014**: Vim mode MUST support navigation commands: `j` (down), `k` (up), `gg` (top), `G` (bottom)
- **FR-015**: Vim mode MUST support `dd` to delete the selected todo item (same outcome as `Delete`/`Backspace`)
- **FR-016**: Vim mode state MUST be isolated from normal mode — a single `d` keypress in vim mode MUST NOT trigger tab navigation; it starts a 1-second command window waiting for a second `d`; if not received within 1 second, the partial command is discarded
- **FR-017**: In any text input field, `Backspace`/`Delete` MUST delete the preceding character, not the todo item

### Key Entities

- **UserSettings**: Persisted configuration containing `vim_mode: bool` and `theme: String`; stored in a well-known local path; loaded at startup, saved **immediately on every change** (not deferred to exit)
- **TodoItem**: An actionable task with unique identifier, text content, and state; can be permanently deleted
- **Theme**: A named visual style (`dark`, `light`, `material`, `onedark`) that controls all color rendering; has a fixed list of valid names
- **PomodoroTimer**: Tracks elapsed time for a work/break session; transitions to frozen state when session ends; resets only on explicit new session start

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can delete a todo or log entry in under 2 keystrokes without triggering unintended navigation
- **SC-002**: After any session ends, the timer display shows the correct frozen elapsed time within 1 second and does not increment further
- **SC-003**: 100% of user settings (vim mode, theme) are preserved across 10 consecutive app restarts without re-entry; settings are confirmed written to disk within 500ms of each change
- **SC-004**: Users can select any of the 4 available themes via `focus config set theme <name>` and toggle vim mode via `focus config set vim-mode true/false`; changes take effect on next startup
- **SC-005**: All vim navigation commands (`j`, `k`, `gg`, `G`, `dd`) work correctly across all panels when vim mode is enabled
- **SC-006**: No keyboard shortcut conflicts exist — every key combination has exactly one unambiguous meaning per context
- **SC-007**: Timer digit rendering is visually consistent across all supported terminal emulators with no character artifacts

---

## Assumptions

- The app runs on macOS, Linux, and Windows; platform is detectable at runtime for key binding differences
- The configuration file is stored in the same data directory as the database (`~/.local/share/focus/`)
- The `Delete` key on macOS generates a distinct key code from `Backspace`; the platform detection chooses the appropriate binding
- There are exactly 4 built-in themes; custom user-defined themes are out of scope for this feature
- Vim mode `dd` shares the same deletion outcome as `Delete`/`Backspace` — no separate confirmation prompt
- **Session history records in the Log tab are read-only** — only todo items can be deleted; this feature does not add session record deletion
- A "frozen timer" means the display stops incrementing but the final value remains visible; it does not hide or reset
- Settings write failures (e.g., disk full) surface as a non-fatal warning message, not a crash; the in-memory setting remains active for the session
- The `focus config set theme <name>` and `focus config set vim-mode true/false` CLI commands are the primary way to change persisted settings; an in-TUI settings panel is a stretch goal outside this feature's scope
