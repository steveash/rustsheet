---
id: T-1693
title: "A sheet-qualified reference to a sheet-scoped name (Data!Local) evaluates, and follows a sheet rename"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=Data!Local`, where `Local` is a name scoped to sheet Data, evaluates as Excel does, and keeps
working after the sheet is renamed.

## Context
Found by `probes/sheet-ops/sheet-ops-rename-sheet-scoped-name` (T-1608). The name is
`"Data!Local": "=Data!$A$1:$A$2"`, then Data is renamed to Inputs.
- Excel: `=Data!Local` is `#SPILL!` (the two cells spill into A2), and `=SUM(Data!Local)` is 3.
- The engine: `#NAME?` for both.
First check whether the engine resolves `Data!Local` with no rename, with a probe or a unit test
against existing evidence. If it does, the bug is in rename rewriting
(`rename_sheet_in_node`, `stringify.rs`). If not, it is in name resolution of sheet-qualified
names.

## Acceptance criteria
- [ ] The probe case passes.
- [ ] A unit test covers the qualified name before and after a rename.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Workbook-scoped names (already pass).

## Outcome
`=Data!Local` already evaluated; the bug was rename rewriting. `rename_sheet_in_node` skipped
`DefinedNameKind`, so the `Data!` prefix of a sheet-scoped name stayed after the rename. It now
follows the rename (`stringify.rs`). Unit tests in `test_sheet_scoped_name_qualified.rs`; probes
baseline 3268 -> 3266 known failures (the two cells of `sheet-ops-rename-sheet-scoped-name`).
Not covered: a workbook-scoped name read through a sheet prefix (`Feb!loc`) keeps its prefix
after that sheet is renamed; no Excel evidence yet for what Excel does.
