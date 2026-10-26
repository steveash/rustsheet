---
id: T-0472
title: "Linked sheet with no cached data: one-cell XLOOKUP, refreshError #REF!, COUNTA count"
milestone: M11
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
rustsheet matches the three cells of fixtures/external-link-uncached-lookups that T-0457 left as known failures.

## Context
- docs/excel-semantics/references.md "Lookups into a linked sheet with no cached data are `#N/A`" (T-0457).
- Refs!B13 `=_xlfn.XLOOKUP(5,[1]Sort!$B$2,[1]Sort!$C$2)`: Excel `#N/A`, rustsheet `#BLOCKED!`. The one-cell lookup
  array reads as its `#BLOCKED!` value in `xlookup_table` (functions/xlookup.rs); the small-range and whole-column
  forms already search a blocked array and give `#N/A` / if_not_found.
- Refs!B38 `=ERROR.TYPE('[1]56737'!$B$2)`: Excel 4 (`#REF!`), rustsheet 11. The sheet's `sheetData` has
  `refreshError="1"`, which `ExternalBook` (external_links.rs) does not keep; the xlsx importer
  (crates/rustsheet-xlsx/src/import/external_links.rs) would need to record it.
- Refs!B44 `=COUNTA([1]Sort!$B:$B)`: Excel 1048576, rustsheet 0. A linked-but-uncached sheet's cells count as
  non-empty (blocked), unlike a reference with no link at all, where COUNTA is 0 (probes/external-ref-no-link-index).
  `external_non_empty_count` returns `None` for an uncached area, so the caller counts 0.
- Route XLOOKUP's one-cell case through `evaluate_lookup_table_arg` / `external_lookup_table` rather than a second
  helper (architecture review of T-0457).

## Acceptance criteria
- [x] The three cells pass; the fixtures baseline shrinks by 3 with 0 regressions.
- [x] references.md's "Not yet matched" sentence is removed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Lookups into a book with no externalLink part (`[other.xlsx]`), which are unprobed.

## Outcome
- XLOOKUP reads a one-cell lookup or return array on a linked sheet the cache holds nothing of as a one-cell
  vector of its `#BLOCKED!`, as it reads a range there (`uncached_cell_vector`, xlookup.rs). Nothing is found, so
  B13 is `#N/A`. `external_lookup_table` decides which sheets these are, so no second helper is needed.
- COUNTA counts every cell of a written reference to such a sheet (`external_blocked_count`, external_links.rs), so B44
  is 1048576.
- The xlsx importer keeps the sheets whose `sheetData` is marked `refreshError` (`ExternalBook::refresh_error_sheets`).
  A one-cell read of one reads `#REF!` (B38). Ranges of such a sheet are unprobed and keep `#BLOCKED!`.
- Eval delta: fixtures known failures 7 → 4 (B13, B38 and B44 fixed), 0 regressions. The remaining four are T-0474's.
