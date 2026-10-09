---
id: T-1250
title: "Probe an Excel-saved workbook's _xlnm. built-in names (Print_Area, Print_Titles, _FilterDatabase, Criteria) on load and save"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for how Excel stores and reads its `_xlnm.` built-in names, so `name_from_xlsx`/`name_to_xlsx` (crates/rustsheet-xlsx/src/util.rs) can cover the full ECMA-376 list or stay narrow.

## Context
T-1193 maps only Print_Area, Print_Titles and _FilterDatabase (prefix dropped on import, restored on export). Open questions: does a user-defined `Criteria`, `Database` or `Extract` save as `_xlnm.`? Does `=ROWS(_FilterDatabase)` resolve in a workbook with an AutoFilter? What does Excel write back for a file holding both `_xlnm.Print_Area` and a plain `Print_Area` in one scope? Also: export (`export/workbook.rs`) unwraps the sheet index of a sheet-scoped name, which panics if the scope sheet is gone; make it an error or drop the scope. The oracle needs an Excel-saved workbook, not spec2xlsx output (xlsxwriter writes `Print_Area` without the prefix).

## Acceptance criteria
- [ ] Findings in docs/excel-semantics/xlsx-storage.md "Built-in defined names", with the evidence cited.
- [ ] BUILT_IN_NAMES matches the evidence; the export unwrap is gone.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
