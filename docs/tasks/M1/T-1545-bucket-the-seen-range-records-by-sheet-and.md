---
id: T-1545
title: "Bucket the seen-range records by sheet and row band"
milestone: M1
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spill commit touches only the seen-empty rectangles near its area, so a pass with many spilling anchors and many recorded rectangles costs about anchors + rectangles, not their product.

## Context
T-1512 performance review: `Model::spill_contradicts_a_read` (evaluation.rs) now meets each `SeenRange` once per commit (`SeenRange::first_cell_written`), O(cells + R) per commit, but still walks all R rectangles of the pass for every commit, so S spilling anchors cost O(S x R). Bucket `SeenRanges` (seen_ranges.rs) by sheet and row band so a commit visits only the rectangles that can intersect it. The readers must keep their current order (first cell written, then read order), which the restart facts depend on.

## Acceptance criteria
- [x] Measurement in the Outcome: 1,000 small spilling anchors in a pass with 100,000 recorded rectangles elsewhere on the sheet, before/after (release).
- [x] test/dynamic_evaluation passes; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.

## Outcome
`SeenRanges` (seen_ranges.rs) files each rectangle once under a levelled row band (`Band`: the
smallest power-of-two height that holds it, keyed by sheet and first row), and the new
`SeenRanges::written_by_spill` probes only the bands over a spill's rows, about twice its height
plus the levels in use; a spill so tall that probing costs more than the rectangles on record
tests them all, as before. `Model::spill_contradicts_a_read` calls it; the reader order (first cell
written, then read order) is unchanged, and `a_spill_meets_the_same_rectangles_as_a_full_scan`
checks the lookup against a full scan.

Measurement (release, this container):
- The spill check alone, 1,000 two-by-two spills against 100,000 one-row rectangles (plus three tall
  ones): full scan 188-192 ms, row bands 0.3-0.5 ms.
- End to end, `many_small_spills_skip_the_rectangles_far_from_them` (A1 and B1 record 100,000
  whole-row rectangles, 1,000 anchors in column D spill two cells each; `Model::evaluate`):
  before 306-338 ms, after 136 ms; the rest is the 100,000 `MAP` evaluations.

Eval delta: none (perf only; `make eval-check` 0 regressions, 0 fixes).
Follow-up: T-1646 (whole-column rectangles still all share one band, so bucket by column band too).
