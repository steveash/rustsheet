---
id: T-0474
title: "Whole-column lookups on a cached linked sheet keep full length (virtual blank tail)"
milestone: M11
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP, HLOOKUP and MATCH over whole columns or rows of a cached linked sheet (`[1]Uses!$A:$C`) give Excel's
values instead of the array bound's `#ERROR!`, with the same full-length search a local whole column gets.

## Context
- fixtures/external-link-uncached-lookups Refs!B17 `VLOOKUP(5,[1]Uses!$A:$C,2,FALSE)` "7", B21 `VLOOKUP(99,…)`
  `#N/A`, B49 `VLOOKUP(6,[1]Uses!$A:$C,3,TRUE)` 1, B50 `VLOOKUP(5,[1]Uses!$A:$Z,26,FALSE)` 0: rustsheet gives
  `#ERROR!` (1048576 x 3 cells materialized by `external_area_value`). corpus-bench/56637 fails the same way.
- T-0457 first cut the area to the cache's extent, which T-0457's Excel-fidelity review rejected: a shorter
  search vector moves `approximate_search`'s midpoints (lookup_table.rs, lookup.md "Performance invariant",
  T-0119/T-0127), so an approximate lookup over unsorted cached data could differ from Excel (cache column A =
  1,5,2,9,3: full length returns 1, the cut vector 2).
- Fix direction: build the table from the cached rows plus a virtual empty tail up to LAST_ROW / LAST_COLUMN
  (`SearchVector::with_len`), so search positions are as for a local range; picks past the cache read empty.

## Acceptance criteria
- [ ] B17, B21, B49, B50 pass; corpus-bench/56637 passes; 0 regressions.
- [ ] An oracle cell (added to the fixture, refilled) or a unit test pinned to local-range behaviour covers an
      approximate whole-column VLOOKUP and MATCH over an unsorted cached column.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheets with no cached data (T-0457 done; T-0472).
