---
id: T-0423
title: "Index externalLink caches per sheet (extent and cell count) instead of scanning the whole book"
milestone: M3
lane: perf
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-1460]
---

## Goal
Reading a cached range of a linked workbook costs O(cells read), not O(cells cached in the whole
book): the extent and cell count of each cached sheet are known without scanning.

## Context
From the T-1460 performance review. `external_links.rs` keeps `ExternalBook::cells` as one map
keyed by (sheet, row, column):
- `cached_extent` scans every key of the book each time an aggregate reads a whole column or row,
  so N such formulas over a cache of M cells cost O(N*M);
- `external_non_empty_count` (COUNTA) compares an area with the cell count of the whole book, not
  of its sheet;
- `external_area_value` builds a whole-column value array (`MATCH(x,[1]S!$A:$A,0)`) with one hash
  lookup per row; a pre-filled array plus a pass over the sheet's cached keys would do.
A per-sheet index (`HashMap<u32, …>` or extent and count computed once at import) fixes all three.
Behavior must not change (fixtures/external-link-cached-ranges, test_external_links.rs).

## Blocked because
Deferred to Phase 2 (decision 0027): performance work waits until Phase 1 parity exits. Unblock
when docs/status.md has the line `**Phase 1:** exited <date>`. The cost is bounded
today (CLAUDE.md directive 5 limits hold).

## Acceptance criteria
- [ ] No per-evaluation scan of a whole book's cache; a bench or unit test shows it.
- [ ] fixtures/external-link-cached-ranges unchanged; `make check` green; reviewers approve.

## Out of scope
- Any behavior change.
