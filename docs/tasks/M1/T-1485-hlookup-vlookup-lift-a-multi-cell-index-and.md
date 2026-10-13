---
id: T-1485
title: "HLOOKUP/VLOOKUP lift a multi-cell index and report #REF! for an IF union index (follow-up of T-1426)"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
HLOOKUP/VLOOKUP with a multi-cell range index (`H1:H2`, or `IF(TRUE,H1:H2)`) lift it like Excel instead of implicitly intersecting it, and `IF(TRUE,(G5,G6))` as the index gives `#REF!`.

## Context
Probe `probes/lookup-index-multicell-if` (excel-graph, T-1426): `COUNT(HLOOKUP(42,F5:G6,H1:H2))` is 1 in every row (A1 and A2; the engine gives 0 in row 2, where it intersects to H2), likewise B2, C2, D2; `=HLOOKUP(42,F5:G6,H1:H2)` spills 2 rows (J1:J2) and `ROWS(...IF(TRUE,H1:H2))` is 2 (O1); the union index M1 is `#REF!` (engine `#VALUE!`). See docs/excel-semantics/references.md (HLOOKUP/VLOOKUP pick).

## Acceptance criteria
- [ ] The failing cells of probes/lookup-index-multicell-if pass and are removed from eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
