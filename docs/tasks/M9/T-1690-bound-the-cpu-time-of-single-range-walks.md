---
id: T-1690
title: "Bound the CPU time of single-range walks stretched by a far cell (A:XFD with data at XFD1048576)"
milestone: M9
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A workbook with one cell far away (`XFD1048576`) cannot make one formula over whole columns or
rows, such as `=SUM(A:XFD)` or `=SUMXMY2(A:XFD,A:XFD)`, walk about 1.7e10 positions. The walk costs
in proportion to the cells that exist, or is capped under a documented DEVIATION (decision 0014).

## Context
Found in T-1681 review (security and performance). T-1681 made the order-statistic and paired
lists hold only non-blank cells, so memory is bounded. Time is not. A single range is walked
densely over its sheet's used-area rectangle: `for_each_position_in_area` with
`of_union = false` in `functions/util.rs`, used by SUM, COUNT, the aggregates and the
`values_from_range` and `paired_values` callers. A far cell stretches that rectangle to the whole
sheet. The union walk (`for_each_existing_position`) already walks existing cells only, under the
`MAX_AREA_WALK` budget (references.md "Union area cap"). The fix should likely reuse it for
single areas, with care for callers that need blanks: COUNTBLANK, and ranges lifted to arrays.
Explicit near-whole ranges (`A2:XFD1048576`) are not clipped by `used_area_corner` at all.

## Acceptance criteria
- [x] A unit test with cells at A1 and XFD1048576 evaluates `SUM(A:XFD)`, `COUNT(A:XFD)` and
      `SUMXMY2(A:XFD,A:XFD)` in bounded time (release and debug).
- [x] Values are unchanged on every gated suite (eval delta 0 regressions), and a benchmark shows
      no slowdown on dense ranges.
- [x] `make check` green; reviewers approve.

## Out of scope
- Memory of the lists (T-1681).

## Outcome
A single range whose used-area-clipped rectangle holds at least 65,536 positions and more than four
per cell its columns hold (`Model::walks_sparsely`, using `SheetData::cells_in_columns` and `DENSE_WALK_FACTOR` from T-2711)
is walked over its existing cells by the union walk (`for_each_existing_position`), recorded as
read empty in one rectangle. It is not charged to the formula's `MAX_AREA_WALK` budget (it costs
less than the dense walk it replaces, which never was), so it adds no `#VALUE!`: a test repeats
`SUM(A:Z)` over 100,001 cells 200 times in a BYROW (review finding). All callers of
`ClipBlocks` (SUM/COUNT/aggregates, `values_from_range`, `paired_values`, SUBTOTAL, AGGREGATE,
COUNTBLANK, CONCAT, MULTINOMIAL, GCD/LCM) read a blank as nothing, so values are unchanged; the
walk keeps row-major order, so the first error is the same. `test_clipped_range_walks` checks ten
functions over `A:XFD` with a cell at XFD1048576 (release 0.16 s, debug 1.5 s for the file). The
aggregate bench gains a far-cell case (`SUM(A:XFD)` over 1,000,000 filled rows: 163 ms); dense cases
match main within noise (SUM 138 vs 151 ms, block past used area 262 vs 286 ms). references.md
"Union area cap" names the new case. Eval delta: 0 (no case changes).
