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
  512 Newton steps and 4,096 step adjustments (64 and 512 for Student's t), NaN for a NaN first estimate (statrs panicked) and for a
  non-finite Newton step (statrs' adjustment loop never ended). A sweep of 13 million arguments of the domains the
  functions hand it found at most 377 steps (F at p near 1E-162; Beta 117; t 27) and 1,189 adjustments where statrs
  finishes; a unit test checks the port is statrs bit for bit on 60,000 of them. `beta_quantile`, `f_quantile` and
  `t_quantile` call the port instead of `inverse_cdf`; their bisections were already bounded.
- **Weights (release):** a full-cost incomplete-beta cdf (statrs `checked_beta_reg` at its 140 continued-fraction steps
  plus three `ln_gamma`, shapes near 1E5 at x = 0.5) measures 3.1 to 3.4 us a call (load 8 on 4 cores; 0.15 us for
  `I_0.4(2,3)`), a step adjustment about 13 ns. Each weight charges the bound of its kernel's caps at that cost, as the
  performance review asked: BETA.INV/F.INV max(512 steps = 1.6 ms, 1,100-cdf bisection = 3.4 ms, F's 1,301-cdf bisection
  = 4.0 ms) → `Weighted(2^18)`, 512 elements, about 2.1 s at the bound; T.INV/CONFIDENCE.T max(64 steps + 512
  adjustments = 0.21 ms, 264-cdf bisection = 0.82 ms) → `Weighted(2^16)`, 2,048 elements, about 1.7 s. The cases measured
  cost far less (lifted over `SEQUENCE(n)*0+p`): 167 us for `BETA.INV(1E-300,0.5,1E6)`, 128 us for
  `F.INV(1E-300,1E10,1E10)`, 105 us for the 377-step `F.INV(3.718810279553989E-162,4532,43)`, 17.5 us for
  `T.INV.2T(1E-9,99999)`, 1 to 2 us in the middle of the range. On main `F.INV(1E-50,6000,4)` and
  `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` did not finish in 20 s and `F.INV(1E-300,5,5)` panicked;
  here each is `#NUM!` in about 0.1 ms. A unit test drives the port past its step and adjustment caps.
- **Deviations** (statistics.md "Quantile searches: per-call cap and lifting"): lifts past 512 (BETA.INV, F.INV) or
  2,048 (T.INV, CONFIDENCE.T) elements are `#ERROR!`; where statrs panicked or hung the result is `#NUM!` and Excel
  gives a number (probes/stat-quantile-lift E1-E3, E6, E9: T-1871).
- **Scalar changes:** only the panics and hangs above become `#NUM!`. Every other scalar value is main's (probes-numeric
  identical cell by cell); error messages are now empty, as in T-1290's kernels.
- **Evidence:** new probes/stat-quantile-lift (124 formula cells, excel-graph 2026-10-18). Known failures in it: the
  follow-up functions' lift cells (T-1872, T-1873), the TRUE elements of BETA.INV, F.INV, F.INV.RT (L2, L3, L19:
  T-1849, which now lists them), the tiny-p F.INV and extreme BETA.INV cells (E1-E6, E9: T-1871), E7 (`F.INV(1E-50,2000,2)`,
  84% off as on main: statrs converges to a wrong root; T-1871) and the last-digit misses E8, E10-E12 (4 to 357 ulp,
  as on main; T-1871).
- **Eval:** `update-baseline --timeout 240`: 0 regressions in every gated suite; probes known failures 2423 → 2433
  (33 fixed, all stat-argument-modes lift cells of the nine functions; 43 in the new case, listed above). Compared cell
  by cell with main's binary: probes-numeric, ironcalc-calc, fixtures, poi, gen-m5 to gen-m9 identical (0 fixed, 0 new,
  0 changed failing values); probes differs only in those 33 fixes and the new case (main panics on it, 81/124 here).
- **Quality:** duplicated lines 4328 → 4248, static_analysis.rs 1199 → 1181 (after merging main at T-1842) (decision 0010 line). NOTICE names the
  statrs port.

