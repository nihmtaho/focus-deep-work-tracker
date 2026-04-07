# Release Note Template — focus CLI

Use this template for GitHub Release Notes (user-facing, shown on the Releases page).
This is distinct from CHANGELOG.md — release notes are higher-level and user-focused.

---

## Template

```markdown
## What's New in v{VERSION}

{ONE_LINE_SUMMARY}

### ✨ Features
- {USER_FACING_DESCRIPTION}

### 🐛 Bug Fixes
- {WHAT_WAS_FIXED_AND_WHY_IT_MATTERS}

### ⚡ Improvements
- {PERFORMANCE_OR_UX_IMPROVEMENT}

### 🔧 Technical
- {DEPENDENCY_CHANGES_OR_DB_SCHEMA}

---

**Install / Upgrade:**
\`\`\`bash
# via npm (coming soon)
npm install -g focus-deep-work

# via cargo
cargo install focus
\`\`\`

**Full Changelog:** https://github.com/nihmtaho/focus-deep-work-tracker/compare/v{PREV_VERSION}...v{VERSION}
```

---

## Writing Guidelines

### ONE_LINE_SUMMARY
- One sentence capturing the theme of the release
- Examples:
  - "Adds Pomodoro timer mode with full TUI support."
  - "Improves session controls and TUI navigation."
  - "Bug fixes and performance improvements."

### Features (✨)
- Describe **what the user can do now** that they couldn't before
- Lead with the command or UI element: "`focus start --pomodoro` lets you run 25/5 Pomodoro cycles"
- Skip internal-only changes

### Bug Fixes (🐛)
- Describe what was broken and what the impact was
- "Fixed Q key on Pomodoro tab — it now asks for confirmation instead of quitting immediately"

### Improvements (⚡)
- Performance gains with numbers when possible: "TUI now redraws at 100ms for smooth countdowns"
- UX improvements: "Settings tab now validates input inline"

### Technical (🔧)
- Only include if relevant to developers/packagers
- New dependencies, DB schema changes, config file changes
- Keep brief — one line per item

### What to OMIT
- Internal refactors with no user impact
- `chore:` and `docs:` commits
- CI/CD changes
- Test additions

---

## CHANGELOG.md Format (for reference)

The CHANGELOG uses a more detailed, component-prefixed format:

```markdown
## [X.Y.Z] - Month DD, YYYY

### Added

- **CLI**: `focus start --pomodoro <TASK>` command for 25/5 Pomodoro cycles
- **TUI**: Dedicated Pomodoro view with live countdown and phase indicator
- **Database**: `sessions.mode` column to distinguish session types
- **Configuration**: Support for `~/.config/focus/pomodoro.toml`

### Changed

- `focus log` output now includes `MODE` column
- TUI menu reorganized to include Pomodoro options

### Fixed

- Q key now shows confirmation dialog instead of quitting immediately
- Countdown uses real wall-clock time (tolerates system sleep)

### Technical

- Added `toml = "0.8"` and `ctrlc = "3"` dependencies
- Database migration applied automatically on first run
```

Key differences from GitHub Release Notes:
- Uses `**Component**:` prefix on Added items
- Includes `### Technical` section with dep/DB details
- More comprehensive — every notable change included
- Date format: `Month DD, YYYY` (e.g., `April 07, 2026`)
