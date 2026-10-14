---
id: T-0055
title: "Workers start each task from a fresh branch off origin/main, never a local main"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A worker or steward run never depends on the state of a local `main` branch. Every run, and every
task within a run, starts from a new branch cut from a freshly fetched `origin/main`. So a stale,
shallow or diverged local `main` can no longer stop a run or tempt it into a destructive reset.

## Context
2026-10-14, owner request. The sonnet B run at 05:46 UTC (session cse_01Ggek4BH7adCpLdJ25eKpHP)
did no work:
- The routine prompt and work-loop step 1 say `git fetch origin main && git checkout main && git
  pull --ff-only`.
- In that container, local `main` was a stale shallow checkout (last commit #440), so `pull
  --ff-only` found no common ancestor with `origin/main`.
- The run then did `git reset --hard origin/main`. The auto-mode classifier treats that as
  irreversible local destruction, denied the run's next (read-only) commands, and the run stopped.
- `origin/main` was never rewritten: older merges such as 4a86e105 (#85) are still ancestors of it.

Places that sync a local `main` today:
- `.claude/skills/work-loop/SKILL.md` step 1;
- the two worker prompts and the steward prompt in `docs/process/routines.md` ("Start from an
  up-to-date `main` …", "sync `main` and go back to (1)", "On an up-to-date `main`");
- anything else `rg -n "checkout main|pull --ff-only|up-to-date .main" .claude docs/process scripts Makefile` finds.

## Acceptance criteria
- [ ] Each of those places starts work with
  `git fetch origin main && git checkout -B <work-branch> origin/main`, or an equivalent that
  never checks out, pulls into or resets a local `main`. A new task's branch is created from
  `origin/main` directly (`git checkout -B agent/<lane>/T-XXXX-slug origin/main`). "Sync main
  between tasks" becomes "fetch `origin/main`, then branch from it".
- [ ] The work-loop skill says, in one line, never to `git reset --hard` or delete a local branch
  to recover from divergence: branch from `origin/main` instead.
- [ ] Handle shallow clones: if `git merge-base` is needed (decision 0028's
  `--is-ancestor origin/main` check), document `git fetch --unshallow` or a deeper fetch as the
  remedy, not a reset.
- [ ] Decision record (protected paths: `.claude/`, `docs/process/`). It notes that the owner
  must re-apply the three routine prompts from routines.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing routines directly (agents never modify routines; the owner re-applies them).
