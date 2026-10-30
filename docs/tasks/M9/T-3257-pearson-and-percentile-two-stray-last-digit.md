---
id: T-3257
title: "PEARSON and PERCENTILE: two stray last-digit gen-m9 cells no task names (PERCENTILE(Data!A9:D11,1E-16), T-1811's known miss)"
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
gen-m9/5395b8a427764a3f Calc!A269 (PERCENTILE) and gen-m9/201d67f1d59436ed Calc!A177 (PEARSON), both 1 ulp off and
within 15 significant digits of Excel, match Excel to the bit, or the Outcome records the forms tried with their
bit-exact counts over every PERCENTILE/PERCENTILE.INC/QUARTILE oracle cell and every PEARSON/CORREL/RSQ oracle cell
(decision 0056 time box).

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch) found these two
  cells named by no open task. Neither is visibly off, so this is Phase 1b work and stays deferred.
- PERCENTILE A269 `=PERCENTILE(Data!A9:D11,1E-16)`: Excel -2.499999999999999, ours -2.4999999999999987. The numbers
  are -2.5, 1E-15, 0.1, 0.2, 100 and 123456789.123 (n = 6; the text '1' and the booleans drop out). It is the one known
  miss of statistics.md "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1)" (T-1811, done):
  the rule gives ours, and `(1-h)*a + h*b` gives Excel's value but misses 4 other cells. So the interpolation form is
  not pinned where h comes from rounding `1 + k(n-1)`: h here is fl(1 + 5E-16) - 1 = 2^-51. Score the forms (`a + h*(b-a)`,
  `(1-h)*a + h*b`, `a*(1-h) + b*h`, with or without an extended product) over every PERCENTILE, PERCENTILE.INC,
  QUARTILE and QUARTILE.INC oracle cell, not only this one.
- PEARSON A177 `=PEARSON(Data!B11:D12,{1,1,2,2,2,3})`: Excel -0.4999999999999999, ours -0.49999999999999994. The first
  array holds 1, "?b" and an empty or formula cell in row 11 and -0.5, 1 and another in row 12, so the pairs with a
  number on both sides are few; check which pairs Excel keeps (`make eval-show` and the case's Data sheet) before
  fitting. The engine's form is statistics.md "Paired regression functions" (T-1723: covariance and standard
  deviations over n).
- Code: `crates/rustsheet-engine/src/functions/statistical/percentile.rs` (`percentile_inc_impl`), and the paired
  regression helpers PEARSON shares with CORREL and RSQ.

Owned cells:
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A269
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A177

## Acceptance criteria
- [ ] Both known-failure lines leave eval/baseline/gen-m9.txt (`make eval-baseline`) with no new line in any baseline,
      or the Outcome lists each form tried with its bit-exact count over the oracle cells named above.
- [ ] docs/excel-semantics/statistics.md's PERCENTILE section drops or updates its "known miss" note, citing A269.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other PERCENTILE or PEARSON semantics (argument kinds, errors): the probe tasks T-2605, T-2051, T-2080.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
