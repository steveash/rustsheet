---
id: T-2063
title: "Aggregates read an array of references: SUM-family top-left cells, COUNT/COUNTA/SUMPRODUCT rules (SPEC-0014)"
milestone: M6
lane: engine
status: done
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0014-reference-array-results.md
depends_on: [T-1956]
---

## Goal
SUM, AVERAGE, MIN, MAX and PRODUCT read each element of an array of references as its top-left cell (a range cell), and COUNT, COUNTA and SUMPRODUCT read the elements as non-numbers, as Excel does.

## Context
Also owns probes/reference-lift-elements G12 and H12 (`SUM(OFFSET(A1,{0;1},0))` 3, `SUM(OFFSET(A1,0,0,{1;2}))` 2),
moved from T-1956, which left the aggregate walker reading a `RefArg::Grid` as its value view
(`RefArg::into_ranges_or_value`, util/aggregate_args.rs).
Design: docs/specs/SPEC-0014-reference-array-results.md rules 4 and 5, "Consumers" (the aggregate walker's `TopLeft`/`NotANumber` descriptor). Builds on T-1956's `RefGrid`. Evidence: probes/reference-lift-consumers E1-E29, E117, E125, E129, E133, E181-E201 and reference-lift-consumers-b2 E117-E121, E145, E157-E169, E181-E189, E201, E213 (excel-graph, 2026-10-19). Code: SPEC-0009's walker (functions/util.rs `evaluate_reference_arg`, `for_each_value` in functions/statistical/count_and_average.rs, functions/aggregate.rs, functions/statistical/count.rs).

## Acceptance criteria
- [x] The cells above pass and leave eval/baseline (`make eval-baseline`), with no new known failures.
- [x] Aggregates not named in SPEC-0014 rules 4-5 keep the value view (unit test).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The SPEC-0009 walker's `AggregatePolicy` gained `grids: GridReading` (`Values` | `TopLeft`). SUM, MIN/MAX,
PRODUCT and AVERAGE (its own `AVERAGE_TOP_LEFT_ARGS`) read each element of a lifted OFFSET's `RefGrid` as its
top-left cell (`Model::walk_top_left_cells`; an error element ranks as a referenced error). COUNT and COUNTA keep the
value view, which is already rule 5. SUMPRODUCT reads a grid element as 0 and an error element as its error
(`Model::sumproduct_array`, through the new `Model::evaluate_value_or_grid`). The other aggregates keep the value
view (unit test `fn_offset_lifted_into_other_aggregates_is_its_value_view`).
Eval delta: probes known failures 30 fewer, none new (reference-lift-consumers E1, E5, E21, E25, E29, E117, E125,
E129, E133, E173, E181, E189, E197, E201; -b1c E69, E73; -b1d E105; -b2 E117, E121, E137, E145, E157, E161, E165,
E181, E185, E189, E201; reference-lift-elements G12, H12). The INDIRECT cells (E9, E205, elements F12, b2 E213)
need INDIRECT as a producer (T-1957) and then pass through this walker.
