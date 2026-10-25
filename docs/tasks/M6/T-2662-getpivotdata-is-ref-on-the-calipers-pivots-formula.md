---
id: T-2662
title: "GETPIVOTDATA is #REF! on the calipers pivots (formula_stress_test Database!B16, pv23_041 Liability Summary)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GETPIVOTDATA reads the calipers pivot tables that real models query instead of `#REF!`.

## Context
Phase 1 assessment, 2026-10-25 (decision 0114). Visibly-off cells, all `#REF!` in ours:
- `calipers/roundtrip/formula_stress_test`: `Database!B16` `GETPIVOTDATA("Sum of Qux",B32)`, Excel 57;
- `calipers/pv_goldens/pv23_041`: `Liability Summary!G5`
  `GETPIVOTDATA("For Review",Pivot!$C$9,"Year","2021","Remediation Status",)` (Excel 244445.62) and
  the cells that sum it (G3, M3, O3: 9 cells).

The eval log of that run printed `RustSheet: skipped pivot cache "xl/pivotCache/pivotCacheDefinition1.xml": "a grouped field"`
and `skipped pivot table "xl/pivotTables/pivotTable2.xml": "cacheId 0 has no loaded cache"`, so
a pivot the importer skips is one suspect; the empty trailing argument in pv23_041 is another.
Find which, then fix the import or the lookup (SPEC-0015). If the fix needs new Excel evidence
for a pivot shape, probe it on excel-graph first (T-2085 lists unprobed shapes).

## Acceptance criteria
- [ ] The cause of each `#REF!` is named in the Outcome.
- [ ] The cells above are not visibly off, or what remains has a filed task with its evidence need.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- Pivot shapes no calipers cell needs (T-2085).
