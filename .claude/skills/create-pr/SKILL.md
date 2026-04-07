---
name: create-pr
description: Create a pull request following project PR standards. Use this skill whenever the user asks to open a PR, create a pull request, submit a PR, push and PR, or says things like "make a PR", "open a PR", "ready to PR", "can you PR this", or "ship this". Enforces correct title format (feat/fix/refactor/chore), links spec and task files, fills in the PR template with a proper test plan, and validates cargo tests pass before opening.
---

## User Input

```text
$ARGUMENTS
```

Consider any user-provided input (PR title, target branch, extra context) before proceeding.

## Goal

Create a pull request that fully satisfies **Principle VII (Pull Request Standards)** from `.specify/memory/constitution.md` and uses the project PR template at `.github/PULL_REQUEST_TEMPLATE.md`.

---

## Step 1 — Pre-flight checks

Run these in parallel:

1. **`cargo test`** — confirm all tests pass. If any test fails, STOP and report the failure to the user. Do NOT create the PR until tests are green.
2. **`git status`** — confirm working tree is clean (all changes committed).
3. **`git log --oneline main..HEAD`** (replace `main` with the base branch if different) — collect the commits that will be in this PR.
4. **`git diff main...HEAD`** — collect the full diff for context.
5. **Read `.github/PULL_REQUEST_TEMPLATE.md`** — load the PR body template.
6. **Read `.specify/memory/constitution.md`** — confirm current PR standards (Principle VII).

---

## Step 2 — Identify feature context

From the current branch name (format `###-short-description`), resolve the feature spec directory:

- Spec file: `.specify/specs/[###-feature]/spec.md`
- Tasks file: `.specify/specs/[###-feature]/tasks.md`

Read both files to extract:
- **What**: 1–2 sentence feature summary from spec.md
- **Why**: problem statement / motivation from spec.md
- **Key implementation decisions**: non-obvious choices visible in the diff or tasks

---

## Step 3 — Validate and build the PR title

The title **MUST** follow: `[feat|fix|refactor|chore]: description`

- `feat` — new feature, capability, or behavior
- `fix` — bug fix or correctness improvement
- `refactor` — code restructuring without behavior change
- `chore` — docs, build, or non-code changes

If the user provided a title in `$ARGUMENTS`, validate it against this format and correct it if needed (inform the user of any correction).

If no title was provided, derive one from the spec and commits.

---

## Step 4 — Build the PR body

Fill in the PR template from `.github/PULL_REQUEST_TEMPLATE.md`. Replace every placeholder:

- **What**: 1–2 sentences from spec.md describing the feature.
- **Why**: Pull from the problem statement in spec.md.
- **Implementation notes**: List key decisions made — non-obvious choices, trade-offs, anything a reviewer needs to understand the diff. Draw from the diff and tasks.md.
- **Test plan**: MUST include:
  - `- [ ] \`cargo test\` passes`
  - At least **2 additional manual test steps** that are actionable and independently reproducible. Derive these from the feature's user stories (e.g., `focus start "test"` → `focus stop` → `focus log` shows entry).
- **Spec reference**: Fill in the actual paths:
  - `.specify/specs/[###-feature]/spec.md`
  - `.specify/specs/[###-feature]/tasks.md`
- **Constitution Check** (Principle VII checkboxes): Mark all as checked `[x]` only after confirming each is satisfied.

---

## Step 5 — Determine base branch

- Default base branch: check `git remote show origin` or use the branch the current branch was cut from.
- If the user specified a base branch in `$ARGUMENTS`, use that.
- Typical base for feature branches: `main` (the project's main development branch per git history) or `main`.

---

## Step 6 — Create the PR

Push the branch if not already pushed, then run:

```bash
gh pr create \
  --title "<validated title>" \
  --base <base-branch> \
  --body "$(cat <<'EOF'
<filled PR body>
EOF
)"
```

**Do NOT** include `Co-Authored-By: Claude` or any AI attribution lines anywhere in the PR title or body (Principle VI).

---

## Step 7 — Report to user

Output:
- The PR URL
- A one-line summary of the title and base branch
- Any corrections made to the title or template (so the user is aware)
- A reminder if `cargo test` was not run (skipped due to user override)
