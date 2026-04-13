# Implementation Plan: TUI Polish & Fixes

**Branch**: `009-tui-polish-fixes` | **Date**: 2026-04-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/009-tui-polish-fixes/spec.md`

## Summary

Fix 7 post-ship usability issues in the `focus` TUI: keyboard conflict between `[d]` (tab shortcut) and deletion in the Log tab; missing todo deletion in the Dashboard; rounded timer digits; Pomodoro timer not freezing after session end; settings (vim mode, theme) not persisting across restarts; no way to manually override the OS-detected theme; and incomplete vim mode. All fixes operate on the existing Rust + ratatui codebase with no new binary or daemon. Settings persistence re-uses `src/config.rs` (already exists with `AppConfig`, `load_config`, `save_config`). A new `config` CLI subcommand replaces the missing `focus config set` entry point.

## Technical Context

**Language/Version**: Rust stable 1.77+
**Primary Dependencies**: ratatui 0.29, crossterm 0.28, rusqlite 0.31 (bundled), clap 4, serde_json 1, toml 0.8, dirs 5, anyhow 1, thiserror 1
**Storage**: SQLite WAL at `~/.local/share/focus/focus.db`; settings JSON at `~/.config/focus/config.json` (already wired via `src/config.rs`)
**Testing**: `cargo test` — unit tests inline + `tests/integration/`
**Target Platform**: macOS, Linux, Windows (single binary)
**Project Type**: CLI + TUI (single binary)
**Performance Goals**: Key events respond <16ms; settings write completes <500ms
**Constraints**: No new binary; no daemon; `cargo build --release` must produce one artifact
**Scale/Scope**: Single-user local tool; no concurrency

## Constitution Check

| Principle | Gate question | Status |
|---|---|---|
| I. Single Binary | Does this feature require a second binary, daemon, or system dependency? | ✅ PASS — all changes are in-process; `config.json` is a file, not a service |
| II. Test-First | Are tests planned before implementation tasks in tasks.md? | ✅ PASS — each phase writes failing tests first |
| III. Structured Error Handling | Do all error paths use `anyhow::Result` + `FocusError` variants? Config saves surfaced via `MessageOverlay::error`? | ✅ PASS — `save_config` returns `Result<()>`; TUI callers must propagate to overlay |
| IV. Color-Independent Output | Is output readable without color? `NO_COLOR` respected? | ✅ PASS — timer block chars render without color; existing `no_color` flag passed through |
| V. Data Safety | WAL mode on DB open; stable primary key on UPDATE+SELECT? | ✅ PASS — no schema changes; todo delete uses `id` as key |
| VI. Commit Hygiene | No Co-authored-by AI attribution in planned commits? | ✅ PASS |
| VII. Pull Request Standards | PR title format, spec/task links, test plan with ≥2 manual steps? | ✅ PASS — enforced on PR creation |

## Project Structure

### Documentation (this feature)

```text
specs/009-tui-polish-fixes/
├── plan.md              ← this file
├── research.md          ← Phase 0 output
├── data-model.md        ← Phase 1 output
├── quickstart.md        ← Phase 1 output
└── tasks.md             ← /speckit.tasks output (not created here)
```

### Source Code — files touched by this feature

```text
src/
├── config.rs                          ← extend: add save_config_immediately helper, add `focus config set` integration
├── main.rs                            ← add `Config { set, get, list }` subcommand dispatch
├── commands/
│   └── config.rs                      ← NEW: `focus config set theme <name>` + `focus config set vim-mode <bool>`
├── models/
│   └── todo.rs                        ← already has delete(); expose via handler
├── tui/
│   ├── app.rs                         ← wire config save-on-change for vim_mode toggle, theme change
│   ├── keyboard.rs                    ← add Delete/Backspace bindings; add vim `dd` with 1s timeout; remove `d`-as-delete conflict
│   ├── handlers_todo.rs               ← extend: call delete() on Delete/Backspace in Dashboard context
│   ├── events.rs                      ← route Delete/Backspace to todo handler vs log handler vs input field
│   ├── themes.rs                      ← load theme from config (override auto-detect)
│   ├── timer_display.rs               ← replace box-drawing glyphs with full-block characters
│   └── mod.rs                         ← freeze timer when session ends (guard on active session)
tests/
├── integration/
│   ├── test_keyboard_bindings.rs      ← extend: Delete/Backspace delete; `d` tab switch; vim dd
│   ├── test_todo_colors.rs            ← extend: todo deletion + Dashboard context
│   ├── test_timer_display.rs          ← extend: block char assertion; no rounded corners
│   ├── test_theme_loading.rs          ← extend: config-override beats auto-detect
│   └── test_config_persistence.rs    ← NEW: vim_mode + theme round-trip
```

**Structure Decision**: Single-project Rust binary (Option 1). No new directories beyond `src/commands/config.rs`.

---

## Phase 0: Research

*All unknowns resolved from codebase survey. No external research agents needed.*

### Research Findings

**R-001 — Delete key cross-platform** (FR-001)
- Decision: Detect `KeyCode::Delete` (macOS `Delete`/`Fn+Backspace`) and `KeyCode::Backspace` in `keyboard.rs`. Both trigger deletion regardless of platform. crossterm exposes both as separate `KeyCode` variants — no runtime OS check needed; bind both unconditionally.
- Rationale: crossterm's `KeyCode::Delete` maps to the forward-delete key (macOS "Delete" = backspace on most keyboards). Binding both `Backspace` and `Delete` covers all platform layouts.
- Alternative rejected: Runtime `cfg!(target_os)` check — unnecessary complexity since crossterm already distinguishes them.

**R-002 — Vim `dd` with 1-second timeout** (FR-016)
- Decision: Track partial vim command in `KeyHandler` with a `pending_d: Option<std::time::Instant>`. On first `d`: set timestamp, emit `KeyAction::None`. On next key within 1s: if `d` again → emit `DeleteItem`; else clear and re-dispatch. Poll elapsed time in the existing event loop `event::poll(250ms)` budget.
- Rationale: Reusing the existing 250ms poll loop means the partial-command window naturally expires within 1–2 poll cycles. No new thread or timer needed.
- Alternative rejected: tokio async timeout — over-engineered for a single-key compositor.

**R-003 — Settings persistence** (FR-008)
- Decision: `src/config.rs` already has `AppConfig`, `load_config`, `save_config`. Gap: `save_config` is not called on change. Fix: add a `save_config_now(cfg: &AppConfig)` wrapper that calls `save_config(&config_file_path(), cfg)` and surfaces any `Err` via `MessageOverlay::error`. Call it whenever `app.config.vim_mode` or `app.config.theme` changes.
- Rationale: Minimal change to existing infrastructure. No new crate needed.

**R-004 — Theme config override** (FR-012)
- Decision: In `src/tui/themes.rs` `load_theme()`, check `app.config.theme` first. If `Some(name)` → resolve that name directly, bypassing OS auto-detect. If `None` → fall back to existing auto-detect.
- Rationale: `app.config` is already available at TUI startup; one `if let Some` guard is sufficient.

**R-005 — Timer freeze** (FR-006)
- Decision: In `src/tui/mod.rs` event loop, move the `last_tick` advance block inside `if app.has_active_pomodoro()`. Implement `App::has_active_pomodoro() → bool` checking `app.pomodoro_timer.is_some()`. On session end, set `app.pomodoro_timer = None`.
- Rationale: Existing structure already exposes `app.pomodoro_timer: Option<PomodoroTimer>`. The freeze is a two-line guard around the existing tick block.

**R-006 — Block-char timer digits** (FR-005)
- Decision: Replace the `glyphs_unicode` pattern arm in `timer_display.rs` that uses box-drawing chars (`┌ ┐ └ ┘ │ ─`) with 5-row full-block patterns using `█` and ` `. Keep the existing `glyphs_block` pattern (already uses `█`) as the primary — verify it has no rounded forms.
- Rationale: The `timer_display.rs` already has both a `Block` and a `Unicode` glyph set. The fix is to ensure the `Unicode` set is removed or also uses solid blocks, and that all rendering paths default to `Block`.

**R-007 — `focus config set` CLI subcommand** (FR-011)
- Decision: Add `Commands::Config { subcommand: ConfigSubcommand }` to `src/main.rs` enum. `ConfigSubcommand::Set { key: String, value: String }`. Dispatch to new `src/commands/config.rs`. Validate `key ∈ {theme, vim-mode}`; for `theme`, validate `value ∈ {dark, light, material, onedark}`.
- Rationale: Mirrors the existing `commands/` module pattern. No new crate needed.

---

## Phase 1: Design & Contracts

### Data Model (`data-model.md`)

See `data-model.md` for full entity specs. Summary:

| Entity | Location | Change |
|---|---|---|
| `AppConfig` | `src/config.rs` | Existing; no schema change. `theme: Option<String>` already present. |
| `Todo` | `src/models/todo.rs` | Existing `delete()` fn exposed. No schema change. |
| `PomodoroTimer` | `src/tui/app.rs` | No change; freeze by setting `app.pomodoro_timer = None`. |
| `KeyHandler` | `src/tui/keyboard.rs` | Add `pending_d: Option<Instant>` field for vim `dd` timeout. |

### Interface Contracts (`contracts/`)

**CLI contract** (`contracts/cli-config.md`):
```
focus config set theme <name>     # name ∈ {dark, light, material, onedark}
focus config set vim-mode <bool>  # bool ∈ {true, false}
focus config get theme            # prints current value
focus config get vim-mode         # prints current value
```

**Key binding contract** (`contracts/keyboard.md`):
- `Delete` or `Backspace` in Dashboard/Log todo panel → delete selected todo
- `d` (always) → navigate to Dashboard tab
- `d` then `d` within 1s in vim mode → delete selected item (same as Delete/Backspace)
- `Backspace` in `TodoInput` context → delete preceding character in input field

### Quickstart (`quickstart.md`)

Manual E2E scenarios:
1. Set theme: `focus config set theme material` → open `focus ui` → verify material palette
2. Set vim mode: `focus config set vim-mode true` → quit → reopen → verify j/k navigation works
3. Delete todo: Dashboard → select todo → press `Backspace` → todo gone → reopen app → still gone
4. Timer freeze: start Pomodoro → wait 30s → press ESC → timer shows frozen 0:30 → does not increment

---

## Complexity Tracking

*(No constitution violations — no entries required)*

