---
id: T-1882
title: "SUBTOTAL over TAKE of a CHOOSE with an array index spills, static analysis says scalar (debug_assert panic)"
milestone: M2
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`=SUBTOTAL(9,TAKE(CHOOSE({1,2},H1:H2,I1:I2),2))` never panics in a debug build and gives Excel's
2x2 of `#VALUE!` (probes/lift-arrays-choose-arms A69:B70).

## Context
Found while fixing T-1874. `is_read_reference` (static_analysis/aggregate.rs) asks `yields_cells`
whether TAKE's first argument is cells; `yields_cells` takes a CHOOSE whose arms are all ranges for
cells and ignores its index. With an array index CHOOSE lifts to an array of values, so TAKE picks
values and SUBTOTAL spills, while static analysis says scalar: the
"Larger-than-1x1 array reached scalar-context cell" debug assertion in model.rs. Release builds give
`#VALUE!` in the anchor only, so the probe cells B69, A70 and B70 are known failures.

Mind the cost: T-1874's first fix ran static analysis on the CHOOSE index inside `yields_cells`, and
the performance reviewer measured exponential time in nested formulas (each level analysed its
index twice). `choose_static_result` (static_analysis.rs) now gives a CHOOSE with an array index an
`Array` shape in one pass; reuse that, or a purely syntactic check, never a second analysis of the
same subtree.

Also unprobed since T-1874: a legacy top-level `=CHOOSE({1,2},H1:H2,I1:I2)` (on and off the
range's rows) now gets the array-context `@` rather than the whole-call one, because CHOOSE with an
array index is an `Array` shape. Add those legacy cells to a probe here and match them.

`yields_cells` now serves `lifted.rs` as well as SUBTOTAL/TAKE; when this task changes it, move it
out of aggregate.rs into a neutral static_analysis submodule (architecture review of T-1874).

## Acceptance criteria
- [ ] A unit test (decision 0035) of the formula above passes in a debug build.
- [ ] probes/lift-arrays-choose-arms passes in full; the baseline only shrinks.
- [ ] The legacy top-level CHOOSE-with-array-index cells are probed and match.
- [ ] Static analysis stays linear in nesting depth (a 60-level nested test finishes at once).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IM*/EDATE/ISEVEN over CHOOSE (T-1874, done).
