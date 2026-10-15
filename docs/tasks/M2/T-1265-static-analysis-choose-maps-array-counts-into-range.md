---
id: T-1265
title: "Static analysis: CHOOSE maps Array counts into Range extents"
milestone: M2
lane: engine
status: in-progress
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

## Open review finding (excel-fidelity, round 1, REQUEST_CHANGES)
The branch's convention change flips the scalar/dynamic classification (`Model::set_cell_with_formula`, xlsx `cm="1"
t="array"`) for some 1x1 shapes, with cell values unchanged:
- `=ABS(CHOOSE(1,{5}))`, `=SUBTOTAL(9,CHOOSE(1,{5}))`, `=AGGREGATE(CHOOSE(1,{9}),0,E1:E6)`: dynamic on main, plain here
  (a 1x1 array arm now gives `Range(0, 0)`, which broadcasting and AGGREGATE read as a single cell);
- `=ABS(E1:E1+1)`, `=CHOOSE(1,E1:E1+1)`: plain on main, dynamic here (an operator over a single cell now gives
  `Array(1, 1)`, was `Array(0, 0)`);
- an empty `ArrayKind` is now non-scalar in `broadcast_static_result` (probably unreachable).
Next step: probe Excel's saved `cm` flag for these five formulas (excel-probe) and match it, or keep main's
classification for these shapes; then pin it with export/`is_dynamic` tests (the AGGREGATE test added here passes on
main too). Until then the Outcome below overstates "no export change".

## Outcome
`StaticResult::Range` holds extents (rows - 1, columns - 1) everywhere and `Array` holds counts; `StaticResult::counts`
converts. Operators and `broadcast_static_result` now turn a range into its counts (a single cell still reads as a scalar
when broadcasting), and CHOOSE turns its arms' counts back into extents (`CHOOSE(1,{1,2})` is `Range(0, 1)`, was
`Range(1, 2)`). Only scalar-versus-not is read outside the parser (`Model::set_cell_with_formula`), and the legacy `@`
placement matches on `Range(_, _)`, so no export or spill result changes: eval check is unchanged (0 fixed, 0 regressions
in every gated suite). New unit tests: `choose_of_a_range_function_num_spills_like_the_range` (AGGREGATE over a 2x2 and a
1x1 CHOOSE arm) and `legacy_choose_of_ranges_is_intersected_and_exports_unchanged`.
