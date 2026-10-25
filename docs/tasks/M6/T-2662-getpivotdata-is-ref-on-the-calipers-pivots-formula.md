---
id: T-2662
title: "GETPIVOTDATA is #REF! on the calipers pivots (formula_stress_test Database!B16, pv23_041 Liability Summary)"
milestone: M6
lane: functions-lookup
status: done
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
- [x] The cause of each `#REF!` is named in the Outcome.
- [x] The cells above are not visibly off, or what remains has a filed task with its evidence need.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- Pivot shapes no calipers cell needs (T-2085).

## Outcome
Both pivots import; the `#REF!`s came from GETPIVOTDATA itself.
- pv23_041 `Liability Summary` G5:G7 (and the cells summing them): the omitted trailing item argument
  (`"Remediation Status",`) is Excel's way of picking the `(blank)` item (`<m/>` in the cache); ours read it as a
  non-text item. An omitted item now picks the field's one blank item. `""`, a blank cell and `"(blank)"` stay
  `#REF!` (unprobed).
- formula_stress_test `Database!B16`: pivot GPD has two row and two column fields, a shape `supported` refused
  (SPEC-0015 unknown 10). Such a pivot now answers its no-pair grand total; any pair stays `#REF!` (T-2085).
- web.md "GETPIVOTDATA on real-model pivots" records both, plus a last-bit difference between two pivots'
  totals of the same records (unknown 11). SPEC-0015 GETPIVOTDATA step 3 updated.
Eval delta: calipers pv23_041 now passes all 446 cells; formula_stress_test B16 passes (see the PR for the
baseline count).
