---
id: T-1230
title: "Probe CELL(contents) of a multi-cell range: same-sheet vs other-sheet vs intersection (follow-up of T-1218)"
milestone: M5
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`CELL("contents",range)` for a multi-cell range is settled by oracle evidence for same-sheet, other-sheet, and formula-row/column inside vs outside the range.

## Context
T-1218 keys on the sheet (other sheet = `#VALUE!`, same sheet = top-left) because ironcalc-calc CELL.xlsx B14 (excel-cached) and probes/info-logic-argument-modes A18 (excel-graph) fit it, but they fit "formula row/column outside the range" too. See docs/excel-semantics/information.md. Code: `cell_info` in functions/information/mod.rs.

## Acceptance criteria
- [ ] a probe separates the hypotheses (same-sheet range not containing the formula cell; other-sheet range containing it; a plain dynamic formula `CELL("contents",A10:A14)` next to a CSE variant), and also covers `CELL({1,"row"},B2)` and `N(range)` with an error top-left cell; `cell_info` and the semantics entry match the oracle
- [ ] `make check` green; reviewers approve.

## Out of scope
-
