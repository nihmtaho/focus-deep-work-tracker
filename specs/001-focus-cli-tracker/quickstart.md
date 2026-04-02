# Quickstart: focus

**Branch**: `001-focus-cli-tracker`  
**Date**: 2026-04-02

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.77+)
- C compiler — required by `rusqlite` to compile bundled SQLite
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux (Debian/Ubuntu): `sudo apt install build-essential`
  - Linux (Fedora/RHEL): `sudo dnf install gcc`

## Build

```bash
# Development build
cargo build

# Release build (optimised, single binary)
cargo build --release
```

Binary output: `./target/release/focus`

## Install

```bash
# Install to ~/.cargo/bin/ (must be in PATH)
cargo install --path .
```

Verify:
```bash
focus --help
```

## Run Tests

```bash
# All tests (unit + integration)
cargo test

# Integration tests only
cargo test --test '*'

# Specific test
cargo test test_start_stop
```

## Usage

```bash
# Start a session
focus start "write unit tests"
focus start "refactor payment module" --tag client-a

# Check what you're working on
focus status

# Stop the session
focus stop

# View recent sessions
focus log
focus log --limit 20

# Weekly report
focus report
focus report --today
focus report --week

# Export data
focus export --format json > sessions.json
focus export --format markdown > sessions.md
```

## Data Location

```
~/.local/share/focus/
└── focus.db    # SQLite database (created automatically on first run)
```

To inspect raw data:
```bash
sqlite3 ~/.local/share/focus/focus.db "SELECT * FROM sessions ORDER BY start_time DESC LIMIT 10;"
```

## Cross-compilation (optional)

```bash
# macOS → Linux (requires cross or a Linux runner)
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
```
