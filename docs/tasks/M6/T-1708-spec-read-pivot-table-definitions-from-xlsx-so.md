---
id: T-1708
title: "Spec: read pivot table definitions from xlsx so GETPIVOTDATA can resolve field/item pairs"
milestone: M6
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ says how the engine keeps the pivot tables of a loaded workbook (location, row/column/data fields, rendered items) and how GETPIVOTDATA resolves its data_field and field/item pairs against them, so the probes of probes/getpivotdata-real-pivot can pass.

## Context
T-1705 made the case builder write pivot tables; probes/getpivotdata-real-pivot (33 known-failure cells, `#NAME?` today: the engine has no GETPIVOTDATA) holds Excel's answers. Semantics: docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table" and "GETPIVOTDATA without a pivot table" (T-1703). The eval engine adapter ignores spec `pivot_tables` (crates/rustsheet-eval/src/case.rs `PivotTableSpec`), so the spec must also say how a spec case's pivots reach the engine (the adapter, or xlsx import of the built file). Rendered pivot cells are plain values in the file; Excel may refresh the cache on load (`refreshOnLoad`).

## Acceptance criteria
- [ ] docs/specs/ has the spec: data model, xlsx import of pivotCacheDefinition/pivotTableDefinition, GETPIVOTDATA lookup rules with the probe cells they cover, untrusted-input limits, and implementation tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it; refreshing or editing pivots; PIVOTBY (a function, not a pivot table).
