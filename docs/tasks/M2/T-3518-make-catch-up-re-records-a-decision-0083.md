---
id: T-3518
title: "make catch-up re-records a decision-0083 regression the PR's Outcome already lists"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Any session can clear a baseline-only `conflict with main` ejection on a PR whose approved diff
regresses cells under decision 0083's standing ulp allowance. That covers the cells the PR
already recorded and its reviewers quoted. Today such a PR strands every time `main` touches
the same baseline list.

## Context
- `make catch-up PR=<n>` (T-3487, decision 0207) takes main's side of the eval lists and runs
  `make eval-baseline`, which refuses any regression. Decision 0083 lets a campaign PR record
  ulp regressions with `update-baseline --allow-regressions` when four conditions hold. Its
  regressed cells are named in the task Outcome and quoted by the excel-fidelity reviewer.
- #2104 (T-2733) was ejected for `conflict with main` in `eval/baseline/gen-m9.txt` twice on
  2026-10-31, at 12:55 and 19:06 UTC. On 2026-10-31 at about 20:30 UTC, `make catch-up PR=2104`
  refused with gen-m9: 17 fixed, 1 regression `gen-m9/5ded7594e597e31e Calc!A123`. That cell is
  one of the five T-2973 cells the PR already records under 0083. The re-record needs
  `--allow-regressions`. Session permission policies have refused it three times (comments on
  #2104), so the PR waits for a run or owner that may make it.
- Code: `scripts/merge_catchup.py`, `scripts/eval_guard.sh`, the `eval-baseline` Makefile rule.

## Acceptance criteria
- [ ] `make catch-up` succeeds on a baseline-only conflict when every regression it finds is a
      cell that the PR's approved head already regressed against its own merge base. It
      records those with the allowance and still refuses any other regression.
- [ ] The rule is written in a decision record (it widens where `--allow-regressions` runs),
      and decisions 0207 and 0083 point to it. If owner approval is required, the task is
      blocked on that owner ruling.
- [ ] Unit tests in `scripts/test_merge_catchup.py` cover the accepted case and the refused
      case (a regression the approved head did not have).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing decision 0083's four conditions.
- Running catch-ups from the merger routine.
