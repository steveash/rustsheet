---
id: T-1463
title: "Bound the seen-range records multi-area walks push during a pass"
milestone: M1
lane: perf
status: done
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
- [x] Unit test: the formula above records a bounded number of seen ranges (or is refused under a documented bound), and the spill restart tests still pass.
- [x] Measured before/after for the formula above in the Outcome.
- [x] No eval regressions; any new limit has a DEVIATION entry (decision 0014).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_AREA_CELLS` or `MAX_AREAS`.

## Outcome
The formula of the Context already records one rectangle on main: T-1497 made `SeenRanges`
deduplicate per root, so `LET(u,<65,536 x A:A>,SUM(MAP(SEQUENCE(16),LAMBDA(r,SUM(u)))))` records 1
before and after (it is `#VALUE!` from the walk budget either way); a unit test pins it. What was
still unbounded is distinct rectangles: each new whole-row/column reference adds one, so
`SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,SUM(OFFSET(1:1,r-1,0,c)))))` recorded 1,048,576 on main
(release: 136 s, ~70 MB of records, every spill commit scanning them). A new per-formula-run budget
(`MAX_SEEN_RANGES` = 65,536, area_budget.rs) pays for new rectangles only; past it the rectangle
goes unrecorded and the run ends `#VALUE!` ("Too many distinct ranges read"), set in
`with_area_budget` so IFERROR cannot turn an untracked read into a value. Same formula after: 65,536
records, `#VALUE!` (release: 130 s, the cap is reached after 1.5 s; the rest is the remaining lambda
calls). DEVIATION entry: references.md "Seen-range cap". The duplicated test helpers
`error_message`/`balanced_union` moved to test/util.rs. Eval: no change (no case reads that many
rectangles).
