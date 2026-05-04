# Design: Focus CLI — Surface Polish

**Date:** 2026-05-04  
**Approach:** Surface Polish (Approach A) — UX improvements + dead code removal  
**Scope:** TUI, CLI commands, Pomodoro, TODOs  
**Breaking changes:** Allowed

---

## Problem Statement

A production audit of the focus CLI revealed three categories of issues:

1. **Dead code** — View files and App fields that are never used, accumulating silently
2. **Duplication** — Logic repeated verbatim, theme resolution called repeatedly per frame
3. **UX friction** — Ambiguous keybindings, too-long TODO→Session flow, redundant CLI commands

This spec addresses all three in a way that delivers immediate user-facing value while reducing codebase surface area.

---

## 1. Dead Code Removal

### 1.1 Remove orphaned view files

`src/tui/views/menu.rs` and `src/tui/views/start_form.rs` are never called from production code. `menu.rs` references a removed navigation model; `start_form.rs` references `app::InputField` which no longer exists.

`src/tui/views/report.rs` renders a full-screen Report tab that was removed from navigation (no `Tab::Report` exists). Its `render()` function is unreachable.

**Action:** Delete all three files. Remove their `pub mod` declarations from `views/mod.rs`.

### 1.2 Remove dead App fields

These fields in `App` are never read from any view or handler:

| Field | Why dead |
|---|---|
| `today_summary` | Populated in `load_dashboard()`, never read — dashboard uses `report_metrics` now |
| `terminal_too_small` | Never set or read — terminal check is inline in the event loop |
| `report_rows` | Only used by the now-deleted `views/report.rs` |
| `report_window` | Same — only used by `views/report.rs` |
| `report_selected_window` | Same |

**Action:** Remove these five fields from `App`, update `App::new()` initializer, and remove `App::load_report()` which only populated `report_rows/window`.

### 1.3 Remove dead functions

| Function | Location | Why dead |
|---|---|---|
| `get_current_colors()` | `tui/themes.rs` | Never called from production code; has `// TODO: Integrate with config` comment |

**Action:** Delete the function. Its only test can be removed or rewritten to test `get_colors_for_theme` directly.

### 1.4 Remove unused KeyboardConfig

`AppConfig::keyboard: KeyboardConfig` contains `enable_number_shortcuts` and `enable_letter_shortcuts` that are never read by the keyboard handler. The shortcuts are always active regardless.

**Action:** Remove `KeyboardConfig` struct and the `keyboard` field from `AppConfig`. Update serialization and tests.

---

## 2. Duplication Elimination

### 2.1 Merge clock animation methods

`App::advance_clock_anim` and `App::advance_pomo_clock_anim` are ~50-line methods with identical logic. The only difference is which `(curr_str, prev_str, anim_frame)` triple they operate on.

**Action:** Extract a private free function:

```rust
fn advance_anim(
    curr: &mut String,
    prev: &mut String,
    frames: &mut [u8],
    new_str: &str,
) { ... }
```

Both public methods delegate to it. This removes ~50 lines of duplication.

### 2.2 Cache theme resolution per frame

`get_colors_for_theme(app.config.theme.as_deref())` is called 9+ times per render frame in `ui.rs` and views. Each call re-resolves the theme name string.

**Action:** Pass `&ThemeColors` as a parameter down the render call stack starting from `render()` in `ui.rs`. Resolve once at the top, share the reference. This also removes the thin `tui/themes.rs` wrapper module — callers can use `src/theme/` directly if needed, or keep the wrapper for `load_theme`.

### 2.3 Move time utilities to `db/time.rs`

`today_start()`, `current_week_start()`, `rolling_7d_start()` are defined in `commands/report.rs` (a CLI command module) but imported from the TUI layer (`tui/app.rs`, `tui/report.rs`). Cross-layer imports from a command module into the TUI layer are an architectural smell.

**Action:** Move the three functions to a new `db/time.rs` module. Re-export from `db/mod.rs`. Update all import sites.

---

## 3. UX Improvements

### 3.1 Fix keyboard navigation ambiguity

**Current problem:** Three overlapping ways to navigate:
- `d/l/s` → Dashboard/Log/Settings (letter shortcuts)
- `Tab/BackTab` → cycle tabs
- `1/2/3` → *context-dependent*: panel focus on Dashboard, tab navigation otherwise

The `d` key additionally conflicts with vim `dd` delete on the Dashboard tab, handled by a fragile conditional.

**Design:** Simplify to two clear, non-overlapping layers:

| Key | Always means |
|---|---|
| `Tab` / `BackTab` | Next / previous tab |
| `1` / `2` / `3` | Tab 1/2/3 (Dashboard/Log/Settings) — always, regardless of context |
| `d/l/s` | **Removed** as tab shortcuts — free up `d` for delete-only in vim mode |

