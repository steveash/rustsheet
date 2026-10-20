---
id: T-1956
title: "OFFSET lifts rows, cols, height and width into an array of references"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0014-reference-array-results.md
depends_on: [T-1955]
---

## Goal
OFFSET's rows, cols, height and width lift over a multi-cell range or an array into an array of references, as Excel does, on the carrier of the T-1955 spec.

## Context
Design: docs/specs/SPEC-0014-reference-array-results.md (the `RefGrid` carrier, its value view, `CallResult::Reference`, and the INDEX, ROWS/COLUMNS, ISREF and LET/IF/CHOOSE consumers: Rollout step 1). Split from T-1810. Evidence: docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19). Failing cells: probes/lookup-argument-modes F20:H21; probes/reference-lift-elements F1:H5, H11, G12, H12, F15:H15 and the spills K1, K4, K13 (and their members); gen-m7/cb748b6a9b81fbb7 Calc!A28 `=OFFSET(..,OFFSET(Data!D6:D9,3,(Data!D$4+7)))` (Excel `#SPILL!`, the lift winning over the `#N/A` scalar argument; engine `#N/A`). `rustsheet-eval show probes/reference-lift-elements.case.json` lists them.

## Acceptance criteria
- [x] The OFFSET cells above pass except reference-lift-elements G12, H12 (SUM: the top-left aggregate walker, moved to T-2063); eval/baseline updated with no new known failures.
- [x] lookup.md's "Implemented in" line names the code.
- [ ] (moved to T-2078 with the LET binding it bounds) SPEC-0014's LET held-grid budget (`MAX_HELD_GRID_ELEMENTS`, past it `#VALUE!`) is recorded as a DEVIATION (decision 0014) under lookup.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- INDIRECT and TRIMRANGE (T-1957).

## Outcome
Built SPEC-0014's carrier: `RefElement`/`RefGrid` and `RefArg::Grid` (reference_operators.rs) with the value view
`RefGrid::to_values`; `Model::lift_references` (lift/driver.rs) broadcasts a reference-returning kernel into a
`RefGrid` with the driver's shape and work checks, sharing `broadcast_shape` with `lift_grids` (a separate entry
rather than a `CallResult::Reference` mode, since its output is not `ArrayNode`s). OFFSET moved to
lookup_and_reference/offset.rs as `offset_reference` (producer) over the scalar kernel `offset_of`; `fn_offset` is its
value view, and `static_analysis_offset` gives a lifted call its broadcast spill shape. Readers: INDEX picks an
element (`index_grid`), ROWS/COLUMNS count the value view, ISREF is FALSE; every other `RefArg` consumer reads the
value view (`into_ranges_or_value`) until T-2063/T-2064. Not done here: SUM's top-left rule (G12, H12 → T-2063) and
the LET/IF/CHOOSE pass-through with its held-grid budget DEVIATION (→ T-2078, filed). Eval delta: probes 73 known
failures fixed (reference-lift-elements, lookup-argument-modes F20:H21 and T-1955's consumer probes), gen-m7
cb748b6a9b81fbb7 A28 passes, 0 regressions. Scalar OFFSET keeps its arguments inline (`ArgBuf`), so it allocates
nothing: `cargo bench -p rustsheet --bench lift -- OFFSET`, 100,000 scalar `=SUM(OFFSET(A{r},0,0,1,2))` cells, median
50.8 ms on main and 48.0 ms here (11 runs each, same machine); the lifted `=ROWS(OFFSET(A1,B1:B100000,0))` takes
19.0 ms (main gave a wrong 1 in 6.3 ms).
