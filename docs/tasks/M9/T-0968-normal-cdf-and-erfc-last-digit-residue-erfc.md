---
id: T-0968
title: "Normal CDF and erfc last-digit residue: ERFC, NORM.S.DIST, NORM.DIST, LOGNORM.DIST, GAUSS, Z.TEST cumulative (59 known failures)"
milestone: M9
lane: functions-stat
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-1040]
---

## Goal
The cumulative normal family and erfc reproduce Excel's last digits from the single erf/erfc kernel
in `functions/statistical/special.rs`, finishing what T-0924 started.

## Context
Kernel map (T-1522, decision 0056): rank 2 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle sweep (T-1040): probes-numeric cases `t1040-norm-s-dist-cdf-a`, `t1040-norm-s-dist-cdf-b`, `t1040-erfc`, `t1040-erfc-precise`, `t1040-gauss`, `t1040-norm-dist-cdf`, `t1040-lognorm-dist-cdf`; summary in docs/excel-semantics/statistics.md.

Triage T-0903. T-0924 (done) replaced statrs' erf with a series / continued-fraction kernel that is
correctly rounded or nearly so; the remaining lines are Excel's own last digits, which are not
correctly rounded: `NORM.S.DIST(1,TRUE)` is 0.841344746068543 while Phi(1) rounds to
0.8413447460685429 (`docs/excel-semantics/statistics.md`, "The normal CDF is not always correctly
rounded"). Code: `special.rs` `erf`, `erfc`, `normal_cdf`; callers `normal.rs`, `log_normal.rs`,
`gauss.rs`, `z_test.rs`, `engineering/bessel.rs` (ERFC/ERFC.PRECISE).
59 baseline lines in `eval/baseline/ironcalc-calc.txt` (48 NumericUlp, 11 NumericClose); cumulative
cells only (densities are T-0975):
- `ironcalc-calc/ENGINEERING/ERFs.xlsx` `Sheet1`: E4, G4, E5, G5, E6, G6, E7(C), G7(C), E8, G8, E11, G11, E13, G13, E20, G20
- `ironcalc-calc/STATISTICAL/GAUSS.xlsx` `GAUSS`: B3, B4, B29, B31, B34
- `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV`: E2, E8, E9, E17(C), E18(C), E41, E43(C), E45, E47, E48, E49, E50, E53, E54, E55, E58, E66, E67, E68
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.DIST NORM.INV`: E4(C), E8, E9, E11, E20, E32(C), E39, E58(C)
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV`: C4, C8, C9, C11(C), C13(C), C15, C19(C), C22, C29
- `ironcalc-calc/STATISTICAL/Z_TEST.xlsx` `Sheet1`: E2, E4
(C = NumericClose, unmarked = NumericUlp.)

Examples (expected vs actual):
- `NORM.S.DIST(1,TRUE)` C8/C29: expected 0.841344746068543, actual 0.8413447460685429; C9 (z = -1):
  expected 0.158655253931457, actual 0.15865525393145707.
- `ERFC(5)` E7/G7 (Close): expected 1.537459794428034e-12, actual 1.537459794428035e-12; E20: expected
  4.1838256077794166e-23, actual 4.183825607779414e-23.
- `NORM.S.DIST(-10,TRUE)` C19 (Close): expected 7.619853024160475e-24, actual 7.619853024160595e-24
  (1.6e-14: the far tail differs by more than rounding); LOGNORM.DIST E18: expected
  1.284175630643432e-117, actual 1.2841756306435955e-117.
- `GAUSS(1)` B3: expected 0.34134474606854304, actual 0.3413447460685429.

Hypothesis (the oracle is the judge): Excel's erfc is a classic rational approximation (Cody 1969
/ W. J. Cody's CALERF, or Hart et al.) rather than a correctly rounded evaluation, and
NORM.S.DIST(z,TRUE) is `0.5*erfc(-z/sqrt2)` on it (or Cody's ANORM directly); GAUSS = Phi(z) - 0.5
or 0.5*erf(z/sqrt2). The far-tail Close cells (C19, E17, E18, E32) show the tail formulation
differs too. **Needs an oracle sweep (T-0018):** fit against the T-1040 normal-CDF sweep
(family 1), never blind; T-0924's Outcome records that blind tweaks only trade cells.

**T-1524 (fit lab `erfc-x87`, 2026-10-14):** the x87 hypothesis is rejected for ERFC. The best T-1187 fraction
evaluated with 64-bit intermediates and exp (idealized or fldl2e/f2xm1/fscale) gets at most 44/205 tail
cells, the same as a 128-bit evaluation and the engine, while the same fraction in doubles gets 58.
A 64-bit evaluation does not help; the best fit so far is in doubles (statistics.md "ERFC core factor").

## Acceptance criteria
- [ ] At least 40 of the 59 lines leave `eval/baseline/ironcalc-calc.txt`, including every NumericClose
      line, and no new lines appear in any baseline (the T-0924 fixes and T-0925's cells must not
      regress; ERF/ERF.PRECISE must stay fixed).
- [ ] erf/erfc and the normal CDF stay single definitions in `special.rs` (`rg "fn erfc"`).
- [ ] The formulation that matched is recorded in `docs/excel-semantics/statistics.md`, with the
      sweep cells as evidence; the `special.rs` module comment is corrected (Excel is not correctly rounded).
- [ ] `make check` green; reviewers approve.

## Out of scope
Normal densities and PHI (T-0975); inverse normal (T-0925); legacy NORMDIST/NORMSDIST evidence (T-1048).

## Blocked because: time-boxed (decision 0056)
Two attempts on the ERFC kernel, which the normal CDF (NORM.S.DIST, NORM.DIST, LOGNORM.DIST, GAUSS)
also runs on, so nothing in this task is actionable until ERFC is fitted. Neither attempt raised the
tail's bit-exact count to a usable level (`erfc-tail` / `erfc-x87`, 205 cells of `t1040-erfc` with x >= 1):
- T-1187 (2026-10-11, fit lab `erfc-tail`, 5,794 configurations): bottom-up continued fractions in five
  forms at depths 5-1000, each combined with seven multiply and seven divide orders for exp(-x^2) and
  1/sqrt(pi), best 58; modified Lentz at tolerances 1e-14 to 1e-16, best 38; Numerical Recipes'
  incomplete gamma (gcf/gser, three gln values, eight tolerances), best 4; union 174.
- T-1186: the engine's fraction in 7 orders, Cody CALERF with and without its x*16 split, and the
  asymptotic series in five orders, all at most 22%.
- T-1524 (2026-10-14, `erfc-x87`): the best fraction (`bu_even`, depths 30/60/100, the two best
  combinations) with every operation and exp at 64 bits, idealized or fldl2e/f2xm1/fscale: at most 44,
  the same as a 128-bit evaluation (depth 1500) and the engine.
Reopens on a new hypothesis recorded here, for example: MSVC's own `exp` (not available on the agent
containers; an oracle probe of EXP near the sweep's x^2 values would supply it), a named rational
approximation not yet tried (Hart et al., W. J. Cody's ANORM for the CDF directly), or new oracle
evidence such as a dense ERFC sweep around one x to expose the formulation's error pattern.
