# Changelog

## [1.2.0] - April 07, 2026

### Added
- feat: implement Pomodoro timer mode with CLI and TUI support (#5)



All notable changes to this project are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- (upcoming features go here)

### Changed
- (upcoming changes go here)

### Fixed
- (upcoming fixes go here)

---

## [1.2.0] - April 7, 2026

### Added

- **CLI**: New `focus start --pomodoro <TASK>` command to start 25/5/15 Pomodoro cycles with configurable durations
- **CLI**: New `focus pomo-stats` command to view daily/weekly Pomodoro statistics, streaks, and abandonment counts
- **TUI**: Dedicated Pomodoro view with live countdown, phase indicator (🍅 WORK / ☕ BREAK / 🌿 LONG BREAK)
- **TUI**: Mode selector on start screen — choose between Pomodoro and Freeform sessions
- **TUI**: Pomodoro customization dialog with inline validation for adjusting durations
- **TUI**: New Settings tab to configure default Pomodoro durations globally
- **TUI**: Pomodoro controls: [P] pause/resume, [S] skip break, [+5] extend phase, [Q] confirm quit
- **Database**: `sessions.mode` column to distinguish Pomodoro from freeform sessions
- **Database**: New `pomodoro_stats` table tracking daily completed/abandoned counts
- **Configuration**: Support for `~/.config/focus/pomodoro.toml` with configurable durations
- **Configuration**: Three-level precedence: config file → environment variables → CLI flags
- **Notifications**: Desktop notifications on phase transitions
- **Statistics**: Per-day streak tracking for Pomodoro sessions
- **Integration**: Pomodoro work phases in `focus log` and `focus report` with mode labels

### Changed

- `focus log` output now includes `MODE` column (freeform/pomodoro)
- `focus report` now includes mode breakdown and time spent in each mode
- TUI menu reorganized to include Pomodoro options
- TUI Start Form shows mode selection dialog before task entry

### Fixed

- Q key on Pomodoro tab now shows confirmation dialog instead of abruptly quitting
- Pomodoro countdown now uses real wall-clock time (tolerates system sleep and skew)

### Technical

- Added dependencies: `toml = "0.8"`, `ctrlc = "3"`
- Database migrations applied automatically on first run
- TUI redraws at 100ms intervals for smooth countdown
- Timer state machine implemented with real time (not tick-based)
- All 61 tests pass; TDD compliance for all user stories
- Backward compatible: existing freeform sessions unchanged

---

## [1.1.0] - April 6, 2026

### Added

- **TUI**: Session controls (pause, stop, confirm-quit with double-press)
- **TUI**: Menu-driven navigation for all major operations
- **Tests**: Comprehensive integration tests for session controls

### Changed

- TUI now requires user confirmation (double-press Q) when quitting with active session

### Fixed

- Race condition in `stop_session` fixed by ensuring in-memory state matches database
- Silent config save errors now properly logged

---

## [1.0.0] - April 2, 2026

### Initial Release

### Added

- **CLI Core**: `focus start <TASK>` to begin sessions, `focus stop` to end, `focus status` to check
- **CLI Core**: `focus log` command to view completed sessions
- **CLI Core**: `focus report` command to aggregate time by tag
- **CLI Core**: `focus export` command to export as JSON or Markdown
- **TUI**: Interactive dashboard with real-time session tracking
- **TUI**: Session log viewer with pagination
- **TUI**: Report view showing tag totals and time windows
- **TUI**: Full keyboard navigation (arrows, vim keys, Enter, Esc)
- **Database**: SQLite backend with automatic schema creation
- **Configuration**: Support for NO_COLOR environment variable
- **Documentation**: Comprehensive README and keyboard shortcuts
- **Tests**: 35 tests covering all CLI commands and TUI

### Technical

- Rust stable 1.77+ with clap, rusqlite, chrono, dirs, colored, anyhow, thiserror
- SQLite WAL mode for efficient concurrent access
- Automatic crash recovery for interrupted sessions
- No external network dependencies
