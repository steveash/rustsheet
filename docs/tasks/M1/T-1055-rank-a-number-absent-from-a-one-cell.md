---
id: T-1055
title: "RANK: a number absent from a one-cell reference gives #N/A, not #NUM!"
milestone: M1
lane: functions-stat
status: done
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
- [x] probes/if-index-ref-args-aggregates A46-A51 pass; the baseline entries are removed.
- [x] A probe cell for a number absent from a multi-cell numeric range; semantics entry added.
- [x] `make check` green; reviewers approve.

## Out of scope
- IF/INDEX reference routing (verified correct by T-1003).

## Outcome
RANK.EQ/RANK.AVG (and RANK, which shares them) no longer give `#NUM!` for a reference with no numeric value: Excel
gives `#N/A` there, and for every other absent number (probe `rank-absent-number`: multi-cell numeric, text/boolean-only,
empty range, one-cell references; oracle excel-graph). Entry in statistics.md. Eval delta: 6 probe cells fixed
(if-index-ref-args-aggregates A46-A51), 12 new cells passing, 0 regressions.
