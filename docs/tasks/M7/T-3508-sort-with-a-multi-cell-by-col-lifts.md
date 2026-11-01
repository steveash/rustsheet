---
id: T-3508
title: "SORT with a multi-cell by_col lifts an argument error to by_col's shape (gen-m7 arg-pairs)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
When SORT's `by_col` is a multi-cell range and SORT fails on another argument, the result is that error spilled over `by_col`'s shape, as in Excel, not a single error.

## Context
- Found by the T-3497 arg-pairs batch (gen-m7, excel-graph, 2026-10-31). SORT lifts over its by_col (T-2319). The generator's `range` class passes a 2x2 range there (`Data!A1:B2`, `Data!C3:D4`).
- Excel, by the oracle sidecars: each of these spills a 2x2 of the error, and we fill only the anchor.
  - `SORT(Data!B3,#NUM!,Data!B1:D1,Data!A1:B2)` spills `#NUM!`.
  - `SORT(Data!C3:E4,"",Data!B7:E10,Data!A1:B2)` spills `#VALUE!`.
  - `SORT(Data!A4:C5,1,Data!G9,Data!C3:D4)` spills `#DIV/0!`.
  - `SORT(#REF!,Data!A7:D9,1,Data!A1:B2)` spills `#REF!`.
- The anchors pass. The failing cells are A2/B1/B2 of each footprint, where Excel holds the error and we hold nothing.
- Quirk recorded in docs/excel-semantics/arrays.md, "SORT's lifted by_col spreads an argument error (T-3497)".
- Cells (27; all owned here):
- gen-m7/75534bc7e1ce6474: A490, A551, A3052, A3113, B489, B490, B550, B551, B3051, B3052, B3112, B3113
- gen-m7/8217549f0b417328: A917, B916, B917
- gen-m7/91b6b88061b805b1: A11348, A11409, B11347, B11348, B11408, B11409
- gen-m7/d2945e939d6fe742: A5980, A6041, B5979, B5980, B6040, B6041

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, or an error where Excel spills), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel, with a unit test for an erroring SORT over a 2x2 by_col.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which error wins (T-3507) and the one-cell error array (T-3506).
