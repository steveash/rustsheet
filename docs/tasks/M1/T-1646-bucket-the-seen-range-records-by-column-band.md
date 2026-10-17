---
id: T-1646
title: "Bucket the seen-range records by column band too"
milestone: M1
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spill commit skips the seen-empty rectangles in other columns, so many spilling anchors next to many whole-column reads cost about anchors + rectangles, not their product.

## Context
T-1545 performance review: `SeenRanges::written_by_spill` (seen_ranges.rs) files each rectangle under a row band only. A whole-column rectangle, and every `clip_to_used_area` remainder below the used area, lands in one high-level band, so every spill meets all of them whatever its columns: S anchors and R distinct-root reads such as `SUM(A:A)` or `MAP(..., LAMBDA(c, SUM(OFFSET(A:A,0,c))))` still cost O(S x R). File rectangles by column band as well (for instance the same levelled bands on the narrower side, or a 2-D level grid), keeping one filing per rectangle and the reader order (first cell written, then read order).

## Acceptance criteria
- [x] Measurement in the Outcome: 1,000 small spilling anchors beside 50,000 whole-column rectangles in other columns, before/after (release).
- [x] `a_spill_meets_the_same_rectangles_as_a_full_scan` (test_range_reads.rs) extended with column-separated rectangles; test/dynamic_evaluation passes; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.

## Outcome
`SeenRanges` (seen_ranges.rs) now files each rectangle once under a 2-D band (`Band`): a levelled
row band and a levelled column band, each the smallest power of two that holds the rectangle's
extent on that axis. `written_by_spill` probes, for each pair of levels in use, only the row bands
times the column bands over the spill (`bands_over`; level 0 skips the band before, since a
one-row or one-column rectangle never reaches into the next band), and still falls back to testing
every rectangle when probing would cost more. One filing per rectangle and the reader order (first
cell written, then read order) are unchanged; `a_spill_meets_the_same_rectangles_as_a_full_scan`
now also checks 2,003 whole-column rectangles side by side met by 1,000 small spills and a few
wide ones.

Measurement (release, this container; whole test process, `Model::evaluate` dominates):
- `many_small_spills_skip_the_whole_column_rectangles_in_other_columns` (new: four anchors record
  50,000 whole-column rectangles in columns DKK:VXD, then 1,000 anchors in row 1 spill two cells
  each): before 326-397 ms, after 114-120 ms. Debug build: 3.11 s before, 1.25 s after.
- `many_small_spills_skip_the_rectangles_far_from_them` (T-1545, whole-row rectangles): 372-442 ms
  before and after, unchanged within noise.

Eval delta: none (perf only).
