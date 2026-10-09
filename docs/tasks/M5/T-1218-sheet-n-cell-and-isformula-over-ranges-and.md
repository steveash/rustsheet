---
id: T-1218
title: "SHEET, N, CELL and ISFORMULA over ranges and arrays as excel-graph gives them"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SHEET, N, CELL and ISFORMULA give excel-graph's results for range and array arguments (probes/info-logic-argument-modes).

## Context
T-1088 probed how the M5 information functions take ranges and arrays (probes/info-logic-argument-modes and -2, excel-graph 2026-10-09;
docs/excel-semantics/information.md "Information function argument modes"). The engine misses these cells:
- `SHEET(Data!A2:C2)` is 1 (the range's sheet); the engine gives `#N/A` (A12).
- `N(Data!A2:C2)` is 2 and `N(Data!B2:C2)` from a row that shares no row or column with it is 3: the top-left cell, with no lift and no
  implicit intersection; the engine gives `#N/IMPL!` (A6; -2 A7).
- `CELL("contents",Data!A2:C2)` is `#VALUE!` (the engine gives 2, A18), and `CELL({"row","col"},Data!B2)` lifts over info_type (2, 2; the engine gives `#N/IMPL!`, A27:B27).
- `ISFORMULA(Data!A3:B3)` lifts (TRUE, FALSE); the engine gives one TRUE (A11:B11).
`CELL("address",ref)` on another sheet names the workbook file (`[name.xlsx]Data!$B$2`, A19). Excel gets that from the file name, which the
engine does not know. Leave it as `#N/IMPL!` and do not write a baseline entry for A19 by hand.

## Acceptance criteria
- [x] The probe cells above (other than A19) pass; `make eval-baseline` records them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SHEET/N of a range, CELL array info_type + other-sheet multi-cell `contents` (#VALUE!), ISFORMULA lift (static analysis now spills). probes known failures 377 -> 370; no regressions. A19 (`CELL("address")` cross-sheet) left as #N/IMPL! as instructed.
