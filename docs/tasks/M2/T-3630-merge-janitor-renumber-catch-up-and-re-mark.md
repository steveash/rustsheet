---
id: T-3630
title: "Merge janitor: renumber, catch-up and re-mark ejected and stale PRs"
milestone: M2
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Ejected and stale-marker PRs in `train` mode clear without their worker when the fix is
mechanical, idle orphans are flagged once, and a stranded PR escalates instead of flipping the
train to `lock` (decision 0224, the owner's ruling).

## Context
- The merger ejects for `conflict with main`, id collisions and `ci-red`; only the worker cleared
  them, and workers' sessions usually end first. `make catch-up` (decision 0207) covers
  baseline-only conflicts but needs a session to run it. Id collisions needed a hand renumber.
- A stranded ready PR triggered the emergency flip (decisions 0157, 0194), a liveness problem
  treated like a correctness one.
- Code: scripts/merge_catchup.py, scripts/merge_ready.py (`carry_problem`), scripts/id_registry.py,
  scripts/tasks.py (`reserve_id`), scripts/merge_train_stats.py (`stranded`, `thresholds`).

## Acceptance criteria
- [x] `make renumber PR=<n>` (scripts/merge_renumber.py, scripts/id_renumber.py) reserves fresh ids
  through the id registry, moves the PR's colliding files, rewrites the old id only in files the
  PR added and lines it added, merges `main` (list conflicts as catch-up), pushes one merge commit
  without force and restores the checkout; refuses forks, closed PRs, non-main bases, no
  collision, and conflicts outside the lists (scripts/test_merge_renumber.py).
- [x] The carry rule accepts a renumber merge by rebuilding the expected tree with the tool's own
  function, and refuses a substitution in a line `main` owns, another edit in a renamed file, a
  rename onto an id `main` has and a rename of `main`'s file (scripts/test_merge_renumber.py).
- [x] `make janitor [DRY_RUN=1]` (scripts/merge_janitor.py) catches up or renumbers ejected PRs
  idle 1 h, notes the pushed head and re-marks with `CARRY` on a later run once CI is green,
  comments orphans idle 4 h once per head, and re-marks or unlabels stale markers idle 2 h
  (scripts/test_merge_janitor.py, with gh_client's fake).
- [x] `merge-train-stats`: a stranded PR's marker names the current head; only tree-check
  failures print `-> flip to lock`, the rest `-> escalate`.
- [x] SPEC-0027, the steward and work-loop skills, routines.md (merger and steward prompts) and
  decisions 0050, 0157, 0194 and 0207 updated; decision 0224 records the owner's ruling.
- [x] The script tests, `tasks.py validate`, `check_decisions.py`, typos, file-size, duplication and governance checks pass.

## Out of scope
- Running the janitor, renumber or catch-up against the real repository (tests only).
- A step mode for the janitor's GitHub writes (it writes through `gh api`, as `merge_ready` does).

## Outcome
Added `make renumber` and `make janitor`, and extended the carry rule to accept a renumber.
scripts/id_renumber.py holds the one definition of a renumber: the moves, the id patterns (whole
ids only, so `0.0221` and `T-0221` never match a decision number), the PR's added lines from a
pinned `git diff -U0`, and a renumbered PR head built in a private index. `make renumber` merges
`main` into that head through `merge_catchup.catch_up`, which now takes a starting commit and
commits with `commit-tree`, so the merge's parents stay the PR head and `main`. The carry rule
derives the renames from the head, checks that the ids are fresh, rebuilds the renumbered merge
with the same function and compares trees under the existing list allowances. The janitor
records its pushes, refusals and orphan comments as `merge-janitor v1` notes on the PR, so a
later run re-marks once CI is green without blocking. merge_train_stats now escalates a stranded
PR and the latency thresholds, and flips only on a tree-check failure. No Rust or eval change, so
the eval delta is 0.
