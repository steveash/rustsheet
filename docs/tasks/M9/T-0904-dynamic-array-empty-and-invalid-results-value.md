---
id: T-0904
title: "DROP/TAKE/SEQUENCE/FILTER: Excel returns #VALUE! where we return #CALC!"
milestone: M9
lane: functions
status: in-progress
priority: P1
size: S
depends_on: []
---

## Goal
Make the degenerate-result cases of DROP, TAKE, SEQUENCE and FILTER return the error Excel's own
cached values show (`#VALUE!`) instead of `#CALC!`.

## Context
All 8 cells have `error_mismatch`, expected `#VALUE!`, actual `#CALC!` (Excel-authored workbooks, no
new oracle access needed):
- `ironcalc-calc/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx`: Sheet1!C2 `DROP(A2:A8,-7)`, E2 `DROP(A2:A8,1200)`,
  F2 `DROP(A2:A8,7)`, I2 `DROP(A2:A8,1,1)`, L2 `TAKE($A$2:$A$8,0)`  (all would yield an empty array).
- `ironcalc-calc/LOOKUP_AND_REFERENCE/FILTER_ERRORS.xlsx`: Sheet1!E1 `FILTER(A1:A4,A1:A4>10)` (no match).
- `ironcalc-calc/MATH_AND_TRIGONOMETRY/SEQUENCE.xlsx`: Sheet1!C16 `SEQUENCE(0,0)`, E18 `SEQUENCE(0.3)`.
Root cause (suspected): the shared "empty array" result is mapped to `#CALC!` in
`crates/rustsheet-engine/src/functions/lookup_and_reference/drop_take.rs`,
`.../math_and_trigonometry/sequence.rs` and the FILTER implementation. Check whether `#CALC!` for
FILTER no-match is correct in live Excel (the cached value in this workbook says `#VALUE!`; if other
cells in the suite expect `#CALC!`, keep those passing and document the distinction in
`docs/excel-semantics/`). Use `make eval-show CASE=ironcalc-calc/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx`.

## Acceptance criteria
- [ ] The 8 known-failure lines above (5 DROP_TAKE, 1 FILTER_ERRORS, 2 SEQUENCE error cells) disappear from
      eval/baseline/ironcalc-calc.txt with no new ones.
- [ ] Unit tests cover each degenerate case; `make check` green.
- [ ] Finding recorded in docs/excel-semantics/.

## Out of scope
SEQUENCE floating-point step accumulation (T-0905), oracle confirmation of other empty-array cases.
