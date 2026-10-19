---
id: T-1987
title: "SUBTOTAL/AREAS over TAKE of a LAMBDA call or an XLOOKUP whose result lifts reaches the scalar-context assertion"
milestone: M2
lane: engine
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Static analysis stops taking a TAKE/DROP over a LAMBDA call or an XLOOKUP whose result lifts for cells, so these formulas spill at run time instead of tripping the debug scalar-context assertion (model.rs:928).

## Context
Found by the T-1914 Excel-fidelity review; each form already panics on main (not a T-1914
regression), the same deadly-signal family as T-1874/T-1882/T-1914 (decision 0035). H1:H2 = 3,4;
I1:I2 = 6,8. In a debug build these reach the assertion:
- `SUBTOTAL(9,TAKE(LAMBDA(s,SWITCH(s,1,H1:H2,2,I1:I2))({1,2}),2))`, and
  `LET(f,LAMBDA(s,SWITCH(s,1,H1:H2,2,I1:I2)),SUBTOTAL(9,TAKE(f({1,2}),2)))`;
- `SUBTOTAL(9,TAKE(LAMBDA(s,CHOOSE(s,H1:H2,I1:I2))({1,2}),2))`; the same under `CHOOSE(1,...)` and through AREAS;
- `SUBTOTAL(9,TAKE(XLOOKUP(1,{1,2},CHOOSE({1,2},H1:H2,I1:I2)),2))` and its AREAS form: the XLOOKUP
  branch of `is_read_reference` only rejects a literal array `return_array`, not one that lifts.
Code: `is_read_reference` and `lifts_to_values` in expressions/parser/static_analysis/aggregate.rs,
`yields_cells` in static_analysis/reference_result.rs. A LAMBDA call can count as `Unknown`
(lifting) unless its body provably yields cells; XLOOKUP's return array can go through
`lifts_to_values`. Keep the analysis linear in nesting depth.

## Acceptance criteria
- [ ] Debug unit tests (decision 0035) of every formula above evaluate without the assertion.
- [ ] A deep nested test stays fast (linear).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
