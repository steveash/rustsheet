---
id: T-3471
title: "Range operands of operators (evaluate_range) read from the per-pass range-values cache (calipers pv26_002 SUMPRODUCT masks)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A range read as an operand of an operator (`A1:A1440="x"`, `B2:G41*2`) whose cells are already inert in this
pass costs a copy of its cached values, not one `evaluate_cell` per cell. Values, errors, charges and the
cells on record stay the walk's.

## Context
After T-3230 and T-3456, calipers `pv26_002` spends about half of its 12 s `evaluate()` in SUMPRODUCT.
Stack samples (2026-10-31) put most of that in `handle_comparison` → `value_or_array` → `evaluate_range`
→ `evaluate_cell`: 1,616 calls of
`SUMPRODUCT('Quarterly ARR Rollup'!$I$11:$AW$1450*('…'!$I$6:$AW$6=D$3)*('…'!$C$11:$C$1450=$C6))` each read
the 1,440-cell key column and the 41-cell header again through `evaluate_cell`.

`evaluate_range` (`cast.rs`) checks the size, clips to the used area (recording the rest as read empty),
charges the array's cells (SPEC-0024), then reads each cell inside the used area with `evaluate_cell`.
The criteria cache (`ifs_cache.rs`, `ifs_range_values`) already holds, per pass, the values of a range of
64 to 4,000,000 cells whose cells are all inert (`read_is_inert`, T-0449). That rule is the one T-3230's
blocks use, and it means reading the cached values is the walk's read.

A prototype that reads the cached view after the charge took `pv26_002` from 12.2 s to 9.1–9.7 s, and the
engine tests pass unchanged.

## Acceptance criteria
- [x] `evaluate_range` reads a same-sheet range's values from `ifs_range_values` after its size check, clip
      and charge. It falls back to the cell walk when the range cannot be cached.
- [x] Tests compare with `no_pass_caches`: comparisons and arithmetic over ranges with numbers, text,
      booleans, blanks, errors, cells past the used area, formulas not evaluated yet, spills, circular cells
      and iteration. They include the circular marks and the work spent.
- [x] calipers `pv26_002` release `evaluate()` time before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Caching the comparison masks themselves, or indexing a key column for `=`.

## Outcome
- **Change:** `evaluate_range` (`cast.rs`) now reads a range from `ifs_range_values` once its size check, used-area
  clip and cell charge are done. This applies to a same-sheet range that ends inside the used area. A range past the
  used area is still walked, so `A:A=1` reads only its used rows and is never cached whole. A range not cacheable
  yet is still walked: one under 64 cells, one past the cache budget, or one with a cell that is not inert. Elements
  convert through the existing `calc_result_to_array_node`. To be read from `cast.rs`, `ifs_range_values`,
  `RangeView`, `RangeView::get` and `RectKey` became `pub(crate)`.
- **Tests:** `test/test_range_operand_cache.rs` compares runs against `no_pass_caches`. It checks stored values to
  the bit, circular marks, the work spent, and that the cached run reads fewer cells (`cell_reads`).
  - The workbook has numbers, text, booleans, numeric text, blanks, formulas evaluated when read, an error, a spill,
    a two-column block and a range past the used area.
  - The readers apply `=`, `>`, `<>`, `*`, unary `-` and `&`.
  - It also covers a circular reader inside its range and a cycle member under iterative calculation, both with a
    recalculating edit.
  - Mutations caught: a boolean read as a number, text read as blank, an error read as blank, and elements
    transposed. The first try missed the boolean mutation, because each reader got only one criterion; the criteria
    now vary independently of the reader.
- **Shared helper:** the cached-versus-walked runner is now `test/pass_cache_runs.rs`, used by these tests and
  T-3456's `test_sum_prefix.rs`.
- **Timing** (calipers `pv26_002`, release `evaluate()`, on T-3456's head): 12.2 s before, 9.5–10.5 s after (three
  runs on a noisy machine).
