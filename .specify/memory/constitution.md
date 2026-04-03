<!--
SYNC IMPACT REPORT
==================
Version change: 1.2.0 → 1.3.0
Bump rationale: MINOR — Materially expanded Principle III and Principle V with
  specific non-negotiable rules derived from bugs found in PR #4 code review
  (lessons L001 and L002 in .specify/memory/lessons-learned.md).

Modified principles:
  - III. Structured Error Handling — added rule: persistence errors MUST be
    surfaced to the user, never discarded with `let _ = result`
  - V. Data Safety — added rule: DB mutations MUST be identified by stable primary
    key, never by a value written in the same statement (e.g. timestamp join key)

Added sections: none

Removed sections: none

Templates updated:
  ✅ .specify/templates/plan-template.md — Updated constitution version reference
     to v1.3.0; expanded gate questions for Principle III and Principle V

Deferred TODOs: none
-->

# Focus Constitution

## Core Principles

### I. Single Binary

The `focus` tool MUST be distributed as a single self-contained binary with no
runtime dependencies. It MUST be buildable with `cargo build --release` alone and
MUST run on macOS and Linux (x86_64, aarch64) without any pre-installed system
libraries (SQLite is statically bundled via `rusqlite bundled` feature).
No daemon, no network requirement, no root/sudo.

**Rationale**: Developer tools that require system dependencies or background processes
create friction and deployment complexity. A single binary is the gold standard for
CLI tool distribution.

### II. Test-First (NON-NEGOTIABLE)

TDD is mandatory for all features. The workflow MUST follow:

1. Write tests → confirm they **fail**
2. Get explicit sign-off before implementing
3. Implement until tests **pass**
4. Refactor under green tests

Unit tests live inline (`#[cfg(test)]`) or in `tests/unit/`.
Integration tests live in `tests/integration/` and use a temporary SQLite file
(never `~/.local/share/focus/focus.db`) for full isolation.
No task may be marked complete if its tests were written after or skipped.

**Rationale**: Tests written after the fact verify nothing about design. The
Red-Green-Refactor cycle is the only reliable way to validate behaviour contracts.

### III. Structured Error Handling

All fallible operations MUST use `anyhow::Result` for propagation.
User-facing errors MUST be printed to **stderr** in the format `Error: <message>`.
Domain errors MUST be defined as typed variants in a `FocusError` enum using
`thiserror`. Panic is forbidden except in `expect()` calls on invariants that
are statically guaranteed by construction (e.g., hard-coded valid time values).

Errors from persistence operations (config saves, DB writes) MUST be surfaced to
the user — never silently discarded with `let _ = result`. In TUI context, surface
the error via `MessageOverlay::error(...)` before returning. In CLI context,
propagate with `?`. `let _ = expr` is only acceptable for operations that are
genuinely fire-and-forget with no observable side-effects on user-visible state.

**Rationale**: Mixing ad-hoc string errors with structured errors makes CLI output
unpredictable and breaks scripting. A typed error enum provides exhaustive handling
and clear user messages at a single exit point. Silent persistence failures cause
data loss that is invisible to the user — a config saved only in memory is
indistinguishable from one written to disk until the next restart.

### IV. Color-Independent Output

All terminal output MUST be fully readable without color. Color MAY be added as
a visual enhancement, but MUST be automatically disabled when:

- stdout is not a TTY (piped, redirected)
- The `NO_COLOR` environment variable is set (any value)

The `colored` crate's automatic TTY detection satisfies this. Tests MUST verify
that `NO_COLOR=1 focus <cmd>` produces no ANSI escape codes.

**Rationale**: CLI output is often captured, diffed, logged, or read in environments
without color support (CI logs, `less`, accessibility tools). Color-only signalling
is an accessibility and scripting hazard.

### V. Data Safety

The SQLite database at `~/.local/share/focus/focus.db` MUST be opened with
**WAL (Write-Ahead Logging) mode** enabled (`PRAGMA journal_mode=WAL`).
This ensures:

- Concurrent reads are never blocked by writes
- A crash during write leaves the database in a consistent state
- No data loss on unexpected process termination

The data directory MUST be created automatically on first run via
`std::fs::create_dir_all`. Any failure to open or migrate the database MUST
surface as `FocusError::DataFileCorrupted` with the full absolute path.

Database mutations MUST be identified by stable primary key when the mutated row
needs to be read back. Never use a value written by an UPDATE (e.g., a timestamp)
as the join key in a subsequent SELECT on that same row — two rows can share the
same timestamp within a second. Capture the row's `id` before the mutation and
re-fetch by that `id`. `last_insert_rowid()` is only valid after an INSERT, not
after an UPDATE.

