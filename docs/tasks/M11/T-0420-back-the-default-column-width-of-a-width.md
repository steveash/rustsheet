---
id: T-0420
title: "Back the default column width of a width-less col element with oracle evidence (8.43 vs 10)"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The width a `<col>` without a `width` attribute takes on import matches Excel, backed by oracle
evidence, instead of the engine's 10 character units.

## Context
T-0418 made `width` optional on import. A width-less `<col>` takes `<sheetFormatPr defaultColWidth>`,
else `DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR` = 10 units
(`crates/rustsheet-xlsx/src/import/columns.rs`). Upstream IronCalc and Excel's usual default are
8.43 (derived from `baseColWidth` 8 plus padding), and `baseColWidth` is ignored. No formula observes
column width today (no `CELL("width")`), so this is parity for export and a future `CELL("width")`.
Use the `excel-probe` skill if a probe is possible; record the finding in `docs/excel-semantics/`.

## Acceptance criteria
- [ ] The fallback width is justified by cited evidence (probe or Excel-authored workbook), with a `docs/excel-semantics/` entry.
- [ ] A test covers the width-less `<col>` with and without `baseColWidth`/`defaultColWidth`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column width round-trip precision on export.
