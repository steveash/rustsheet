---
id: T-0438
title: "GCD and LCM over a tall block read the blanks past the used area as one 0"
milestone: M3
lane: perf
status: in-progress
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
- [ ] `GCD`/`LCM` over a tall block past the used area is bounded in time (test).
- [ ] Unit tests show the same values as the dense walk for blanks before/after a too-large value, a negative and an error.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN's 3-D walk with ignore_empty FALSE (same pattern; separate if needed).
