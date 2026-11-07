---
id: T-4177
title: "Round-trip suite: engine-exported pivots re-rendered by Excel"
milestone: M16
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4176]
---

## Goal
A suite whose cases are exported by the engine and sent to the oracle: a refusal or repair fails; Excel's re-render and rowItems must equal the engine's.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 4. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Suite and harness support; first cases filled.
- [x] Cases for what T-4176's export left UNVERIFIED: an API-built pivot's generated `<items>` re-rendered
  (`refreshOnLoad` on); a pivot without `refreshOnLoad` whose saved `rowItems` are stale (the export writes them
  back as read); a pivot whose render collapsed to `#SPILL!` (the export writes it without `rowItems`/`colItems`);
  cache dates at serials 1 to 60, the phantom serial 60 (written as `1900-02-28T00:00:00`) and a
  1904 workbook's cache (docs/excel-semantics/xlsx-storage.md "Pivot cache dates below serial 61 use the
  1899-12-30 epoch"). The three T-4176 scratch runs (pivots.md "Excel for the web opens engine-exported pivot
  parts") become committed cases.
- [x] Cases for T-4734's grouped-cache export (pivots.md "Excel for the web opens engine-exported grouped pivot
  caches"): its scratch runs become committed cases. They are the engine exports of probes/t4170-pivot-manual-groups,
  t4170-pivot-date-levels, t4236-pivot-date-groups-no-blank, t4228-pivot-range-groups and t4128-pivot-spike (G),
  plus the API-built cache: Amount by tens, Date by months with Quarters and Years, and Product2 as a manual group.
  Add one more case for a date field grouped in place whose `sharedItems` carry no `containsNonDate`,
  `minDate` or `maxDate`. It commits the "flags needed" result: Excel's refresh dropped the months.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A new suite kind, `export`, and the gated cells-baseline suite eval/suites/pivot-export (M16) score the engine's xlsx
export against Excel opening it (docs/architecture/eval-harness.md "Engine-export suites").
- Harness: a case is a `<name>.export.json` (crates/rustsheet-eval/src/export_case.rs): a `source` (a spec case or
  an xlsx, relative to the repository root, no `..`) or an inline `spec`, `pivots` added through `Txn::add_pivot_table`, and `patches`,
  text edits of the saved package that must apply (a `refreshOnLoad` cache the API cannot set; flags removed).
  `rustsheet-eval export <case> --out <file>` writes the package (deterministic); the oracle fill sends that file
  (`snapshot.engine_export`), and writes a `refused` sidecar when Excel will not open it (`OracleRefusedError` on
  createSession's refusal codes). The runner builds the export on an isolated worker and scores it as an xlsx case:
  formula cells and every sidecar pivot-area cell (values and text). The engine's own `rowItems`/`colItems`, read
  back from its save (`Evaluated::saved_package`, `pivot_scoring::saved_lines`), give `LinesDiffer` in "Pivot
  layout" (reported, not ratcheted: T-4944). `refused` makes the case `excel_refused` (SPEC-0033), and a sidecar of
  another package than the current export is noted on the result. `rustsheet-eval cases` moved to case_listing.rs.
  T-4180 can extend the kind to the general goldens.
- Cases (18, excel-graph 2026-11-07; Excel opened every one, no `excel-refused`): the T-4176 runs t4177-example,
  -pivot-tables, -api-items (and -api-items-refresh, generated `<items>` re-rendered); -stale-row-items (t4586 A's
  stale lines saved back as read); -api-spill and -api-spill-refresh (a `#SPILL!` collapse); -cache-dates-1900,
  -cache-dates-1904 and their -refresh variants (serials 1 to 62 with the phantom 60, and a 1904 cache); the T-4734
  runs -manual-groups, -date-levels, -date-groups-no-blank, -range-groups, -spike, -grouped-cache (the API-built
  cache's shape as builder definitions: the facade takes one range grouping, so no levels or manual groups) and
  -date-flags-needed (Excel's refresh dropped the months).
- Scored: 1989 cells, 1814 passing (91.2%) at the first fill; 1826 (91.8%) after rebasing onto T-4768's label sort,
  which fixes the t4177-spike K cells. 9 cases pass in full. The 175 failing cells (163 after T-4768): the probes' own GETPIVOTDATA
  failures and typed-item / collapsed grand-total GETPIVOTDATA (T-4154, owner lines added), t4128 K's sort (T-4768),
  the flags-needed grouping (T-4941), the exported collapsed pivot (T-4942). Text only: date items render as serials
  (T-4943).
- pivots.md "Excel for the web opens engine-exported pivot parts" and "... grouped pivot caches" are VERIFIED from
  these cases; xlsx-storage.md "Pivot cache dates below serial 61" verifies serials 1 to 61 and 1904 cache dates.
- Filed: T-4941, T-4942, T-4943 (engine), T-4944 (harness).

