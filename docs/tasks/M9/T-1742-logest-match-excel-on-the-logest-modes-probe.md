---
id: T-1742
title: "LOGEST: match Excel on the logest-modes probe (spill shape, flags, multi-column x, ULP-level numerics)"
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
Every cell of probes/logest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LOGEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/logest-modes (excel-graph values). Mismatches against the engine today: 49 TypeMismatch, 63 NumericClose, 20 NumericUlp, 1 NumericMismatch, 1 ErrorMismatch. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 8 cells, 1 numeric past half a unit in the 15th digit and 7 with a wrong type, error or text, blamed on LOGEST 8.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/logest-modes Sheet1!K141 | `=LOGEST(D2:D9,A2:A9,"x")` | #VALUE! | 1.7574908101856999 |
| probes/logest-modes Sheet1!K162 | `=LOGEST(B15,A15,TRUE,TRUE)` | 1 | #VALUE! |
| probes/logest-modes Sheet1!K169 | `=LOGEST({1,2,4,8},{1,2,3,4},TRUE,TRUE)` | 1.9999999999999998 | #REF! |
| probes/logest-modes Sheet1!K183 | `=LOGEST({1,2,4,8},{1;2;3;4})` | #REF! | 1.9999999999999993 |
| probes/logest-modes Sheet1!K190 | `=LOGEST({2,4},{1,2})` | 2 | #REF! |
| probes/logest-modes Sheet1!K197 | `=LOGEST({0.5,0.25,0.125},{1,2,3},TRUE,TRUE)` | 0.5 | #REF! |
| probes/logest-modes Sheet1!K71 | `=LOGEST(D2:D9,,TRUE,TRUE)` | 1.7574908101857 | #REF! |
| probes/logest-modes Sheet1!K85 | `=LOGEST(D2:D9,A2:C9,TRUE,TRUE)` | 0.982454941667086 | 0.9824549416670796 |

## Acceptance criteria
- [ ] Visible parity: the 8 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 8 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/logest-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong LOGEST shapes and errors, and K85 past the 15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.
