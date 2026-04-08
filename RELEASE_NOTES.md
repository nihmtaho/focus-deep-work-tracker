# Focus CLI v0.2.2 Release Notes

**Release Date:** April 08, 2026

## Overview

Release v0.2.2 combines Pomodoro timer functionality with comprehensive release infrastructure and CI/CD automation. This release brings significant productivity features alongside improved developer experience and distribution mechanisms.

## 🎯 Major Features

### Pomodoro Timer Mode
- **New Command**: `focus start --pomodoro <TASK>` for 25/5/15 Pomodoro cycles
- **Statistics**: `focus pomo-stats` displays daily/weekly streaks and completion metrics
- **TUI Integration**: Dedicated Pomodoro view with real-time countdown and phase indicators
- **Mode Selection**: Start screen now offers choice between Pomodoro and Freeform modes
- **Customization**: Adjust work/break durations via Settings tab or `~/.config/focus/pomodoro.toml`
- **Desktop Notifications**: Phase transitions trigger system notifications
- **Pause/Resume**: [P] to pause, [S] to skip break, [+5] to extend current phase

### Release & Distribution Infrastructure
- **Git Flow Workflow**: Structured release process with release branches and PR validation
- **Conventional Commits**: Enforced commit message format for clear changelog generation
- **Cross-Platform Builds**: GitHub Actions automatically builds binaries for:
  - Linux (x86_64)
  - macOS (x86_64 + arm64)
  - Windows (x86_64)
- **One-Command Release**: `make release VERSION=0.2.2` for complete release workflow

## 📊 Database & Configuration

### Database Enhancements
- New `sessions.mode` column distinguishes Pomodoro from freeform sessions
- New `pomodoro_stats` table for tracking daily completions and abandonments
- Automatic schema migrations on startup

### Configuration
- New `~/.config/focus/pomodoro.toml` with three-level precedence:
  1. Config file values
  2. Environment variables
  3. CLI flag defaults
- Preserves backward compatibility with existing session data

## 🔧 Technical Improvements

### Development Experience
- **Local Git Hooks**: Commit message validation before pushing
- **Makefile**: Simplified release and testing workflows
  - `make setup` — Install git hooks
  - `make test` — Run full test suite
  - `make release VERSION=x.y.z` — Prepare release
- **CI/CD Integration**: Automated testing and binary builds on every push

### Code Quality
- All 61 tests passing with TDD coverage
- Performance: TUI redraws at 100ms for smooth countdown
- Real wall-clock timer implementation (tolerates system sleep)
- No breaking changes — freeform sessions remain unchanged

## 📦 Installation / Upgrade

### Via Cargo
```bash
cargo install --force focus
```

### Via GitHub Releases
Download pre-built binaries from: https://github.com/nihmtaho/focus-deep-work-tracker/releases/tag/v0.2.2

### Supported Platforms
- Linux (x86_64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

## 📋 CLI Updates

### New Commands
- `focus start --pomodoro <TASK>` — Start Pomodoro session
- `focus pomo-stats` — View Pomodoro statistics

### Updated Commands
- `focus log` — Now includes `MODE` column (freeform/pomodoro)
- `focus report` — Now shows mode breakdown and time per mode

## 🎨 TUI Improvements

### New Elements
- Mode selector on start screen
- Dedicated Pomodoro tab with countdown and phase indicator
- Settings tab for duration configuration
- Confirmation dialog for quit action in Pomodoro mode

### Controls
- **[P]** Pause/resume current Pomodoro phase
- **[S]** Skip to next phase
- **[+5]** Extend current phase by 5 minutes
- **[Q]** Quit with confirmation

## 🐛 Bug Fixes

- Fixed race condition in `stop_session` state consistency
- Pomodoro timer now uses real wall-clock time (handles system sleep)
- Q key on Pomodoro tab shows confirmation instead of immediate quit
- Silent config save errors now properly logged

## ✅ Testing

- **Test Coverage**: 61 tests covering all features
- **TDD Compliance**: All user stories have test coverage
- **CI/CD**: Automated test validation on all commits
- **Platforms**: Tests run on Linux, macOS, and Windows

## 📝 Changelog

Full changelog with all technical details: https://github.com/nihmtaho/focus-deep-work-tracker/blob/main/CHANGELOG.md

## 🙏 Dependencies

- Rust stable (1.77+)
- clap 4 (CLI), rusqlite 0.31 (database), ratatui 0.29 (TUI)
- crossterm 0.28, chrono 0.4, colored 2, ctrlc 3, toml 0.8
- No external network dependencies

## 📞 Support

For issues, feature requests, or feedback: https://github.com/nihmtaho/focus-deep-work-tracker/issues
