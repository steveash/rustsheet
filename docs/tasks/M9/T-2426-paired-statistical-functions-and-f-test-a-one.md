---
id: T-2426
title: "Paired statistical functions and F.TEST: a one-cell reference to an empty cell is #VALUE! (gen-m9, 109 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
COVAR, COVARIANCE.P/.S, CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, FORECAST, FORECAST.LINEAR, CHISQ.TEST, CHITEST,
T.TEST, TTEST, F.TEST and FTEST return `#VALUE!` when an array argument is a one-cell reference to an empty cell, before
their size check and before the other argument's error, as Excel does.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "A one-cell reference to an empty cell is #VALUE! in the paired
functions and F.TEST (T-2255)" records the rule and the cells; it contradicts the inference in "Array arguments given a
scalar, a single cell or an error" that an empty cell is one value with no number. 109 cells, in 23 functions'
positions (FORECAST/FORECAST.LINEAR: known_y and known_x; the others: both arrays), 31 nested in another call. A one-cell
reference holding a number, text or a boolean keeps today's one-value rule (`FTEST(6,Data!A$11)` with A11 = "x" is
`#DIV/0!`, 35e8168aae2b00f8 A253, passes and must keep passing). This answers T-2051's question (CORREL of a number cell
against an empty cell) with gen-m9 evidence; fcf11f2ec6f7a59d A9 is owned here.
Code: the shared readers `fn_get_two_matrices` (functions/math_and_trigonometry/sum.rs), `Model::paired_samples`
(functions/statistical/paired.rs), `fn_f_test` (functions/statistical/fisher.rs), the CHISQ.TEST reader (chisq.rs) and
T.TEST's (t_dist.rs). The check is on the reference itself (one cell, empty), before the values are read.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 109 known-failure lines in eval/baseline/gen-m9.txt, 109 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A257 `=COVAR(Data!B4:B4,((Data!D$9-Data!E12)-(0*2.5)))`: Excel #VALUE!, ours #DIV/0!
- gen-m9/06b4e8f5de409d5e Calc!A102 `=COVARIANCE.P(Data!B2:E9,Data!A11:A11)`: Excel #VALUE!, ours #N/A
- gen-m9/4250f880ab640c49 Calc!A258 `=GAMMA.INV(YIELDDISC(FORECAST((-1),Data!A10,Data!D11),44817,98,110,(-0.5)),(SUM(Data!E8:E9,Data!A2:A12,Data!B10:B11)-42),(ROUND(59,Data!D8)*DATE(1969,Data!D6,Data!D2)))`: Excel #NUM!, ours #DIV/0!
- gen-m9/eced4bb6a0234c9e Calc!A175 `=FORECAST.LINEAR(SUM(Data!E11:E11,Data!D11:D11,ROUNDUP(0,42)),Data!$D$8,SUM((0^Data!D8)))`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (2): Calc!A257, A387
- gen-m9/06b4e8f5de409d5e.case.json (3): Calc!A70, A102, A131
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A149
- gen-m9/165db58d1540f014.case.json (3): Calc!A74, A82, A218
- gen-m9/27601819397dd56e.case.json (1): Calc!A257
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A273
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A161
- gen-m9/2a18b2f5a4dc43ac.case.json (3): Calc!A178, A220, A321
- gen-m9/3a25a03eb94fcfbd.case.json (4): Calc!A48, A145, A263, A314
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A328
- gen-m9/4250f880ab640c49.case.json (5): Calc!A50, A111, A176, A189, A258
- gen-m9/4ab2ba98d26c8a5e.case.json (4): Calc!A90, A146, A281, A314
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A134
- gen-m9/4e55500bb8ad3cd8.case.json (3): Calc!A37, A206, A312
- gen-m9/530c636749a85209.case.json (2): Calc!A6, A321
- gen-m9/589f42e67f4b4b8d.case.json (4): Calc!A13, A327, A335, A375
- gen-m9/58e3253bbd3a7a82.case.json (4): Calc!A9, A81, A98, A337
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A302
- gen-m9/623663cacea53bbc.case.json (2): Calc!A176, A178
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A15
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A330
- gen-m9/899f9555e67da2d6.case.json (4): Calc!A127, A211, A217, A355
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A284, A322
- gen-m9/8dd2312c96effb1b.case.json (3): Calc!A82, A165, A282
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A336
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A169, A280
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A1, A335
- gen-m9/a54c738c697617c3.case.json (1): Calc!A316
- gen-m9/a5828d3925575713.case.json (1): Calc!A15
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A172, A248
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A22
- gen-m9/a790a4d4a6a82756.case.json (4): Calc!A87, A173, A260, A303
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A399
- gen-m9/b4173fe25109a238.case.json (1): Calc!A348
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A44
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A49, A84
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A16
- gen-m9/bc40d37cb4d8301d.case.json (3): Calc!A93, A160, A201
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A225
- gen-m9/c18dbf21f11093f3.case.json (5): Calc!A22, A160, A299, A317, A369
- gen-m9/c6cfda6bd10cae93.case.json (3): Calc!A23, A240, A314
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A60
- gen-m9/cde551ea787f713a.case.json (1): Calc!A180
- gen-m9/d8266bc15533702a.case.json (2): Calc!A61, A360
- gen-m9/e6cf3b20a014e6a3.case.json (4): Calc!A33, A85, A202, A338
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A175, A292
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A84
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A121, A155
- gen-m9/fcf11f2ec6f7a59d.case.json (1): Calc!A9
- gen-m9/fd9fa22d212080d6.case.json (3): Calc!A2, A212, A312
- gen-m9/fec39efaba9284c3.case.json (3): Calc!A149, A241, A255

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 109 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `COVAR(A1:A3,Z1)`, `CORREL(Z1:Z1,{1,2,3})`, `F.TEST(Z1,A1:A3)` and `INTERCEPT(Z1,1/0)` with Z1 empty
      are `#VALUE!`; `COVAR(A1:A3,B1)` with B1 = 5 is still the one-value rule.
- [ ] statistics.md's entry gets an "Ours:" line naming this task; T-2051 is closed or rescoped with a pointer here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A one-cell reference that holds a value (unchanged).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
