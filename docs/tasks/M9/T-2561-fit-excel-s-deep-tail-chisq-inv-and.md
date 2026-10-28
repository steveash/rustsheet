---
id: T-2561
title: "Fit Excel's deep-tail CHISQ.INV and GAMMA.INV kernel (T-1634 residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 30 gen-m9 cells left visibly off by T-1634 (and the one added by the 2026-10-26 triage) reach visible parity: `CHISQ.INV`, `CHISQ.INV.RT`,
`CHIINV`, `GAMMA.INV` and `GAMMAINV` agree with Excel to 15 significant digits in the deep tails.

## Context
T-1634 replaced statrs' search with a crossing of the capped incomplete gamma and fixed 67 gen-m9
cells. What is left is Excel's own rounding, not ours: at shape 25 (`CHISQ.INV(1E-300,50)`) Excel
gives 2.0354283669760313E-11 where the correctly rounded quantile, which the engine now returns, is
2.035428366976834E-11 (a 50-digit Decimal solve of the series, T-1634). So these cells need Excel's
algorithm reproduced, the second kind of work in decision 0083, not a more accurate kernel.
Leads from T-1634:
- The crossing of the engine's lower cdf with `x^a` by `powf` is nearer Excel than the crossing of
  statrs' `exp(a ln x - x - lngamma(a))` form at 8 of 8 deep-tail cells tried (relative 4E-15 to
  2.8E-14 against 7E-15 to 3.9E-13), so Excel's prefactor is not statrs' exp form either.
- `CHISQ.INV.RT(1-1E-16,1E+5)` and `CHISQ.INV(1E-300,1E+5)` are off by about 4E-15 relative at a
  shape of 50,000: the upper-tail and large-shape paths may need their own fit.
- `CHIINV(NPV((7*1E-15),{-100,300,-300,101}),3)` (4e55500bb8ad3cd8 A111) is 7.5% off: its p comes
  from NPV, so check NPV's last digits before the quantile.
- df-1 right tail: `CHIINV(1E-9,1)` is 37.324892889999234 in Excel, one step of the double lower cdf's
  resolution near 1 below the engine's complement crossing 37.324893106518715 (probes-numeric/m9-legacy-chiinv
  A1, A2): Excel searches its own double-valued cdf.
- Crossing convention: `refine_increasing`'s midpoint loses an ulp at `GAMMA.INV(1E-300,1,1)`
  (9.999999999999999E-301); `special::nearest_crossing` gives Excel's 1E-300.
Also in scope: the CHISQ.INV.RT `#NUM!` boundary that chisq.rs `RIGHT_TAIL_P_MIN` and
`RIGHT_TAIL_SHAPE_ALL_P` currently guess (the p between 1E-20 and 1E-100, and df 11 to 21;
`TODO(T-2561)` there), since the fit needs those cells anyway.
Run the fit lab on the T-1522 kernel map (decision 0056: two attempts, then time-box).

Owned gen-m9 cells (T-1634 residue, 2026-10-24): 30 visibly-off cells.
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (4): Calc!A128, A194, A215, A250
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A240
- gen-m9/165db58d1540f014.case.json (2): Calc!A132, A283
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A254
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A142
- gen-m9/4250f880ab640c49.case.json (1): Calc!A5
- gen-m9/4e55500bb8ad3cd8.case.json (2): Calc!A111, A217
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A88
- gen-m9/623663cacea53bbc.case.json (1): Calc!A98
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A13
- gen-m9/a612e89321a6df9c.case.json (2): Calc!A169, A317
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A184
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A293
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A399
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A107
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A14, A187
- gen-m9/cde551ea787f713a.case.json (1): Calc!A347
- gen-m9/d8266bc15533702a.case.json (1): Calc!A212
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A77, A301
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A123, A173
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A56

One more visibly-off deep-tail cell (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph); no task listed it:
- gen-m9/a5828d3925575713 Calc!A338 `=GAMMAINV(1E-16,Data!E9,Data!$A1)`: Excel 6.436215297435573E-07, ours
  6.436215297435568E-07 (5 ulp)
Cells:
- gen-m9/a5828d3925575713.case.json (1): Calc!A338

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-1634 residue): these 30 known-failure lines disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The candidates tried, with their bit-exact and visible-parity counts, are recorded here and in
      docs/excel-semantics/statistics.md (decision 0056).
- [ ] The CHISQ.INV.RT `#NUM!` boundary is probed (the p between 1E-20 and 1E-100 at df 3 to 10, and
      df 11 to 21 at 1E-100), chisq.rs matches the oracle values and the `TODO(T-2561)` there is gone.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The t1543-inverse-cdf-tiny-p BETA.INV cells (T-1634's Context lists them; they are BETA.INV's).
- The 65 cells T-1634 brought to visible parity but not to bit-exactness: they stay in
  eval/baseline/gen-m9.txt as bit-level known failures and belong to T-0972 (gamma quantile
  parity, the last-digit task for this kernel). Fix them here only if the same fit covers them.

## Visible-parity stage
Decision 0083: the cells above are visibly off, so the task is in the campaign. Its target is that
every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the
15th significant digit of Excel's value.
