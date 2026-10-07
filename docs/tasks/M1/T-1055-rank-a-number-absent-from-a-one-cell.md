---
id: T-1055
title: "RANK: a number absent from a one-cell reference gives #N/A, not #NUM!"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`RANK(number, ref)` returns `#N/A` when `number` is not among the numeric values of `ref`, as Excel does.

## Context
T-1003's probe `probes/if-index-ref-args-aggregates` A46-A51 (`RANK(3,Data!A1)` and the IF/INDEX forms over a text or boolean cell, no numeric value) expect `#N/A` from Excel (excel-graph, 2026-10-07); the engine gives `#NUM!`. Check the multi-cell case too (a number absent from a numeric range) with a probe before changing the engine.

## Acceptance criteria
- [ ] probes/if-index-ref-args-aggregates A46-A51 pass; the baseline entries are removed.
- [ ] A probe cell for a number absent from a multi-cell numeric range; semantics entry added.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
