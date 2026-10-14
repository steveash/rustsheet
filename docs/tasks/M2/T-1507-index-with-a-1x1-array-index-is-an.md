---
id: T-1507
title: "INDEX with a 1x1 array index is an array, not a reference (COUNTBLANK of it is #VALUE!)"
milestone: M2
lane: functions-lookup
status: done
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
- [x] probes/countblank-array-values H1 passes; any probed neighbours recorded in docs/excel-semantics/lookup.md.
- [x] No eval regressions; `make eval-baseline` records the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- Multi-element array indices (already lift).

## Outcome
A 1x1 array index now lifts INDEX like any other array index (`lifts` in index.rs), so
`INDEX(D1:D5,{2})` is a 1x1 array value: probes/countblank-array-values H1 passes (`#VALUE!`).
New probe probes/index-1x1-array-index (29 cells, excel-graph) evidences the neighbours: ISREF FALSE,
TYPE 64, ROWS 1, AREAS/range operator/OFFSET/CELL/SUMIF/COUNTIF `#VALUE!`, COUNTA 1 over the blank
element, the same for `{n}` as column_num, area_num, `{2}+0` and `SEQUENCE(1)+1`; 28 of 29 pass. The
one left, `N(INDEX(D1:D5,{1}))` = 10 (engine `#N/IMPL!`, N of any array is unimplemented), is
follow-up T-1557. Probes known failures 5617 -> 5617 (fixed 1, new case with 1 failure); poi,
ironcalc-calc and the other suites unchanged. Recorded in lookup.md "INDEX lifts over array and range indices".
