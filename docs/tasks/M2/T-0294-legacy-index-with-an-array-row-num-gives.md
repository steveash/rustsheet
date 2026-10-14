---
id: T-0294
title: "Legacy INDEX with an array row_num gives the first element; CHOOSE with an array index in SUMPRODUCT lifts (follow-up of T-0286)"
milestone: M2
lane: engine
status: todo
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
- [ ] The four INDEX cells above pass (H1, H10 already do); eval baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
