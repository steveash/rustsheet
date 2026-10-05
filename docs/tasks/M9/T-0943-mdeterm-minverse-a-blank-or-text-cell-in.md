---
id: T-0943
title: "MDETERM/MINVERSE: a blank cell in the matrix is #VALUE! (20 poi known failures)"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MDETERM and MINVERSE return `#VALUE!` when any cell of the matrix range is blank, as Excel does
(MMULT already does). Today a blank counts as 0, so MDETERM gives 0 (singular) and MINVERSE `#NUM!`.

## Context
Evidence: poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16; `make eval-show
CASE=poi/MatrixFormulaEvalTestData.xlsx`), sheet Sheet1. H10, H30, H14:J16 and H34:J36 are legacy array formulas; H50 is a normal formula.
- H10 `=MDETERM(B10:D12)` (B10:D12 all blank): expected `#VALUE!`, actual 0.
- H30 `=MDETERM(B30:D32)` (B30 = 1, the rest blank): expected `#VALUE!`, actual 0.
- H14:J16 `=MINVERSE(B14:D16)` (all blank): expected `#VALUE!` in all 9 cells, actual `#NUM!`.
- H34:J36 `=MINVERSE(B34:D36)` (B34 = 1, the rest blank): expected `#VALUE!` in all 9 cells, actual `#NUM!`.
Passing today and must stay so: H50 `=MDETERM(B50)` (B50 = 1) is 1; H18/H38 `MMULT` over blanks is `#VALUE!`.

Rule recorded in docs/excel-semantics/arrays.md "MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!".

Code: `coerce_to_f64` in `crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs`
maps `ArrayNode::Empty` to 0.0; it is only used by `to_square_matrix` (MDETERM, MINVERSE). Make
`Empty` a `#VALUE!` there. Check how `fn_mmult` (mmult.rs) rejects blanks and reuse its rule if a
helper exists rather than adding a second one.

## Acceptance criteria
- [ ] These 20 lines disappear from eval/baseline/poi.txt (`make eval-baseline`), with no new F
      lines in any baseline: `poi/MatrixFormulaEvalTestData.xlsx` Sheet1!H10, H30, H14, I14, J14,
      H15, I15, J15, H16, I16, J16, H34, I34, J34, H35, I35, J35, H36, I36, J36.
- [ ] Unit tests: MDETERM and MINVERSE of a range holding a blank cell give `#VALUE!`; a fully
      numeric matrix is unchanged; `MDETERM` of one numeric cell is that number.
- [ ] arrays.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other 95 failing cells of the same workbook (H46:J136): a legacy array formula whose result is
  smaller than its range (a 1x1 or single row/column result is repeated, other cells are `#N/A`).
  That is CSE range filling, an engine change; see T-0913.
- Text or boolean cells inside the matrix (no evidence in this workbook).
