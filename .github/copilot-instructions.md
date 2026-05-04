# focus — Copilot Instructions

## Project Overview
`focus` is a Rust CLI productivity tracker with a TUI dashboard. It uses SQLite for persistence and ratatui for the terminal UI.

## Tech Stack
| Crate | Version | Notes |
|---|---|---|
| clap | 4 | derive API |
| rusqlite | 0.31 | `bundled` feature, WAL mode via `db::open_db()` |
| chrono | 0.4 | `serde` feature |
| colored | 2 | terminal coloring |
| dirs | 5 | XDG-style paths |
| thiserror | 1 | error types |
| anyhow | 1 | error propagation |
| serde / serde_json | 1 | serialization |
| ratatui | 0.29 | TUI dashboard |
| crossterm | 0.28 | terminal backend for ratatui |

## Project Structure
```
src/
  main.rs        # entry point, CLI dispatch
  lib.rs         # library root
  error.rs       # error types (thiserror)
  commands/      # one module per CLI subcommand
  db/            # database layer (open_db, migrations, queries)
  display/       # colored terminal output helpers
  models/        # domain types
  tui/           # ratatui TUI dashboard
tests/           # integration tests
```

## Commands
```bash
cargo test      # run all tests
cargo clippy    # lint
```

## Code Style
- Rust stable (1.77+), standard idioms
- Use `thiserror` for library errors, `anyhow` for command/bin errors
- Keep database logic in `db/`, display logic in `display/` and `tui/`
- No schema changes without updating migrations in `db/`
- Prefer `?` over `.unwrap()` / `.expect()` in non-test code

# Karpathy Guidelines

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.