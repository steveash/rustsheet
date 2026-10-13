---
id: T-1448
title: "IF and IFERROR/IFNA broadcast a whole column against a whole row unbounded (=IF(D:D,4:4))"
milestone: M1
lane: engine
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
`=IF(D:D,4:4)`, `=IF(D:D,1,4:4)`, `=IFERROR(D:D,4:4)` and `=IFNA(D:D,4:4)` evaluate in bounded
memory and time: a broadcast result above the engine's array limit is the limit's `#ERROR!`
instead of an allocation of 2^20 x 2^14 cells.

## Context
Found while reviewing T-1420, which bounded the binary operators' broadcast (`broadcast_arrays` in
crates/rustsheet-engine/src/arithmetic.rs). `fn_if` and the IFERROR/IFNA path in
crates/rustsheet-engine/src/functions/logical/mod.rs compute `max_rows` x `max_cols` over their
array arguments and allocate the output without `check_array_size`
(functions/math_and_trigonometry/array_size.rs). Each argument alone is bounded by
`evaluate_range` (T-0047); their broadcast is not. Same out-of-memory class as the T-1420 fuzz
reproducer (`formula_eval`, `=Data!$D:$D >Data!$4:44`). DEVIATION per decision 0014:
docs/excel-semantics/arrays.md "Array results above 1,048,576 cells".

## Acceptance criteria
- [ ] `=IF(D:D,4:4)`, `=IF(D:D,1,4:4)`, `=IFERROR(D:D,4:4)` and `=IFNA(D:D,4:4)` are unit tests (decision 0035) that return `#ERROR!` quickly in a debug build.
- [ ] Both paths call `check_array_size` on the output shape before allocating; broadcasts under the limit are unchanged (existing tests and eval).
- [ ] docs/excel-semantics/arrays.md "Array results above 1,048,576 cells" names IF/IFERROR/IFNA.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising or changing the 2^20-cell limit.

## Outcome
`fn_if` and the shared IFERROR/IFNA path now call `check_array_size` on the broadcast output shape
before allocating, so `=IF(D:D,4:4)`, `=IF(D:D,1,4:4)`, `=IFERROR(D:D,4:4)` and `=IFNA(D:D,4:4)`
are `#ERROR!` (unit test in test_operator_broadcast_bound.rs; arrays.md updated). Eval delta: none.
