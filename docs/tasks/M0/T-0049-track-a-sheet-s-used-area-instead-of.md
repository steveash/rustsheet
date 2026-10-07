---
id: T-0049
title: "Track a sheet's used area instead of scanning it on every whole-row/column lift"
milestone: M0
lane: perf
status: in-progress
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
- [ ] `clip_to_used_area` no longer scans the sheet; a benchmark or unit test shows many whole-column lifts on a sheet with many cells do not scale with sheet cells x formulas.
- [ ] Spill writes during a pass are still covered (existing `test/dynamic_evaluation/test_range_reads.rs` stays green).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
