---
id: T-2573
title: "Probe SUMPRODUCT with a blank one-cell argument beside an error element"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's SUMPRODUCT result is evidenced when a blank one-cell argument sits beside an argument holding an error element, in both orders.

## Context
T-2450 made a blank one-cell argument after an element error win (`#VALUE!`), inferred from t2200 A20/A21, no oracle cell. Probe `SUMPRODUCT(Data!R1:S2,Data!A1)`, `SUMPRODUCT(Data!A1,Data!R1:S2)`, `SUMPRODUCT({#N/A},Data!A1)`, `SUMPRODUCT({1,#N/A;2,3},Data!X1:X2,Data!A1)` (use the excel-probe skill, extend probes/t2296-sumproduct-error-element), then fix `fn_sumproduct` and coercion.md if Excel differs.

## Acceptance criteria
- [x] Probe cells with oracle values exist and pass; coercion.md drops the INFERRED/UNPROBED note.
- [x] `make check` green; reviewers approve.

## Out of scope
None.

## Outcome
Probed A17-A20 in t2296-sumproduct-error-element with excel-graph: all `#VALUE!`, matching the T-2450 inference and the engine, so no engine change. coercion.md now cites the probe. Eval delta: +4 cells bit-exact, no new failures.
