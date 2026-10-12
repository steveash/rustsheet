---
id: T-1267
title: "DateSystem in WorkbookSettings: import/export date1904, facade option (SPEC-0005 step 1)"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1266]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
A workbook knows its date system: `DateSystem` is in `WorkbookSettings`, read from and written to
xlsx `workbookPr@date1904`, and settable for new workbooks through the facade. No value changes.

## Context
SPEC-0005 "Design" (Model, Import/export, Facade) and rollout step 1. Code: `types.rs`
(`WorkbookSettings`), `new_empty.rs`, `rustsheet-xlsx/src/import/{mod,workbook}.rs`, `export/`,
`crates/rustsheet/src/workbook.rs` (`WorkbookOptions`), `bindings/python`.

## Acceptance criteria
- [x] Import maps `date1904` `1`/`true` to 1904, absent/`0`/`false`/garbage to 1900 (unit tests, no error on garbage).
- [x] `rustsheet-eval`'s `xlsx_snapshot.rs` (and `calibrate.rs`'s hardcoded `date1904: false`) reuse the importer's value; the `1`/`true` rule exists once.
- [x] Export writes `<workbookPr date1904="1"/>` only for 1904; 1900 exports are unchanged; round-trip test.
- [x] `WorkbookOptions::date_system` (default 1900) and a read-only getter; `rustsheet` re-exports a documented `DateSystem`; Python mirrors both as `"1900"`/`"1904"`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any formatter or function behavior (SPEC-0005 steps 3-4).

## Outcome
`DateSystem` (`Excel1900` default / `Excel1904`) is in `WorkbookSettings`, with `WorkbookSettings::new`
for the 1900 default. The xlsx importer reads `workbookPr@date1904` through the public
`date_system_from_attribute` (`1`/`true` is 1904; anything else is 1900, no error), which
`rustsheet-eval`'s `xlsx_snapshot.rs` now reuses; the exporter writes `<workbookPr date1904="1"/>`
only for 1904, so 1900 exports are unchanged. `WorkbookOptions::date_system` and
`Workbook::date_system()` in the facade (re-exporting `DateSystem`), and `rustsheet.Workbook(date_system="1900"|"1904")`
plus a `date_system` property in Python. No value changes; eval delta none.
