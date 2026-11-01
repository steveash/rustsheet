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
The importer now bounds `array_cell` instead of restructuring it. `import/array_refs.rs` keeps one `ArrayClaims` tally
per load, passed through `load_sheet` and `read_sheet_data` (`SheetPart`). It counts the cells array formula `ref`s
cover against the bytes of the workbook's sheets read so far, using the XML reader's position. Every `ref` is charged
before any of its cells goes into the map. Two cases are refused:
- one `ref` of more than one whole column of cells, which the load check (T-2896) refuses later anyway;
- `ref`s covering more than one cell per 16 bytes read, plus 4 x 2^20 once per load.

So memory and time are proportional to the bytes the file actually holds.

**Why this bound.**
- **What Excel's files can hit.** Each anchor is checked against the bytes read before it, and it comes before its
  cells. So more than four whole-column spills anchored ahead of their cells (`=A:A+1` in five columns of row 1,
  about 130 MB) are refused even though Excel wrote the file. This is accepted as a DEVIATION, the price of not
  trusting the declared size. Once the cells are read, a covered cell has paid for itself: at least 22 bytes in
  Excel's files, which write `r`, and 14 without it.
- **Accepted ceiling.** A hostile 256 MiB package still makes the map about 21M cells (1 to 1.5 GB) in the worst
  case (security review).
- **The repository's files fit.** The Excel-fidelity review scanned 2,039 xlsx files: 45,815 array formulas on
  317 sheets. Under the prefix rule none of the 235 workbooks with arrays is refused; the tightest claims about
  30,000 cells ahead of 4.19M allowed.
- **The slack is per load, not per sheet.** Otherwise about 5,000 tiny sheets would each get a column's worth of map
  entries (Excel-fidelity review).
- **It counts bytes read, not the size the zip declares.** A tiny part declaring 256 MiB would otherwise buy about
  17M map entries (security review).

**Why not stream a structure of open anchors.** The suggested BTreeMap of anchors whose rows are still open would need
cells in row order. The importer does not enforce that: a row's `r` is free, and so is a cell's.

**Records.** The refusal is a DEVIATION, recorded in docs/excel-semantics/xlsx-storage.md. Whether Excel opens such a
file is part of probe T-3436.

**Tests.**
- `tests/malformed_import.rs`:
  - a `ref` of two whole columns;
  - a thousand whole-column `ref`s in one sheet, refused quickly (no timing assertion);
  - an array formula with its cells, which still imports.
- `array_refs.rs` unit tests:
  - many sheets share one slack;
  - bytes read pay for more arrays;
  - one array past a whole column is refused.

**Found while working.** The covered-cell clearing of merged ranges walks every cell of each range. That is T-3442,
P1.
