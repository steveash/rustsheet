---
id: T-2834
title: "Write a sheet-prefixed workbook name with the workbook's file name as its prefix, and keep its value across a sheet rename"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula that reads a workbook-scoped name through a sheet prefix (`=Feb!loc`, no `Feb!loc` defined) shows the
workbook's file name as the prefix in FORMULATEXT, as Excel does, and keeps its value when that sheet is renamed.

## Context
T-1701 probed it (excel-graph, 2026-10-26; docs/excel-semantics/references.md "A sheet-qualified workbook name is
rewritten to the workbook's file name"). Excel rewrites `Feb!loc` to `<file>.xlsx!loc` on load, the file name quoted as
a sheet name would be (`'53aa…xlsx'!loc` for a name starting with a digit, `ebfe…xlsx!loc` otherwise); values are
unchanged (5, `SUM` 5, `+1` 6, ISREF TRUE) and a rename of Feb leaves the formula alone.
Ours: FORMULATEXT keeps `=Feb!loc` (6 cells), and after renaming Feb to Mar both values are `#NAME?` (Excel 5):
the rename keeps the `Feb!` prefix of a scope-`None` name (`rename_defined_name_sheet`, expressions/utils/mod.rs) and
the re-parse finds no sheet Feb (`prefixed_name_operand`, parser/operands.rs).
Failing cells (eval/baseline/probes.txt): probes/sheet-ops/sheet-ops-workbook-name-via-prefix-no-rename Calc!B1:B3,
probes/sheet-ops/sheet-ops-rename-workbook-name-via-prefix Calc!A1:B2,
probes/defined-name-qualified-like-reference Sheet1!A8 (`=FORMULATEXT` of `=Data!Q1_Total`; added by T-2635).
Design points: the parser needs the workbook file name (`Workbook::name` + ".xlsx", as `workbook_file_name` in
functions/information/mod.rs) and must read `<file>.xlsx!loc` back as the workbook name; the xlsx writer must not emit a
prefix Excel cannot read (probe what Excel stores; `[0]!loc` is a guess); the quoting rule rests on two file names, so
probe a file name with a space and one with only letters and digits first; an unnamed workbook is unprobed.

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the ranges above hold):
- probes/sheet-ops/sheet-ops-rename-workbook-name-via-prefix Calc!A2

## Acceptance criteria
- [x] The 8 cells above leave eval/baseline/probes.txt (`make eval-baseline`), no regressions.
- [x] An xlsx round trip of such a formula keeps its value (unit test); references.md says ours matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- A sheet-scoped name read through its own sheet's prefix (follows a rename since T-1693).

## Outcome
The parser knows the workbook's file name (`Parser::set_workbook_file`, from `Workbook::file_name`, now
the one definition CELL also uses) and writes a workbook name read through a sheet (`Feb!loc`, the
formula's own sheet, `[0]!loc`) as `<file>.xlsx!loc`, quoted as a sheet name would be; it reads that
text back (any case) as the workbook name. Stored (R1C1) and exported text is `[0]!loc`: a new probe
(probes/workbook-name-own-book-prefix, excel-graph) shows Excel loads `[0]!loc` from a file as the
workbook name with the file-name text, even on a sheet with its own `loc`. So sheet renames, workbook
renames (`UserModel::set_name`) and xlsx round trips keep it (unit tests in the engine and the xlsx
crate). Eval: probes known failures 2988 -> 2980 (the 8 cells above), new case 13/13, no regressions.
