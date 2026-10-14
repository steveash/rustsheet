---
id: T-1512
title: "Index the seen-range records the spill check scans"
milestone: M1
lane: perf
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A dynamic array's spill commit costs about what its cells and the rectangles they land in cost, not (spill cells) x (every seen-empty rectangle recorded in the pass).

## Context
T-1463 performance review: `Model::spill_contradicts_a_read` (evaluation.rs) tests every written cell against every `SeenRange` of the pass. T-1463 caps the rectangles per formula run at `MAX_SEEN_RANGES` (65,536), but the cap is not pass-wide, so N formulas can hold N x 65,536 rectangles, and a large spill checks each of its cells against all of them. Options: index `SeenRanges` by sheet and row/column band, or test the spill's bounding rectangle against each range once and only then its cells. Read the evaluation.rs module doc first; the restart semantics must not change.

## Acceptance criteria
- [ ] Unit test or measurement: a spill of 100,000 cells in a pass with 100,000 recorded rectangles commits in time near-linear in the two, measured before/after in the Outcome.
- [ ] Spill restart tests (test/dynamic_evaluation) still pass; no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.
