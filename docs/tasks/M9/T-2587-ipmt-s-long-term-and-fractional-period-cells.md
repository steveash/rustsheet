---
id: T-2587
title: "IPMT's long-term and fractional-period cells off in the 13th to 15th digit"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The ten gen-m9 IPMT cells below have Excel's value to 15 significant digits, with no PMT, IPMT or PPMT
cell of m9-fin-pmt, IPMT_PPMT.xlsx or gen-m9 made visibly off.

## Context
- Split from T-2242. T-2242 found that Excel's PPMT is not the exact closed form but
  `-(pv+fv)*rate*Q(per-1-n)/(1-Q(-n))` with `Q(x) = EXP(x*L)` and its own `L = ln(1+rate)` (an x87 atanh
  series below 0.375, Excel's LN above), financial.md "PPMT discounts with an atanh-series log". IPMT still
  uses the exact balance form (`accurate_period_split` in functions/financial/annuity.rs), and these cells
  are 2E-15 to 1E-13 relative off it: long terms, fractional periods, huge nper and type 1.
- First hypotheses to test in the lab (exact args are in the cases): IPMT = PMT - PPMT with T-2242's PPMT,
  IPMT as `-rate*B` with the balance's powers from the same Q, and for fractional periods Excel's own `^`.
- Owned gen-m9 cells (T-2257 triage, moved here from T-2242; excel-graph, 2026-10-23), 9 visibly off:
  - gen-m9/0a957119475760c5 Calc!A202 `=IPMT(1,6,50,272273,AVERAGE(..))`: Excel -272272.99996209145, ours -272272.9999622236
  - gen-m9/1cd3096e3e87c965 Calc!A227 `=IPMT(Data!A$6,1.999,167,..)`: Excel -1.1780962499999998, ours -1.1780962500000083
  - gen-m9/5ded7594e597e31e Calc!A177 `=IPMT(BETAINV(..),1.5,77,75622)`: Excel -75621.97390807692, ours -75621.97388109102
  - gen-m9/6376e27be7a05871 Calc!A158 `=IPMT(..,3,209,Data!A3,667,1)`: Excel -1.0471966666666503, ours -1.0471966666666666
  - gen-m9/6376e27be7a05871 Calc!A207 `=IPMT(0.1,8,291,1,638)`: Excel -0.09999999994538485, ours -0.09999999994537818
  - gen-m9/b72d952c5f52cb4d Calc!A364 `=IPMT(10,4,1E+300,..)`: Excel -10.001, ours -10.000999999999749
  - gen-m9/b73c6cc96e093f03 Calc!A268 `=IPMT(TOROW(..),8,290,392736,(-0.5),..)`: Excel -383602.6046511628, ours -383602.60465116205
  - gen-m9/c0681a8b150ce301 Calc!A104 `=IPMT(Data!A6,3,232,259859,333,..)`: Excel 64714.99999999975, ours 64715.0
  - gen-m9/e168be7f6d3f84c2 Calc!A385 `=IPMT((-0.05),10,336,ROWS(Data!B1:C1))`: Excel 0.03151246988087522, ours 0.03151246988087528
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A202
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A227
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A177
- gen-m9/6376e27be7a05871.case.json (2): Calc!A158, A207
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A364
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A268
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A104
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A385

One more visibly-off gen-m9 IPMT cell (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph), long term at a
negative rate with type 1; no task listed it:
- gen-m9/b6a02893a8d6b151 Calc!A35 `=IPMT((-0.05),4,341,208149,115,1)`: Excel 9391.86032881097, ours 9391.860328810977
Cells:
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A35

## Acceptance criteria
- [ ] Visible parity (gen-m9): these 9 cells are within half a unit of the 15th significant digit of Excel's
      value (their lines leave eval/baseline/gen-m9.txt, or stay only as last-digit NumericUlp lines), with
      no new known-failure lines in any baseline. Partly: 8 of the 10 pass; A202 moved to T-2943 and A177
      (a BETAINV miss, not IPMT's) to T-2936.
- [x] financial.md records IPMT's formulation and its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity (T-2243, T-1528).

## Outcome
- Of the ten cells, A158, A207 (6376e27be7a05871), A364 (b72d952c5f52cb4d) and A268 (b73c6cc96e093f03) already
  passed on `main` (fixed by T-2242 and T-2490). This task fixes A385 (e168be7f6d3f84c2), A104 (c0681a8b150ce301),
  A35 (b6a02893a8d6b151) and A227 (1cd3096e3e87c965), all now bit-exact.
- Rule (financial.md "IPMT follows Excel's own discount where it moves the value"): where the balance does not
  cancel, Excel's IPMT is `-rate*((pv+fv)*Y - fv)` in doubles with `Y` from Excel's expm1 and PPMT's L wherever that
  differs from the exact balance by more than 6E-16 relative; below that the exact balance stays (it has Excel's bits
  more often). The switch is fitted: 5E-16 to 7E-16 fit the same cells, and it moves no other cell of gen-m9, probes,
  probes-numeric, ironcalc-calc, corpus-bench or poi. `discounted_interest` and `DISCOUNTED_INTEREST_FROM` in
  functions/financial/annuity.rs; unit test `interest_on_excels_discount`.
- Split out: A202 (0a957119475760c5) cancels on fv and needs the same form there without breaking A207 and four
  huge-rate probe cells (T-2943). A177 (5ded7594e597e31e) is not IPMT's miss: Excel's `BETAINV(1E-15,1E+8,1)`
  differs from `p^(1/alpha)` by 3.6E-10 relative (T-2936).
- Eval: gen-m9 known failures 3724 -> 3720 (4 fixed, 0 regressions); IPMT visibly-off cells 12 -> 9 over gen-m9,
  probes, probes-numeric and ironcalc-calc. calipers: the same 26 known regressions (T-0458) with and
  without the change.
- Resumed 2026-10-27 by the next opus run for `make check`, the review panel, the PR and the merge.
