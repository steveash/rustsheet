---
id: T-2042
title: "CELL(\"address\") of a reference on another sheet names the workbook file, as Excel does"
milestone: M5
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`CELL("address",Data!B2)` on another sheet gives Excel's `[<workbook file name>]Data!$B$2` instead of
`#N/IMPL!`, from a workbook file name the engine knows, and the eval scores the oracle cell.

## Context
- Failing cell (gap triage 2026-10-19, `make eval`): probes/info-logic-argument-modes Calc!A19
  `CELL("address",Data!B2)`: Excel (excel-graph, 2026-10-09)
  `[e93a64b8fcc19bb3f7cf233ba1950bac7796f76ae1473c306f25f66653d77869.xlsx]Data!$B$2`, ours `#N/IMPL!`
  (`cell_info` "ADDRESS" arm, crates/rustsheet-engine/src/functions/information/mod.rs, refuses
  `reference.sheet != cell.sheet`). No open task covered it; information.md ("CELL" bullet and the
  T-1218 note) records the rule and that it is left unimplemented because the engine does not know
  the file name.
- The bracketed name is the oracle's upload name: `ExcelGraphBackend.recalc`
  (tools/src/rustsheet_tools/oracle/backends.py) uploads as `sha256(xlsx).hexdigest() + ".xlsx"`.
  It is not the sidecar's `input_sha256` (2322387e... for this case), so today nothing in `eval/`
  lets the adapter reproduce it. information.md also notes the T-1433 run saw the same shape for
  `CELL("address",Data!A2)`, unscored.
- Design questions this task settles (opus: no spec yet, touches engine state and the harness):
  1. Where the workbook file name lives: `Workbook.name` (crates/rustsheet-engine/src/types.rs) set
     from the xlsx path at load, or a new field set through the facade; what a workbook created in
     memory reports (Excel's unsaved `Book1` has no extension; unprobed, so either probe it or keep
     `#N/IMPL!` for an unnamed workbook rather than guess).
  2. How the harness gets the oracle's name: record the upload name in the oracle sidecar (a
     `source.workbook_name` field written by the fill) and have the adapter set it, or score such
     cells by pattern; decide, without hand-editing expectations.
  3. Quoting: a sheet name that needs quotes (`'My Sheet'`) and the name's place in the quotes
     (`'[book.xlsx]My Sheet'!$A$1` in Excel's formula syntax) are unprobed; probe them
     (excel-probe skill) before implementing, or leave them `#N/IMPL!`.
- Related: `CELL("filename")` uses the same name (check whether it is implemented; same source).

## Acceptance criteria
- [x] probes/info-logic-argument-modes Calc!A19 passes, and its known-failure line leaves
      eval/baseline/probes.txt with no new ones (`make eval-baseline`).
- [x] A probe with a quoted sheet name and a same-sheet control is oracle-filled and passes, or the
      quoted form is filed as a follow-up and stays `#N/IMPL!`.
- [x] information.md's CELL entry says what is implemented and where the name comes from; the
      T-1218 "unimplemented" note is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `CELL("filename")` path text beyond the bracketed file name (directory part), if it needs more evidence: file a follow-up.

## Outcome
- Design: the oracle fill records the file name the workbook had in the oracle as the sidecar's
  `source.workbook_name` (excel-graph: the upload name `<sha256>.xlsx`; desktop and LibreOffice:
  `in.xlsx`). The eval runner passes it to the engine through `Engine::evaluate_{xlsx,spec}` and the new
  facade option `WorkbookOptions::name` (file name without `.xlsx`, applied at `new` and at load). An
  unnamed workbook (`None`, the default) gives `#N/IMPL!` for cross-sheet `CELL("address")` and for
  `CELL("filename")` (Excel's unsaved form is unprobed); before, the facade reported `[workbook.xlsx]`.
- Evidence: new probe probes/info-cell-address-cross-sheet (excel-graph, 2026-10-19): the sheet name is
  never quoted (`[f.xlsx]My Sheet!$B$2`, `[f.xlsx]Bob's!$A$1`), a range gives its top-left, same-sheet
  stays bare. `CELL("filename")` is `#VALUE!` on the web (unscored; follow-up T-2052). The
  info-logic-argument-modes sidecar was refilled to record its name; only A19 and `source` changed.
- Eval delta: fixed 1 (probes/info-logic-argument-modes Calc!A19), new coverage 9 passing cells
  (probes/info-cell-address-cross-sheet), regressions 0.
- Follow-ups: T-2052 (CELL("filename") on the web). A union on another sheet stays `#N/IMPL!` (unprobed).
