---
id: T-1802
title: "Probe, then enter QUOTIENT, FACTDOUBLE and SERIESSUM's x, n, m (they refuse a range; probes/scalar-lift-open-a Y19-Y30)"
milestone: M9
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
QUOTIENT, FACTDOUBLE and SERIESSUM's x, n and m are `LiftArrays` entries in the SPEC-0011 table once an array probe shows they lift, and MULTINOMIAL, GCD, LCM and SERIESSUM's coefficients are not lifted.

## Context
SPEC-0011 open question 3 left these without an entry until probed. T-1777 measured (probes/scalar-lift-open-a Y19-Y30, Z19-Z30 the scalar controls): `ROWS(QUOTIENT(A1:A60,2))`, `ROWS(QUOTIENT(10,B1:B60))` and `ROWS(FACTDOUBLE(D1:D60))` are `#VALUE!`, and so are SERIESSUM's x, n and m over a range (Y24-Y26); whether they lift over an array is NOT probed. SERIESSUM's coefficients over a range is read whole (Y27, one value, `SERIESSUM(2,0,1,B1:B3)` = 7, -d W31). MULTINOMIAL, GCD and LCM give one result (ROWS 1; Y22, Y23, Y28-Y30), and only MULTINOMIAL's value was probed: `MULTINOMIAL(B1:B3,2)` = 59.999999999999986 (-d W30). T-1770 owns the table, so the entries must be added with its first consumers; the family is math.

## Acceptance criteria
- [x] First probe (excel-graph, a case under probes/): `INDEX(QUOTIENT({7;9},2),2)`, `INDEX(FACTDOUBLE({5;6}),2)`, `INDEX(SERIESSUM({1;2},0,1,{1,2}),2)` with ROWS of each, before entering them as `LiftArrays`.
- [x] The table has QUOTIENT, FACTDOUBLE (`LiftArrays`) and SERIESSUM (x, n, m `LiftArrays`, coefficients `Whole`); MULTINOMIAL, GCD and LCM have no lifted argument.
- [x] probes/scalar-lift-open-a Y19-Y30 pass (the range refusals; ROWS 1 for MULTINOMIAL, GCD, LCM).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions' later arguments.

## Outcome
- Probes first (excel-graph, 2026-10-17): probes/math-atp-array-lift (44 cells) and -b (18 cells). QUOTIENT,
  FACTDOUBLE and SERIESSUM's x, n, m lift over an array (`INDEX(QUOTIENT({7;9},2),2)` = 4,
  `INDEX(FACTDOUBLE({5;6}),2)` = 48, `INDEX(SERIESSUM({1;2},0,1,{1,2}),2)` = 5, ROWS 2 each). SERIESSUM's
  coefficients, MULTINOMIAL, GCD and LCM stay whole over an array. One finding the task did not expect: FACTDOUBLE
  refuses a boolean in every form (`FACTDOUBLE(TRUE)`, a TRUE cell and `{TRUE;5}` are `#VALUE!`, -b E1-E3, E8).
  It is recorded in numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes".
- Table: QUOTIENT `ATP_2` and FACTDOUBLE `ARRAYS_1`. SERIESSUM is the new `SERIES`:
  `[LiftArrays, LiftArrays, LiftArrays, WholeByCaller]`. MULTINOMIAL, GCD and LCM have no entry. Their fallback
  arms already give no lift (Vector signatures, a Scalar shape), which a unit test checks with `ROWS(...)` = 1.
- The coefficients are `WholeByCaller`, not `Whole`, for the reason T-1772 gives in SPEC-0011's "As built". The
  existing reader looks at the value as `evaluate_node_in_context` gives it: a one-cell text reference or typed
  text is read as a number, and text inside a range or an array is `#VALUE!`. A `Whole` read through
  `evaluate_node_with_reference` would change those scalar results. Reading the coefficients first also gives
  their cell count, so each lifted element is charged `LiftCost::Cells(count)` through `lift_call_costed`. With
  `Whole`, the driver would charge `Unit`, and `SERIESSUM(SEQUENCE(1000),0,1,A1:A200000)` would run 2*10^8
  terms unbounded. That call is now `#ERROR!` up front (unit test). `ArgKind::Whole` stays dead code with its
  TODO(T-1236). The kernel still checks x, n, m first and then reports a coefficient read error, so the check
  order is unchanged.
- Kernels borrow their values (`cast_number_no_bools_ref`, financial's `two_numbers` made `pub(crate)` for
  QUOTIENT) and use `lift::{num_error, value_error, div_error}`. FACTDOUBLE left `single_number_fn!`.
- Scalar changes, all from evidence: `FACTDOUBLE(TRUE|FALSE|<boolean cell>)` changed from 1 to `#VALUE!` (-b E1-E3),
  a 3-D range in these arguments is `#VALUE!` (SPEC-0011's `LiftArrays` rule, was `#ERROR!` for FACTDOUBLE), and
  legacy formulas give a bare range in x/n/m/QUOTIENT/FACTDOUBLE no `@` (it is refused).
- Eval: 39 cells fixed, 0 regressions, no other cell's value changed (cell-by-cell against main): probes/scalar-lift-open-a
  Y19-Y21, Y24-Y26 (6), and the new cases are clean (27 + 6 cells that failed on main).
  static_analysis.rs 1406 -> 1385 lines (decision 0010).

