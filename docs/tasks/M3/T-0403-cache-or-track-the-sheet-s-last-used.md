---
id: T-0403
title: "Cache or track the sheet's last used column for whole-row lookups"
milestone: M3
lane: perf
status: todo
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
- [ ] `last_column` (or its replacement) is O(1) amortised per call; unit tests cover set/remove/spill.
- [ ] Bench or timing note: 1k `MATCH(x,1:1,0)` on a sheet with 100k rows, before/after.
- [ ] `make check` green; reviewers approve.

## Out of scope
The same for `last_row` (already chunk-based and cheap).
