---
id: T-0438
title: "GCD and LCM over a tall block read the blanks past the used area as one 0"
milestone: M3
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: [T-0405]
---

## Goal
`GCD`/`LCM` over a tall explicit block (`LCM(A1:Z1048575)`) cost the block's data, not its size, with the same value as now.

## Context
T-0405 review (performance, minor). GCD and LCM read a blank cell as 0 (`LCM(F1:F3)` with 4,
blank, 6 is 0; docs/excel-semantics/coercion.md), so they walk an explicit block whole through
`for_each_cell_or_error_in_whole_range` (functions/util.rs), every cell of
`A1:Z1048575` (27 million). They could walk the used-area clip (`clip_block_to_used_area`) and
then read one 0 when anything was left out. Care: `handle_number` in
functions/math_and_trigonometry/gcd_lcm.rs is order-dependent (a 0 resets `result_too_large`), so
where that single 0 goes relative to the walked cells must give the same value as the dense walk
in every case (too-large before and after the blank, negatives, errors after it).

## Acceptance criteria
- [x] `GCD`/`LCM` over a tall block past the used area is bounded in time (test).
- [x] Unit tests show the same values as the dense walk for blanks before/after a too-large value, a negative and an error.
- [x] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN's 3-D walk with ignore_empty FALSE (same pattern; separate if needed).

## Outcome
GCD and LCM walk a range with the new `for_each_cell_or_error_in_range_noting_blanks`
(functions/util.rs): an explicit block is cut at the used area as the other aggregates cut it
(`walk_area` now reports whether a `ClipBlocks` cut left out blanks the whole-range walk hands
on), and the blanks left out are read as one 0 after the walk. Any number of 0s anywhere gives
what one gives: GCD(n,0) is n, and a 0 makes LCM 0 and clears a too-large result before it, while
errors, negatives and too-large arguments do not depend on where the 0 falls. Whole-row and
whole-column ranges are clipped as before (no 0), and blocks the cut declines are walked densely.
`a_tall_block_past_the_used_area_gives_what_the_dense_walk_gives` checks 15 tall/short pairs
(too-large LCM before the blank, negative, too-large argument, error and text after it, a block
wholly past the data); `a_tall_block_costs_its_data_not_its_size` (`LCM(A1:Z1048575)`, 27M
cells) takes 69s in a debug build on the old code and 0.01s now. Eval delta: none (0 changes in
every suite). TEXTJOIN's whole-range walk stays as it was (out of scope).
