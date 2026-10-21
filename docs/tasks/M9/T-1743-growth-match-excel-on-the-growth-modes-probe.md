---
id: T-1743
title: "GROWTH: match Excel on the growth-modes probe (default new_x spill, const flag, new_x shapes)"
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
Every cell of probes/growth-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: GROWTH's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/growth-modes (excel-graph values). Mismatches against the engine today: 34 TypeMismatch (default new_x spills only the first cell; GROWTH(D2:D9,,,FALSE) is #REF!), 29 NumericClose, 28 NumericUlp. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 0 numeric past half a unit in the 15th digit and 1 with a wrong type, error or text, blamed on TRANSPOSE 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/growth-modes Sheet1!K349 | `=GROWTH(TRANSPOSE(D2:D9),A2:A9,A2:A9)` | #REF! | 2.983473176983954 |

## Acceptance criteria
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/growth-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (GROWTH over a transposed known_y is `#REF!` in Excel), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. The last-digit numerics stay with T-2006.
