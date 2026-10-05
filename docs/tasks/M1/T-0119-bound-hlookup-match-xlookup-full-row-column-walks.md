---
id: T-0119
title: "Bound HLOOKUP/MATCH/XMATCH/LOOKUP whole-row and whole-column walks by the used range"
milestone: M1
lane: engine
status: done
priority: P1
size: S
tier: sonnet
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

## Outcome
`SheetData::last_column` (+ `Worksheet::last_used_column`) added next to `last_row`. HLOOKUP's search row, MATCH's lookup vector and LOOKUP's lookup vector are truncated at the last used column/row (`LookupTable::first_row`, `as_vector(.., search = true)`); LOOKUP's 3-argument result vector is cut after element `l`. XMATCH already clipped via `clip_to_used_area`; covered by a test. Timing (debug build): 200 x `MATCH(i,A:A,0)` over 10k cells 159.5 s before, 0.33 s after. Eval delta none.
Side effect: sorted (approximate) search over a whole column/row no longer sees the empty tail, so `MATCH(99,A:A,1)` returns the last used row (3) instead of 1048576, and `HLOOKUP(99,1:2,2)` returns the last column's value, matching VLOOKUP (T-0111). Not oracle-probed; no eval cell covers it.
