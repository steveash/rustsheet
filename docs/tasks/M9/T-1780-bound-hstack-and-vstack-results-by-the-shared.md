---
id: T-1780
title: "Bound HSTACK and VSTACK results by the shared array limit (HSTACK(A:A,1:1) pads 2^34 cells)"
milestone: M9
lane: functions-lookup
status: todo
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
HSTACK and VSTACK refuse a result above the shared array limit before they pad or copy it, as `&`, ROUND and the element-wise text functions do. So an untrusted formula such as `=HSTACK(A:A,1:1)` (2^20 rows x 16,385 columns, about 2^34 cells) cannot exhaust memory (CLAUDE.md directive 5).

## Context
Found while doing T-1771. `fn_hstack` and `fn_vstack` (crates/rustsheet-engine/src/functions/lookup_and_reference/hstack_vstack.rs) take the tallest (HSTACK) or widest (VSTACK) argument and pad every other argument with `#N/A` to it, without calling `limits::check_array_size`. Each argument is bounded on its own (`evaluate_range`), but a column next to a row is their product. Reproduced with a scratch unit test on the T-1771 branch:
- `=ROWS(HSTACK(A1:A2000,A1:ALL1))` builds 2000 x 1001 = 2,002,000 cells and gives 2000.
- `=ROWS(VSTACK(A1:A2000,A1:ALL1))` builds 2001 x 1000 = 2,001,000 cells and gives 2001.
- `=ROWS(A1:A2000&COLUMN(A1:ALL1))` gives `#ERROR!`.
Compute the output height and width from the argument shapes first and return `check_array_size`'s error before allocating. The bound is the existing DEVIATION (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"); add HSTACK/VSTACK to the broadcasts it lists. Related but distinct: T-1564 (the arms are copied in full before the output is sized); this task is only the output bound.

## Acceptance criteria
- [ ] HSTACK and VSTACK return `check_array_size`'s error (`#ERROR!` above 2^20 cells, `#VALUE!` past the grid) before padding or copying any argument into the result.
- [ ] A unit test shows `=ROWS(HSTACK(A1:A2000,A1:ALL1))` and `=ROWS(VSTACK(A1:A2000,A1:ALL1))` are `#ERROR!`, and `=ROWS(HSTACK(A:A,1:1))` (out of column A and row 1) returns at once.
- [ ] arrays.md's DEVIATION entry lists HSTACK/VSTACK. No eval cell changes (`make eval`: 0 fixes, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sizing from range bounds without copying the arms (T-1564).

