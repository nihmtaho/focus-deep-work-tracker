# Changelog

## [1.2.0] - April 07, 2026

### Added
- 



All notable changes to this project are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- (upcoming features go here)

### Changed
- (upcoming changes go here)

### Fixed
- (upcoming fixes go here)

---

## [1.1.0] - 2026-04-06

### Added

- **TUI**: Session controls (pause, stop, confirm-quit with double-press)
- **TUI**: Menu-driven navigation for all major operations
- **Tests**: Comprehensive integration tests for session controls with race condition handling

### Changed

- TUI now requires user confirmation (double-press Q) when quitting with an active session

### Fixed

- Race condition in `stop_session` fixed by ensuring in-memory state matches database before allowing stop
- Silent config save errors now properly logged instead of being ignored

---

## [1.0.0] - 2026-04-02

### Initial Release

### Added

- **CLI Core**: `focus start <TASK>` to begin a work session, `focus stop` to end it, `focus status` to check active session
- **CLI Core**: `focus log` command to view completed sessions in reverse-chronological order
- **CLI Core**: `focus report` command to aggregate time by tag (today, current week, last 7 days)
- **CLI Core**: `focus export` command to export sessions as JSON or Markdown
- **TUI**: Interactive dashboard (`focus ui`) with real-time session tracking and tag-based summaries
- **TUI**: Session log viewer with pagination (10 rows per page)
- **TUI**: Report view showing tag totals and time windows
- **TUI**: Start form for entering task name and optional tag
- **TUI**: Full keyboard navigation (arrow keys, vim keys, Enter, Esc)
- **Database**: SQLite backend with automatic schema creation
- **Configuration**: Support for NO_COLOR environment variable
- **Documentation**: Comprehensive README with Quick Start, command reference, and keyboard shortcuts
- **Tests**: 35 tests covering all CLI commands, TUI state machine, and edge cases

### Technical

- Rust stable 1.77+ with dependencies: clap, rusqlite (bundled), chrono, dirs, colored, anyhow, thiserror
- SQLite WAL mode for efficient concurrent access
- Automatic crash recovery for sessions interrupted by system shutdown
- No external network dependencies; all data stays local
