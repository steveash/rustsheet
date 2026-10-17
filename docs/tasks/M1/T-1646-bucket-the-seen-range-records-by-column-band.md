---
id: T-1646
title: "Bucket the seen-range records by column band too"
milestone: M1
lane: perf
status: in-progress
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
- [ ] Measurement in the Outcome: 1,000 small spilling anchors beside 50,000 whole-column rectangles in other columns, before/after (release).
- [ ] `a_spill_meets_the_same_rectangles_as_a_full_scan` (test_range_reads.rs) extended with column-separated rectangles; test/dynamic_evaluation passes; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.
