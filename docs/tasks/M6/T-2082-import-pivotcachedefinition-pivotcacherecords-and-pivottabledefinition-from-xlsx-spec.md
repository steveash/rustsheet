---
id: T-2082
title: "Import pivotCacheDefinition, pivotCacheRecords and pivotTableDefinition from xlsx (SPEC-0015)"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-2081]
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
A workbook's pivot caches and pivot table definitions are imported from xlsx into the model of T-2081, bounded for untrusted files.

## Context
SPEC-0015 "xlsx import" and "Untrusted-input limits": new `crates/rustsheet-xlsx/src/import/pivots.rs` reading workbook `<pivotCaches>`, pivotCacheDefinition (worksheet sources only), pivotCacheRecords and each sheet's pivotTableDefinition; skipped (never a panic) on OLAP, grouped or calculated fields, bad indices or a limit; limits logged and recorded as a DEVIATION. Depends on T-2081.

## Acceptance criteria
- [x] Unit tests: a hand-written minimal package with shared and inline record values, `saveData="0"`, out-of-range indices, an OLAP source, and each limit.
- [x] Relationship targets through package.rs `resolve_target`; External, escaping or missing targets and duplicate cacheIds skip the pivot (tests).
- [x] A test proves `resolve_target` rejects `..` escapes and absolute paths (it returns a `String`; tighten it if it only normalizes).
- [x] The limits are totals counted while parsing (SPEC-0015), and the DEVIATION entry is in web.md in the same PR.
- [x] A test workbook in crates/rustsheet-xlsx/tests with a pivot imports its cache and definition.
- [x] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).

## Outcome
`crates/rustsheet-xlsx/src/import/pivots.rs` streams the workbook's `<pivotCaches>`, each pivotCacheDefinition
(worksheet sources), its pivotCacheRecords and each sheet's pivotTableDefinition parts into `Workbook::pivot_caches`
and `pivot_tables`, through the T-2081 `CacheBuilder` (one cache builder, its limits counted before each push). Every
part is pulled with quick-xml events, never built into a tree. A cache or pivot the model cannot hold faithfully
(OLAP/external source, `fieldGroup` or calculated field, no records part, an index past its cache, `showDataAs`, a
limit, an External/absolute/escaping/missing target, a duplicate `cacheId`, a part reached twice) is skipped whole
and logged, so GETPIVOTDATA on it is `#REF!`. Relationship targets go through the new
`package.rs::resolve_relative_target`, which refuses absolute targets and `..` past the package root (the existing
`resolve_target` keeps accepting absolute targets, which OPC allows and other parts use); `Relationship` gains
`external` and moves to `import/mod.rs` with a shared `parse_relationships`. web.md gains the DEVIATION entry for
the limits and the list of skipped forms. `tests/pivot_tables.xlsx` (probes/getpivotdata-real-pivot built by
spec2xlsx) imports both caches and pivots and answers GETPIVOTDATA with excel-graph's values.
Eval delta: none (spec cases reach pivots through T-2084's adapter, not the importer); 0 regressions.
