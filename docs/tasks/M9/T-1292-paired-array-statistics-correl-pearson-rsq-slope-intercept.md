---
id: T-1292
title: "Paired-array statistics (CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, CHISQ.TEST) of unequal sizes with text are #N/A; F.TEST tail"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Paired-array statistics give Excel's error when the arrays differ in size and one holds text: `#N/A`, not `#VALUE!`.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): `CORREL(C1:C6,B1:B8)` (6 cells with text, booleans and a blank against 8 numbers) is `#N/A` in Excel for CORREL,
PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, CHISQ.TEST and CHITEST; ours is `#VALUE!`. COVAR, COVARIANCE.P and COVARIANCE.S
already match (`#N/A`). F.TEST and T.TEST take unequal sizes and skip the text: T.TEST matches within a few ulps, but
`F.TEST(C1:C6,B1:B8)` (the numbers 1 and 4 against eight) is 0.9195979047175995 in Excel and 1.0804020952824007 in the
engine, which is 2 minus Excel's value, above 1 for a p-value: the two-tailed doubling takes the wrong tail when the first
sample's variance is the smaller (H248, H251; FTEST too). Probe the error
order (size check before or after the text check) with equal sizes too before implementing.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/95efb37a9874f82e Calc!A21 `=INTERCEPT(Data!C5:E6,Data!D8:E9)` (6 cells with a boolean and a blank against 4 with FALSE and " x "): Excel `#N/A`, engine `#VALUE!`.

## Acceptance criteria
- [ ] The eight `#N/A` cells and the F.TEST/FTEST cells (H248, H251) of probes/stat-argument-modes pass.
- [ ] The rule is in docs/excel-semantics/statistics.md.
- [ ] gen-m9/95efb37a9874f82e A21 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Unequal-size pairs in `fn_get_two_matrices` are `#N/A` (CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, CHISQ.TEST, CHITEST, FORECAST); F.TEST takes twice the smaller tail. Rule in statistics.md. Eval: 62 known failures cleared by `make eval-baseline` (probes/stat-argument-modes 8, regression-pairs-a 8, regression-pairs-b 4, regression-forecast 4, plus 38 groupby cells that already passed on this base).
