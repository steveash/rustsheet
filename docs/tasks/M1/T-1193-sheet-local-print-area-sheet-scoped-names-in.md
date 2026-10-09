---
id: T-1193
title: "Sheet-local Print_Area: sheet-scoped names in the case loader and _xlnm. names (SPEC-0004 probe row 14)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
probes/multiarea-printarea (SPEC-0004 probe row 14) passes: a sheet-local `Print_Area` = `Sheet1!$A$1,Sheet1!$B$2` gives `SUM(Print_Area)` 6 and `AREAS(Print_Area)` 2, both from a case spec and from an xlsx file.

## Context
Split from T-1065, which made union-bodied names evaluate as their areas (a global `mu` works). The
probe still fails with `EngineError: invalid defined name`, for two reasons outside multi-area work:
1. The case spec names it `"Sheet1!Print_Area"` (xlsxwriter's syntax for a sheet-scoped name; spec2xlsx
   writes `_xlnm.Print_Area` with `localSheetId`), but `rustsheet-eval`'s spec loader
   (crates/rustsheet-eval/src/engine.rs) calls `Txn::define_name(name, formula)`, which has no scope
   and validates `Sheet1!Print_Area` as an identifier. It needs a scoped define (an addition to the
   `Txn` API in crates/rustsheet/src/edit.rs, with undo) and the loader must split `Sheet!Name`.
2. xlsx stores built-in names as `_xlnm.Print_Area`; formulas write `Print_Area`. Check what the
   importer and the parser do with the `_xlnm.` prefix (an xlsx with a Print_Area and `=SUM(Print_Area)`),
   and how export writes it back (it must stay `_xlnm.Print_Area` with `localSheetId`).
Evidence: probes/multiarea-printarea.oracle.json (excel-graph, 2026-10-07).

## Acceptance criteria
- [ ] probes/multiarea-printarea H1:H2 pass; baseline updated.
- [ ] An xlsx with a sheet-local `_xlnm.Print_Area` loads, `=SUM(Print_Area)` evaluates, and export keeps the name (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other `_xlnm.` names (Print_Titles, _FilterDatabase) beyond loading without error.
