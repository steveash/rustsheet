---
id: T-0967
title: "T.INV/T.INV.2T/CONFIDENCE.T: Student-t quantile parity (89 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0969, T-1043]
---

## Goal
T.INV, T.INV.2T and CONFIDENCE.T return Excel's values from one Student-t quantile routine built on
the T-0969 CDF kernel.

## Context
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

## Acceptance criteria
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
