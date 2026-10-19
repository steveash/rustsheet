---
id: T-1742
title: "LOGEST: match Excel on the logest-modes probe (spill shape, flags, multi-column x, ULP-level numerics)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every cell of probes/logest-modes that is not a bit-exact match for Excel passes, or is a recorded DEVIATION: LOGEST's result shape, error cases and numerics equal Excel's.

## Context
T-1621 added probes/logest-modes (excel-graph values). Mismatches against the engine today: 49 TypeMismatch, 63 NumericClose, 20 NumericUlp, 1 NumericMismatch, 1 ErrorMismatch. Numeric differences are mostly last-digit: Excel's regression algorithm (QR) differs from ours, so find the evaluation order that reproduces the oracle bit-exactly. See docs/excel-semantics/statistics.md "LINEST, LOGEST, GROWTH: probe findings".

## Acceptance criteria
- [ ] probes/logest-modes baseline entries shrink to zero (or documented DEVIATIONs), apart from the last-digit cells T-2006 owns; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes outside this function, and the FORECAST.ETS family (T-1553).
- Last-digit (ulp/close) differences from the shared least-squares solve: T-2006 fits that kernel for LINEST, LOGEST,
  TREND and GROWTH together (gap triage 2026-10-19).
