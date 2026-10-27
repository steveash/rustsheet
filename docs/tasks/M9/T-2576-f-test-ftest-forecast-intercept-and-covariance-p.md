---
id: T-2576
title: "F.TEST, FTEST, FORECAST, INTERCEPT and COVARIANCE.P are #NUM! when a one-value argument is a number returned directly by ERROR.TYPE or CELL (gen-m9, 3 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
F.TEST, FTEST, FORECAST, INTERCEPT and COVARIANCE.P return `#NUM!` when a one-value argument is a number that ERROR.TYPE or CELL returned directly, as Excel does, and stay as today for every other one value.

## Context
From T-2436's probe of the untraced gen-m9 cells. statistics.md "ERROR.TYPE and CELL results are #NUM! in the paired functions (T-2436)". Why Excel does this is unknown (the number is plain once `+0` or N() touches it), so first probe which other functions' results do the same (INFO, ISREF-style, SHEET, ROW of a range, TYPE, ERROR.TYPE inside LET/IF/CHOOSE, as an array element) and which other paired functions (COVARIANCE.S, COVAR, CORREL, SLOPE, RSQ, T.TEST, CHITEST) react; then mark such values where the evaluator returns them. Code: the paired readers in functions/statistical/, fisher.rs; the evaluator's value for ERROR.TYPE/CELL.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): a number returned directly by ERROR.TYPE or CELL makes these functions `#NUM!` where any other one value gives `#DIV/0!` (or 0 for COVARIANCE.P): `FTEST({1;2;3},ERROR.TYPE(#REF!))`, `FTEST({1;2;3},CELL("col",B12))`, `FORECAST(1,ERROR.TYPE(#REF!),5)`, `INTERCEPT(ERROR.TYPE(#REF!),5)`, `COVARIANCE.P(ERROR.TYPE(#REF!),5)` are `#NUM!` while `FTEST({1;2;3},ERROR.TYPE(#REF!)+0)`, `N(ERROR.TYPE(..))`, `CELL(..)+0`, `SUM(4)`, `ROW(B4)` and `INDIRECT("B1")` are `#DIV/0!` (probes/t2436/computed-scalar A1:A29); in the sweep `FTEST(Data!C3:C9,ERROR.TYPE(#REF!))`, `F.TEST(CELL("col",Data!$B$12),..)` and `FORECAST(Data!A10,ERROR.TYPE(..),STDEV.P(..))` (ours `#DIV/0!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/4250f880ab640c49 Calc!A325 `=FTEST(Data!C3:C9,ERROR.TYPE(#REF!))`: Excel #NUM!, ours #DIV/0!
- gen-m9/623663cacea53bbc Calc!A400 `=F.TEST(CELL("col",Data!$B$12),(PI()+FILTER(Data!B1:B12,(Data!D1:D12>1.005),Data!E7)))`: Excel #NUM!, ours #DIV/0!
- gen-m9/bcb24385baafb2f0 Calc!A252 `=FORECAST(Data!A10,ERROR.TYPE((0.5-Data!A1)),STDEV.P(Data!D8:D12))`: Excel #NUM!, ours #DIV/0!
Cells:
- gen-m9/4250f880ab640c49.case.json (1): Calc!A325
- gen-m9/623663cacea53bbc.case.json (1): Calc!A400
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A252

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
- Probe probes/t2576-error-type-cell-paired (excel-graph, 2026-10-26, 68 cells), findings in statistics.md
  "ERROR.TYPE and CELL results are #NUM! in the paired functions". Covered: which values carry the mark, what keeps or
  drops it, and which functions react, in which argument.
- `is_marked_number` (functions/statistical/paired.rs) marks a number that ERROR.TYPE or `CELL("col"|"row",..)` returns
  straight to the argument, parentheses aside. Where it applies:
  - `#NUM!` as either F.TEST/FTEST sample.
  - `#NUM!` as `fn_get_two_matrices`' first argument, after its size check: CORREL, PEARSON, SLOPE, INTERCEPT, RSQ,
    STEYX and FORECAST's known_y.
  - `#NUM!` as the first sample of COVARIANCE.P, COVARIANCE.S, COVAR and T.TEST, and T.TEST's second.
  - A cell with no number as COVARIANCE's second sample.
- Eval delta (`rustsheet-eval check`, 0 regressions):
  - gen-m9 fixed 6, the 3 owned cells among them.
  - probes fixed 20 (probes/t2436/computed-scalar among them).
  - The new probe case holds 11 known failures, owned by follow-up T-2766: the LET, IF, CHOOSE, IFERROR, LAMBDA and
    INDEX forms, `ERROR.TYPE({#REF!})`, SUMXMY2, and LINEST with its spill.
