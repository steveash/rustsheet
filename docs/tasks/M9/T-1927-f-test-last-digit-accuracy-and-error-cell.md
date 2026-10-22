---
id: T-1927
title: "F.TEST last-digit accuracy and error-cell paired size order"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
F.TEST/FTEST match Excel to the last digit, and the order of the size check against a range holding an error cell is probed.

## Context
After T-1292, probes/stat-argument-modes H248 and H251 (`F.TEST(C1:C6,B1:B8)`) are 2 ulps off (0.9195979047175997 vs 0.9195979047175995), and H246/H247/H249/H250 are NumericClose; the F cdf path is in functions/statistical/fisher.rs. Separately, `fn_get_two_matrices` checks sizes before reading cells, so CORREL of an error-holding range against a range of another size is `#N/A`; no oracle evidence decides it (use the excel-probe skill with an equal-size and an unequal-size pair where one side holds `#DIV/0!`).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 4 cells, 4 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on FTEST 4.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes-numeric/m9-legacy-ftest Calc!A11 | `=FTEST(B1:C5,B1:B5)` | 0.9873866645220285 | 0.9873866645220292 |
| probes/stat-argument-modes Sheet1!H249 | `=FTEST(A1:A8,B1:B8)` | 0.7947858780819458 | 0.7947858780819467 |
| probes/stat-argument-modes Sheet1!H250 | `=FTEST({1,2,4,3,7,5,8,6},{2,3,5,4,9,6,9,8})` | 0.7947858780819458 | 0.7947858780819467 |
| probes/stat-argument-modes Sheet1!H251 | `=FTEST(C1:C6,B1:B8)` | 0.9195979047175995 | 0.9195979047175988 |

## Acceptance criteria
- [ ] Visible parity: the 4 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 3 lines in eval/baseline/probes.txt disappear (`make eval-baseline`); the 1 cell of the ungated probes-numeric suite passes in `make eval`, with no new known-failure lines in any baseline.
- [ ] H248 and H251 pass exactly (or the ulp tolerance is justified in docs/excel-semantics/statistics.md); the error-cell order is probed and recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (FTEST values that differ in the 15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).

## Handoff (resumed)
Tried (sonnet, 2026-10-22): an exact tail for whole-number df, `I_x(a,b)` for a,b in {1/2,1,3/2,..} by
stepping up from the closed forms at a,b in {1/2,1} with `I_x(a+1,b)=I_x(a,b)-x^a y^b/(a B)` and
`I_x(a,b+1)=I_x(a,b)+x^a y^b/(b B)`, each F tail its own evaluation, used only in `fn_f_test`
(fisher.rs), then `2*min(lo, sf)`.
Result against mpmath (40 digits, f taken as the double our variance gives):
- A11 (df 9,4) and H246/H247/H249/H250 (df 7,7): Excel is the correctly rounded exact value
  (...0285, ...9458); the recurrence gets 1 ulp off (...0286, ...9459), was 6-9 ulp off.
- H248/H251 (df 1,7, f=0.6116504854368933): exact is 0.91959790471759887, Excel is ...5995, ours
  ...5987 now (was ...5988): Excel is itself ~7e-16 off the exact value here, so no exact method hits it.
- ironcalc-calc/STATISTICAL/F_TEST.xlsx F1 (df 3,3, f=0.4266211604095564): exact is
  0.5023992107800743, Excel 0.5023992107800741, the old code matched bit-exactly, the recurrence gives ...743:
  a REGRESSION (make eval-baseline refuses it).
So Excel's F.TEST is not correctly rounded and is not the exact value; a bit-exact fit needs
Excel's own algorithm (probably the same one as F.DIST/BETA.DIST, see the fit notes in
incomplete_beta.rs and tools fitlab kernels/beta.py) plus possibly a different `f` (variance
formula) or `2*min` ordering. Next: try the fitlab with these 7 F.TEST cells (and variants of the
variance: two-pass vs sum-of-squares, p = 1 - |1 - 2*lo|, 2*min) before touching the beta kernel;
the error-cell order probe (Context) is untouched. Visible parity (15 digits) is met by the
recurrence for A11/H249/H250 but not asked to trade a bit-exact cell for it.
