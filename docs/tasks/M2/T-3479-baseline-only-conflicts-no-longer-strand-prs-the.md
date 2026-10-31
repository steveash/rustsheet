---
id: T-3479
title: "Baseline-only conflicts no longer strand PRs the merge train ejects"
milestone: M2
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3455]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A PR ejected for a conflict with `main` that lies only in `eval/baseline/` can be caught up and
re-marked by any session, without a new review and without its original worker.

## Context
- On 2026-10-31 trains ejected #2585, #2562, #2564, #2565, #2546 and #2532 for "conflict with
  main"; most such conflicts are in the known-failure lists. Their workers' sessions had ended,
  so they stranded. Decision 0205 has the rule.

## Acceptance criteria
- [x] `carry_problem` accepts a merge of main whose tree differs from the plain merge only in
      `eval/baseline/`, and refuses a conflict or edit anywhere else (scripts/test_merge_ready.py).
- [x] `make catch-up PR=<n>` merges main in, refuses non-baseline conflicts, takes main's side of
      the lists, runs `make eval-baseline` (refusing regressions), pushes without force and
      restores the checkout (scripts/test_merge_catchup.py, 7 tests).
- [x] SPEC-0027 "Carry-over", the steward skill's "Ejections" and decision 0205 record it.
- [x] The scripts tests pass.

## Out of scope
- Conflicts outside `eval/baseline/` (code, docs, task files): still the PR's worker's.
- Running catch-ups automatically from the merger routine (`make eval-baseline` takes minutes).

## Outcome
Usable at once by the steward and any session: `make catch-up PR=<n>`, then `make mark-ready
PR=<n> CARRY=1` after CI.
