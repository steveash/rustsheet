---
id: T-0048
title: "Clip whole-row/column walks that read every cell (MULTINOMIAL, date holidays, COUNT-family helper)"
milestone: M0
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A function that walks every cell of a whole-row or whole-column range argument without building
an array reads only the sheet's used area, so `=NETWORKDAYS(1,9,A:XFD)` or
`=MULTINOMIAL(1:1048576)` cannot run for billions of cells or grow the evaluation seen-table per
cell (CLAUDE.md directive 5).

## Context
Found while doing T-0047, which bounded and clipped every range lifted into an array
(`Model::evaluate_range`, cast.rs). Walks that call `evaluate_cell` per cell of a `Range`
argument without clipping remain: `functions/math_and_trigonometry/multinomial.rs` (~line 49),
`functions/date_and_time.rs` (~line 555, the holidays walk), and the range arms in
`functions/statistical/count_and_average.rs` (~lines 166, 243, 362, 405, 455). Each `evaluate_cell`
in a pass calls `Model::record_seen` (evaluation.rs), so the seen-table grows per cell.
The existing pattern to reuse is `Model::clip_to_used_area` (evaluation.rs), already used by
`functions/util.rs`, `xlookup.rs`, `if_ifs.rs` and `text/common.rs`: walk to the clipped corner and
treat the rest as empty (what each function does with an empty cell must stay unchanged).

## Acceptance criteria
- [x] Each walk listed above clips whole-row/column ranges with `clip_to_used_area` (no new helper).
- [x] A unit test per family evaluates a whole-sheet range argument (`A:XFD` or `1:1048576`) and returns the same value as the used-area range, quickly (debug test under a few seconds).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
MULTINOMIAL, the NETWORKDAYS/WORKDAY holidays walk, AVERAGEA, COUNT, COUNTA, COUNTBLANK and the AVERAGEA-family helper (`for_each_value_a`) now read ranges through the shared walkers in functions/util.rs, which clip whole-row/column ranges with `clip_to_used_area`. `for_each_cell_in_range` (aborts on the first error cell) is now built on a new `for_each_cell_or_error_in_range` (hands error cells to the callback, for COUNT/COUNTA/COUNTBLANK), so the clipping lives in one place and the per-function walk boilerplate shrank. COUNTBLANK counts the non-blank cells of the walk and subtracts them from the range's cell count, so the clipped-off (empty) cells are blank. COUNT/COUNTA/COUNTBLANK moved to statistical/count.rs (count_and_average.rs had to stay under the 1000-line gate). Unit tests in test/test_clipped_range_walks.rs (before the change the whole-sheet cases ran for minutes). No eval delta.
