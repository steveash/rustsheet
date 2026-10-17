---
id: T-1771
title: "Bound the broadcast of the element-wise text functions by the shared array limit (LEFT(A:A,1:1) allocates 2^34 cells)"
milestone: M9
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
The element-wise text functions (LEFT, MID, FIND, REPT, TEXT, CONCATENATE and the other `broadcast_text` callers) refuse a broadcast above the shared array limit before allocating, as `&` and ROUND do. So an untrusted formula such as `=LEFT(A:A,1:1)` (2^34 cells) cannot exhaust memory (CLAUDE.md directive 5).

## Context
Found while writing SPEC-0011 (T-1767). `broadcast_text` (crates/rustsheet-engine/src/functions/text/text_arg.rs) takes the largest rows and columns of its operands and allocates rows × columns elements. It never calls `limits::check_array_size`. Only the 64 MiB text budget (T-0232) limits it, and a numeric result such as FIND's or LEN's does not count against that budget. Each operand is bounded on its own (`evaluate_range`), but a column against a row is their product. Reproduced with a scratch case on rustsheet-eval at ea76d91f:
- `=ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))` builds 2,000,000 cells and gives 2000.
- `=ROWS(A1:A2000&COLUMN(A1:ALL1))` and `=ROWS(ROUND(A1:A2000,COLUMN(A1:ALL1)))` give `#ERROR!`.
The bound is the existing DEVIATION (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"), which lists the broadcasts it covers. Add the text functions there.

## Acceptance criteria
- [x] `broadcast_text` returns `check_array_size`'s error (`#ERROR!` above 2^20 cells, `#VALUE!` past the grid) before it allocates or computes any element.
- [x] A unit test shows `=ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))` and `=ROWS(FIND("a",A1:A2000,COLUMN(A1:ALL1)))` are `#ERROR!`. A test with whole-column and whole-row operands (`=ROWS(LEFT(A:A,1:1))`) returns at once.
- [x] arrays.md's DEVIATION entry lists the text broadcast. No eval cell changes (`make eval`: 0 fixes, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving the text functions onto the SPEC-0011 driver (T-1775).

## Outcome
`broadcast_text` (functions/text/text_arg.rs) calls `limits::check_array_size` on the broadcast shape before it allocates or computes any element, the check `broadcast_arrays` (`&`) and `lift_numbers` (ROUND) already use; no new helper. Every element-wise text caller (LEFT, RIGHT, MID, FIND/SEARCH, LEN, REPT, EXACT, CONCATENATE, TEXT, SUBSTITUTE, REPLACE, PROPER, CHAR/UNICHAR, CODE, BAHTTEXT, FILTERXML, DATEVALUE/TIMEVALUE) gets the bound. Tests: test/text_functions/broadcast_bound.rs (`ROWS(LEFT|FIND|MID|CONCATENATE|REPT(A1:A2000,COLUMN(A1:ALL1)))` are `#ERROR!`; `=ROWS(LEFT(A:A,1:1))`, `=ROWS(FIND("a",A:A,1:1))` and `=LEFT(A:A,1:1)` return `#ERROR!` at once; a 2x3 broadcast and `ROWS(LEN(A:A))` are unchanged) and functions/text/text_arg/tests.rs (`compute` is never called above the limit). arrays.md's DEVIATION lists the text broadcast. Eval: 0 fixes, 0 regressions. Other unbounded results found by the same search, outside this helper: HSTACK/VSTACK padding (T-1780) and TEXTSPLIT padding (T-1781).