**Panel focus** (`focused_panel_idx`) on the Dashboard is removed as a keyboard-driven concept. All Dashboard panels are always visible; keyboard context routes naturally based on the active state (e.g., TODO keys are always active when on Dashboard unless in an overlay). The `focused_panel_idx` field and related panel-highlight rendering can be removed.

Remove the `if !(vim_mode && on_dashboard)` special case entirely. Vim `dd` always works on Dashboard TODO list when vim_mode is active.

**Breaking change:** `d`, `l`, `s` no longer jump to tabs. Users who relied on letter shortcuts use `Tab` or number keys instead.

### 3.2 Streamline TODO → Session flow

**Current:** Select TODO → press `→` → enter task name (pre-filled from TODO title) → enter tag → done. 5 interactions for a linked session.

**Design:** When the user presses `→` on a selected TODO:
- Use the TODO title as the session task name directly — skip the task name prompt
- Open only the tag prompt: `"Tag for «{todo_title}» (optional):"` with a single Enter to skip

Result: 2 interactions (press `→`, press Enter to skip tag or type tag). The TODO title is still used as the session name, but the redundant name-entry step is eliminated.

If the user wants to customize the task name, they can press `n` (new session flow) instead.

### 3.3 Unify `report` and `pomo-stats` CLI commands

**Current:** Two commands with identical flag shapes:
```
focus report [--today | --week]
focus pomo-stats [--today | --week]
```

**Design:** Consolidate into one `focus report` command with a mode flag:

```
focus report [--pomo] [--today | --week]
```

- Without `--pomo`: shows session time aggregated by tag (current behavior)
- With `--pomo`: shows Pomodoro statistics (current `pomo-stats` behavior)

**Breaking change:** `focus pomo-stats` is removed. Add a clear deprecation period: for one version, `focus pomo-stats` can print a migration hint and delegate.

---

## 4. Architecture (Scoped)

Two targeted improvements that don't require a full refactor:

### 4.1 Theme colors passed as parameter, not re-resolved

Described in §2.2. This change naturally improves render performance and reduces coupling to `app.config` in every render function. No need to change the module structure.

### 4.2 `events.rs` split by domain (deferred)

`events.rs` at 1365 lines is a candidate for splitting into `events/dashboard.rs`, `events/log.rs`, `events/settings.rs`, `events/overlays.rs`. This is pure mechanical refactoring with no behavior change. Deferred to a separate spec — too large to bundle here without risk.

---

## Success Criteria

| Goal | Verification |
|---|---|
| Dead view files removed | `cargo check` passes; no `mod menu/start_form/report` in views |
| Dead App fields removed | All field references compile-clean; no `today_summary`, `terminal_too_small`, etc. |
| Clock anim deduplicated | Single `advance_anim` fn; both public methods < 5 lines each |
| Theme resolved once/frame | No repeated `get_colors_for_theme` calls inside nested render fns |
| Time utils in `db/time.rs` | No imports from `commands::report` in TUI modules |
| `d/l/s` shortcuts removed | `Tab` and `1/2/3` work; no special-case `d` conflict code; `focused_panel_idx` removed |
| TODO→Session: 2 steps | Pressing `→` on a TODO skips task name prompt |
| `pomo-stats` merged | `focus report --pomo` works; `focus pomo-stats` removed |
| All tests pass | `cargo test` green |
| Clippy clean | `cargo clippy -- -D warnings` passes |

---

## Out of Scope

- Splitting `events.rs` (deferred)
- Splitting `App` struct into sub-structs (deferred)
- New features (tags autocomplete, session templates, etc.)
- Export format additions

---

## Files Affected

**Deleted:**
- `src/tui/views/menu.rs`
- `src/tui/views/start_form.rs`
- `src/tui/views/report.rs`

**Modified:**
- `src/tui/views/mod.rs` — remove dead mod declarations
- `src/tui/app.rs` — remove 5 dead fields, `focused_panel_idx`, `load_report()`, merge clock anim
- `src/config.rs` — remove `KeyboardConfig`
- `src/tui/themes.rs` — remove `get_current_colors()`
- `src/tui/ui.rs` — resolve theme once, pass `&ThemeColors` down
- `src/tui/views/dashboard.rs` — accept `&ThemeColors` param
- `src/tui/views/log.rs` — accept `&ThemeColors` param
- `src/tui/views/settings.rs` — accept `&ThemeColors` param
- `src/tui/views/pomodoro.rs` — accept `&ThemeColors` param
- `src/tui/events.rs` — remove `d/l/s` tab shortcuts, fix TODO→Session flow
- `src/commands/report.rs` — add `--pomo` flag, absorb pomo-stats logic
- `src/commands/pomo_stats.rs` — replaced by `report --pomo`
- `src/main.rs` — remove `PomoStats` command variant
- `src/db/mod.rs` — add `pub mod time`

**New:**
- `src/db/time.rs` — `today_start()`, `current_week_start()`, `rolling_7d_start()`
