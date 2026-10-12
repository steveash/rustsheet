---
id: T-1230
title: "Probe CELL(contents) of a multi-cell range: same-sheet vs other-sheet vs intersection (follow-up of T-1218)"
milestone: M5
lane: functions-logic
status: done
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
- [x] a probe separates the hypotheses (same-sheet range not containing the formula cell; other-sheet range containing it; a plain dynamic formula `CELL("contents",A10:A14)` next to a CSE variant), and also covers `CELL({1,"row"},B2)` and `N(range)` with an error top-left cell; `cell_info` and the semantics entry match the oracle
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/info-cell-contents-range (excel-graph, 50 cells; sonnet run) showed that `CELL("contents", ref)` reads the top-left cell of `ref` on the formula's own sheet, whatever sheet `ref` names, and is never implicitly intersected (legacy and dynamic formulas agree). It also settled the T-1218 open questions: `CELL({1,"row"},B2)` is `#VALUE!`, and `N(range)` with an error top-left cell is that error. Per decision 0039 (excel-graph is the oracle of record) the engine follows the web, an opus run having taken the handoff:
- `cell_info` reads `CONTENTS` on `cell.sheet`;
- CELL's `reference` argument has a vector signature, so a legacy formula no longer intersects it.
The desktop check is T-1075 item 9. No currently passing cell depends on the old cross-sheet reading: the gen-* uses of a cross-sheet `CELL("contents")` sit inside formulas whose results do not separate the two readings. `args_signature_sheet` and `args_signature_no_args` were folded into `args_signature_scalars`, so the oversized `static_analysis.rs` shrinks (2156 -> 2147). Eval: new case, 50/50 pass; no other change and no regressions. Note for incremental recalc (none exists today, every recalc is full): a cross-sheet `CELL("contents")` depends on the same-coordinate cell of its own sheet, which a reference walk would miss. Follow-up: T-1433 (legacy CELL with the other info_types).
