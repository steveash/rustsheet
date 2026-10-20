---
id: T-2082
title: "Import pivotCacheDefinition, pivotCacheRecords and pivotTableDefinition from xlsx (SPEC-0015)"
milestone: M6
lane: engine
status: todo
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
- [ ] Unit tests: a hand-written minimal package with shared and inline record values, `saveData="0"`, out-of-range indices, an OLAP source, and each limit.
- [ ] A test workbook in crates/rustsheet-xlsx/tests with a pivot imports its cache and definition.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).
