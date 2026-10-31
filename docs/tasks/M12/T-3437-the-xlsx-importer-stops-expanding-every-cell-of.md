---
id: T-3437
title: "The xlsx importer stops expanding every cell of an array formula ref into a map"
milestone: M12
lane: engine
status: done
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
- [x] A test imports an xlsx with thousands of whole-sheet array `ref`s and is refused quickly, with bounded memory.
- [x] Every suite workbook imports as before (`make eval-check`, no regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The importer now bounds `array_cell` instead of restructuring it. `SheetDataReader` gives each sheet a budget of cells
its array formulas may cover: the part's declared size / 16, plus one whole column (`array_cell_budget`).
`claim_array_cells` charges each `ref` against it before any entry goes into the map, and refuses two cases:

- **One `ref` of more than one whole column of cells.** The load check (T-2896) refuses such a file anyway, so this
  only refuses earlier.
- **`ref`s that exhaust the budget.** Memory and time are then proportional to the part.

**Why the budget is safe.** A file that stores every cell its arrays cover spends more than 16 bytes per covered cell,
in its `<c>` and `<row>` elements, so such files fit. The declared size is untrusted, but `ImportLimits` already
checks it against the package's byte budget (`Part::declared_size`).

**Why not stream a structure of open anchors.** The suggested BTreeMap of anchors whose rows are still open would need
cells in row order. The importer does not enforce that: a row's `r` is free, and so is a cell's.

**Plumbing.** `read_sheet_data` now takes a `SheetPart` (rich errors, date system, part size) to stay within the
argument limit.

**Records.** The refusal is a DEVIATION, recorded in docs/excel-semantics/xlsx-storage.md. Whether Excel opens such a
file is part of probe T-3436.

**Tests** (`tests/malformed_import.rs`):
- a `ref` of two whole columns;
- a thousand whole-column `ref`s with no cells behind them, refused in milliseconds;
- an array formula with its cells, which still imports.

**Found while working.** The covered-cell clearing of merged ranges walks every cell of each range. That is T-3442,
P1.

