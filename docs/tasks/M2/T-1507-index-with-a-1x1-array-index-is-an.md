---
id: T-1507
title: "INDEX with a 1x1 array index is an array, not a reference (COUNTBLANK of it is #VALUE!)"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`COUNTBLANK(INDEX(D1:D5,{2}))` is `#VALUE!`, as in Excel: INDEX with a 1x1 array index returns a 1x1 array, not the reference the scalar index `2` picks.

## Context
probes/countblank-array-values H1 (excel-graph, 2026-10-13, T-1437) fails: the engine gives 1. `lifts` in
functions/lookup_and_reference/index.rs treats a 1x1 array index as a scalar index, so INDEX returns a reference to D2.
Probe first what else a 1x1 array index changes (ISREF, ROWS, CELL, `INDEX(..,{2}):D5`, aggregates such as SUM and
COUNTA of a blank picked cell, a reference argument of MATCH or SUMIF) before changing `lifts`: only the COUNTBLANK cell
is evidenced. See docs/excel-semantics/references.md ("COUNTBLANK of a value that is not a reference is #VALUE!") and
lookup.md ("INDEX lifts over array and range indices").

## Acceptance criteria
- [ ] probes/countblank-array-values H1 passes; any probed neighbours recorded in docs/excel-semantics/lookup.md.
- [ ] No eval regressions; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Multi-element array indices (already lift).
