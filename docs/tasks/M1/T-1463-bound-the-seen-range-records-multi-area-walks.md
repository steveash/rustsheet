---
id: T-1463
title: "Bound the seen-range records multi-area walks push during a pass"
milestone: M1
lane: perf
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A formula walking many whole-column areas records a bounded number of seen ranges per pass, so the spill bookkeeping cannot grow to tens of millions of entries for one formula.

## Context
T-1162 performance review: `clip_to_used_area` (evaluation.rs) pushes up to two `SeenRange` entries per whole-row/column area per walk, without deduplication. The per-formula cell budget (`MAX_AREA_CELLS`, area_budget.rs) caps it, since every area costs at least one cell, but one formula can still push about 2 x 2^24 entries in a pass (`MAP` over `SUM(u)` with `u` = 65,536 copies of `A:A` on an empty sheet). Options: merge or deduplicate identical seen ranges per root, or charge records against a budget. Read the evaluation.rs module doc (restarts depend on these records) before changing anything.

## Acceptance criteria
- [ ] Unit test: the formula above records a bounded number of seen ranges (or is refused under a documented bound), and the spill restart tests still pass.
- [ ] Measured before/after for the formula above in the Outcome.
- [ ] No eval regressions; any new limit has a DEVIATION entry (decision 0014).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_AREA_CELLS` or `MAX_AREAS`.
