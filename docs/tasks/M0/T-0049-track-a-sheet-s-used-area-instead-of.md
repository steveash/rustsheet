---
id: T-0049
title: "Track a sheet's used area instead of scanning it on every whole-row/column lift"
milestone: M0
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Clipping a whole-row or whole-column range to the used area costs O(1) (or amortised O(1)) instead of a scan of every cell of the sheet, so N formulas such as `=SUM(ABS(A:A))` cost O(N), not O(N x sheet cells).

## Context
Raised by the performance reviewer on T-0047. `Model::clip_to_used_area` (evaluation.rs) calls
`Worksheet::dimension()`, which scans every stored cell (worksheet.rs, has a FIXME). Since T-0047
every whole-row/column lift (`Model::evaluate_range`, cast.rs) goes through it, alongside the
existing walkers (util.rs, xlookup.rs, if_ifs.rs, text/common.rs). Options: maintain max row/column
incrementally on cell writes (deletes may only over-approximate, which stays correct: cells past the
true used area are empty), or cache the dimension per sheet for one evaluation pass, invalidated on
spill writes. Correctness constraint: the clip may over-approximate the used area, never under.

## Acceptance criteria
- [x] `clip_to_used_area` no longer scans the sheet; a benchmark or unit test shows many whole-column lifts on a sheet with many cells do not scale with sheet cells x formulas.
- [x] Spill writes during a pass are still covered (existing `test/dynamic_evaluation/test_range_reads.rs` stays green).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`Worksheet::used_area_end` gives the used area's bottom-right corner from the tracked last row
(chunk scan, bounded by the chunk layout) and last column (`column_counts`, O(log columns)), never
visiting the cells. `clip_to_used_area`, the IFS walker and the XIRR/XNPV walker use it instead of
`dimension()`. 10,000 `COUNT(Sheet1!A:A)` over a 160,000-cell sheet: ~57 s → 0.7 s unoptimised
(`test_clipped_range_walks::whole_column_walks_do_not_scan_the_sheet`). Eval delta: none
(0 fixed, 0 regressions). Follow-up: T-1114 (walkers that act on empty cells depend on where the
clip ends).
