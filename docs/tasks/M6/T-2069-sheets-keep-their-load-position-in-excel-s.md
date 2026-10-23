---
id: T-2069
title: "Sheets keep their load position in Excel's calc order after a move (SPEC-0017)"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
spec: docs/specs/SPEC-0017-weeknum-stale-state-calc-order.md
depends_on: [T-2068]
---

## Goal
A sheet moved after loading keeps its load position in Excel's calc order, as it does in Excel.

## Context
probes/weeknum-stale-sheet-moved: Third is moved to the first tab before the recalc. The reader on
Second still sees Third's marker (`222222`), because Excel runs the sheets in reverse load order, not
in reverse tab order. SPEC-0017 "Order, only when it matters" defines `load_rank: u32` per worksheet. It
is set from the sheet's position when the workbook is read; a sheet added later takes the next free
rank. The Excel-order pass of T-2068 sorts sheets by it in descending order.

## Acceptance criteria
- [x] Worksheets carry a load rank that survives moves and renames. New sheets get the next rank.
- [x] The Excel-order pass orders sheets by descending load rank. probes/weeknum-stale-sheet-moved
      passes, and the baseline is updated.
- [x] A unit test moves a sheet and checks a reader's value.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any other use of sheet order.

## Outcome
`Worksheet::load_rank` is set from the position when an xlsx is read, `0` for a new model's first sheet, and the next free rank for added or duplicated sheets; moves, renames and restores keep it. The Excel-order chain sorts formula cells by (rank, row, column). probes/weeknum-stale-sheet-moved Second!A1 now passes (probes known failures 2001 -> 2000); no other suite changed. Unit test `a_moved_sheet_keeps_its_load_position`.
