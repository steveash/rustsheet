---
id: T-1929
title: "Route CHISQ.DIST.RT df 1, Z.TEST, POISSON and GAMMA.DIST shape 0.5 through the T-1778 ERF/ERFC identities"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1916]
---

## Goal
The distribution functions that Excel builds from its ERF/ERFC kernels compute through the same
compositions, so they inherit the bit-exact kernels T-1916 shipped and Excel's identities hold bit for bit.

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
- [ ] Each identity above holds bit for bit in the engine, with a unit test per identity on probe inputs.
- [ ] The CHISQ.DIST.RT and Z.TEST cells of `t1916-erfc-tail` and `t1916-erf-wrappers` pass.
- [ ] `make eval-baseline`: known failures do not grow in any suite (NORM_DIST, LOGNORM, GAUSS, ERFs, CHISQ,
  GAMMA, POISSON workbooks and the probes). If routing one function regresses a suite, it keeps its path and
  the Outcome records why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Odd-df upper chi-square (Q(1.5), Q(2.5)) and the shape-0.5 density, which use separate generic paths.
- NORM.S.INV's central region (T-0925).
