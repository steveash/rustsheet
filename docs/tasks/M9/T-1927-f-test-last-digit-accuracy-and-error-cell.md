---
id: T-1927
title: "F.TEST last-digit accuracy and error-cell paired size order"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
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
