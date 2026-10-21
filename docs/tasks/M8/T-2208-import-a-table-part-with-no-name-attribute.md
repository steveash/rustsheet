---
id: T-2208
title: "Import a table part with no name attribute by its displayName, as Excel does (poi/LIBRE_OFFICE-128382-0.xlsx)"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
A workbook whose table part has a `displayName` but no `name` attribute loads, and its structured
references resolve through the displayName, as in Excel. Today the whole workbook fails to import.

## Context
Gap triage 2026-10-20 (poi suite fetched and run locally). `poi/LIBRE_OFFICE-128382-0.xlsx` (written by
LibreOffice) is `!EngineError` in eval/baseline/poi.txt: `import failed: invalid workbook: XML Error:
Corrupt XML structure: missing table name`. Its `xl/tables/table1.xml` is
`<table id="1" displayName="Таблица1" ref="A2:E13" totalsRowCount="1">` with no `name`, and the totals
row of sheet "Basic features" reads it by that displayName: B13 `SUBTOTAL(109,Таблица1[[Sum for Q1 ]])`,
C13 `SUBTOTAL(109,Таблица1[[ Sum for Q2 ]])`, D13 `SUBTOTAL(109,Таблица1[Sum for Q3])`, E13
`SUBTOTAL(109,Таблица1[Sum for Q4])`. Excel for the web loads the file and computes them:
eval/suites/poi/oracle/LIBRE_OFFICE-128382-0.oracle.json (excel-graph, 2026-10-04) has B13 19239.04,
C13 25215.790000000005, D13 28393.92, E13 27527.01 (D22 `TODAY()` and E22 `NOW()-TODAY()` are volatile
and skipped).

Code: `crates/rustsheet-xlsx/src/import/tables.rs` (around line 38) requires `name` and errors without
it; it also reads `display_name` from the `name` attribute rather than `displayName` (both lookups say
`"name"`). Read `displayName` for the display name, and fall back to it when `name` is missing (and to
`name` when `displayName` is missing); a part with neither stays an import error. This file's formulas name
the table by its displayName (B13:E13 above); if the engine keys tables by `name`, check which one
structured references resolve through, and check the exporter writes both attributes. Keep the change in the importer; no new
helper if an attribute-with-fallback one exists (`rg "attribute\(" crates/rustsheet-xlsx/src/import`).

To run the poi suite locally: `target/release/rustsheet-eval fetch --suite poi` (clones apache/poi at
the pinned rev into eval/.cache/suites/poi), then `make eval` or
`rustsheet-eval run --suite poi --filter LIBRE_OFFICE-128382`.

## Acceptance criteria
- [ ] `F poi/LIBRE_OFFICE-128382-0.xlsx !EngineError` leaves eval/baseline/poi.txt; the case loads and
      Basic features!B13:E13 pass bit-exact (no new F line for this case, or the Outcome lists any other
      cell of it with its cause).
- [ ] An importer test loads a table part with `displayName` and no `name` and resolves a structured
      reference to it; it fails before the change.
- [ ] Importer tests for untrusted names: a part with neither `name` nor `displayName` is still an import
      error (no panic), and an empty, non-ASCII or duplicate `displayName` is an error or a loaded table,
      never a panic.
- [ ] docs/excel-semantics/xlsx-storage.md "A table part without a name attribute is named by its
      displayName" gets the implementation line.
- [ ] No new line in any baseline; `make check` green; reviewers approve.

## Out of scope
- The other poi `!EngineError` cases (47889 without `cellXfs`, 60825 with an empty `r:id`,
  invalid_cell_reference, style-alternate-content, 56278 and 59021 with missing zip parts): no Excel
  evidence that they load (decision 0012), and the XML bombs are a directive-5 refusal.

## Visible-parity stage
Decision 0083: a workbook Excel opens that we refuse is wrong for every cell in it. The acceptance
criteria above are the visible-parity stage.
