---
id: T-1929
title: "Route CHISQ.DIST.RT df 1, Z.TEST (with its error rules), POISSON, GAMMA.DIST shape 0.5 and T.DIST x < 0 through the T-1778 ERF/ERFC identities"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1916]
---

## Goal
The distribution functions that Excel builds from its ERF/ERFC kernels compute through the same
compositions, so they inherit the bit-exact kernels T-1916 shipped and Excel's identities hold bit for bit.
In scope: CHISQ.DIST.RT/CHIDIST/CHISQ.TEST/CHISQ.DIST at df 1, GAMMA.DIST's cdf (shape 0.5 and its x/b scaling),
Z.TEST/ZTEST (the NORM.S.DIST form, its z, and the documented #DIV/0! and #NUM! rules), the T.DIST x < 0
identity (evaluated, kept on its old path, Outcome), and tests for the POISSON and GAUSS identities.

## Context
Successor of the research branch's routing task, which was numbered T-1780 there (that id is taken on
main by "Bound HSTACK and VSTACK results by the shared array limit"). T-1778 established the identities from
oracle probes (docs/excel-semantics/statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM,
CHISQ, GAMMA.DIST and POISSON (T-1778)"):
- `ERFC.PRECISE(x)` = `CHISQ.DIST.RT(2x^2,1)`; `CHISQ.DIST.RT(2z,1)` = `ERFC.PRECISE(SQRT(z))` with the
  internal sqrt rounded twice (32,768 of 32,768); CHIDIST and CHISQ.TEST at df 1 are the same.
- `ERF.PRECISE(x)` = `GAMMA.DIST(x^2,0.5,1,TRUE)` = `CHISQ.DIST(2x^2,1,TRUE)` (15,360 of 15,360 with
  `GAMMA.DIST(z,0.5,1,TRUE)` = `ERF.PRECISE(SQRT(z))`); shapes 0.5 and 1 are special-cased on exact equality.
- `CHISQ.DIST(x,k,TRUE)` = `GAMMA.DIST(x/2,k/2,1,TRUE)`; `GAMMA.DIST(x,a,b,TRUE)` = `GAMMA.DIST(x/b,a,1,TRUE)`;
  `POISSON.DIST(k,m,TRUE)` = `CHISQ.DIST.RT(2m,2(k+1))`.
- `Z.TEST` and `GAUSS` are NORM.S.DIST wrappers; `T.DIST(x,n,TRUE)` for x < 0 = `T.DIST.2T(-x,n)/2`.
- Already done in T-1916: `normal_cdf` (NORM.DIST, NORM.S.DIST, LOGNORM.DIST) uses
  ERFC(fl53(fl64(-z*fl(sqrt(0.5)))))/2 with z = (x-m)/s.
- Failing evidence on main: probes-numeric `t1916-erfc-tail` (1,076 `CHISQ.DIST.RT(2*x*x,1)` cells near the
  cutoff) and `t1916-erf-wrappers` (`CHISQ.DIST.RT`, `Z.TEST` cells), copied from `kmyst-erfc-cutoff`,
  `kmyst-erfc-chiruns1`, `kmyst-erfc-r9side1` and `kmyst-erfc-r9side4`.
Code: crates/rustsheet-engine/src/functions/statistical/{chisq,gamma,poisson,z_test,gauss,t_dist}.rs and
`incomplete_gamma.rs`.

## Acceptance criteria
- [x] Each identity above holds bit for bit in the engine, with a unit test per identity on probe inputs
  (except T.DIST x < 0, kept on its old path: Outcome).
- [x] The CHISQ.DIST.RT and Z.TEST cells of `t1916-erfc-tail` and `t1916-erf-wrappers` pass.
- [x] `make eval-baseline`: known failures do not grow in any suite (NORM_DIST, LOGNORM, GAUSS, ERFs, CHISQ,
  GAMMA, POISSON workbooks and the probes). If routing one function regresses a suite, it keeps its path and
  the Outcome records why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Odd-df upper chi-square (Q(1.5), Q(2.5)) and the shape-0.5 density, which use separate generic paths.
- NORM.S.INV's central region (T-0925).

## Outcome
- Routed, all through the T-1916 kernels: CHISQ.DIST.RT, CHIDIST and CHISQ.TEST at df 1 are
  `erfc(sqrt64(x/2))` and CHISQ.DIST(x,1,TRUE) is `erf(sqrt64(x/2))` (`erf.rs` `gamma_half_upper`/`_lower`, the
  root rounded twice by `extended_sqrt`); GAMMA.DIST's cdf scales by the twice-rounded x/b at every shape (not
  statrs' x*fl(1/b)) and is `erf(sqrt64(x/b))` at shape exactly 0.5; Z.TEST/ZTEST is NORM.S.DIST(-z,TRUE) with
  z = fl(fl(mean-x)*sqrt64(n))/sigma, every step x87 (240 of 240 wrappers cells and 30 of 30 t1048/t1049 numeric
  cells; the textbook forms score 24 of 30 and 141 of 240). POISSON.DIST already shared Q(k+1, m) with
  CHISQ.DIST.RT(2m,2(k+1)) and GAUSS was already NORM.S.DIST-0.5: unit tests only.
- Kept on its old path: T.DIST(x<0,n,TRUE) = T.DIST.2T(-x,n)/2. Our T.DIST.2T is statrs' 2*(1-cdf(x)), which
  cancels to 0 for large x, so the routed T.DIST(-1E6,10,TRUE) was 0 (Excel 1.230468749943611E-56) and T.INV of it
  #NUM! instead of Excel's -1000000.0000000001 (T_DIST.xlsx "T.DIST T.INV T.DIST.RT" D19/E19; the
  rustsheet_xlsx `generated_tests` failure in CI). Computing T.DIST.2T as 2*cdf(-x) keeps the identity and T.DIST,
  but loses 8 Excel-matching `T.DIST.2T(1,10)` cells (T_DIST.xlsx C11, C25, C26, C34, C35, C38, C39;
  probes-numeric m9-t-dist-2t A64). The identity waits for Excel's T kernel (T-0969); statistics.md records it.
  The inverse searches (T.INV, T.INV.2T, CHISQ.INV, GAMMA.INV) call their own statrs-port cdfs, not the changed
  worksheet paths, so only a chained cell like E19 could see a routing change.
