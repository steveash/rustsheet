---
id: T-1741
title: "LINEST: match Excel on the linest-modes probe (spill shape, #N/A fill, collinear x, flags, 1e-16 residual stats)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Every cell of probes/linest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LINEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/linest-modes (excel-graph values). Mismatches against the engine today: 102 TypeMismatch (stats block cells empty or #N/A where Excel gives numbers, e.g. LINEST(B2:B9,A2:E9,TRUE,TRUE) collinear x gives ~1e-16 values, we give #NUM!), 38 NumericUlp, 32 NumericClose, 11 NumericMismatch, 2 ErrorMismatch (LINEST(B2:B9,A2:A9&"") is #VALUE!). Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 13 cells, 1 numeric past half a unit in the 15th digit and 12 with a wrong type, error or text, blamed on LINEST 13, TRANSPOSE 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/linest-modes Sheet1!K106 | `=LINEST(B2:B9,A2:E9,TRUE,TRUE)` | 1.4342828472961902e-16 | #NUM! |
| probes/linest-modes Sheet1!K113 | `=LINEST(B2:B9,A2:A9&"")` | #VALUE! | #NUM! |
| probes/linest-modes Sheet1!K134 | `=LINEST(A12:A13,B12:B13,TRUE,TRUE)` | 0.5 | 0.49999999999999867 |
| probes/linest-modes Sheet1!K141 | `=LINEST(B15,A15,TRUE,TRUE)` | 0 | #VALUE! |
| probes/linest-modes Sheet1!K155 | `=LINEST(B2:B9,A2:A9,"x")` | #VALUE! | 1.9976190476190487 |
| probes/linest-modes Sheet1!K162 | `=LINEST(B2:B9,A2:A9,TRUE,"x")` | #VALUE! | 1.9976190476190487 |
| probes/linest-modes Sheet1!K169 | `=LINEST(TRANSPOSE(B2:B9),TRANSPOSE(A2:A9),TRUE,TRUE)` | 1.9976190476190474 | #REF! |
| probes/linest-modes Sheet1!K204 | `=LINEST(5,A2:A9)` | 0 | #VALUE! |
| probes/linest-modes Sheet1!K218 | `=LINEST({1,2,3,5},{1,2,3,4},TRUE,TRUE)` | 1.3 | #REF! |
| probes/linest-modes Sheet1!K232 | `=LINEST({1,2,3,5},{1;2;3;4})` | #REF! | 1.2999999999999998 |
| probes/linest-modes Sheet1!K239 | `=LINEST({2,4},{1,2})` | 2 | #REF! |
| probes/linest-modes Sheet1!K246 | `=LINEST({2,4,6},{1,2,3},TRUE,TRUE)` | 2 | #REF! |
| probes/linest-modes Sheet1!K71 | `=LINEST(B2:B9,,TRUE,TRUE)` | 1.9976190476190474 | #REF! |

## Acceptance criteria
- [ ] Visible parity: the 13 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 13 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/linest-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong LINEST shapes and errors, and K134 past the 15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.
