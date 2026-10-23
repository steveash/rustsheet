---
id: T-2430
title: "An error element inside an array argument of LARGE, SMALL, QUARTILE, COVARIANCE and the paired functions is that error, not #VALUE! (gen-m9, 19 cells)"
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
An error element inside an array argument of LARGE, SMALL, QUARTILE.EXC, PERCENTRANK.INC, PERCENTRANK.EXC, MODE.MULT,
VARPA, COVAR, COVARIANCE.S and CORREL is that error, not `#VALUE!`, as Excel does.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "An error element of an array argument is that error (T-2255)"
records the rule (`LARGE({1,#N/A,3},3)` `#N/A`, `COVARIANCE.S({1,#N/A,3},{-1,0,1})` `#N/A`); it answers the
"Unprobed: an error inside an array constant" note of "Paired functions: a size mismatch is #N/A before an error cell"
for equal sizes. 19 cells, 11 nested (the enclosing function passes the error on). F.TEST's own cells are T-2093's.
Two cells need care: `LINEST(..,(MAP({1,#N/A,3},..)+..),FALSE,..)` is `#REF!` in Excel (LINEST's size check before the
error, T-1741), and `FREQUENCY(0.1,VARPA(ACOTH(..),..,{1,#N/A,3}))` is `#NUM!` (VARPA's first argument's error first).
Code: the array readers of these functions (functions/statistical/: `values_from_array` callers, paired.rs,
`fn_get_two_matrices`).

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 19 known-failure lines in eval/baseline/gen-m9.txt, 19 formulas.
Examples:
- gen-m9/165db58d1540f014 Calc!A286 `=LARGE({1,#N/A,3},3)`: Excel #N/A, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A285 `=COVAR({1,#N/A,3},COVARIANCE.S((0-1),0))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/85f48d5cdfb24555 Calc!A351 `=LINEST(Data!C6:E11,(MAP({1,#N/A,3},Data!A5:C12,Data!A3:D11,LAMBDA(w,y,v,v))+(0.1-Data!$E$10)),FALSE,Data!$D8)`: Excel #REF!, ours #VALUE!
- gen-m9/cde551ea787f713a Calc!A351 `=FREQUENCY(0.1,VARPA(ACOTH(Data!E9),SUMX2PY2(Data!C1,Data!B7),{1,#N/A,3}))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/165db58d1540f014.case.json (1): Calc!A286
- gen-m9/27601819397dd56e.case.json (2): Calc!A175, A285
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A139, A282
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A53
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A315
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A351
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A268
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A50
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A27
- gen-m9/a54c738c697617c3.case.json (1): Calc!A352
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A135
- gen-m9/b4173fe25109a238.case.json (1): Calc!A84
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A97
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A49
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A265
- gen-m9/cde551ea787f713a.case.json (1): Calc!A351
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A300

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 19 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `LARGE({1,#N/A,3},1)`, `SMALL({1,#N/A,3},3)`, `COVARIANCE.S({1,#N/A,3},{1,2,3})` are `#N/A`.
- [ ] statistics.md's entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.TEST/FTEST (T-2093).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
