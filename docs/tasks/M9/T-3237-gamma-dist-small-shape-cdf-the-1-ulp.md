---
id: T-3237
title: "GAMMA.DIST small-shape cdf: the 1-ulp EXP(a LN x)/EXP(GAMMALN(a+1)) residue behind D20, D23, D25 of t3012-gamma-dist-at-quantile"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.DIST(x,a,1,TRUE) at a shape below 1 and a tiny x is bit-exact with Excel where T-3060's form leaves 1 to 3
ulps, so the three `GAMMA.DIST(..)/1E-k-1` cells of probes/t3012-gamma-dist-at-quantile that magnify those ulps
(D20, D23, D25) are within 15 digits.

## Visible-parity stage
`visible`: D20, D23 and D25 are visibly off on main and after T-3060. Target: those three cells within half a unit of
the 15th significant digit, with no cell of any suite regressing (the form's region today stops at a·LN(x) = -16
because, nearer 0, it would lose A31 and C25, bit-exact on the accurate pair). Time-boxed (decision 0056).

## Context
- T-3060 (docs/excel-semantics/statistics.md, its bullets after the T-3012 evidence) showed that Excel's cdf there
  is `EXP(a*LN(x))/EXP(GAMMALN(a+1))` times e^-x and the series, with a·LN(x) a double: against 50-digit mpmath
  the exponent Excel used is that double to within 0.1 ulp at every probed cell. Note the probe's literals are read
  at 15 significant digits: fit at the 15-digit x.
- What is left is the engine's own evaluation of the form: with `excel_exp`, `excel_ln` and `excel_gammaln` it is
  1 ulp off at B23 (high), B20 (low; a·LN(x) = -11.6, outside the region), A4, A21, A28 and 2 to 3 at row 24, and
  1 ulp off A31 and C25 where the pair is exact. The worksheet form itself is 1 ulp off too: ours vs Excel at
  `EXP(0.25*LN(6.749697893111757E-65))/EXP(GAMMALN(1.25))` (B28) and B30-B32 of the same case, while Excel's
  GAMMA.DIST equals its worksheet form at A28. So start from the worksheet cells: find which of EXP, LN, GAMMALN
  or `/` is off (an excel-probe case of `LN(x)`, `0.25*LN(x)`, `EXP(that)`, `GAMMALN(1.25)` and
  `EXP(GAMMALN(1.25))` at the probe's x values pins each step).
- Tried in T-3060, none fits every cell: 108 orderings (quotient plain, rounded twice, or as a product with
  EXP(-GAMMALN); e^-x by Excel's EXP or libm; series with and without its tail sum, or as 1 + aS) and x87
  extended chains with EXP and the quotient unrounded.
- Code: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs `excel_small_shape_lower`,
  `EXCEL_EXPONENT_MAX`; src/x87_exp.rs, src/x87_log10.rs, functions/statistical/gammaln.rs.

## Acceptance criteria
- [ ] probes/t3012-gamma-dist-at-quantile D20, D23 and D25 within half a unit of the 15th significant digit
      (or this task records, with probe evidence, why they cannot be reached).
- [ ] No regression in any suite (`make eval-baseline` refuses none); `EXCEL_EXPONENT_MAX` moved or removed only
      with the cells that support it.
- [ ] Findings recorded in docs/excel-semantics/statistics.md (and numbers.md if EXP or LN moves).
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.INV (T-3013).
- The cdf at x of 1 or more, and CHISQ.DIST, CHIDIST, POISSON.
