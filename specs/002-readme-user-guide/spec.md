# Feature Specification: Focus CLI — README & User Guide

**Feature Branch**: `002-readme-user-guide`
**Created**: 2026-04-02
**Status**: Draft
**Input**: User description: "create a README.md or users guide for this Focus CLI"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - New User Gets Started (Priority: P1)

A developer discovers the `focus` binary (via GitHub or cargo install). They want to
understand what it does, install it, and run their first session in under 5 minutes
without reading anything beyond the README.

**Why this priority**: If a new user cannot onboard from the README alone, the tool
is effectively undiscoverable. First-run success is the highest-value outcome for
any user-facing document.

**Independent Test**: A person unfamiliar with the project can read the README,
install the binary, run `focus start`, `focus stop`, and `focus status` correctly
without asking any questions.

**Acceptance Scenarios**:

1. **Given** a developer has Rust installed, **When** they follow the README install
   section, **Then** they have a working `focus` binary in their PATH
2. **Given** a developer has the binary, **When** they follow the Quick Start section,
   **Then** they can complete their first session (start → status → stop) without error
3. **Given** a developer reads the README, **When** they want to use `focus log` or
   `focus report`, **Then** they can find the correct syntax and flags without running
   `--help`

---

### User Story 2 - Existing User Looks Up a Command (Priority: P2)

A developer who already uses `focus` wants to quickly look up the syntax for a
specific subcommand (e.g., "how do I export to JSON again?").

**Why this priority**: Reference utility — the README doubles as a cheat sheet.
Users return to it regularly; it must be scannable without reading top to bottom.

**Independent Test**: A user can locate any subcommand's flags and an example
invocation within 10 seconds of opening the README.

**Acceptance Scenarios**:

1. **Given** a user wants to export data, **When** they scan the README, **Then**
   they find `focus export --format json` with a redirect example
2. **Given** a user wants to filter the report by time window, **When** they scan
   the README, **Then** they find `--today` and `--week` explained with examples

---

### User Story 3 - User Understands Data Storage and Privacy (Priority: P3)

A developer wants to know where their session data is stored and how to back it up
or inspect it directly.

**Why this priority**: Local data tools must answer "where is my data?" clearly.
Unanswered, this becomes a support question.

**Independent Test**: A user can find the database path, understand the storage
format, and find the export command for backups — all from the README alone.

**Acceptance Scenarios**:

1. **Given** a user wants to know where data is stored, **When** they read the README,
   **Then** they find the exact path (`~/.local/share/focus/focus.db`)
2. **Given** a user wants to back up or migrate data, **When** they read the README,
   **Then** they find the `focus export` command and a file redirect example

---

### Edge Cases

- **Resolved**: Users on Windows are out of scope — README notes macOS/Linux only.
- **Resolved**: Users without Rust installed are directed to rustup.rs with a one-liner.
- What if the user does not have a C compiler (required by rusqlite bundled)? Include
  platform-specific prerequisite instructions (Xcode CLT for macOS, build-essential
  for Linux).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The README MUST include a one-paragraph description of what `focus` does
  and who it is for, visible above the fold (before any installation instructions)
- **FR-002**: The README MUST include installation instructions covering both
  `cargo install` (from crates.io or path) and building from source
- **FR-003**: The README MUST include a Quick Start section with a complete
  start-to-stop session walkthrough (minimum: `start`, `status`, `stop`)
- **FR-004**: The README MUST document all six subcommands with their flags, default
  values, and at least one example invocation each
- **FR-005**: The README MUST state where session data is stored
  (`~/.local/share/focus/focus.db`) and note that it is created automatically
- **FR-006**: The README MUST include a Prerequisites section listing: Rust stable
  1.77+, and the platform-specific C compiler requirement for macOS and Linux
- **FR-007**: The README MUST include a section showing how to export and inspect
  data (backup use case)
- **FR-008**: The README MUST note that color output is automatically disabled when
  stdout is not a TTY or `NO_COLOR` is set
- **FR-009**: The README MUST include a supported platforms note (macOS, Linux;
  Windows out of scope)

### Key Entities

- **README.md**: A single Markdown file at the repository root. It is the sole
  entry point for new users and the primary reference for existing ones.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer unfamiliar with `focus` can complete their first session
  (install → start → stop) using only the README in under 5 minutes
- **SC-002**: Every subcommand's flags and a usage example can be located by
  scanning (not reading) the README in under 10 seconds
- **SC-003**: The README answers "where is my data stored?" without the user
  needing to run any command or read external documentation
- **SC-004**: The README requires no prior knowledge beyond basic terminal usage

## Assumptions

- The README is the only user-facing documentation file — no separate wiki, docs
  site, or man page is in scope for this feature
- The audience is developers comfortable with a terminal; prose-heavy tutorials
  are not needed — command examples and concise descriptions are preferred
- The README is written in GitHub-flavored Markdown and will be rendered on the
  GitHub repository page
- `cargo install --path .` is the primary local install method; a crates.io
  publish is assumed out of scope for now (README may include the command as a
  future option but should not promise it)
- A badge section (CI status, crates.io version) is out of scope for this
  initial README
