---
id: T-1743
title: "GROWTH: match Excel on the growth-modes probe (default new_x spill, const flag, new_x shapes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every cell of probes/growth-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: GROWTH's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/growth-modes (excel-graph values). Mismatches against the engine today: 34 TypeMismatch (default new_x spills only the first cell; GROWTH(D2:D9,,,FALSE) is #REF!), 29 NumericClose, 28 NumericUlp. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

## Acceptance criteria
- [ ] probes/growth-modes baseline entries shrink to zero (or documented DEVIATIONs); `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
