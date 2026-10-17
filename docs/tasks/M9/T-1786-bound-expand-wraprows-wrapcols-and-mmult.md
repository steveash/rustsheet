---
id: T-1786
title: "Bound EXPAND, WRAPROWS, WRAPCOLS and MMULT results by check_array_size before allocating; check XMATCH"
milestone: M9
lane: functions-lookup
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
The array functions outside the lifting paths that T-1780 and T-1781 do not cover refuse a result above the shared array limit before they allocate, so an untrusted formula cannot exhaust memory with them (CLAUDE.md directive 5). They are EXPAND, WRAPROWS, WRAPCOLS and MMULT. XMATCH is checked too.

## Context
Found by the T-1767 security review of SPEC-0011 ("Not driver paths, same hole"). HSTACK/VSTACK padding is T-1780's and TEXTSPLIT padding is T-1781's, both filed by the T-1771 work. Reading the code at e0627030 shows:
- **EXPAND** (functions/lookup_and_reference/expand.rs) casts `rows` and `columns` with `n as usize` and calls `Vec::with_capacity(target_rows)` before any check. `EXPAND(1,1E9,1E9)` asks for 10^18 cells.
- **WRAPCOLS** (wrapcols_wraprows.rs) casts `wrap_count` with `n as usize` and allocates `vec![Vec::with_capacity(num_cols); wrap_count]`. WRAPROWS pads each row to `wrap_count`.
- **MMULT** (math_and_trigonometry/mmult.rs) builds rows(array1) × columns(array2) cells with no check: `MMULT(A1:A1048576,A1:B1)` is 2^21 cells, and `MMULT(A:A,1:1)` is 2^34.
- **XMATCH:** not traced. Read its allocation path and fix it, or record that it is bounded.

The bound is the existing DEVIATION (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"): `#ERROR!` above 2^20 cells and `#VALUE!` past the grid. Check the size in `usize` with saturating or checked arithmetic, before any `with_capacity` or `vec!`. Excel's own results for valid sizes must not change: arrays.md has EXPAND's, WRAPROWS's and WRAPCOLS's evidenced rules (WRAPCOLS of a single value, wrap_count clamping), so keep them.

## Acceptance criteria
- [x] Unit tests with explicit ranges on a filled fixture: `=EXPAND(1,1E9,1E9)`, `=ROWS(WRAPCOLS(A1:A10,1E9))`, `=ROWS(WRAPROWS(A1:A10,1E9))` and `=ROWS(MMULT(A1:A1048576,A1:B1))` are `#ERROR!` (or the evidenced `#VALUE!`/`#NUM!` where Excel's rules decide first). Each test finishes in under a second in debug.
- [x] The XMATCH finding is in the Outcome, as a fix or as "bounded by …".
- [x] arrays.md's DEVIATION entry lists these functions. `make eval`: 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- HSTACK/VSTACK (T-1780), TEXTSPLIT (T-1781), the text broadcast (T-1771) and the lifting driver (T-1770).

## Outcome
EXPAND, WRAPCOLS, WRAPROWS and MMULT call `check_array_size` on their result shape before they
allocate it. For EXPAND that is after its rows/columns rules, and for MMULT after its input checks,
so Excel's `#VALUE!` for a blank or text input still comes first. `EXPAND(1,1E9,1E9)` and a
wrap_count of 1E9 are `#VALUE!` (a dimension past the grid). An in-grid shape above 2^20 cells is
`#ERROR!`: `ROWS(EXPAND(1,2000,1000))`, and `ROWS(MMULT(A1:A2000,B1#))` with a 1,000-column B1#.
WRAPCOLS/WRAPROWS cannot reach `#ERROR!`: their vector is at most 2^20 cells. Excel clamps a
wrap_count above the vector's length instead of padding (arrays.md, T-1630), which bounds them the
same way once T-1630 lands. MMULT's inputs were already bounded by `eval_to_array`, and with the
output bound its work is at most 2^30 multiply-adds.
XMATCH is bounded by its inputs: it lifts nothing and builds no result array. Its `lookup_array`
must be one row or one column. A range becomes a `SearchVector` over its used part
(`range_search_vector`, the rest a virtual run of blanks), and an array constant or computed array
was bounded when it was built. arrays.md's DEVIATION entry lists the four functions and the XMATCH
finding. Tests: test/lookup_and_reference/array_result_bound.rs, 0.37 s in debug. Eval delta:
`make eval-check` shows 0 regressions (see the PR).
