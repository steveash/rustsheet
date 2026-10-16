---
id: T-1741
title: "LINEST: match Excel on the linest-modes probe (spill shape, #N/A fill, collinear x, flags, 1e-16 residual stats)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every cell of probes/linest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LINEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/linest-modes (excel-graph values). Mismatches against the engine today: 102 TypeMismatch (stats block cells empty or #N/A where Excel gives numbers, e.g. LINEST(B2:B9,A2:E9,TRUE,TRUE) collinear x gives ~1e-16 values, we give #NUM!), 38 NumericUlp, 32 NumericClose, 11 NumericMismatch, 2 ErrorMismatch (LINEST(B2:B9,A2:A9&"") is #VALUE!). Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

## Acceptance criteria
- [ ] probes/linest-modes baseline entries shrink to zero (or documented DEVIATIONs); `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
