---
id: T-0908
title: "BESSELJ/BESSELY parity for |x| >= 8"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: M
tier: opus
campaign: visible
depends_on: [T-0900, T-1004]
---

## Goal
T-0900 made `|x| < 8` bit-exact. The remaining BESSELJ/BESSELY failures in
`ironcalc-calc/ENGINEERING/BESSEL.xlsx` are x = ±10 and ±32 (Excel is ~1e-6..1e-10 off the true
values, differing from fdlibm and from NR's asymptotic branch).

## Notes
Kernel map (T-1522, decision 0056): rank 9 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Once settled, drop the fdlibm fallback for |x| >= 8 in `bessel_excel.rs` if Excel never uses it.

Also probe Excel's real cap on the order `n` (currently UNVERIFIED: > 100_000 returns #NUM!).

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1004 sweep (`eval/suites/probes-numeric/cases/m9-bessel.case.json`, filled), never blind.

## Acceptance criteria
- [ ] Identify Excel's large-argument algorithm from the cached values (fit candidate asymptotic
  expansions; use the oracle for more points via `excel-probe` if available).
- [ ] Remaining BESSELJ/BESSELY known failures removed from `eval/baseline/ironcalc-calc.txt`.
- [ ] `docs/excel-semantics/engineering.md` updated.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 370 in total.
- BESSELJ: 187 cells (13-14 digits right 6, 10-12 18, 6-9 120, fewer or 0 vs nonzero 43); most in `probes-numeric/m9-bessel.case.json` 170; `ironcalc-calc/ENGINEERING/BESSEL.xlsx` 17. e.g. `BESSELJ(8,0)` Excel 0.17165080731769355, ours 0.1716508071375539
- BESSELY: 183 cells (13-14 digits right 0, 10-12 20, 6-9 163, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-bessel.case.json` 174; `ironcalc-calc/ENGINEERING/BESSEL.xlsx` 9. e.g. `BESSELY(8,0)` Excel 0.22352148924968093, ours 0.2235214893875662

Excel is the inaccurate side here (errors 1e-9 to 7e-5 against the true value); the engine's fdlibm values are accurate, so only reproducing Excel's algorithm reaches 15 digits.
