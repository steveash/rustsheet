---
id: T-1072
title: "CELL address of a union, OFFSET #VALUE!, ROW #REF! (SPEC-0004 row 18)"
milestone: M1
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
CELL("address",union)="$A$1, $B$2", OFFSET(union,1,1)=#VALUE!, ROW(union)=#REF!.

## Context
Oracle (probes/multiarea-cellfn); SPEC-0004 planned #VALUE! for CELL and ROW, both differ. Probe other CELL info types in the same PR if cheap (excel-probe skill). Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [x] The three cells of multiarea-cellfn pass; other info types recorded in references.md
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).

## Outcome
`CELL("address", union)` lists each area's top-left cell joined by ", ", other info types read the first area;
`ROW`/`COLUMN` of a union are `#REF!` (via the existing `reference_ranges`). `OFFSET` already gave `#VALUE!`.
Extra info types probed in probes/multiarea-cellfn-types and recorded in references.md. Eval: probes/multiarea-cellfn
3/3 (was 1/3), multiarea-cellfn-types 11/11.
