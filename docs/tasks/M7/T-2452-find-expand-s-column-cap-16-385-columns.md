---
id: T-2452
title: "Find EXPAND's column cap: 16,385 columns are built, 123,456,789 are #NUM! (probe sizes Excel refuses without building)"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
EXPAND refuses columns with `#NUM!` from exactly Excel's cap, not from the smallest refused size probed so far.

## Context
Found by T-2408: `COLUMNS(EXPAND(1,1,16385))` is 16385 but `COLUMNS(EXPAND(1,1,123456789))` and `(1,4,123456789)`, `(1,4,2^27)`
are `#NUM!` (probes/expand-makearray-past-grid-big, excel-graph, 2026-10-23), so Excel has a columns cap of its own somewhere in
16,386..123,456,789. `fn_expand` (functions/lookup_and_reference/expand.rs, `COLUMNS_REFUSED`) refuses from 123,456,789. Sizes
Excel builds hang excel-graph's workbook open (T-2408's 2^17..2^20-column and 1E6-element MAKEARRAY cells never filled,
so they were dropped), so probe only sizes expected to be refused, a few per case, bisecting down from
123,456,789 (e.g. 2^24, 2^22, 2^20+1 in one row), or ask desktop Excel (M14).

## Acceptance criteria
- [ ] The cap is evidenced to one column (or to the bound below which ours is `#ERROR!` anyway); `COLUMNS_REFUSED` matches it and
  arrays.md "Stacked results past the grid" states it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
