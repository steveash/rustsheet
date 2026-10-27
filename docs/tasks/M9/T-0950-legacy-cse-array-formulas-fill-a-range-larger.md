---
id: T-0950
title: "Legacy (CSE) array formulas fill a range larger than their result (repeat 1x1/row/column, #N/A elsewhere)"
milestone: M9
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A legacy (CSE) array formula entered over a range larger than its result fills that range as Excel
does: a 1x1 result fills every cell, a single-row result is repeated down and a single-column
result across, and every other extra cell is `#N/A`.

## Context
poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16), Sheet1 H46:J136: 95 known failures
(eval/baseline/poi.txt), a mix of repeated-value cells and 47 `#N/A` fill cells. Examples: `TRANSPOSE(B46)` over H46:J48 is 1 in
all 9 cells (also `MINVERSE(B54)` H54:J56, `MMULT(B58,E58)` H58:J60); `B62:C62+E62:F62` over
H62:J64 repeats the row down with J62:J64 `#N/A`; `TRANSPOSE(B66:C66)` over H66:J68 has row 68
`#N/A`. Rule recorded in docs/excel-semantics/arrays.md "Legacy (CSE) array formula over a larger
range". Found by the gap-triager while filing T-0943 (the MDETERM/MINVERSE blank-cell part).
Gap triage 2026-10-27: a real-world instance in a gated suite. corpus-bench/13894/1_13894_input.xlsx (Excel for Mac
cache) has the legacy array formula `IFERROR(INDEX(ucode,SMALL(IF(MATCH(ucode,ucode,0)=ROW(INDIRECT("1:"&ROWS(ucode))),
MATCH(ucode,ucode,0),""),ROW(INDIRECT("1:"&ROWS(ucode))))),"")` entered over Sheet1!F14:F25; `ucode` is 6 cells, so the
result is 6x1 and F20:F25 are `#N/A` in Excel, `#VALUE!` in ours (6 cells, all the case's failures: its whole-case
`F ... *` line in eval/baseline/corpus-bench.txt). The `#N/A` fill applies even though IFERROR wraps the formula: it
is the range fill, not an error of the formula.
This is an engine change to how array-formula ranges are filled from a result (the xlsx importer
and the evaluator's array-formula path): check whether it needs a short spec first (CLAUDE.md
directive 4) and split if it is bigger than M.

## Acceptance criteria
- [ ] The H46:J136 F lines of poi/MatrixFormulaEvalTestData.xlsx disappear from eval/baseline/poi.txt
      (those that stay need a documented reason) with no new F lines in any baseline.
- [ ] corpus-bench/13894/1_13894_input.xlsx passes and its `F` line leaves eval/baseline/corpus-bench.txt.
- [ ] Unit tests for the 1x1, row and column broadcast and the `#N/A` fill.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array spilling (`#SPILL!`), which is a different mechanism.
