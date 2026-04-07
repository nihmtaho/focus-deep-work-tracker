---
name: focus-release
description: "Execute the Git Flow release workflow for the focus CLI project. Use this skill whenever the user wants to: cut a new release (e.g., 'release v1.3.0', 'do a patch release', 'bump to minor version', 'ship the new version'), publish to GitHub, create a release tag, update CHANGELOG.md, or write release notes. This skill knows the exact workflow: analyze commits → AI-write CHANGELOG + Release Notes → make release → PR → tag → GitHub Actions → merge back. Trigger even if the user just says 'let's release', 'time to ship', or asks how to release."
---

# focus Release Workflow

Complete Git Flow release for the `focus` CLI project — from AI-generated content to published
GitHub Release with cross-platform binaries.

```
develop → AI content → release/vX.Y.Z branch → PR → main → tag → GitHub Actions → develop
```

Release Note template: `assets/release-note-template.md`

---

## Step 1: Determine Version

If version not specified, check latest tag:

```bash
git tag --sort=-v:refname | head -3
```

Ask: "Latest tag is `vX.Y.Z`. Patch (vX.Y.Z+1) / minor (vX.Y+1.0) / major (vX+1.0.0)?"

Pre-release format: `v1.3.0-rc.1`, `v1.3.0-beta.1`

---

## Step 2: Create Release Branch

```bash
make release VERSION=<version>
```

This creates `release/v<version>` from `develop`, bumps `Cargo.toml`, runs `git-cliff` for an
initial CHANGELOG entry, and commits `chore(release): bump version to <version>`.

If `git-cliff` is not installed: `cargo install git-cliff`

After this step, **do not move to Step 3 yet** — proceed to Step 2.5 to generate richer content.

---

## Step 2.5: AI Generate CHANGELOG Entry & Release Notes

This is the core AI step. Gather the raw material, then write both documents.

### Gather commits

```bash
LAST_TAG=$(git tag --sort=-v:refname | sed -n '2p')  # second tag = previous release
git log ${LAST_TAG}..HEAD --format="%s|%h|%b" --no-merges
```

Also get scope of changes:
```bash
git diff ${LAST_TAG}..HEAD --stat | tail -3
```

### Write CHANGELOG entry

Read `assets/release-note-template.md` → "CHANGELOG.md Format" section for the exact format.

Rules for the CHANGELOG entry:
- Date format: `Month DD, YYYY` (e.g., `April 07, 2026`)
- Use `**Component**:` prefix on `### Added` items — match existing entries in CHANGELOG.md
  - Components: `CLI`, `TUI`, `Database`, `Configuration`, `Statistics`, `Integration`
- `### Changed` — behavioral changes to existing commands/views, no component prefix
- `### Fixed` — bugs, with what broke and what the correct behavior is
- `### Technical` — deps added/removed, DB schema changes, config file additions
- Omit `chore:`, `docs:`, `ci:`, `test:` commits — they're not user-visible
- Write full sentences, not raw commit subjects — expand abbreviations, add context

Example of expanding a raw commit:
- Raw: `feat(tui): add pomodoro view`
- Written: `**TUI**: Dedicated Pomodoro view with live countdown, phase indicator (🍅 WORK / ☕ BREAK / 🌿 LONG BREAK)`

### Write GitHub Release Notes

Read `assets/release-note-template.md` → "Template" section.

Rules for Release Notes:
- Start with a one-line summary of the release theme
- Use emoji section headers: ✨ Features, 🐛 Bug Fixes, ⚡ Improvements, 🔧 Technical
- User-focused: describe what the user can do / what got better, not implementation details
- Include install/upgrade block with current version
- Footer: Full Changelog link comparing `v<prev>...v<new>`
- Omit Technical section if no dep/schema changes

### Present drafts to the user

Show both drafts clearly separated:

```
═══════════════════════════════════════
CHANGELOG.md entry (to be prepended):
═══════════════════════════════════════
[CHANGELOG DRAFT]

═══════════════════════════════════════
GitHub Release Notes (release page body):
═══════════════════════════════════════
[RELEASE NOTES DRAFT]
```

Ask: "Do these look good? Any edits before I apply them?"

### Apply content

After approval:

1. **Update CHANGELOG.md** — replace the git-cliff-generated entry at the top with the AI draft:
   - The git-cliff entry starts with the version header (e.g., `## [1.3.0]`)
   - Replace everything from that line down to the start of the next `## [` block

2. **Create RELEASE_NOTES.md** in the repo root with approved content:
   ```bash
   cat > RELEASE_NOTES.md << 'EOF'
   [AI-written release notes here]
   EOF
   ```

