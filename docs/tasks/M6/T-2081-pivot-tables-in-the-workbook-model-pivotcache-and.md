---
id: T-2081
title: "Pivot tables in the workbook model: PivotCache and PivotTable types (SPEC-0015)"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Pivot caches and pivot tables are part of the workbook model, survive structural edits like tables do, and are available to evaluation.

## Context
SPEC-0015 "Data model": add `pivot.rs` types (`PivotCache` with columnar interned records, `CacheField`, `CacheItem`, `PivotTable`, `PivotField` (hidden bitset), `CellArea`, `AxisField`, `DataField`, `Aggregate`) and `Workbook::pivot_caches`/`pivot_tables`; location is a 0-based `CellArea` and the sheet a stable `sheet_id` (api-design.md principle 3); until T-2086, a structural edit (row/column insert or delete, sheet rename or delete) on a sheet holding pivots drops them; a new workbook has none. xlsx export keeps dropping pivots (note it in the exporter's skipped-features list).

## Acceptance criteria
- [x] The types and Workbook fields exist with doc comments; unit tests cover a structural edit dropping the sheet's pivots (T-2086 moves them later).
- [x] One cache builder (columnar, interning, counting the SPEC-0015 limits) for the importer (T-2082) and `Txn::add_pivot_table` (T-2084).
- [x] `rg` for an existing area type before adding `CellArea` (expressions/types.rs `Area` carries a sheet and sizes; reuse one if it fits). The hidden bitset is sized at import, and a `get()` past its length reads as not hidden.
- [x] The interim gap (a structural edit drops the sheet's pivots) is recorded in docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table".
- [x] No behavior change for workbooks without pivots (eval unchanged).
- [x] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).

## Outcome
`crates/rustsheet-engine/src/pivot.rs` adds the SPEC-0015 types (`PivotCache`, `CacheSource`, `CacheField`, `CacheItem`,
`PivotTable`, `PivotField` with its hidden bitset, `CellArea`, `Axis`, `AxisField`, `DataField`, `Aggregate`) and
`Workbook::pivot_caches`/`pivot_tables` (empty for a new or imported workbook until T-2082). `CacheBuilder` is the one cache
builder: columnar records, items interned per field (shared items keep their positions), and the cache limits (fields,
records, items, record values against a workbook budget) checked before each push, with a thiserror `PivotCacheError`.
`CellArea` is new: the engine's `Area` carries a sheet index and 1-based i32 sizes and `RangeRef` lives in the API crate.
Insert, delete and move of rows or columns, and sheet rename and deletion, drop that sheet's pivots (`Workbook::drop_pivots_on_sheet`,
unit-tested); recorded in web.md, with export dropping pivots noted on `save_xlsx_to_writer`.
Eval delta: none (no workbook has pivots yet).