**Rationale**: Developers trust a local tool to never corrupt their session history.
WAL mode is the minimum bar for SQLite durability in a single-writer tool. Using
timestamps as post-UPDATE join keys is a silent correctness bug: it works almost
always and fails unpredictably, making it harder to detect and diagnose than an
outright crash.

### VI. Commit Hygiene

Commit messages MUST NOT include AI model attribution lines such as
`Co-Authored-By: Claude <noreply@anthropic.com>` or any equivalent.
Commits MUST be authored solely under the developer's identity.

**Rationale**: AI attribution in commit history creates ambiguity about code
ownership, pollutes `git log`, and is not relevant to the repository's change record.

### VII. Pull Request Standards

Every pull request MUST follow these standards to ensure traceability and quality:

1. **Title format** MUST be: `[feat|fix|refactor|chore]: description`
   - `feat`: New feature, capability, or behavior
   - `fix`: Bug fix or correctness improvement
   - `refactor`: Code restructuring without feature or behavior change
   - `chore`: Documentation, build, or non-code changes

2. **Spec and task links** MUST be included in PR description:
   - Reference `.specify/specs/[###-feature]/spec.md` for user stories and requirements
   - Reference `.specify/specs/[###-feature]/tasks.md` for implementation breakdown
   - Links enable code review to verify requirements were met

3. **Test plan** MUST include at least 2 manual test steps:
   - Verify `cargo test` passes (automated gate)
   - Document manual scenarios that validate feature behavior
   - Manual tests MUST be actionable and independently reproducible

4. **Merge gate**: A PR MUST NOT be merged if `cargo test` fails.
   - The CI/automation MUST enforce this; no manual overrides
   - Failing tests indicate incomplete or regressed implementation

5. **Claude Code automation**: When creating PRs via Claude Code, MUST use
   `gh pr create` with the standard template from `.github/PULL_REQUEST_TEMPLATE.md`.
   This ensures consistent formatting and spec/task linkage across all PRs.

**Rationale**: Standardized PR format creates historical traceability (spec → code → merge),
prevents accidental merges of incomplete work, and makes code review faster by providing
context upfront. The template ensures feature specifications and tasks are never orphaned
from their implementation.

## Technology Stack

- **Language**: Rust stable (1.77+)
- **CLI parsing**: `clap 4` with derive API
- **Database**: `rusqlite 0.31` with `bundled` feature; WAL mode enabled (Principle V)
- **Time**: `chrono 0.4` with serde feature; all timestamps stored as Unix epoch
  integers (UTC); display converted to local time where appropriate
- **Error handling**: `anyhow 1` + `thiserror 1` (Principle III)
- **Output color**: `colored 2` with automatic TTY detection (Principle IV)
- **Home directory**: `dirs 5`
- **Serialization**: `serde 1` + `serde_json 1`
- **TUI framework**: `ratatui 0.29` + `crossterm 0.28`; used for the `focus ui`
  interactive dashboard (Principle I — no daemon; Principle IV — NO_COLOR respected)

New dependencies MUST be justified against this list. Introducing a dependency
that duplicates existing crate functionality requires a Complexity Tracking entry
in `plan.md`.

## Development Workflow

- **Branch naming**: `###-short-description` (e.g., `001-focus-cli-tracker`)
- **Commit messages**: imperative mood, present tense; no AI attribution (Principle VI)
- **Phase gates**: Setup → Foundational → User Stories → Polish; no story work
  begins before Foundational phase is complete and `cargo build` passes
- **Checkpoints**: `cargo build`, `cargo clippy -- -D warnings`, and
  `cargo test` MUST pass at every phase checkpoint before proceeding
- **Formatting**: `cargo fmt` MUST be run before any checkpoint commit

## Governance

This constitution supersedes all other project practices. Amendments require:

1. A clear rationale describing which principle is being added, changed, or removed
2. A version bump per semantic versioning (MAJOR/MINOR/PATCH as defined in the
   Sync Impact Report header)
3. Propagation to all affected templates via the `/speckit.constitution` command
4. All PRs and plan reviews MUST include a Constitution Check section confirming
   no violations (or documenting justified exceptions in `plan.md` Complexity Tracking)
5. PR Standard compliance (Principle VII) is enforced at merge time; all PRs MUST
   follow the title format, include spec/task links, and pass `cargo test` before merge

**Version**: 1.3.0 | **Ratified**: 2026-04-02 | **Last Amended**: 2026-04-03
