# Developer Quickstart: TUI Session Controls

**Branch**: `004-tui-session-controls` | **Date**: 2026-04-03

---

## Prerequisites

- Rust stable 1.77+ (`rustup show`)
- `cargo build` passes on `main` before starting

```bash
git checkout 004-tui-session-controls
cargo build
cargo test
```

---

## What This Feature Changes

This feature refactors the TUI navigation model and adds inline controls. The key structural shift is:

**Before**: `View` enum controlled the entire screen (Dashboard → Menu → StartForm/Log/Report)  
**After**: `active_tab: Tab` controls which tab is shown; `overlay: Overlay` handles prompts/dialogs on top

**Files touched** (approximate — implement in this order):

1. `Cargo.toml` — add `ctrlc = "3"`
2. `src/error.rs` — add `SessionNotFound`
3. `src/db/session_store.rs` — add `delete_session`, `rename_session`
4. `src/config.rs` — new file; `AppConfig` + load/save functions
5. `src/tui/app.rs` — replace `View` with `Tab` + `Overlay`; add `AppConfig` field
6. `src/tui/events.rs` — full rewrite of event dispatch for tab/overlay model
7. `src/tui/ui.rs` — add tab bar rendering; dispatch to tab views; render overlays
8. `src/tui/views/dashboard.rs` — remove `[M] Menu` hint; add `[N] New  [S] Stop`
9. `src/tui/views/log.rs` — add item selection highlight; vim nav; delete/rename keys
10. `src/tui/views/report.rs` — remove 1/2/3 window shortcuts (use h/l only)
11. `src/tui/views/settings.rs` — new file; render vim mode toggle
12. `src/tui/views/mod.rs` — expose `settings` module
13. `src/tui/mod.rs` — add `ctrlc` signal handler; auto-save on any exit path

---

## TDD Workflow (Constitution Principle II)

For each change:
1. Write the test first in `#[cfg(test)]` block or `tests/`
2. Confirm `cargo test` **fails** (red)
3. Implement until `cargo test` **passes** (green)
4. Run `cargo clippy -- -D warnings` and `cargo fmt`

---

## Running the TUI Locally

```bash
cargo run -- ui
```

**Test vim mode toggle:**
1. Launch `cargo run -- ui`
2. Press `4` to go to Settings tab
3. Press `v` to enable vim mode — should show "Vim mode: ON"
4. Press `q` to quit
5. Re-launch — vim mode should still be ON (persisted)
6. Verify `cat ~/Library/Application\ Support/focus/config.json` (macOS) shows `"vim_mode": true`

**Test quick start:**
1. Launch `cargo run -- ui` (Dashboard tab)
2. Press `n` — inline prompt should appear at bottom
3. Type session name, press Enter — session starts, prompt closes
4. Press `s` — session stops, elapsed time shown in notification

**Test auto-save on force-close:**
1. Launch `cargo run -- ui`, start a session with `n`
2. Wait 5 seconds
3. From another terminal: `kill -TERM $(pgrep focus)`
4. Run `focus log` — last session should appear with ~5s duration

**Test Log tab delete/rename:**
1. Create a session, stop it
2. Press `2` (Log tab) — session appears
3. Press `d` — confirm dialog appears
4. Press `y` — session deleted

---

## Checkpoint Gates

Before marking any phase complete:

```bash
cargo build          # must pass
cargo clippy -- -D warnings   # must pass (zero warnings)
cargo test           # must pass (all tests green)
cargo fmt --check    # must pass (no formatting changes needed)
```
