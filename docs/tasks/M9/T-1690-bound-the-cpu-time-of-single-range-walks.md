---
id: T-1690
title: "Bound the CPU time of single-range walks stretched by a far cell (A:XFD with data at XFD1048576)"
milestone: M9
lane: perf
status: todo
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
- [ ] A unit test with cells at A1 and XFD1048576 evaluates `SUM(A:XFD)`, `COUNT(A:XFD)` and
      `SUMXMY2(A:XFD,A:XFD)` in bounded time (release and debug).
- [ ] Values are unchanged on every gated suite (eval delta 0 regressions), and a benchmark shows
      no slowdown on dense ranges.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Memory of the lists (T-1681).