3. **Commit both files** to the release branch:
   ```bash
   git add CHANGELOG.md RELEASE_NOTES.md
   git commit --amend --no-edit
   # OR if not yet committed:
   # git commit -m "chore(release): add CHANGELOG and release notes for v<version>"
   ```

**Important**: Both `CHANGELOG.md` and `RELEASE_NOTES.md` must be committed to the release branch
and merged into main. When Step 5 pushes the tag, GitHub Actions will find `RELEASE_NOTES.md`
in the repo and use it for the release. If not found, it falls back to git-cliff (old behavior).

---

## Step 3: Show Final CHANGELOG Preview

```bash
head -50 CHANGELOG.md
```

Confirm the entry looks right before pushing.

---

## Step 4: Push & Open PR

```bash
git push origin release/v<version>

gh pr create \
  --base main \
  --head "release/v<version>" \
  --title "chore(release): v<version>" \
  --body-file RELEASE_NOTES.md
```

Using `--body-file RELEASE_NOTES.md` puts the full release notes directly in the PR description.

Tell the user: "PR is open. Review, run any final checks, then merge it."

---

## Step 5: Tag After Merge

Once the user confirms the PR is merged:

```bash
git checkout main
git pull origin main
git tag -a v<version> -m "Release <version>"
git push origin v<version>
```

GitHub Actions (`release.yml`) will now:
1. Build binaries for Linux x86_64, macOS x86_64, macOS arm64, Windows x86_64
2. Package artifacts
3. Create GitHub Release with binaries and release notes

Watch: https://github.com/nihmtaho/focus-deep-work-tracker/actions

**If GitHub Actions fails or is slow:** Manually create the release:
```bash
# Check workflow status first
gh run list --workflow release.yml | head -3

# If needed, create release manually with committed RELEASE_NOTES.md
gh release create v<version> --notes-file RELEASE_NOTES.md

# Download artifacts from GitHub Actions and upload
gh run download <RUN_ID> --dir /tmp/release-binaries
gh release upload v<version> /tmp/release-binaries/**/*
```

This is a fallback — normally GitHub Actions handles it automatically.

---

## Step 5.5 (Optional): Edit Release Notes on GitHub

If you need to tweak the release notes after the release is published (e.g., fix typos or
add late-breaking info):

```bash
gh release edit v<version> --notes-file RELEASE_NOTES.md
```

Otherwise, GitHub Actions will have already used the committed `RELEASE_NOTES.md` file.
Release will appear at: https://github.com/nihmtaho/focus-deep-work-tracker/releases/tag/v<version>

---

## Step 6: Merge Back to develop

```bash
git checkout develop
git pull origin develop
git merge --no-ff main -m "chore: merge main back to develop after v<version>"
git push origin develop
```

Optional cleanup:
```bash
git push origin --delete release/v<version>
git branch -d release/v<version>
```

---

## Step 7: Summary

```
✅ CHANGELOG.md updated with AI-written entry
✅ RELEASE_NOTES.md committed to release branch
✅ PR merged: release/v<version> → main
✅ Tag pushed: v<version>
✅ GitHub Actions: building 4-platform binaries
✅ GitHub Release notes: updated with AI-written content
✅ develop: synced with main
```

GitHub Release: https://github.com/nihmtaho/focus-deep-work-tracker/releases

---

## Pre-release Handling

Versions containing `-` (e.g., `v1.3.0-rc.1`) are automatically marked as pre-release by GitHub
Actions. Workflow is identical. Release Notes should include a `> ⚠️ Pre-release — not for production` banner.

---

## Hotfix Shortcut

For urgent production fixes:

```bash
git checkout main
git checkout -b hotfix/v<version>
# apply fix, commit with fix: prefix
git checkout main && git merge --no-ff hotfix/v<version>
git tag -a v<version> -m "Hotfix <version>"
git push origin main v<version>
git checkout develop && git merge --no-ff main && git push origin develop
git branch -d hotfix/v<version>
```

Still run Step 2.5 to write the CHANGELOG/Release Notes — even for hotfixes, the notes matter.

---

## Common Issues

**"Working directory not clean"** → commit or stash first  
**"Not on develop"** → `git checkout develop && git pull origin develop`  
**"git-cliff not found"** → `cargo install git-cliff`  
**"Tag already exists"** → `git tag -l "v<version>"`, then `git tag -d v<version>` if mistake  
**"Merge conflict in CHANGELOG.md"** → accept incoming (release branch) version  
**"gh: not authenticated"** → `gh auth login`  
**`git commit --amend` on pushed branch** → use `git push --force-with-lease origin release/v<version>` if you haven't pushed yet; if already pushed, make a new commit instead  
