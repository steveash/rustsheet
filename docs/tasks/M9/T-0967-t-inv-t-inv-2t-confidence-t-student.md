---
id: T-0967
title: "T.INV/T.INV.2T/CONFIDENCE.T: Student-t quantile parity (89 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0969, T-1043]
---

## Goal
T.INV, T.INV.2T and CONFIDENCE.T return Excel's values from one Student-t quantile routine built on
the T-0969 CDF kernel.

## Context
Kernel map (T-1522, decision 0056): rank 5 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::StudentsT::inverse_cdf`, which calls statrs'
`inv_beta_reg(df/2, 0.5, 2p)` and back-transforms (`sqrt(df(1-y)/y)`), losing precision near p = 0.5.
Code: `functions/statistical/t_dist.rs` (`fn_t_inv`, `fn_t_inv_2t`), `functions/statistical/normal.rs`
(`fn_confidence_t`, `t_dist.inverse_cdf(1 - alpha/2)` times `std_dev/sqrt(size)`).
89 baseline lines in `eval/baseline/ironcalc-calc.txt` (48 NumericClose, 35 NumericUlp, 6 NumericMismatch):
- `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` `CONFIDENCE.NORM CONFIDENCE.T`: E2(C), E3, E5(C), E8(M), E11(C), E13(C), E16(C), E17(C), E22(C), E30(C), E34(C), E35(C), E36(C), E37(C), E48(C), E49(C), E53(C), E54(C), E56(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`: E2(C), E3, E4, E5, E6(C), E7(C), E8(C), E10(C), E11, E12, E13(C), E14, E15, E16(C), E17(C), E19(C), E20(M), E21(M), E22, E23, E28, E36, E38, E39, E40, E41, E42, E43, E46, E47, E55, E57, E58, E75(C), E76(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`: D2(C), D3(C), D4, D6, D7(C), D8(C), D11, D13, D16(C), D17(C), D18(C), D20(M), D25, D26, D34, D35, D38, D39, D47(C), D48(C), D49(C), D51(C), D52(C), D53(C), D54(C), D55(C), D56(C), D57(C), D58(C), D59(C), D60(C), D62(M), D63(M), D72, D73
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- `T.INV.2T(0.9999999,10)` D62: expected 1.2849890167888884e-07, actual 1.2467205919833128e-07 (3% off:
  cancellation near p = 0.5 in the inv_beta_reg route). D63 (p = 0.9999) 1.4e-8 relative.
- `T.INV(T.DIST(1E-6,11),11)` E20: expected 9.999999999666944e-07, actual 1.0000999564468016e-06.
- `CONFIDENCE.T(0.05,1,50)` E2 (9 lines with this value, plus scaled copies E13, E16, E17): expected 0.2841968554957298, actual 0.28419685549566487
  (2.3e-13). E8 (Mismatch): expected 1.781519460717496e-07, actual 1.7824180786974336e-07.
- `T.INV(T.DIST(1,1,TRUE),1)` E15: expected 1.0, actual 0.9999999999999996; `T.INV(...)` E11 expected
  1.0000000000000004, actual 1.0000000000000009 (Excel's own last digits; round trips are not exact,
  see `docs/excel-semantics/statistics.md`, "Inverse distribution functions do not round-trip exactly").

Hypothesis (the oracle is the judge): Excel solves CDF(t) = p iteratively on its own T.DIST kernel
(T-0969), so parity needs that kernel first; for the 2-tailed and near-0.5 cases solve on the tail
directly (no `1 - p`). The last digits depend on the solver's start, step and stopping rule:
**Needs an oracle sweep (T-0018):** fit against the T-1043 Student-t sweep, never blind.
Previous evidence from T-0925 (inverse normal): a closed-form inverse plus one Newton step regressed
cells that the old code matched by luck; measure regressions on every attempt.

Oracle sweep (T-1043): probes-numeric cases `m9-t-inv`, `m9-t-inv-2t`, `m9-confidence-t`, `m9-t-nonint-df`, `m9-t-edges`; findings in docs/excel-semantics/statistics.md "Student t sweep".

gen-m9 sweep (gap triage 2026-10-23, T-2255). CONFIDENCE.T, whose Student-t quantile is visibly off (35 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 35 known-failure lines in eval/baseline/gen-m9.txt, 35 formulas.
Examples:
- gen-m9/146b46e74f2dac3b Calc!A340 `=CONFIDENCE.T(0.999999,1E-10,((2*(-2.5))+BITXOR(882,42)))`: Excel 4.2975693350469395e-18, ours 3.1591558575874862e-18
Cells:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A340
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A389
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A45
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A181
- gen-m9/4bad303d32cc3f46.case.json (3): Calc!A5, A188, A371
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A57
- gen-m9/623663cacea53bbc.case.json (1): Calc!A293
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A53
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A372
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A256
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A246
- gen-m9/984cba67bbe6972b.case.json (5): Calc!A128, A177, A310, A317, A338
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A76, A320
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A4
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A132, A254
- gen-m9/b4173fe25109a238.case.json (1): Calc!A264
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A135
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A341
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A99, A221
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A205
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A90
- gen-m9/ecc87f80077959e0.case.json (4): Calc!A5, A166, A327, A348
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A11

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: CONFIDENCE.T last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas.
Examples:
- gen-m9/8fec7b207f4dd753 Calc!A185 `=CONFIDENCE.T(1E-16,1E+5,12)`: Excel 2388816.5175161078, ours 2388816.5175161064
Cells:
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A2, A185
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A156, A191
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A198

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 35 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] All 6 NumericMismatch and all 48 NumericClose lines above leave the NumericClose/Mismatch
      classes, at least 45 of the 89 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in
      any baseline; remaining lines listed in the PR with expected/actual.
- [ ] One quantile routine shared by T.INV, T.INV.2T and CONFIDENCE.T (no remaining statrs
      `StudentsT::inverse_cdf` call; `rg inverse_cdf` shows it).
- [ ] Unit tests for p near 0, 0.5 and 1 and df = 1, 2, 1e6; bounded iteration (no unbounded loops
      on untrusted input).
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0969); CONFIDENCE.NORM (T-0925); legacy TINV (evidence in T-1048).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 345 in total.
- T.INV: 109 cells (13-14 digits right 66, 10-12 33, 6-9 4, fewer or 0 vs nonzero 6); most in `probes-numeric/m9-t-inv.case.json` 67; `probes-numeric/t1531-t-tiny-p.case.json` 27. e.g. `T.INV(0.4,5)` Excel -0.2671808657041464, ours -0.2671808657041447
- T.INV.2T: 148 cells (13-14 digits right 69, 10-12 45, 6-9 12, fewer or 0 vs nonzero 22); most in `probes-numeric/m9-t-inv-2t.case.json` 100; `ironcalc-calc/STATISTICAL/T_DIST.xlsx` 19. e.g. `T.INV.2T(0.5,2)` Excel 0.8164965809277259, ours 0.8164965809277231
- TINV: 22 cells (13-14 digits right 15, 10-12 4, 6-9 1, fewer or 0 vs nonzero 2); most in `probes-numeric/t1531-t-tiny-p.case.json` 17; `probes-numeric/t1048-tinv.case.json` 5. e.g. `TINV(0.5,10)` Excel 0.6998120613124317, ours 0.6998120613124311
- CONFIDENCE.T: 66 cells (13-14 digits right 31, 10-12 29, 6-9 3, fewer or 0 vs nonzero 3); most in `probes-numeric/m9-confidence-t.case.json` 30; `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` 18. e.g. `CONFIDENCE.T(0.1,1,2)` Excel 4.464496510753559, ours 4.464496510753552

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.
