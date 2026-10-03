---
id: T-0119
title: "Bound HLOOKUP/MATCH/XMATCH/LOOKUP whole-row and whole-column walks by the used range"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
depends_on: []
---

## Goal
Lookup functions other than VLOOKUP no longer evaluate up to 1M (column) or 16k (row) empty cells per call for `A:A` / `1:1` references.

## Context
T-0111 bounded VLOOKUP's search column via `SheetData::last_row` (`LookupTable::first_column`). HLOOKUP's `first_row`, MATCH, XMATCH and LOOKUP still walk the full range: a hostile workbook can cost ~1M evaluations per call (directive 5). Empty cells never match, so truncation at the last used row/column is safe; add a last-used-column helper alongside `last_row`.

## Acceptance criteria
- [ ] Unit tests with full-row/column references for each function, including a range starting below the data.
- [ ] A timing note in the PR for `MATCH(x, A:A, 0)` in 10k cells before/after.
- [ ] `make check` green; reviewers approve.

## Out of scope
Per-recalc search-column caching (M3).
