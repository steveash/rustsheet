---
id: T-0905
title: "SEQUENCE/MULTINOMIAL/COMBIN/COMBINA: match Excel's floating-point evaluation order"
milestone: M9
lane: functions-math
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Reproduce Excel's exact last-bit results for SEQUENCE steps, MULTINOMIAL, COMBIN and COMBINA.

## Context
Filed in draft PR #11 (2026-10-02, never merged); salvaged by T-0031. Cell lists are from that date: re-check them against eval/baseline/ironcalc-calc.txt first.
Excel-cached expected values reveal the algorithm (no new oracle access needed):
- SEQUENCE (`MATH_AND_TRIGONOMETRY/SEQUENCE.xlsx`, 5 NumericUlp cells: I9, I10, I11, L14, M14): expected
  14.399999999999999, 16.599999999999998, 18.799999999999997, 12.999999999999998, 13.099999999999998; we give
  14.4 / 16.6 / 18.8 / 13.0 / 13.1. Suggests Excel accumulates `prev + step` rather than `start + i*step`.
- MULTINOMIAL (`MULTINOMIAL.xlsx`, 6 cells: A1, B3, A8, D12, B16, B22): `MULTINOMIAL(1,2,3)` expected
  59.99999999999993, we give 60. Excel is not computing an exact integer ratio; try factorial/gamma
  based evaluation (e.g. `exp(lgamma)` or a product of ratios) and fit all six values.
- COMBIN/COMBINA (`COMBIN_COMBINA.xlsx`, 5 cells: D4, C5, D5, C16, D16): e.g. COMBINA expected 92378, we give
  92377.99999999999; COMBIN expected 20, we give 20.000000000000004; large cases differ in the 14th digit.
  Fit the multiplicative loop order/direction (and COMBINA = COMBIN(n+k-1,k)) to all five.
Code: `crates/rustsheet-engine/src/functions/math_and_trigonometry/{sequence,multinomial,mathematical}.rs`.
If one function cannot be fit with the existing evidence, split it out and leave its lines in baseline.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/3a62e0950c1074fd Calc!A34 `=COMBIN((DEGREES(0.2)+IF(..)),9)` = COMBIN(11,9) after truncation: Excel 55, engine 54.999999999999986 (`COMBIN(11,9)` alone).
- gen-m7/8ed8a680e9794b77 Calc!A21 `=COMBIN(23,ISOWEEKNUM(36264))` = COMBIN(23,15): Excel 490314, engine 490313.9999999999 (`COMBIN(23,4)` is exact 8855). Excel gives exact integers for both; fit these with the five COMBIN_COMBINA cells.

## Acceptance criteria
- [ ] 16 known-failure lines (5 SEQUENCE: I9, I10, I11, L14, M14; 6 MULTINOMIAL; 5 COMBIN_COMBINA)
      disappear from eval/baseline/ironcalc-calc.txt with no new ones.
- [ ] Algorithms documented in docs/excel-semantics/; `make check` green.

## Out of scope
SEQUENCE `#VALUE!` (done, T-0913); trig/hyperbolic (T-0914, T-0915, T-0917, T-0939).
