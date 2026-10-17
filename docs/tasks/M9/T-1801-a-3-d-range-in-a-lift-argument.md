---
id: T-1801
title: "A 3-D range in a Lift argument is #REF!, and in a LiftArrays argument #VALUE! (probes/scalar-lift-open-b, -d)"
milestone: M9
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A 3-D range (`Sheet1:Sheet2!A1:A2`) in a `Lift` argument gives `#REF!`, as Excel does, and in a `LiftArrays` argument `#VALUE!`.

## Context
SPEC-0011 open question 2 assumed `#VALUE!` in both, as `lift_with_cost` gives today. T-1777 (probes/scalar-lift-open-b W2-W11, -d W17-W29) measured: `ROWS(PMT(Sheet1:Sheet2!C1:C2,10,100))` and `ROWS(NORM.DIST(Sheet1:Sheet2!C1:C2,0,1,TRUE))` are `#REF!`, also for one cell per sheet (`PMT(Sheet1:Sheet2!C1,10,100)`, -d W21-W22), in a later argument (`PMT(0.1,3D,100)`, -b W6) and beside an array (`INDEX(PMT({0.1;0.2},10,3D),1)` and 2, -d W17-W18; `INDEX(PMT(3D,{10;20},100),1)`, -d W25). In EDATE (`LiftArrays`) the 3-D range is `#VALUE!`, alone (-b W4-W5, W7) and beside an array (elements 1 and 2 `#VALUE!`, -d W19-W20, W26). The shape beside an array is the array's (-b W8-W9: ROWS 2). `ISREF(3D)` is FALSE and `3D+0` is `#REF!` (-d W24, W27-W28), while `SUM(3D)` works (-b W1) and `ROWS(3D)` is `#VALUE!` (-b W12). Also `BITAND(13,3D)` (a `Lift` argument) is `#REF!` (-d W29). T-1770 (driver read phase) must account for it, and T-1372, T-1383, T-1290 for their families.
T-1770 implemented both in the read phase (`functions/lift/call.rs`); only `Lift` has no consumer yet. Two of its reads are inferred, not probed, and this task probes them (excel-probe skill, a new probes case):
- a one-cell 3-D range in a `LiftArrays` argument, `EDATE(Sheet1:Sheet2!A1,1)` (ours `#VALUE!`);
- the reference-form reads of a `LiftArrays` argument other than TAKE, INDEX and IF: `ROWS(EDATE(DROP(A1:A60,1),1))`, `ROWS(EDATE(TRIMRANGE(A1:A60),1))` and `ROWS(EDATE(XLOOKUP(1,{1;2},A1:B60,,0),1))`-style multi-cell XLOOKUP results (ours `#VALUE!`, refused like a written range).

## Acceptance criteria
- [x] probes/scalar-lift-open-b W2-W11 and probes/scalar-lift-open-d W17-W29 pass, except -b W9 and -d W17, W18 (`PMT` beside an array and a 3-D range), which need a `Lift` consumer and moved to T-1372's acceptance criteria.
- [x] A test covers a 3-D range in a `Lift` and in a `LiftArrays` argument, alone and beside an array.
- [x] The inferred reads above are probed and their cells pass, or the read phase changes to match them.
- [x] `make check` green.
- [ ] Reviewers approve.

## Out of scope
- 3-D ranges in aggregates or in any non-lifted function.

## Outcome
Probed the inferred reads (probes/scalar-lift-open-e, excel-graph, 22 cells): Excel matches what T-1770's read phase infers, so no engine code changed; the notes that called them inferred now cite the cells.

- **One-cell 3-D range in a `LiftArrays` argument is `#VALUE!`**: `EDATE(Sheet1:Sheet2!A1,1)`, `WEEKNUM`, `EOMONTH`, `YEARFRAC` (W1, W4-W7, W9), also in the later argument (W7) and beside an array (`INDEX(EDATE(3D1,{1;2}),1)` and 2 are `#VALUE!`, W2-W3; `ROWS` is 2, W8).
- **Reference forms**: `EDATE` over a multi-cell `DROP`, `TRIMRANGE` or `XLOOKUP` result is `#VALUE!` (`INDEX(..,1)` and `ROWS`, W10-W15, W20), as for `TAKE` (W22). A one-cell result is its value (W16-W19, W21).
- **Code and docs**: `read_lift_args` and `evaluate_as_reference` already do all of this; the doc comment in `functions/lift/call.rs`, `docs/excel-semantics/arrays.md` and the SPEC-0011 read table and open question 2 cite the cells. New unit test `one_cell_3d_range_and_reference_forms_in_lift_arrays` (lift/tests/mod.rs); the existing `a_3d_range_is_value_in_lift_arrays_and_ref_in_lift` covers a `Lift` position alone and beside an array through the driver.
- **Left to the `Lift` consumers (T-1372 PMT, T-1383 BITAND, T-1290 NORM.DIST)**: -b W9 and -d W17, W18 (`PMT` beside an array and a 3-D range) still fail, because PMT does not lift yet; the other -b W2-W11 and -d W17-W29 cells pass today (a `Lift` 3-D read is unit-tested through the driver, no consumer here). Their tasks should expect `#REF!` in each element.
