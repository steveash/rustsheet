---
id: T-0976
title: "FISHERINV: use (EXP(2y)-1)/(EXP(2y)+1) like Excel (6 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FISHERINV returns Excel's values bit for bit, including its loss of precision near 0.

## Context
Triage T-0903. `functions/statistical/fisher.rs` `fn_fisher_inv` returns `y.tanh()`. Excel evaluates
`(exp(2y) - 1) / (exp(2y) + 1)` in doubles, which reproduces every numeric FISHERINV cell in both
workbooks (32 cells, incl. the 6 failing ones), and is 1 where exp(2y) overflows
(`docs/excel-semantics/statistics.md`, "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1)", Status VERIFIED).
6 baseline lines in `eval/baseline/ironcalc-calc.txt`:
- `ironcalc-calc/STATISTICAL/FISHER.xlsx` `Sheet1`: G2 (expected 0.30000000000000004, actual 0.3000000000000001), G3 (expected 0.4, actual 0.39999999999999997)
- `ironcalc-calc/STATISTICAL/FISHER_FISHERINV.xlsx` `FISHER FISHERINV`: C10 (expected -0.9000000000000001, actual -0.9),
  C40 `FISHERINV(1E-7)` (NumericClose: expected 1.0000000005039292e-07, actual 9.999999999999965e-08),
  C41 `FISHERINV(-1E-7)` (NumericClose: expected -1.0000000001086735e-07, actual -9.999999999999965e-08),
  C51 (expected 0.30000000000000004, actual 0.3000000000000001)
Must keep passing: C38 `FISHERINV(1000)` = 1 and C39 `FISHERINV(1E10)` = 1 (exp overflows to inf, so
inf/inf would be NaN: return 1 when exp(2y) is infinite), C44/C45 booleans, the error cells.

## Acceptance criteria
- [ ] The 6 lines above leave `eval/baseline/ironcalc-calc.txt` (`make eval-baseline`), no new lines in any baseline.
- [ ] Unit tests: FISHERINV(1E-7) = 1.0000000005039292e-07, FISHERINV(1000) = 1, FISHERINV(-1000) = -1,
      FISHERINV(0) = 0.
- [ ] The code comment cites the statistics.md entry; the entry gets `Implemented in:`.
- [ ] `make check` green; reviewers approve.

## Out of scope
FISHER (passes); other hyperbolic functions (T-0917).
