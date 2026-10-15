---
id: T-1265
title: "Static analysis: CHOOSE maps Array counts into Range extents"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`StaticResult::Range` holds one convention everywhere (extents minus one, a single cell is
`Range(0, 0)`), so no caller compares a range shape off by one.

## Context
T-0296 found AGGREGATE comparing a `Range` against `(1, 1)` as if it held counts (a 2x2 range
function_num reached a scalar cell, debug-assert panic), and documented `Range` as extents minus one.
`Function::Choose` in `expressions/parser/static_analysis.rs` still turns `Array(n, m)` (counts) into
`Range(n, m)`, and `broadcast_static_result` takes the max of counts and extents together. Reviewers
of T-0296 flagged it. Audit every producer and consumer of `StaticResult::Range` (legacy-import `@`
placement at the `Range(_, _)` matches, AGGREGATE, SUBTOTAL/AREAS) and either convert CHOOSE or
switch `Range` to counts, with tests showing no export or spill change (or the evidence for one).

## Acceptance criteria
- [x] One documented convention for `StaticResult::Range`, used by every producer.
- [x] Unit tests for `AGGREGATE(CHOOSE(1,A1:B2),0,E1:E6)` and a legacy CHOOSE import keep Excel's results.
- [x] `make check` green; reviewers approve.

## Out of scope
- Tracking shapes of names and tables.

## Review finding (excel-fidelity, round 1), resolved
The convention change flipped the scalar/dynamic classification of some 1x1 shapes (`=ABS(CHOOSE(1,{5}))`,
`=AGGREGATE(CHOOSE(1,{9}),0,E1:E6)` dynamic on main, plain on the first branch head; `=ABS(E1:E1+1)` the reverse).
Probed (probes/legacy-one-cell-shape-intersection, excel-graph): Excel puts the legacy `@` on every one of these shapes,
`ABS(@E1:E1)` included, so a one-cell range is a 1x1 array, not a scalar, when broadcasting. `broadcast_static_result`
and AGGREGATE's `function_num`/options no longer skip `Range(0, 0)`, so all five formulas are dynamic (the safe direction:
the debug assert fires only when a scalar-classified formula evaluates to an array). `=SUBTOTAL(9,CHOOSE(1,{5}))` stays
dynamic as on main (a 1x1 array argument lifts to a 1x1 `#VALUE!`). The empty-`ArrayKind` case is unreachable (the parser
rejects `{}`).

## Outcome
`StaticResult::Range` holds extents (rows - 1, columns - 1) everywhere and `Array` holds counts; `StaticResult::counts`
converts. Operators and `broadcast_static_result` turn a range into its counts, a one-cell range included, and CHOOSE turns
its arms' counts back into extents (`CHOOSE(1,{1,2})` is `Range(0, 1)`, was `Range(1, 2)`). Classification changes against
main, all toward Excel's legacy `@` (probes/legacy-one-cell-shape-intersection): `=ABS(E1:E1+1)`, `=ABS(E1:E1)` and other
one-cell ranges in a broadcast argument are now dynamic. Eval: new probe case (26 cells, 23 pass; H3, M3 and P3 are legacy
`@` placements main also misses, fix T-1633); 0 regressions. Unit tests: `one_cell_shapes_are_not_scalar`,
`choose_of_a_range_function_num_spills_like_the_range`, `legacy_choose_of_ranges_is_intersected_and_exports_unchanged`.
