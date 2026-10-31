---
id: T-3442
title: "The xlsx importer clears covered cells of a merged range without walking its whole area"
milestone: M12
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Importing an xlsx whose merged ranges are huge, for example `<mergeCell ref="A1:XFD1048576"/>` repeated, costs time
in proportion to the file, not to the area the merges name.

## Context
Found while doing T-3437. `load_sheet` in crates/rustsheet-xlsx/src/import/worksheets.rs clears the covered cells of
each merged range, the loop after `load_merge_cells`. It looks up every cell of the range (`for row in
merged_cell.row..=last_row { for column in .. { sheet_data.cell_mut(..) } }`). A whole-sheet merge is therefore
1.7e10 lookups, and a few hundred bytes of `<mergeCell>` stall the import (directive 5).

Fix: visit only the stored cells inside each range. Walk the stored rows in the range, then each row's stored columns
in it (the engine's `SheetData` row and column walks; `places_in_block` is `pub(crate)` today). Overlapping merges
(Excel writes none) could still repeat the work, so either refuse them or walk the stored cells once against a
sorted list of the ranges.

## Acceptance criteria
- [x] A test imports an xlsx with many whole-sheet `<mergeCell>`s in well under a second, with covered cells still
      cleared.
- [x] `make eval-check` shows no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`import/merged_cells.rs` `clear_covered_cells` replaces the loop in `load_sheet`, which looked up every cell of every
merged range.

**How it works.** One sweep goes over the stored cells in row order (`SheetData::cells` is row-major). The ranges
whose rows are open are kept in a `BTreeMap` keyed by first column. For each cell, the open range with the largest
first column at or before it is the only one that can cover it, because `load_merge_cells` drops intersecting ranges.
A range whose rows have closed is dropped when first met. The cost is O((cells + ranges) log ranges), not the area the
ranges name.

**Why not the task's suggested walk.** Walking each range's stored rows (`places_in_block`) would still cost
rows x ranges for many tall merges, so it is not used.

**Behaviour.** Unchanged: covered cells become `EmptyCell` with their style kept, and each anchor stays.

**Tests.**
- Unit tests:
  - covered cells are emptied and the rest kept;
  - closed and later ranges do not mix up.
- `tests/malformed_import.rs`: 1,000 whole-column merges, about 2^30 cells named. They import in 0.08 s, with
  covered cells cleared and the anchor kept. The old loop made about 10^9 lookups.

**Found while working.** `load_merge_cells` itself is quadratic in the number of ranges (its intersection check). That
is T-3454.

