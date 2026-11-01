---
id: T-3765
title: "VSTACK and HSTACK keep a one-cell error reference as a 1x1 array, also through a LAMBDA parameter"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
VSTACK and HSTACK read a one-cell reference to an error cell as a 1x1 array holding that error, as Excel does, also when
the reference reaches them through a LAMBDA parameter (a cell of the range SCAN, REDUCE or MAP walks). A literal or
computed scalar error argument stays the whole result (T-1695).

## Context
- Found by the T-3683 arg-pairs batch (gen-m7, excel-graph, 2026-11-01). On the Data sheet of the case G5 is the text
  "12" and G9 the constant error `#DIV/0!`.
- Excel: `SCAN(Data!G5,Data!G9,LAMBDA(x,z,VSTACK(x,z)))` is `#CALC!`. SCAN passes z as a reference to Data!G9 (the walked
  range's cell, T-1442), and VSTACK stacks it into the 2x1 array {"12";#DIV/0!}, a LAMBDA result that is an array, so
  `#CALC!`. We return `#DIV/0!`: `fn_vstack` returns an error argument whatever its source.
- The direct form behaves the same in our engine: `VSTACK("12",Data!G9)` is `#DIV/0!` here. Excel's answer for the
  direct forms is inferred from the cell above and from T-3506's rule for TOCOL, TOROW and FILTER, not yet measured.
- T-1695 (probes/vstack-scalar-error) evidences that a scalar error argument (`NA()`, `1/0`) makes VSTACK or HSTACK that
  one error. Only a one-cell error *reference* is in question here.
- Helper to reuse: `wrap_one_cell_error_ref` in functions/spill_functions.rs (T-3506) wraps by the argument's node kind,
  so it does not see a LAMBDA parameter bound to a reference; extend or generalise it rather than copy it.
- Quirk recorded in docs/excel-semantics/arrays.md, "A one-cell error reference is a 1x1 array in TOCOL, TOROW and
  FILTER (T-3497)", which now names VSTACK through a LAMBDA parameter.
- Cells (1; owned here):
  - gen-m7/3f368d034700f19e: A2990

## Visible-parity stage
Decision 0083: the cell is visibly off (a wrong error type), so this task is in the campaign. Its target is that every
cell it owns has Excel's type and error.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) evidences `VSTACK` and `HSTACK` with a direct one-cell error
      reference first, middle and last (`VSTACK("12",Data!G9)`, `ROWS(..)`, `INDEX(..,2)`), beside a literal error for
      contrast, and through a LAMBDA parameter (SCAN, REDUCE, MAP); the engine matches every probed cell.
- [ ] gen-m7/3f368d034700f19e A2990 passes and leaves eval/baseline/gen-m7.txt (`make eval-baseline`).
- [ ] The arrays.md entry states the VSTACK/HSTACK rule with the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scalar error arguments (T-1695 stays as is) and other stacking functions not probed here.
