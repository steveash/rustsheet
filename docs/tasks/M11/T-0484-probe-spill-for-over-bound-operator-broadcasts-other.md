---
id: T-0484
title: "Probe #SPILL! for over-bound operator broadcasts other than * (&, comparisons, error operands, 2^34 cells)"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel shows in a cell whose operator broadcast is above 1,048,576 cells and runs past
the grid, for the operator forms T-0480 extrapolated from the `*` case of corpus-bench/55049.

## Context
T-0480 makes such a cell `#SPILL!` from the operand shapes (docs/excel-semantics/arrays.md "Array results above
1,048,576 cells are #ERROR!"). Only `=SUMPRODUCT(..)*H:J` is oracle-backed. Unprobed, asserted in
crates/rustsheet-engine/src/test/test_lifted_range_bound.rs `broadcast_above_the_bound_that_cannot_spill_is_spill`
and test_operator_broadcast_bound.rs `whole_column_against_whole_row_is_bounded`: `=(1+H:J*2)`, `=H:J&"x"`,
`=1/0*H:J`, `=A:A*H1:J1+1` in rows 3-7, and the 2^34-cell `=D:D+4:4`, `=D:D>4:4`, `=D:D&4:4`, `=4:4^D:D` in G1-G7.
Excel for the web may refuse very large arrays (`SessionMemoryQuotaExceeded`, arrays.md), so the 2^34-cell
cells may need one workbook each.
T-0483 added the lift and unary forms of the same rule, asserted in test_lifted_range_bound.rs
`lift_above_the_bound_that_cannot_spill_is_spill`: `=-H:J`, `=H:J%`, `=ABS(-H:J)`, `=1+DATE(11:1899,12,1)` and
`=(DATE(2000,H:J,1))` in C3:C7 (`#SPILL!`), and `=-H:J` in K1 (fits the grid; ours `#ERROR!`); the 2^34-cell lifts
`=ROUND(A1:A1048576,C1:XFD1)` in B2 (test_fn_round.rs) and `=LEFT(A:A,1:1)` in C5 (text_functions/broadcast_bound.rs),
now `#SPILL!`; and a call that reads an argument itself before its lift, with an error there (`=TEXTJOIN(1/0,H:J,"a")`
in C3, ours `#SPILL!`). Probe them too.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with those formulas has oracle values; the tests and arrays.md agree with
      them, or a task is filed for each disagreement.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a task instead).
