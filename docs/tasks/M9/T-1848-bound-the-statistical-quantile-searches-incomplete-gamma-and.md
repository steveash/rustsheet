---
id: T-1848
title: "Bound the statistical quantile searches, incomplete-gamma and HYPGEOM cdf loops, then lift them (SPEC-0011)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The statistical functions whose every argument is scalar and that T-1290 left out of the SPEC-0011 table lift over
arrays and ranges like the rest, once each kernel has a per-call cap of its own: BETA.INV, BETAINV, F.INV, F.INV.RT,
FINV, T.INV, T.INV.2T, TINV, CONFIDENCE.T, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, CHISQ.INV, CHISQ.INV.RT, CHIINV,
GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON, POISSON.DIST, HYPGEOM.DIST and HYPGEOMDIST.

## Context
SPEC-0011's security rule: a kernel whose loop length depends on its argument values must be `KernelWork::Weighted`
and keep a per-call cap of its own. These do not have one (T-1290 read the code; statrs 0.18):
- statrs' `inverse_cdf` for Beta, F and Student-t (`inv_beta_reg`'s open `loop`s, and the default doubling search),
  used below the shape/df thresholds of `beta_quantile`, `f_quantile` and `t_quantile` (statistical/beta.rs,
  fisher.rs, t_dist.rs), and the default `inverse_cdf` of `Gamma` and `ChiSquared` (GAMMA.INV, CHISQ.INV*).
- statrs' regularized incomplete gamma (`gamma_lr`, `gamma_ur`: series and continued fraction run to a 1E-15
  tolerance) behind GAMMA.DIST, CHISQ.DIST*, POISSON's cdf. Measured in release: `Gamma::new(a,1).cdf(a)` takes
  30 us at a = 1E6, 1.6 ms at 1E10, growing as sqrt(a); GAMMA.DIST's alpha has no upper limit, and once a + 1 == a
  the series no longer converges.
- statrs' `Hypergeometric::cdf` sums the pmf term by term: `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)`
  takes about 4 s.
Each needs a bounded kernel first (an iteration cap that keeps today's values, or an own quantile search like
`bisect_increasing`), with its weight, then the `Lift` entries (evidence: probes/stat-argument-modes, every argument
over `{v;v}` and the first over a range, all lift; probe the later arguments over a range first, as
probes/stat-later-argument-lift did for T-1290). Split by family if it is larger than M.

## Acceptance criteria
- [x] (subset: the quantile searches; the incomplete-gamma family moved to T-1872 and HYPGEOM to T-1873, each with its
  reason in kinds.rs) Every function above either has a kernel with its own per-call cap, entered as `Weighted` with a
  citing comment, or stays out with its reason in kinds.rs; its lift cells of probes/stat-argument-modes pass.
- [x] A test shows the worst-case scalar call of each capped kernel is bounded (no hang), with the values that
  probes-numeric and ironcalc-calc already check unchanged.
- [x] Each directive-5 cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Scope (size M, a coherent subset):** BETA.INV, BETAINV, F.INV, F.INV.RT, FINV, T.INV, T.INV.2T, TINV and
  CONFIDENCE.T are `Lift` entries (every argument over an array and a range). The incomplete-gamma functions and their
  searches (CHISQ.*, CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON, POISSON.DIST) are T-1872,
  HYPGEOM.DIST and HYPGEOMDIST T-1873; the new probe already holds their later-argument evidence (H27-H54).
- **Bound:** statrs 0.18's `inv_beta_reg` is ported line for line to `statistical/inverse_beta.rs` with caps of its own:
  512 Newton steps (64 for Student's t), 4,096 step adjustments, NaN for a NaN first estimate (statrs panicked) and for a
  non-finite Newton step (statrs' adjustment loop never ended). A sweep of 13 million arguments of the domains the
  functions hand it found at most 377 steps (F at p near 1E-162; Beta 117; t 27) and 1,189 adjustments where statrs
  finishes; a unit test checks the port is statrs bit for bit on 60,000 of them. `beta_quantile`, `f_quantile` and
  `t_quantile` call the port instead of `inverse_cdf`; their bisections were already bounded.
- **Timings (release, lifted over `SEQUENCE(n)*0+p`, load 2-6 on 4 cores):** BETA.INV/F.INV: 167 us an element at the
  worst case (`BETA.INV(1E-300,0.5,1E6)`, a 1,100-cdf bisection), 128 us (`F.INV(1E-300,1E10,1E10)`), 105 us for the
  377-step search (`F.INV(3.718810279553989E-162,4532,43)`, 0.28 us a step, about 145 us at the cap), 1-2 us in the
  middle (`BETA.INV(0.4,2,3)`, `F.INV(0.4,5,5)`): `Weighted(2^14)`, 8,192 elements, 1.37 s at the worst case. T.INV/
  CONFIDENCE.T: 17.5 us at the worst case (`T.INV.2T(1E-9,99999)`), 14 us (`T.INV(1E-300,99999)`), 1.2 us for
  `T.INV(0.4,5)`, about 18 us at the 64-step cap: `Weighted(2^11)`, 65,536 elements, 1.15 s. On main
  `F.INV(1E-50,6000,4)` and `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` did not finish in 20 s and
  `F.INV(1E-300,5,5)` panicked; here each is `#NUM!` in about 0.1 ms.
- **Deviations** (statistics.md "Quantile searches: per-call cap and lifting"): lifts past 8,192 (BETA.INV, F.INV) or
  65,536 (T.INV, CONFIDENCE.T) elements are `#ERROR!`; where statrs panicked or hung the result is `#NUM!` and Excel
  gives a number (probes/stat-quantile-lift E1-E3, E6, E9: T-1871).
- **Scalar changes:** only the panics and hangs above become `#NUM!`. Every other scalar value is main's (probes-numeric
  identical cell by cell); error messages are now empty, as in T-1290's kernels.
- **Evidence:** new probes/stat-quantile-lift (124 formula cells, excel-graph 2026-10-18). Known failures in it: the
  follow-up functions' lift cells (T-1872, T-1873), the TRUE elements of BETA.INV, F.INV, F.INV.RT (L2, L3, L19:
  T-1849, which now lists them), the tiny-p F.INV and extreme BETA.INV cells (E1-E6, E9: T-1871) and ulp misses in
  E7, E8, E10-E12 (the existing F.INV accuracy, T-0974).
- **Eval:** `update-baseline --timeout 240`: 0 regressions in every gated suite; probes known failures 2423 → 2433
  (33 fixed, all stat-argument-modes lift cells of the nine functions; 43 in the new case, listed above). Compared cell
  by cell with main's binary: probes-numeric, ironcalc-calc, fixtures, poi, gen-m5 to gen-m9 identical (0 fixed, 0 new,
  0 changed failing values); probes differs only in those 33 fixes and the new case (main panics on it, 81/124 here).
- **Quality:** duplicated lines 4328 → 4248, static_analysis.rs 1199 → 1181 (after merging main at T-1842) (decision 0010 line). NOTICE names the
  statrs port.

