---
id: T-1230
title: "Probe CELL(contents) of a multi-cell range: same-sheet vs other-sheet vs intersection (follow-up of T-1218)"
milestone: M5
lane: functions-logic
status: todo
priority: P2
size: S
tier: opus
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

## Handoff (resumed)
- Done: probe `eval/suites/probes/cases/info-cell-contents-range.case.json` + excel-graph sidecar, finding recorded in docs/excel-semantics/information.md ("CELL(\"contents\", ref) on excel-graph").
- Finding: excel-graph reads `CELL("contents", Data!A2)` from the formula's own sheet (Calc!A2), even for a single cell, with no implicit intersection, legacy or dynamic. This is probably a web bug; the engine matches desktop-like behavior for single cells (many gen-* cases pass on it).
- Why stopped: matching it changes `cell_info` for every cross-sheet CONTENTS, and the new case has 32 failing cells that `make check` rejects (no baseline hand-edit allowed). Whether to follow the web (decision 0039: excel-graph is the oracle of record) or split the probe so the cross-sheet cells go to M14 is a design call. The same-sheet cells (D3 legacy twin intersects: we give 13.5, Excel 11.5) are a plain fix.
- Next: decide; then either implement (read top-left on `cell.sheet`, no legacy intersection) and `make eval-baseline`, or drop the cross-sheet cells from the case, keep the same-sheet ones and file the desktop question under M14.
