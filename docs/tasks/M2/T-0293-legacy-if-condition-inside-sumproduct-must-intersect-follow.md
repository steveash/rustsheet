---
id: T-0293
title: "Legacy IF condition must intersect (follow-up of T-0286)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In a legacy (non-dynamic) formula the condition of IF intersects, in any context and under SUMPRODUCT too, as Excel does.

## Context
probes/array-context-legacy-sumproduct F1, F10, R1, R10, S1, S10 (excel-graph, T-0286): `SUMPRODUCT(IF(B1:B6>1,B1:B6,0))` is 0 on row 1 and `#VALUE!` on row 10; `SUMPRODUCT(IF(A1:A6="a",1,0))` is 1 and `#VALUE!`; `SUM(IF(A1:A6="a",1,0))` is the same. The engine lifts them (10, 3, 3). Value branches of IF stay arrays (U1/U10 = 12). The R cells have no SUMPRODUCT, so `is_array_context` (parser/static_analysis.rs) is not the only cause: find why the legacy IF condition is not intersected (IF's argument handling or the legacy `@` insertion) and fix both paths. Documented in criteria.md.

## Acceptance criteria
- [x] The six cells above pass and U1/U10 keep passing; eval baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done in T-1526's PR, which needed the same fix for IFERROR under SUMPRODUCT: a scalar argument
now intersects arithmetic and comparisons over a range in a legacy formula, and under SUMPRODUCT
IF's condition and IFERROR's and IFNA's value still intersect (`for_each_array_context_condition`
in parser/static_analysis.rs). F1, F10, R1, R10, S1, S10 pass; U1/U10 still pass.
