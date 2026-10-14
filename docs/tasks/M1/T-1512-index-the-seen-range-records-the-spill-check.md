---
id: T-1512
title: "Index the seen-range records the spill check scans"
milestone: M1
lane: perf
status: done
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
- [x] Unit test or measurement: a spill of 100,000 cells in a pass with 100,000 recorded rectangles commits in time near-linear in the two, measured before/after in the Outcome.
- [x] Spill restart tests (test/dynamic_evaluation) still pass; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SEEN_RANGES`.

## Outcome
`Model::spill_contradicts_a_read` takes the spill's area instead of a list of written cells and
meets each `SeenRange` once, by intersecting it with the area (`SeenRange::first_cell_written`),
then sorts the hits by that first cell (stable), so the readers come out in the same order as
the old cell-by-cell scan and the restart facts are unchanged. Reader de-duplication uses a set
instead of `Vec::contains`. Cost: O(cells + rectangles · log rectangles) instead of
O(cells × rectangles).

Measurement (`a_large_spill_is_checked_against_many_rectangles_in_near_linear_time`: 100,000
rectangles on record from two 50,000-cell anchors, then a 100,000-cell spill; two evaluations,
`--release`): 34.06 s before, 0.47 s after. All dynamic_evaluation tests pass; eval delta: none
(no case changes value).
