---
id: T-0944
title: "SUMX2MY2/SUMX2PY2/SUMXMY2: skip non-numeric pairs, #DIV/0! when none remain, pair by element count (17 poi known failures)"
milestone: M9
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMX2MY2, SUMX2PY2 and SUMXMY2 match Excel's pairing rules: the two arrays are paired element by
element in row-major order and only need the same number of elements (not the same shape); a pair
where either side is not a number (blank, text, boolean in a range) is skipped; no numeric pair
left is `#DIV/0!`; different element counts are `#N/A`; scalar arguments work.

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15), sheet EverythingTests
(`make eval-show CASE=poi/FormulaEvalTestData_Copy.xlsx`). Data block B7:N20 (column D blank,
column E text, G10 = "alphanum", J10 = "row", B16 = TRUE).
- D1336 `SUMX2MY2(B7:C10,C8:D11)`: 285149.7900399997 (actual 285152.0000599999: blanks counted as 0).
- E1336 `SUMX2MY2(D8:E11,C8:D11)`: `#DIV/0!` (actual -11.20996…). G1336 `SUMX2MY2(E8:E14,F8:F14)`: `#DIV/0!`.
- I1336 `SUMX2MY2(G8:G11,H8:H11)`: -14.438935530000002 (actual -23.43…: the "alphanum"/3 pair is skipped).
- J1336 `SUMX2MY2(H8:H10,G7:G8)`: `#N/A` (actual `#VALUE!`). K1336 `SUMX2MY2(G8,H8)`: -7.992000000000001 (actual `#VALUE!`).
- E1340 `SUMX2PY2(C8:C10,B15:D15)`: 3.2100200002; H1340 `SUMX2PY2(B7:B9,B16:D16)`: 2.21 (TRUE and blank pairs skipped).
- F1340, G1340, K1340: `#DIV/0!` (no numeric pair).
- J1340 `SUMX2PY2(G7:H9,J7:J12)`: 105.7931362501, a 3x2 range against a 6x1 range, paired row-major
  (G7,J7), (H7,J8), (G8,J9), (H8,J10="row" skipped), (G9,J11), (H9,J12). Actual `#VALUE!`.
- F1344 `SUMXMY2(2,4)`: 4 (actual `#VALUE!`). H1344 `SUMXMY2(C7:C10,C7:C9)`: `#N/A` (actual `#VALUE!`).
- G1344, I1344, J1344: `#DIV/0!`.
Passing today and must stay so: F1336, H1336, L1336, M1336 (`#REF!`), D1340, I1340, D1344, E1344.

Rule recorded in docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing".

Code: `crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs`. The three functions use
`fn_get_two_matrices`, which is shared with the statistical functions in correl.rs, pearson.rs,
forecast.rs and chisq.rs: change the SUMX rules without changing what those callers get (for example a SUMX-specific
element-count check and pair filter in sum.rs on top of the shared value extraction).

## Acceptance criteria
- [ ] These 17 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!D1336, E1336, G1336, I1336, J1336, K1336,
      E1340, F1340, G1340, H1340, J1340, K1340, F1344, G1344, H1344, I1344, J1344.
- [ ] Unit tests for each rule: pair skipping, `#DIV/0!` with no numeric pair, `#N/A` for different
      counts, different shapes with the same count, scalar arguments.
- [ ] coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pairing in the other `fn_get_two_matrices` callers (no evidence here; file a follow-up if they differ).

## Outcome
SUMX2MY2/SUMX2PY2/SUMXMY2 now share `sumx_fold` (pair row-major, skip non-numeric pairs, `#N/A` on unequal counts, `#DIV/0!` when none remain, scalars allowed). poi known failures 916 -> 899 (17 fixed, 0 regressions). Unit tests in `test_sumx_pairing.rs`.
