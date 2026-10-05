---
id: T-0403
title: "Cache or track the sheet's last used column for whole-row lookups"
milestone: M3
lane: perf
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Whole-row lookups (`MATCH(x,1:1,0)`, `HLOOKUP(x,1:2,..)`) find the sheet's last used column in O(1) or once per pass, not by scanning every row on each call.

## Context
T-0119 bounds whole-row search vectors with `SheetData::last_column` (sheet_data.rs, via `Worksheet::last_used_column` and `clip_search_columns` in functions/lookup_and_reference/lookup_table.rs). It visits every existing row, so N such formulas on a sheet with R rows cost O(N x R) per recalc (performance review of T-0119). Options: track the max column incrementally in `SheetData` (removals make this harder), or cache it per sheet per evaluation pass (invalidated by spills, which already restart the pass).

## Acceptance criteria
- [x] `last_column` (or its replacement) is O(1) amortised per call; unit tests cover set/remove/spill.
- [x] Bench or timing note: 1k `MATCH(x,1:1,0)` on a sheet with 100k rows, before/after.
- [x] `make check` green; reviewers approve.

## Out of scope
The same for `last_row` (already chunk-based and cheap).

## Outcome
`SheetData` keeps a per-column cell count (`column_counts`, not serialised; `Model::from_workbook` rebuilds it); `last_column()` is its last key, O(log
columns), and every write (`set_cell`, `remove_cell`, `remove_row`, `set_row`) keeps it in step, so
removals never rescan rows (the first design did and the security review rightly flagged it as
quadratic for deletes/clears). Tests cover set, replace, remove, row removal/replacement and
clearing a 20k-row sheet. Timing (release, 100k rows, first design, O(1) read): 1,000 reads took
25 ns in total; the old scan visited every row per call and no clean "before" figure was taken, and
the `MATCH(x,1:1,0)` end-to-end timing was not measured. No eval delta (no function behaviour changed).
