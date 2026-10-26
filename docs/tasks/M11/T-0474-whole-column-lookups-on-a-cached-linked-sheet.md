---
id: T-0474
title: "Whole-column lookups on a cached linked sheet keep full length (virtual blank tail)"
milestone: M11
lane: functions-lookup
status: done
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
- [x] B17, B21, B49, B50 pass; corpus-bench/56637 passes; 0 regressions.
- [x] An oracle cell (added to the fixture, refilled) or a unit test pinned to local-range behaviour covers an
      approximate whole-column VLOOKUP and MATCH over an unsorted cached column.
- [x] `make check` green; reviewers approve.

## Out of scope
- Sheets with no cached data (T-0457 done; T-0472).

## Handoff (resumed)
Claimed by an opus run that had too little session budget left to finish it; no code written. Notes from reading
the code (data, check them):
- The table argument reaches `Model::lookup_table` (lookup_and_reference/lookup.rs) already evaluated: an external
  range (`Node::WrongRangeKind`) is built by `Model::external_area_value` (external_links.rs) as a full array, which
  `check_array_size` refuses for 1048576 x 3, hence `#ERROR!`.
- So VLOOKUP/HLOOKUP/MATCH need to see the node first: `Model::cached_external_area(node, cell)` gives the
  `ExternalArea` (first/last row and column, `cache`) without building anything; `is_external_array` shows the pattern.
- A third `LookupTable` variant (lookup_table.rs) holding the `ExternalArea` could serve `rows`/`columns`/`get` from
  `cached_value` and `first_row`/`first_column` as `SearchVector::with_len(cached prefix, full length)`; the cached
  prefix ends at the cache's last used row or column. `lookup.rs` `ResultVector` and the line-67 vector builder
  match on the variants too.

## Outcome
VLOOKUP, HLOOKUP and MATCH read a range of several cells of a cached linked sheet in place
(`LookupTable::External`, `Model::cached_external_lookup_area`) instead of building it as an array, so
whole columns and rows no longer hit the array bound. The search vector is the cached cells
followed by an empty tail to the area's full length (`SearchVector::with_len`), so approximate searches probe
the same midpoints a local whole column gives. Unit test `whole_column_lookups_keep_full_length` checks 13
lookups (VLOOKUP, HLOOKUP, MATCH 1/0/-1, over unsorted data) against the same lookups over local whole
columns and rows. Eval: fixtures/external-link-uncached-lookups Refs!B17, B21, B49 and B50 now pass, and so does
corpus-bench/56637 and corpus-bench/59932 (whole-column VLOOKUPs into
`[1]<2010Price!$A:$BE`) pass, with 0 regressions (fixtures known failures 4 -> 0, corpus-bench 9 -> 7).

