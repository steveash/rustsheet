---
id: T-2588
title: "PPMT at rates at or below -0.375, past 1E+300 and at fractional periods"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The five gen-m9 PPMT cells below have Excel's value to 15 significant digits, with no PPMT cell made
visibly off.

## Context
- Split from T-2242 (financial.md "PPMT discounts with an atanh-series log"). The engine's PPMT is
  `-(pv+fv)*rate*Q(per-1-n)/(1-Q(-n))`, `Q(x) = EXP(x*L)`, for rates above -0.375 (`discounted_principal`
  in functions/financial/annuity.rs); at or below -0.375 it keeps the exact form, because
  `PPMT(-0.5,1,277,73630,609)` is exactly -37119.5 in Excel (gen-m9 762cd0e4ec9a7aa9 A5; the model is 25 ulp
  off). But other -0.5 cells are 2E-14 relative from the exact value and 33 ulp from the model, so neither
  form is Excel's there. A probe of PPMT(-0.5, per, n, pv, fv) over per, n and fv (and other rates
  from -0.9 to -0.3, and a fine grid from -0.45 to -0.30 and 0.370 to 0.380 to pin both switches; no PPMT cell
  in eval lies between -0.375 and -0.5) is the first step. Also commit the series and LN candidates to the fit lab
  (`fitlab/kernels/annuity.py`): T-2242's fit counts came from one-off scripts.
- Also here: `PPMT(r,4,230,1E+300)` (pv near the double range; Excel -6.249999999999966E+298, ours the exact
  -6.25E+298) and `PPMT(r,1.999,97,226365)` (a fractional period), both outside the T-2242 fit.
- Owned gen-m9 cells (T-2256/T-2257 triage, moved here from T-2242; excel-graph, 2026-10-23), 5 visibly off:
  - gen-m9/38e4fc309fe199ea Calc!A73 `=MINA(PPMT((-0.5),12,158,(Data!$B4-0.1)))`: Excel -0.00046386718749999705, ours -0.0004638671875
  - gen-m9/5422bdbf173fc233 Calc!A383 `=PPMT((-0.5),13,246,37796,36)`: Excel -4.618164062499897, ours -4.6181640625
  - gen-m9/79d87fc527927626 Calc!A46 `=PPMT((-0.5),5,294,171132)`: Excel -5347.875000000112, ours -5347.875
  - gen-m9/67a4c03afd63f988 Calc!A180 `=PPMT(Data!A$10,4,230,1E+300)`: Excel -6.249999999999966e298, ours -6.25e298
  - gen-m9/d0854366830dcdee Calc!A335 `=PPMT(INDEX(Data!B3:B8,6),1.999,97,226365)`: Excel -56630.48966324223, ours -56630.489663241766
Cells:
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A73
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A383
- gen-m9/79d87fc527927626.case.json (1): Calc!A46
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A180
- gen-m9/d0854366830dcdee.case.json (1): Calc!A335

## Acceptance criteria
- [x] Visible parity (gen-m9): these 5 cells are within half a unit of the 15th significant digit of Excel's
      value, with no new known-failure lines in any baseline.
- [x] financial.md's PPMT entry covers negative rates, the 1E+300 cell and fractional periods.
- [x] `make check` green; reviewers approve.

## Out of scope
- IPMT (T-2587); bit parity (T-2243, T-1528).

## Outcome
- Rule (financial.md "PPMT discounts with an atanh-series log", negative-rate bullets): Excel's discounted PPMT
  holds for every rate above -1, not only above -0.375. Over the 62 cells at or below -0.375 of the new probe
  probes-numeric/ppmt-negative-rate-sweep (79 cells, excel-graph, 2026-10-27), that form is within 15 digits on
  61 and the exact form on 32. The -37119.5 cell that had kept the exact form fits the discounted one too.
- Past the double range Excel's x87 register holds `(pv+fv)*rate*Q`: `discounted_principal` rescales `pv+fv`
  by 2^-600 when the double product overflows (`PPMT(-0.5,4,230,1E+300)`).
- Code: `DISCOUNTED_RATE_FROM` -0.375 -> -1, `OVERFLOW_SCALE` in functions/financial/annuity.rs; unit test
  `principal_at_negative_rates`.
- Eval: gen-m9 known failures 3671 -> 3666, 0 regressions: A73, A383, A46, A180 and, in passing, 5422bdbf173fc233 A138
  (`PPMT(-0.5,1.5,180,79711)`). A335 stays a known failure as an ulp miss (NumericUlp), no longer visibly off.
  The probe: all 79 cells within 15 digits, 43 bit-exact.
- Not done: the fit-lab candidates (`fitlab/kernels/annuity.py`) and pinning the series/LN switch on the
  negative side, filed as T-2949.

