---
id: T-2576
title: "F.TEST, FTEST, FORECAST, INTERCEPT and COVARIANCE.P are #NUM! when a one-value argument is a number returned directly by ERROR.TYPE or CELL (gen-m9, 3 cells)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] Visible parity (gen-m9, T-2436): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Progress (2026-10-26, opus run; stopped at the session budget, branch pushed for resume)
- Probe: probes/t2576-error-type-cell-paired (excel-graph, 2026-10-26), findings in statistics.md "ERROR.TYPE and
  CELL results are #NUM! in the paired functions".
- Implemented (syntactic mark, `is_marked_number` in functions/statistical/paired.rs): ERROR.TYPE or
  `CELL("col"|"row",..)` passed straight on is `#NUM!` as F.TEST/FTEST's either sample, `fn_get_two_matrices`'s first
  argument (SUMXMY2, CORREL, SLOPE, INTERCEPT, RSQ, PEARSON, STEYX, FORECAST's known_y) after the size check, and
  `paired_samples`' first sample, plus T.TEST's second; COVARIANCE's second is one cell with no number.
- `rustsheet-eval check`: gen-m9 fixed 6 (the 3 owned cells among them), probes fixed 20, 0 regressions. The new probe
  case holds 11 known failures: LET, IF, CHOOSE, IFERROR, a LAMBDA call, INDEX and `ERROR.TYPE({#REF!})` keep the mark
  in Excel and are not followed yet (`TODO(T-2576)` in paired.rs), and `LINEST(ERROR.TYPE(#REF!))` is `#VALUE!` (E34,
  unexplained). File these as a follow-up task and point the TODO at it.
- Still to do: replace `is_marked_error`'s message match in paired.rs with a typed flag from `sample_arg`; unit tests;
  `make check`; review; PR.
