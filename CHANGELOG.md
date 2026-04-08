# Changelog

All notable changes to this project are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- (upcoming features go here)

### Changed
- (upcoming changes go here)

### Fixed
- (upcoming fixes go here)

---

## [0.2.2] - April 08, 2026

### Added

- **Pomodoro Mode**: `focus start --pomodoro <TASK>` command to start 25/5/15 Pomodoro cycles with configurable durations
- **Pomodoro Stats**: `focus pomo-stats` command to view daily/weekly statistics, streaks, and abandonment counts
- **TUI Pomodoro**: Dedicated Pomodoro view with live countdown and phase indicator (🍅 WORK / ☕ BREAK / 🌿 LONG BREAK)
- **Mode Selection**: TUI mode selector on start screen — choose between Pomodoro and Freeform sessions
- **Customization**: Pomodoro customization dialog with inline validation for adjusting durations
- **Settings Tab**: New Settings tab to configure default Pomodoro durations globally
- **Pomodoro Controls**: [P] pause/resume, [S] skip break, [+5] extend phase, [Q] confirm quit
- **Database Schema**: `sessions.mode` column and new `pomodoro_stats` table for tracking statistics
- **Configuration File**: Support for `~/.config/focus/pomodoro.toml` with three-level precedence (file → env vars → CLI flags)
- **Release Infrastructure**: Git Flow release workflow with Conventional Commits validation
- **GitHub Actions**: CI/CD pipelines for testing and cross-platform binary builds (Linux/macOS/Windows)
- **Developer Tools**: Makefile with `make setup`, `make test`, `make lint`, `make release VERSION=x.y.z`

### Changed

- `focus log` output now includes `MODE` column (freeform/pomodoro)
- `focus report` now includes mode breakdown and time spent in each mode
- TUI menu reorganized to include Pomodoro options
- TUI Start Form shows mode selection dialog before task entry

### Fixed

- Q key on Pomodoro tab now shows confirmation dialog instead of abruptly quitting
- Pomodoro countdown now uses real wall-clock time (tolerates system sleep and skew)

### Technical

- **Pomodoro Implementation**: Real wall-clock timer with automatic database migrations
- **Dependencies**: Added `toml = "0.8"`, `ctrlc = "3"` for configuration and signal handling
- **UI Performance**: TUI redraws at 100ms intervals for smooth countdown display
- **Test Coverage**: All 61 tests pass; TDD compliance for all user stories
- **Release Automation**: Local `commit-msg` hook enforces Conventional Commits format
- **Build Infrastructure**: GitHub Actions CI/CD with cross-platform binaries (Linux/macOS/Windows)
- **Backward Compatibility**: Existing freeform sessions unchanged; Pomodoro is opt-in

---

## [0.2.0] - April 6, 2026

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

## [0.1.0] - April 2, 2026

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
