---
id: T-2751
title: "Merge main into a PR branch only under the merge lock unless it conflicts"
milestone: M2
lane: quality
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A PR branch takes `origin/main` in once, under the merge lock (decision 0075, steward skill "Merge
lock" step 2), unless it has a real conflict or a known id collision. A branch that is only behind
`main` is not updated outside the lock.

## Context
- Over the 48 h up to 2026-10-25, PRs on `main` made 383 "merge origin/main" commits, about two per
  PR. Each one starts a full CI run of about 7.7 minutes, so these runs used more than half of all CI
  minutes.
- 190 of those merges were made before the PR took the merge lock. `main` moved again before the
  lock was taken, so the merge was made a second time under the lock. Under decision 0075 the lock
  holder already merges `main` in (step 2), so a merge before the lock is wasted unless the PR
  conflicts.
- Several texts still say "merge `main` in" when the branch is only behind: the steward skill's
  merge-policy check (c) ("Merge `origin/main` in, run `make check`, push, and merge only after CI
  is green"), the orphaned-PR rule, the work-loop's step 5c resume, and decisions 0028/0075.
- Decision 0020 stays: a `dirty` PR gets no CI, so a conflict is resolved at once, never waited
  out. Decision 0053's merge-queue text stays dormant and unchanged.

## Acceptance criteria
- [x] A new decision record amends 0075 and 0028. Outside the lock, `origin/main` is merged into a
      PR branch only for (1) a textual conflict (`mergeable_state: dirty`, or a local test-merge
      conflicts) or (2) a known semantic collision that the up-to-date check would catch (a task or
      decision id that `main` now also uses, as found by `tasks.py validate`/`check_decisions.py` on
      a local test-merge, or by red CI under the lock). Being behind is handled once, under the lock.
- [x] The steward skill (check (c), merge lock, orphaned PRs, branch updates, merge conflicts), the
      work-loop skill (steps 5c and 6), `docs/process/agent-workflow.md` agree with it; the routine prompts in
      `docs/process/routines.md` were checked and left unchanged (they only require "up to date with
      main" and defer to the steward skill).
- [x] Decision 0020 (conflicts are never "waiting on CI") and the dormant merge-queue text are kept.
- [x] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass; reviewers approve (architecture and security for the protected paths).

## Out of scope
- Changes to `scripts/merge_lock.py` or CI.
- Making CI skip runs on merge commits.

## Outcome
Decision 0121 amends 0075 and 0028. Outside the merge lock, a PR branch takes `origin/main` in only
to resolve a textual conflict (`dirty`, or a local test-merge conflicts) or a known task/decision id
collision, never only because it is behind. Being behind is handled once, under the lock (steward
"Merge lock" step 2). The steward skill has a new "When to merge `main` in" rule, and check (c),
merge-lock step 2, the orphaned-PR rule, "Branch updates" and "Merge conflicts" point to it. The
work-loop's steps 5c and 6.3 and agent-workflow.md loop step 9 follow it, and 0028 and 0075 carry
the amendment note. The routine prompts are unchanged: they already defer to the steward skill. The
0020 conflict rule and the dormant merge-queue text are kept. No eval delta.
