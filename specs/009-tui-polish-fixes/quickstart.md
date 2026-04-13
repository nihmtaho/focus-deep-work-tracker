# Quickstart — 009 TUI Polish Fixes

Manual acceptance scenarios for QA validation.

## 1. Keyboard conflict resolved (Issue #10)

```bash
focus ui
# Navigate to Log tab with [l]
# Add a todo if none exists
# Press [d]  →  should switch to Dashboard tab (NOT delete)
# Switch back to Log tab, select a todo
# Press [Delete] or [Backspace]  →  should show confirm delete prompt
```

**Expected**: `d` tab-switches; `Delete`/`Backspace` deletes.

## 2. Todo deletion from Dashboard (Issue #11)

```bash
focus ui
# On Dashboard tab, select a todo in the todo list panel
# Press [Delete] or [Backspace]
# Confirm deletion prompt
# Close app (Ctrl+C) and reopen
```

**Expected**: deleted todo does not reappear after restart.

## 3. Block-character timer (Issue #12)

```bash
focus ui
# Navigate to Timer/Pomodoro tab
# Start a session
# Observe the large timer digits
```

**Expected**: digits are composed entirely of █ block characters. No rounded corners, no box-drawing glyphs.

## 4. Timer freezes on session end (Issue #13)

```bash
focus ui
# Navigate to Timer tab, start a 25-min Pomodoro
# Wait ~10 seconds (observe timer incrementing)
# Press [e] or [Esc] to end session
# Observe timer area
```

**Expected**: timer stops incrementing immediately after session ends and shows the elapsed time at the moment of end.

## 5. Settings persistence — vim mode (Issue #16)

```bash
focus config set vim-mode true
focus ui
# Verify j/k navigation works
# Quit (Ctrl+C)
focus ui
# Verify j/k navigation still works without reconfiguring
```

**Expected**: vim mode is active on second launch without re-running `focus config set`.

## 6. Custom theme (Issue #15)

```bash
focus config set theme material
focus ui
# Verify material color palette is shown (green accent, dark background)
focus config set theme light
focus ui
# Verify light palette (white/cream background)
focus config set theme auto
focus ui
# Verify OS auto-detection runs
```

**Expected**: theme changes take effect immediately on next `focus ui` launch.

## 7. Config CLI error handling

```bash
focus config set theme neon
# Expected: "Error: unknown theme 'neon'..." printed to stderr, exit code 1

focus config set color blue
# Expected: "Error: unknown key 'color'..." printed to stderr, exit code 1

focus config get theme
# Expected: prints current theme value
```
