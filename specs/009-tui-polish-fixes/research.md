# Research — 009 TUI Polish Fixes

All decisions resolved from direct codebase survey. No external crate additions required.

## R-001 — Delete key cross-platform (Issue #10, #11)

**Decision**: Bind `KeyCode::Delete` AND `KeyCode::Backspace` in `keyboard.rs` to trigger deletion. Both are fired by crossterm regardless of OS. No `cfg!` guard needed.

**Rationale**: crossterm exposes `KeyCode::Delete` (forward-delete / macOS `Fn+Backspace`) and `KeyCode::Backspace` as distinct variants. Binding both covers all keyboard layouts on macOS, Linux, and Windows.

**Alternatives considered**:
- `cfg!(target_os = "macos")` runtime check — rejected (unnecessary complexity; crossterm already distinguishes the keys).

## R-002 — Vim `dd` with 1-second timeout (Issue #14)

**Decision**: Add `pending_d: Option<std::time::Instant>` to `KeyHandler`. On first `d` in vim mode → store `Instant::now()`, emit `KeyAction::None`. On next key:
- If `d` again AND elapsed < 1s → emit `KeyAction::DeleteItem`.
- Else if elapsed ≥ 1s or key ≠ `d` → clear `pending_d`, re-dispatch the new key.

**Rationale**: The existing `event::poll(250ms)` budget guarantees 4 checks per second. Partial-command expires naturally within 1–1.25 s (1s elapsed + up to one 250ms poll cycle). No new thread or timer crate needed.

**Alternatives considered**:
- tokio/async timeout — over-engineered; adds a heavy async runtime.
- Poll timeout down to 50ms — wastes CPU; the 250ms budget is already approved.

## R-003 — Settings persistence (Issue #15, #16)

**Decision**: `src/config.rs` already provides `AppConfig`, `load_config(path)`, and `save_config(path, cfg)`. Fix requires only:
1. Call `save_config(&config_file_path(), &app.config)` immediately after mutating `app.config.vim_mode` or `app.config.theme`.
2. Surface `Err` via `app.set_message(MessageOverlay::error(...))` — never silent.

**Rationale**: Infrastructure already in place. One call site per mutation site.

**Alternatives considered**:
- Deferred write on exit — rejected per spec clarification (immediate persistence required).
- Separate TOML crate (`toml` 0.8) — already a project dependency but unnecessary here since config uses JSON.

## R-004 — Theme config override (Issue #15)

**Decision**: In `themes.rs` `load_theme()`, check `config.theme.as_deref()` first:
- `Some("dark")` → `Theme::Dark`
- `Some("light")` → `Theme::Light`
- `Some("material")` → `Theme::Material`
- `Some("onedark")` → `Theme::OneDark`
- `None` → existing OS auto-detect path.

**Rationale**: `AppConfig.theme: Option<String>` already exists. One `match` block inserted before the auto-detect branch.

**Alternatives considered**:
- Storing a `Theme` enum in config — requires custom serde impl; rejected in favour of `String` round-trip (already exists).

## R-005 — Timer freeze after session end (Issue #13)

**Decision**: In the TUI event loop (`src/tui/mod.rs`), guard the tick/elapsed update with `if app.has_active_pomodoro()`. Add `App::has_active_pomodoro() → bool` returning `app.pomodoro_timer.is_some()`. On `PomodoroEvent::SessionEnded` transition, ensure `app.pomodoro_timer` is set to `None`.

**Rationale**: `app.pomodoro_timer: Option<PomodoroTimer>` is already the canonical "timer running" signal. The freeze is two lines: one guard and one `None` assignment.

**Alternatives considered**:
- Separate `app.timer_frozen: bool` flag — redundant with `Option<PomodoroTimer>` being `None`.

## R-006 — Block-character timer digits (Issue #12)

**Decision**: Audit `timer_display.rs`'s existing glyph arrays. Remove any segment patterns using `┌ ┐ └ ┘ │ ─` (box-drawing). Ensure `render_digit()` uses only `█` (U+2588 FULL BLOCK) and ` ` (space) for all 10 digits. If a `GlyphStyle::Unicode` enum variant uses rounded chars, delete or replace it; default everything to `GlyphStyle::Block`.

**Rationale**: Full-block characters render identically on all terminal fonts. Box-drawing chars produce rounded appearance on many macOS terminal fonts.

**Alternatives considered**:
- Configurable glyph style — adds UX complexity for no user benefit; spec says "always use blocks".

## R-007 — `focus config set` CLI subcommand (Issue #15 / FR-011)

**Decision**: Add to `src/main.rs`:
```rust
Config {
    #[command(subcommand)]
    cmd: ConfigCmd,
}

enum ConfigCmd {
    Set { key: String, value: String },
    Get { key: String },
}
```
Dispatch to `src/commands/config.rs`. Validate `key ∈ {"theme", "vim-mode"}`. For `theme`, validate `value ∈ {"dark", "light", "material", "onedark"}`. Mutate `AppConfig`, call `save_config`, print confirmation.

**Rationale**: Mirrors existing `commands/` pattern. One new file, ~50 lines.

**Alternatives considered**:
- `focus set-theme` top-level command — inconsistent with `focus` UX pattern of noun-verb grouping.
- Interactive TUI settings panel — exists already; CLI entry point is an additional convenience.
