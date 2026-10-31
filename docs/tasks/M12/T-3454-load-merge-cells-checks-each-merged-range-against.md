---
id: T-3454
title: "load_merge_cells checks each merged range against every earlier one, quadratic in the ranges"
milestone: M12
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Importing a sheet with many merged ranges costs time close to linear in their number. A crafted file with 10^5 small
`<mergeCell>`s must not cost 10^10 intersection checks.

## Context
Found in T-3442. `load_merge_cells` (crates/rustsheet-xlsx/src/import/worksheets.rs) keeps the first of any
intersecting ranges. It does this by checking each new range against every range kept so far
(`merged_cells.iter().any(|m| m.intersects(..))`), which is O(n^2) in the number of ranges. A few MB of `<mergeCell>`
elements (about 30 bytes each) then mean about 10^10 checks (directive 5). Keep the same first-wins result, which
T-3442's sweep relies on, since kept ranges never overlap. Use a structure that answers "does this rectangle hit a
kept one" in log time, for example a sweep over rows with an interval tree of open column spans, or sorted kept
ranges bucketed by row block.

## Acceptance criteria
- [x] A test imports a sheet with 10^5 merged ranges, overlapping ones among them, in well under a second, and keeps
      exactly the ranges the current rule keeps.
- [x] `make eval-check` shows no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`load_merge_cells` files the kept ranges in `KeptRanges` (import/merged_cells.rs). This is a segment tree over the
columns `1..=LAST_COLUMN` (depth 15). Each node holds two merged unions of row spans: `whole` holds the ranges
canonical at that node, and `within` holds the ranges canonical at or under it. A new range intersects a kept one
exactly when its rows overlap either `within` at one of its canonical nodes, or `whole` at one of their ancestors.
Each range costs O(log^2) and its spans merge amortized. The first-wins result is unchanged, which T-3442's sweep
relies on.

Tests:
- Unit tests compare the index with the old pairwise rule on edge shapes and on two sets of 4,000 pseudo-random
  ranges: one on a small grid, and one over all the columns with some ranges thousands of columns wide, which
  reaches the tree's upper levels. The edge shapes are crossing ranges, whole rows and columns, shared corners,
  and touching but disjoint ranges.
- `many_merged_ranges_import_quickly_and_keep_the_first_of_intersecting_ones` (tests/malformed_import.rs) imports
  10^5 disjoint ranges, then 10^4 that each intersect them, then three whole rows. It asserts the kept list exactly.

Timings on that sheet:

| Build | Old pairwise checks | Index (merges) | Whole load |
|---|---|---|---|
| Debug | 46 s | 1.5 s | 2.2 s |
| Release | | 0.17 s | 0.29 s |

The test's bound is 20 s, which the old code fails.

The tree's nodes are allocated on the first range kept: 2^15 empty nodes, about 1.5 MB, dropped after the sheet.
No Excel behavior changed, so there is no new `excel-semantics` entry.

