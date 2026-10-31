---
id: T-3437
title: "The xlsx importer stops expanding every cell of an array formula ref into a map"
milestone: M12
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3426]
---

## Goal
Importing an xlsx costs memory and time in proportion to the file, however large the `ref` of its array formulas:
a crafted `<f t="array" ref="A1:XFD1048576">` repeated a few thousand times is refused or read cheaply, not expanded.

## Context
Found in T-3426's Excel-fidelity review. `SheetDataReader` (crates/rustsheet-xlsx/src/import/sheet_data.rs, the
`"array"` arm of the formula handling) inserts every cell of each array formula's `ref` except the anchor into
`array_cell: HashMap<(i32, i32), (i32, i32)>`. A cell read later looks itself up there to become a spill cell. This
runs before `Model::from_workbook`, so T-2896's per-anchor cap and T-3426's `SpillTally` cannot stop it: n anchors of
2^34 cells each mean n x 2^34 inserts (directive 5).

Suggested approach: cells arrive in row order, so keep only the anchors whose rows are still open, keyed by first
column (a `BTreeMap`), and look a cell up there. Refuse a `ref` larger than `limits::check_array_cells` allows, and an
anchor whose `ref` overlaps another's (Excel writes none). Per-cell lookups then cost O(log anchors) and memory
O(anchors). The current map is only filled once the anchor is read, so a cell before its anchor is not a spill cell
today either.

## Acceptance criteria
- [ ] A test imports an xlsx with thousands of whole-sheet array `ref`s and is refused quickly, with bounded memory.
- [ ] Every suite workbook imports as before (`make eval-check`, no regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
