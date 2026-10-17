---
id: T-1770
title: "SPEC-0011 lifting driver and argument-kind table; first consumers EDATE, EOMONTH, WEEKNUM and YEARFRAC"
milestone: M9
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1767]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
One shared driver lifts a scalar function's arguments over arrays and ranges, from one per-function argument-kind table that static analysis reads too (SPEC-0011). EDATE, EOMONTH, WEEKNUM and YEARFRAC are its first consumers: they lift over an array, and a multi-cell range in any of their arguments gives `#VALUE!`, with no implicit intersection, in dynamic and in legacy formulas.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, sections Design and Test plan. Evidence: docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule", and dates.md "Date function argument modes".
- Add `functions/arg_kinds.rs` with `ArgKind`, `ArgKinds` and `Function::arg_kinds`. Enter EDATE, EOMONTH, WEEKNUM and YEARFRAC as `LiftArrays` in every argument.
- Add `functions/util/lift.rs` with `LiftedArg`, `lift_values` and `lift_call`. `lift_values` is `lift_with_cost`, `CriteriaArg` and `count_walks` moved out of statistical/if_ifs.rs, plus `reuse_equal_runs`, the text budget and `element_node`.
- Keep the existing consumers of `lift_with_cost` (*IF(S), AGGREGATE, IFS, SWITCH, `lookup_each`, XLOOKUP) working through `lift_values`, with no result change.
- Static analysis reads the table:
  - `get_function_args_signature`: `Lift` → Scalar, the other kinds → Vector.
  - `static_analysis_on_function`: the broadcast of the lifted arguments' shapes, where a `LiftArrays` range adds no shape.
  - `legacy_value_operand`: gains the argument index, and puts the `@` on a value operation in a `LiftArrays` position.
  - The four functions' old signature and shape lines are deleted.
- The engine intersects today: `ROWS(EDATE(A1:A60,1))` is 1 on the range's rows. Legacy `=EDATE(A1:A60,1)` gets an `@` that Excel does not add (FORMULATEXT `=EDATE(@A1:A60,1)`).
Cells, all failing in eval/baseline/probes.txt:
- probes/date-argument-modes F/G 20, 22, 24, 26, 54, 56, 66, 68.
- probes/scalar-lift-shapes H1, H2, H3, H5, H6, H22, H30, H36, H37, H45, H49, H50, J2, J6, J7, K2, K6, K7.
- probes/scalar-lift-shapes-b C3, C70, D3, D70, E3, E70, F3, F70, G3, K3, L3, L70, N3, N70, P3, Q3.
- probes/scalar-lift-shapes-c E3, E70, H3, H70, P3, S3.
- gen-m9/4df1b22d3ed63f58 A24 (EOMONTH over an array start: `#SPILL!`) and gen-m9/b58b6d7cd39d195f A9 (EOMONTH's months a TAKE reference: `#VALUE!`). Both moved here from T-1236.

## Acceptance criteria
- [ ] The cells listed under Context pass and leave eval/baseline/probes.txt (and gen-m9's baseline), with no new F line in any suite.
- [ ] The unit tests of SPEC-0011's Test plan are in place, for `lift_values` and the static analysis. Shapes and `#N/A` padding, errors per element, `LiftArrays` alone and next to an array, `Whole` evaluated once, the unlifted path calling the kernel once. Size bound: `=ROWS(EDATE(A1:A1048576+0,A1:B1+0))` (a 2^21-cell broadcast) is `#ERROR!` with 0 kernel calls, and the cell-visit cap trips before the first element.
- [ ] `rg -n "fn lift_with_cost|struct CriteriaArg|enum CriteriaArg" crates/rustsheet-engine/src` finds nothing outside functions/util/lift.rs. `scripts/check_duplication.py` does not rise.
- [ ] docs/excel-semantics/dates.md and arrays.md say "Implemented in" for the four functions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WORKDAY, NETWORKDAYS and their .INTL forms (T-1236). The other families (T-1372, T-1290, T-1772, T-1383). Moving the old helpers (T-1773, T-1774, T-1775).
