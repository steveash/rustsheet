---
id: T-1545
title: "Bucket the seen-range records by sheet and row band"
milestone: M1
lane: perf
status: todo
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
- [ ] Measurement in the Outcome: 1,000 small spilling anchors in a pass with 100,000 recorded rectangles elsewhere on the sheet, before/after (release).
- [ ] test/dynamic_evaluation passes; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.
