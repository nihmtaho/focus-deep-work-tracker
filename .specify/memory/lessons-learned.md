# Lessons Learned

Bug patterns and fixes discovered during code review. Reference these when writing
new DB or event-handling code to avoid repeating the same mistakes.

---

## L001 — Never use timestamps as SQL join keys after an UPDATE

**Discovered in**: PR #4 (`004-tui-session-controls`), `src/db/session_store.rs`

**Bug**: `stop_session` issued an `UPDATE ... WHERE end_time IS NULL` and then a
`SELECT ... WHERE end_time = ?1` using the same wall-clock timestamp as the join
key. If two sessions happened to share the same `end_time` second, the `SELECT`
returned the wrong row.

**Fix**: Capture the active session's `id` _before_ the `UPDATE` (via
`get_active_session`), then `UPDATE WHERE id = ?` and re-fetch by that same `id`.
The row identity is stable; a timestamp is not.

**Rule**: Any function that UPDATE then SELECT must join on a stable primary key,
not on a value that was just written. Never use `last_insert_rowid()` after an
UPDATE — it is only valid after an INSERT.

---

## L002 — Never silently discard save/write errors with `let _ = ...`

**Discovered in**: PR #4 (`004-tui-session-controls`), `src/tui/events.rs`

**Bug**: `handle_settings_tab` toggled `app.config.vim_mode` and immediately
showed a success overlay, then called `let _ = save_config(...)` — discarding any
I/O error. The user would see "Vim mode enabled" even if the setting was never
persisted to disk.

**Fix**: Check the result of `save_config` before setting the overlay message.
Show the success message only if the save succeeded; on failure, set an error
overlay with `MessageOverlay::error(...)`.

**Rule**: All fallible operations that affect user-visible state MUST surface errors
via `app.message` (in TUI context) or propagate with `?` (in non-TUI context).
`let _ = result` is only acceptable for operations that are genuinely fire-and-forget
with no observable side-effects. Config persistence is not fire-and-forget.
This is a direct application of **Principle III** of the constitution.
