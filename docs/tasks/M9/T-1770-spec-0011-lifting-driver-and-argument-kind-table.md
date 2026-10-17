---
id: T-1770
title: "SPEC-0011 argument-kind table, lift_call and static-analysis hooks; first consumers EDATE, EOMONTH, WEEKNUM and YEARFRAC"
milestone: M9
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1767, T-1784]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SPEC-0011's argument-kind table and `lift_call` front end are in place, static analysis derives its signatures, shapes and legacy `@` from the table, and EDATE, EOMONTH, WEEKNUM and YEARFRAC are the first consumers. They lift over an array, and a multi-cell range in their probed arguments gives `#VALUE!`, with no implicit intersection, in dynamic and in legacy formulas.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md (Design, Test plan). Evidence: docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule", and dates.md "Date function argument modes".
- Build on T-1784's `functions/lift/driver.rs`.
- Add `functions/lift/kinds.rs`: `ArgKind`, `KernelWork`, `ArgKinds` and `Function::arg_kinds`, one `match` returning `&'static`. Enter EDATE, EOMONTH, WEEKNUM and YEARFRAC as `LiftArrays` in every argument (all are probed over a range: EDATE's start and months, YEARFRAC's basis and dates, EOMONTH and WEEKNUM in date-argument-modes).
- Add the read phase: non-generic, `#[inline(never)]`, values held in an inline `ArgBuf`, and no heap or grid machinery on the unlifted path. Add `lift_call`.
- Add the borrowing casts (`cast_number_ref`, …). `cast_to_*` delegates to them.
- Add the three bounds: input cells, output cells, and work with `Prepared` and `Weighted`.
- Add the in-place single-grid path.
- Static analysis:
  - The table is consulted first.
  - The old arms move into `fallback_signature` and `fallback_shape`.
  - `lifted_static_result` uses `broadcast_static_result`.
  - `legacy_value_operand` takes the argument index.
  - The four functions' old arms are deleted.
- Element results go through `cast::calc_result_to_array_node`, with no new helper.
- The engine intersects today: `ROWS(EDATE(A1:A60,1))` is 1 on the range's rows. Legacy `=EDATE(A1:A60,1)` gets an `@` that Excel does not add.
Cells, all failing in eval/baseline/probes.txt:
- probes/date-argument-modes F/G 20, 22, 24, 26, 54, 56, 66, 68.
- probes/scalar-lift-shapes H1, H2, H3, H5, H6, H22, H30, H36, H37, H45, H49, H50, J2, J6, J7, K2, K6, K7.
- probes/scalar-lift-shapes-b C3, C70, D3, D70, E3, E70, F3, F70, G3, K3, L3, L70, N3, N70, P3, Q3.
- probes/scalar-lift-shapes-c E3, E70, H3, H70, P3, S3.
- gen-m9/4df1b22d3ed63f58 A24 (EOMONTH over an array start: `#SPILL!`) and gen-m9/b58b6d7cd39d195f A9 (EOMONTH's months a TAKE reference: `#VALUE!`). Both moved here from T-1236.

## Acceptance criteria
- [ ] The cells listed under Context pass and leave eval/baseline/probes.txt (and gen-m9's baseline), with no new F line in any suite.
- [ ] The unit tests of SPEC-0011's Test plan pass. They cover shapes and `#N/A` padding, errors per element, `LiftArrays` alone and next to an array, `Whole` evaluated once, the unlifted path calling the kernel once without a `LiftedArg` or grid, and the table-wins test (no function in the table also has a fallback arm).
- [ ] Size tests on a filled fixture with explicit ranges:
  - `=ROWS(EDATE(A1:A1048576+0,A1:B1+0))` is `#ERROR!` with 0 kernel calls.
  - The summed-input-cells bound refuses before the next grid is read.
  - `LiftCost::Cells` trips the work bound before the first element, and `Prepared` runs.
- [ ] `test_nested_formula_stack` passes with a deeply nested lifted EDATE formula added.
- [ ] docs/excel-semantics/dates.md and arrays.md say "Implemented in" for the four functions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WORKDAY, NETWORKDAYS and their .INTL forms (T-1236). The other families (T-1372, T-1290, T-1772, T-1383). Moving the old helpers (T-1773, T-1785, T-1774, T-1775).
