---
id: T-2052
title: "CELL(\"filename\") is #VALUE! in Excel for the web: score it and match it"
milestone: M5
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`CELL("filename")`, `CELL("filename",B2)` and `CELL("filename",Data!B2)` give what excel-graph (the oracle of
record, decision 0039) gives, and the eval scores them.

## Context
- probes/info-cell-address-cross-sheet Calc!B1:B4 (excel-graph, 2026-10-19, T-2042) are all `#VALUE!`. They are
  outside the case's `check` list, so unscored. Ours: `[<file>.xlsx]<sheet>` for a named workbook, `#N/IMPL!` for an
  unnamed one (`cell_info` "FILENAME" arm, crates/rustsheet-engine/src/functions/information/mod.rs).
- docs/excel-semantics/information.md "CELL("address") of a reference on another sheet names the workbook file"
  records the finding. Desktop Excel returns the full path (`C:\dir\[Book.xlsx]Sheet1`), unverified here: that
  question is M14 (desktop oracle), not this task.
- The engine test `cell_filename` (crates/rustsheet-engine/src/test/test_cell_info_n_sheets.rs) pins the IronCalc
  form; it changes with the rule.

## Acceptance criteria
- [ ] Calc!B1:B4 join the case's `check` list and pass; no new known failures.
- [ ] information.md says what "filename" returns and why (web evidence; desktop path form left to M14).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's path form.

## Outcome
`CELL("filename")` is `#VALUE!` in every form (web oracle); Calc!B1:B4 now scored and pass. information.md updated.
