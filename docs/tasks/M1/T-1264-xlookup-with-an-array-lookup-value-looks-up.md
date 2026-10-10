---
id: T-1264
title: "XLOOKUP with an array lookup_value looks up each element"
milestone: M1
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XLOOKUP with an array `lookup_value` (`SUM(XLOOKUP({1;2},A1:A3,B1:B3))`) returns one result per element, as Excel does.

## Context
probes/let-name-uncalled-picked-refs Sheet1!H21: Excel gives 30, the engine `#N/A` (`xlookup_impl`, functions/xlookup.rs, evaluates lookup_value to a single value). Probe the shapes first (row/column/2-D lookup_value, with if_not_found and the reference form) with the excel-probe skill, and record them in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] H21 passes and its baseline entry is removed; probed shapes are recorded in docs/excel-semantics/lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
XLOOKUP lifts an array or multi-cell `lookup_value` per element through `lift_with_cost` (functions/xlookup.rs); a wide return row keeps its first cell per element. New probe probes/xlookup-array-lookup-value (27 cells, all pass); let-name-uncalled-picked-refs H21 now passes. Shapes recorded in docs/excel-semantics/lookup.md.