- Z.TEST also takes Excel's documented errors (statistics.md, T-1048/T-1049 sections): one value is #DIV/0! with
  sigma given too (t1916-erf-wrappers column T, 120 cells), and sigma < 0 is #NUM!. A non-finite p (data whose
  sum overflows with sigma omitted) is #NUM!, not a NaN number, and the fork's plain-double sigma/sqrt(n) underflow
  guard stays #DIV/0!: both unprobed, owned by T-1942.
- Unit tests: `test/statistical/test_erf_identities.rs`, one per identity, both sides on probe inputs against the
  oracle's bits; `test/util.rs` gains `check_cells_with_numbers` for 17-digit probe inputs.
- eval (per-cell, against origin/main before the change): probes-numeric 49,353 -> 51,146 of 58,079 cells
  (+1,793, 0 regressions; t1916-erfc-tail 9,066 -> 10,142 of 10,142, t1916-erf-wrappers 12,516 -> 13,137 of
  13,137, plus m9-chisq*, m9-gamma-dist-a0p5 2 -> 17 of 30, m9-legacy-chi*, t1048/t1049 ztest and chisq cases);
  probes known failures 2133 -> 2129 (stat-argument-modes Z.TEST/ZTEST); ironcalc-calc 1144 -> 1144 (no cell
  changed); fixtures 9 -> 9. poi is not fetched here.
- Still failing and owned elsewhere: generic-shape GAMMA.DIST/CHISQ/POISSON and the shape-1 special case (T-0971),
  T.DIST/T.DIST.2T/T.DIST.RT values and the T.DIST identity (T-0969), Z.TEST argument coercion (text "5", TRUE, a blank or omitted sigma;
  T-1813), the shape-0.5 density and odd-df upper chi-square (out of scope above).
