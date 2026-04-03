# Research: Focus TUI Dashboard

**Branch**: `003-focus-tui-dashboard` | **Date**: 2026-04-02

## Decision 1: TUI Library

**Decision**: Ratatui 0.29 with Crossterm 0.28 backend

**Rationale**: Ratatui is the most actively maintained Rust TUI library (>3k GitHub stars, regular releases). Its immediate-mode rendering model — where the entire frame is re-drawn on every tick — maps directly to the spec's poll-based refresh design. Crossterm provides the cross-platform terminal backend (raw mode, alternate screen, event polling) and compiles into the binary with no system dependencies, satisfying Constitution Principle I.

**Alternatives considered**:
- **Cursive** — widget-based, simpler API, but event-driven design makes it harder to integrate a tight poll loop for live elapsed-time updates
- **Crossterm alone (manual rendering)** — zero dependencies on a TUI framework, but requires hand-rolling all layout, scrolling, and widget logic; estimated 3× more code for no functional gain
- **tui-rs** — Ratatui's predecessor, archived; Ratatui is its direct fork and continuation

---

## Decision 2: Event Loop Pattern

**Decision**: Poll-based loop with 100ms timeout (`crossterm::event::poll(Duration::from_millis(100))`)

**Rationale**: The spec requires 100ms refresh and keyboard responsiveness. `crossterm::event::poll` blocks for up to the given duration, returns `true` if an event is ready, and returns `false` on timeout — both cases trigger a re-render. This gives:
- Keyboard events processed within ~100ms (acceptable for interactive use)
- Elapsed time updates at ~10Hz (far exceeds the 1Hz minimum in FR-002)
- DB re-query on every iteration (~0.1–0.5ms for local SQLite, well within 50ms render budget)

No async runtime (`tokio`) is introduced. This satisfies Constitution Principle I (single binary, minimal deps) and avoids Send/Sync complexity across the TUI state.

**Alternatives considered**:
- **Async event loop** — would allow true concurrent DB queries and timers, but adds `tokio` as a dependency and complicates the borrow model for a single-user local tool; no benefit justifies the cost
- **500ms timeout + manual timer** — coarser, would cause noticeable lag on keypress

---

## Decision 3: Terminal Lifecycle Management

**Decision**: Explicit setup/teardown with panic hook restoration

**Pattern**:
```
enable_raw_mode() → execute!(EnterAlternateScreen) → [event loop] → execute!(LeaveAlternateScreen) → disable_raw_mode()
```

A custom panic hook installed at `tui::run()` entry restores the terminal before printing the panic message, preventing a corrupted terminal on unexpected crashes. This is the canonical Ratatui pattern.

**Rationale**: Without restoration, a panic leaves the terminal in raw mode with the alternate screen active, requiring the user to run `reset`. The panic hook ensures any crash is transparent.

---

## Decision 4: TTY / Non-Interactive Detection

**Decision**: `std::io::IsTerminal::is_terminal(&std::io::stdout())` (stable since Rust 1.70)

**Rationale**: Rust 1.70+ provides `IsTerminal` in the standard library, making an external crate (`atty`, `is-terminal`) unnecessary. If stdout is not a terminal, `tui::run()` prints `"Error: focus ui requires an interactive terminal."` to stderr and exits with code 1. This satisfies FR-012 without partial TUI state being rendered.

---

## Decision 5: DB Re-query Strategy

**Decision**: Query `get_active_session` and `aggregate_by_tag` (today) on every tick in the dashboard view; query `list_sessions` and `aggregate_by_tag` only on view entry or explicit page change for Log and Report views.

**Rationale**: The spec requires dashboard state to reflect external CLI changes within one tick (clarification Q2). Dashboard queries are cheap (indexed single-row reads). Log and Report are static snapshots — re-querying on every tick would be wasteful and visually distracting. Log re-queries when the page changes; Report re-queries when the time window selection changes.

---

## New Dependencies Summary

| Crate | Version | Justification | Covered by constitution? |
|-------|---------|---------------|--------------------------|
| `ratatui` | 0.29 | TUI rendering framework | No existing equivalent — new capability |
| `crossterm` | 0.28 | Terminal raw mode, event polling, alternate screen | Pulled by ratatui; also needed directly for setup/teardown |

Both crates are pure Rust, statically linked, and add no system library requirements. Binary size increase estimate: ~1.5–2 MB (acceptable for a developer CLI tool).
