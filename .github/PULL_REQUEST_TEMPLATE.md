## What

[1-2 sentences from spec.md — what this feature does]

## Why

[Pull from the problem statement in spec.md]

## Implementation notes

[Key decisions made during implementation, non-obvious choices]

## Test plan

- [ ] `cargo test` passes
- [ ] Manual: `focus start "test"` → `focus stop` → `focus log` shows entry
- [ ] Manual: `focus report --today` shows correct totals

## Spec reference

- `.specify/specs/[feature]/spec.md`
- `.specify/specs/[feature]/tasks.md`

---

**Constitution Check** (Principle VII):
- [ ] PR title follows `[feat|fix|refactor|chore]: description` format
- [ ] Spec and task links included above
- [ ] Test plan includes ≥2 manual test steps
- [ ] `cargo test` passes before merge
