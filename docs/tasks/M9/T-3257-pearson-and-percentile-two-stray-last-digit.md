---
id: T-3257
title: "PEARSON: a stray last-digit gen-m9 cell no task names (PEARSON(Data!B11:D12,{1,1,2,2,2,3}))"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
gen-m9/201d67f1d59436ed Calc!A177 (PEARSON), 1 ulp off and within 15 significant digits of Excel, matches Excel to
the bit, or the Outcome records the forms tried with their bit-exact counts over every PEARSON/CORREL/RSQ oracle cell
(decision 0056 time box).

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch) found these two
  cells named by no open task. Neither is visibly off, so this is Phase 1b work and stays deferred.
- PERCENTILE gen-m9/5395b8a427764a3f A269, found in the same triage, is T-3291's and T-3292's (the PERCENTILE
  interpolation probe and fit), so it is not listed here.
- PEARSON A177 `=PEARSON(Data!B11:D12,{1,1,2,2,2,3})`: Excel -0.4999999999999999, ours -0.49999999999999994. The first
  array holds 1, "?b" and an empty or formula cell in row 11 and -0.5, 1 and another in row 12, so the pairs with a
  number on both sides are few; check which pairs Excel keeps (`make eval-show` and the case's Data sheet) before
  fitting. The engine's form is statistics.md "Paired regression functions" (T-1723: covariance and standard
  deviations over n).
- Code: the paired regression helpers PEARSON shares with CORREL and RSQ.

Owned cells:
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A177

## Acceptance criteria
- [ ] The known-failure line leaves eval/baseline/gen-m9.txt (`make eval-baseline`) with no new line in any baseline,
      or the Outcome lists each form tried with its bit-exact count over the oracle cells named above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other PEARSON semantics (argument kinds, errors): the probe tasks T-2605, T-2051, T-2080.
- PERCENTILE A269: T-3291 and T-3292.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
