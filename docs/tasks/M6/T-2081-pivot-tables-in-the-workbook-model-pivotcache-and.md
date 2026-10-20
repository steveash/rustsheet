---
id: T-2081
title: "Pivot tables in the workbook model: PivotCache and PivotTable types (SPEC-0015)"
milestone: M6
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Pivot caches and pivot tables are part of the workbook model, survive structural edits like tables do, and are available to evaluation.

## Context
SPEC-0015 "Data model": add `pivot.rs` types (`PivotCache`, `CacheField`, `CacheValue`, `PivotTable`, `PivotField`, `AxisField`, `DataField`, `Aggregate`) and `Workbook::pivot_caches`/`pivot_tables`; move or drop pivots on row/column insert/delete and sheet rename/delete as `Table` is handled; a new workbook has none. xlsx export keeps dropping pivots (note it in the exporter's skipped-features list).

## Acceptance criteria
- [ ] The types and Workbook fields exist with doc comments; unit tests cover structural edits moving, shrinking and dropping a pivot.
- [ ] No behavior change for workbooks without pivots (eval unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).
