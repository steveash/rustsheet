---
id: T-3442
title: "The xlsx importer clears covered cells of a merged range without walking its whole area"
milestone: M12
lane: engine
status: todo
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
- [ ] A test imports an xlsx with many whole-sheet `<mergeCell>`s in well under a second, with covered cells still
      cleared.
- [ ] `make eval-check` shows no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
