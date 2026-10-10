---
id: T-1267
title: "DateSystem in WorkbookSettings: import/export date1904, facade option (SPEC-0005 step 1)"
milestone: M1
lane: engine
status: todo
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
- [ ] Import maps `date1904` `1`/`true` to 1904, absent/`0`/`false`/garbage to 1900 (unit tests, no error on garbage).
- [ ] `rustsheet-eval`'s `xlsx_snapshot.rs` (and `calibrate.rs`'s hardcoded `date1904: false`) reuse the importer's value; the `1`/`true` rule exists once.
- [ ] Export writes `<workbookPr date1904="1"/>` only for 1904; 1900 exports are unchanged; round-trip test.
- [ ] `WorkbookOptions::date_system` (default 1900) and a read-only getter; `rustsheet` re-exports a documented `DateSystem`; Python mirrors both as `"1900"`/`"1904"`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any formatter or function behavior (SPEC-0005 steps 3-4).
