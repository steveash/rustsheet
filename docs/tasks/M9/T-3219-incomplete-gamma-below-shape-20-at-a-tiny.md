---
id: T-3219
title: "Incomplete gamma below shape 20, at a tiny x/a and in CHIINV: Excel's prefactor form (T-2638 remainder, 42 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The incomplete-gamma cells T-2638 left visibly off reach visible parity (within half a unit of Excel's 15th
significant digit): the shapes below 20 (`gamma_prefactor`'s direct branch), the density at a tiny x / a at shape 50
and CHIINV's quantile.

## Context
T-2638 found Excel's prefactor from shape 20 away from the mode (docs/excel-semantics/statistics.md "Incomplete gamma:
accurate kernel (T-0971)", the T-2638 paragraph) and closed 31 cells. Its fit attempt 1 (same section, the CHIDIST
paragraph) found no form for the cells below; every candidate tried is recorded there with its ulp at each cell.
Excel's error at the small-shape cells is within about an ulp of ln(value), so the prefactor is likely a summed log,
but at `CHIDIST(100,4)` Excel is 38 ulp below the truth while every log form is above it, so the continued
fraction's own rounding differs too. Next hypotheses: Excel's continued fraction in another recurrence (Cephes
`igamc`'s pk/qk form, Legendre's), a finite Poisson sum at integer and half-integer shapes, x87 extended precision
for the exponent; for shape 50 at y / a below 1/100 (t2753 A46-A48), a third branch of the ratio form. A probe sweep
(CHIDIST(x,k) for k = 2 to 10, x = 20 to 200, and GAMMA.DIST densities at shapes 2 to 20, x / a from 1E-3 to 10) may
be needed first (`excel-probe` skill). Decision 0056: this is attempt 2 of the small-shape fit; record each candidate.

Cells (from T-2638; run `make eval-show CASE=...` for the values):
- gen-m9/05d44f6dd6cfc567 Calc!A178, 3a25a03eb94fcfbd A158 `CHIDIST(100,4)`, bbdfc06cae0e0d71 A125 `CHIDIST(100,5)`,
  f34f66fb7f81e673 A3 `CHIDIST(Data!A2,8)`, 4bad303d32cc3f46 A293 and 984cba67bbe6972b A49 `CHISQ.DIST(1E-10,8,...)`,
  984cba67bbe6972b A42 `CHISQ.DIST(37.5,6,FALSE)`, A140, A217, 68d8ffb69ff739d8 A341, a54c738c697617c3 A212,
  a7bf0d8d81d33ba6 A152, and 0a614cfe974b59c3 A178 (`CHISQ.DIST(AGGREGATE(...),...)`, 3 ulp off before T-2638 and
  4 after).
- CHIINV: gen-m9/285d87cea1aaa73c A261 and 4ab2ba98d26c8a5e A85 `CHIINV(0.975,5)`; 73f053591a54bc43 A108 (MOD of
  `CHISQ.INV.RT(0.975,5)`).
- Type mismatches T-2638 owned by the decision-0159 rule (other causes may own them; check first):
  gen-m9/2801c1c11c21e134 A241, 40425341c5034eed A265, 4e55500bb8ad3cd8 A341, 7bd219ec0c0c5e22 A239,
  a013a45a8fbcab4f A338. Gap triage 2026-10-30d checked them. None is this task's: the two GAMMADIST cells (#NUM! at
  a huge shape) are T-3390's, and the three CHIINV cells at a df from 1.7E+13 to 1E+16 are T-3070's.
- The gen-m9 cells above that no task was credited with, by full case id. A bare hash names no case when probes/t2436 reruns that hash,
  so `scripts/exit_owners.py` did not credit them to this task (gap triage 2026-10-30d):
  - gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A158
  - gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A125
  - gen-m9/f34f66fb7f81e673.case.json (1): Calc!A3
  - gen-m9/984cba67bbe6972b.case.json (4): Calc!A42, A49, A140, A217
  - gen-m9/a54c738c697617c3.case.json (1): Calc!A212
  - gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A152
  - gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A85
- ironcalc-calc STATISTICAL/BETA_GAMMA.xlsx BETA_DIST!L3, CHISQ.xlsx Sheet1!G3 and G4, GAMMA.xlsx
  'GAMMA.DIST GAMMA.INV'!E16.
- probes-numeric/m9-gamma-dist-a9 Calc!A1, A2, A11, A21, A22; t1049-chisqdist Calc!A16, A46, A48;
  t3148-gamma-quantile-flat-band Calc!A240.
- probes/t2753-gamma-density-ratio-sweep S!A20, A46, A47, A48, B20, B21, B46, B47, B48.
Also visibly off and T-2638's before: probes-numeric m9-legacy-chidist A71, A72 `CHIDIST(1000,30)` (shape 15).

## Acceptance criteria
- [ ] Each cell above is within half a unit of Excel's 15th significant digit, with no new baseline line.
- [ ] The candidates tried, with their per-cell ulp, are recorded in docs/excel-semantics/statistics.md; after this
      second attempt without a gain the task is time-boxed (decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes from 20 away from the mode (T-2638) and the 1-2 ulp residue of T-2367.
