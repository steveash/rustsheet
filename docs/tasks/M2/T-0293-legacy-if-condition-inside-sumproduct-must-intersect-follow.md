---
id: T-0293
title: "Legacy IF condition inside SUMPRODUCT must intersect (follow-up of T-0286)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In a legacy (non-dynamic) formula the condition of IF intersects even under SUMPRODUCT, as Excel does.

## Context
probes/array-context-legacy-sumproduct F1, F10, R1, R10, S1, S10 (excel-graph, T-0286): `SUMPRODUCT(IF(B1:B6>1,B1:B6,0))` is 0 on row 1 and `#VALUE!` on row 10; `SUMPRODUCT(IF(A1:A6="a",1,0))` is 1 and `#VALUE!`; `SUM(IF(A1:A6="a",1,0))` is the same. The engine lifts them (10, 3, 3). Value branches of IF stay arrays (U1/U10 = 12). Code: `is_array_context` in parser/static_analysis.rs and the legacy `@` insertion. Documented in criteria.md.

## Acceptance criteria
- [ ] The six cells above pass and U1/U10 keep passing; eval baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
