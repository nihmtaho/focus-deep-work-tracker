# Research: TUI Session Controls with Vim Mode and Tab Views

**Branch**: `004-tui-session-controls` | **Phase**: 0 | **Date**: 2026-04-03

---

## Decision 1: Tab Navigation Model

**Decision**: Replace the current Menu-driven navigation (View::Menu → sub-views) with a persistent tab bar. A new `Tab` enum (`Dashboard | Log | Report | Settings`) represents the active tab. The existing `View` enum is refactored: tab content replaces the old view variants; overlays (inline prompt, confirm dialog, help) become a separate `Overlay` enum layered on top.

**Rationale**: The spec requires Log and Report to be always-accessible tabs without a menu intermediary. The current architecture routes all navigation through `View::Menu`, which is structurally incompatible with persistent tabs. Separating "active tab" from "overlay state" cleanly handles the modal boundary requirement (FR-017) without deeply nesting state.

**Alternatives considered**:
- Keep `View` enum and add `Tab` variants alongside Menu — rejected because it duplicates navigation state and causes ambiguity between "which tab is active" vs "which view is showing".
- Use a single `Screen` enum with both tabs and overlays — rejected because overlay rendering must stack on top of any tab, not replace it.

---

## Decision 2: Vim Mode Persistence

**Decision**: Store user preferences in a JSON config file at `{config_dir}/focus/config.json` where `config_dir` is resolved via `dirs::config_dir()`. Use the existing `serde_json` dependency for serialization. Config struct: `AppConfig { vim_mode: bool }`. Default: `vim_mode: false`. Load on TUI startup; save immediately on any change.

**Rationale**: No new dependency needed — `serde_json` is already in `Cargo.toml`. The `dirs` crate already provides `config_dir()`. JSON is human-readable and trivially editable. `~/.config/focus/` is the idiomatic XDG location on Linux/macOS.

**Alternatives considered**:
- TOML format — requires adding `toml` crate, which would duplicate serialization functionality already covered by `serde_json`.
- Environment variable — rejected because it's not persistent across sessions.
- SQLite table for config — rejected because config is application-level, not data-level; mixing them complicates schema migrations.

---

## Decision 3: Signal Handling for Auto-Save (FR-015)

**Decision**: Add `ctrlc = "3"` crate. In `tui/mod.rs`, set a `ctrlc` handler that sets a shared `Arc<AtomicBool>` flag. The event loop checks this flag each iteration and exits cleanly (triggering the auto-save path) when set. The auto-save call (`session_store::stop_session`) is placed in the `run_app` cleanup path — reached on any normal exit (including the signal flag check).

For the Ctrl+C case, the existing key event handler already returns `Ok(true)` (quit). The cleanup path in `run_app` adds the auto-save there too.

**Rationale**: `ctrlc` is the canonical Rust crate for handling SIGINT+SIGTERM in a unified, cross-platform way. It does not duplicate any functionality already in the stack. The pattern of "flag + check in event loop" avoids the unsafe access-from-signal-handler problem and lets the DB cleanup happen on the normal Rust stack.

**Alternatives considered**:
- `signal-hook` crate — more powerful but heavier; `ctrlc` is sufficient for the use case.
- Raw `libc::signal()` — unsafe and non-portable.
- Extending the panic hook — panic hooks cannot safely access the DB connection; auto-save on panic would require `Arc<Mutex<Connection>>` restructuring.

---

## Decision 4: Key Binding Conflict Resolution (1/2/3 keys)

**Decision**: Number keys `1`/`2`/`3`/`4` switch tabs only when no overlay is active. Within the Report tab, the time window selector is controlled by `h`/`l` (left/right) and `[`/`]` — not number keys. The Report tab's existing `1`/`2`/`3` shortcut for windows is removed to avoid conflict with tab switching.

**Rationale**: The spec requires number keys for tab navigation (FR-005). The Report view already uses `1`/`2`/`3` for window selection. Keeping both would require context-dependent number key behavior within the same global handler, making the key map unpredictable. Removing window-selection shortcuts from Report in favor of global tab shortcuts is simpler and consistent. The `h`/`l` navigation for Report windows remains.

**Alternatives considered**:
- Context-dependent number keys (tab switch unless in Report) — rejected as confusing and hard to document in the `?` help overlay.
- Use `F1`/`F2`/`F3`/`F4` for tabs — not universally available across terminals.
- Use `Shift+1..4` — awkward on international keyboards.

---

## Decision 5: Inline Prompt Implementation

**Decision**: The inline prompt is an `Overlay::Prompt { label: String, value: String, on_confirm: PromptAction }` state stored in `App`. When an overlay is active, the event handler enters input mode (all printable chars go to `value`; Escape cancels; Enter confirms). The `PromptAction` enum (`StartSession | RenameSession { id: i64 }`) drives the action taken on confirm.

The existing `View::StartForm` and `View::Menu` variants are removed in the refactor. All session creation flows through the `Overlay::Prompt` path.

**Rationale**: An overlay state is cleaner than a full `View` variant because it preserves the underlying tab's rendered content (the user can see the dashboard while typing). It also naturally implements the modal boundary — the event handler switches to input mode exactly when a prompt overlay is present.

**Alternatives considered**:
- Keep `View::StartForm` as a full-screen form — rejected because the spec requires inline prompts that don't switch screens.
- Bottom status-bar text entry — rejected (user chose Option A in clarification: explicit modal boundary).

---

## Decision 6: Session Delete/Rename DB Functions

**Decision**: Add two new functions to `session_store.rs`:
- `delete_session(conn, id: i64) -> Result<()>` — `DELETE FROM sessions WHERE id = ?1 AND end_time IS NOT NULL`
- `rename_session(conn, id: i64, new_task: &str) -> Result<()>` — `UPDATE sessions SET task = ?1 WHERE id = ?2`

Active sessions (end_time IS NULL) cannot be deleted via the log (they don't appear in the completed-sessions list). Rename is allowed on any session.

Add `FocusError::SessionNotFound { id: i64 }` for when the targeted row does not exist.

**Rationale**: Only completed sessions appear in the Session Log tab (FR-006 references "past sessions"). The `AND end_time IS NOT NULL` guard in `delete_session` prevents accidental deletion of an in-progress session via a race condition. No schema changes required.

**Alternatives considered**:
- Allow deleting active sessions — rejected; the spec does not mention it and it would require stopping the active session state in the UI simultaneously.
- Soft-delete (deleted_at column) — rejected; no audit/recovery requirement in the spec; hard delete keeps the schema unchanged.
