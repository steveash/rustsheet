---
id: T-0294
title: "Legacy INDEX with an array row_num gives the first element; CHOOSE with an array index in SUMPRODUCT lifts (follow-up of T-0286)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Legacy INDEX with an array row_num gives the first element, and CHOOSE with an array index inside SUMPRODUCT lifts, as Excel does.

## Context
probes/array-context-legacy-sumproduct G1, G10, T1, T10, H1, H10 (excel-graph, T-0286): `SUMPRODUCT(INDEX(B1:B6,{1;2;3}))` and `SUMPRODUCT(INDEX(B1:B6*1,{1;2;3}))` are 1 on and off the row (engine: `#N/IMPL!` in all four); `SUMPRODUCT(CHOOSE({1,2},B1:B6,B1:B6))` is 24 (engine 2 on row 1, `#VALUE!` on row 10; H1 and H10 pass since T-1508, which lifts CHOOSE over an index array, so G1, G10, T1, T10 remain). Documented in criteria.md.

## Acceptance criteria
- [x] The four INDEX cells above pass (H1, H10 already do); eval baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Legacy formulas put an automatic `@` on the array constants in INDEX's `row_num`, `column_num` and `area_num` (also
under SUMPRODUCT and inside arithmetic), as Excel shows in FORMULATEXT; the export drops it again
(`add_index_array_at` / `remove_index_array_at`, static_analysis/legacy.rs). New probe probes/legacy-index-array-index
(30 cells, excel-graph) passes except L1/L10 (`N` over an array, follow-up T-1616). array-context-legacy-sumproduct
G1, G10, T1, T10 pass. Eval delta: see the PR.
