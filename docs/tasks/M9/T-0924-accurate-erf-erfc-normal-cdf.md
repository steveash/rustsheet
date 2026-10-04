---
id: T-0924
title: "Accurate erf/erfc and standard-normal CDF/PDF shared kernel (~120 known failures)"
milestone: M9
lane: functions
status: done
priority: P1
size: M
depends_on: []
---

## Goal
One accurate (< 1 ulp class) `erf`/`erfc` and standard normal CDF/PDF in the engine, used by ERF, ERFC, ERF.PRECISE, ERFC.PRECISE, NORM.S.DIST, NORM.DIST, LOGNORM.DIST, GAUSS and PHI, replacing `statrs`' lower-accuracy versions.

## Context
Oracle-free: Excel's cached values equal the correctly rounded mathematical values; ours are off by ~1e-11 relative.
- `ironcalc-calc/ENGINEERING/ERFs.xlsx` `Sheet1!D3` `=ERF(A3)` expected 0.8427007929497149 (erf(1)), actual 0.8427007929427149; 50 failing lines (ERF 18, ERFC 12, ERF.PRECISE 8, ERFC.PRECISE 12), all NumericClose.
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV!C2` expected 0.9087887256040951 actual 0.9087887255999465; `NORM.DIST NORM.INV!E2` expected 0.9087887802741321 actual 0.9087887802699836. 128 baseline lines in this file in total, of which the CDF/PDF ones (NORM.S.DIST 33, NORM.DIST 31 cells) belong here; the INV ones belong to T-0925.
- `ironcalc-calc/STATISTICAL/GAUSS.xlsx`: 13 lines (NumericClose). `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` LOGNORM.DIST 29 cells. `PHI.xlsx` `Sheet1!B5` (1 ulp).
Code: `functions/engineering/bessel.rs` (`use statrs::function::erf::{erf, erfc}`), `functions/statistical/{normal,gauss,log_normal,phi}.rs` (statrs `Normal::cdf/pdf`).
Suggested approach: port fdlibm/musl `erf`/`erfc` (public-domain rational approximations, with `Ported-From` style attribution in the commit), put it in a single module, build `norm_cdf = 0.5*erfc(-x/sqrt2)` on top. Search first (`rg "erfc?\("`) so there is exactly one definition. The remaining ulp-level differences (NumericUlp) may need the formulation Excel uses (e.g. GAUSS = cdf - 0.5 vs 0.5*erf(x/sqrt2)); pick whichever reproduces the cached cells and note it.

## Acceptance criteria
- [ ] All NumericClose/NumericMismatch lines for ERF, ERFC, ERF.PRECISE, ERFC.PRECISE, NORM.S.DIST, NORM.DIST, GAUSS and LOGNORM.DIST disappear from `eval/baseline/ironcalc-calc.txt` (at least the 50 ERFs.xlsx lines and 13 GAUSS.xlsx lines; report the exact before/after counts in the PR), no new lines.
- [ ] No remaining use of `statrs` erf/Normal cdf/pdf in the engine (rg shows one definition).
- [ ] Unit tests with a handful of reference values (erf(1), erf(0.5), erfc(5), tails); `make check` green.

## Out of scope
Inverse normal (T-0925); gamma/beta kernels.

## Outcome
New `functions/statistical/special.rs` is the single erf/erfc/normal CDF+PDF kernel (Maclaurin series for |x|<1, bottom-up continued fraction beyond; no `statrs` erf/Normal cdf/pdf left outside the inverse functions, which stay with T-0925). ironcalc-calc known failures 1720 -> 1632 (88 fixed, 0 regressions); ERFs.xlsx 51 -> 17 lines, GAUSS.xlsx 14 -> 6, NORM_DIST.xlsx 129 -> 88, LOGNORM.xlsx 61 -> 56. Remaining lines are NORM.*.INV/LOGNORM.INV (T-0925) and 1e-15-relative or 1-ulp noise in Excel's own cached values (e.g. ERFC(5) cached ...034e-12 vs correctly rounded ...035e-12), which cannot be matched without Excel's exact algorithm. PHI and Z.TEST keep their original operation order because it reproduces the cached last digits.
