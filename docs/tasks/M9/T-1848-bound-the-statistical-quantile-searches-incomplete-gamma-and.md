---
id: T-1848
title: "Bound the statistical quantile searches, incomplete-gamma and HYPGEOM cdf loops, then lift them (SPEC-0011)"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] Every function above either has a kernel with its own per-call cap, entered as `Weighted` with a citing comment,
  or stays out with its reason in kinds.rs; its lift cells of probes/stat-argument-modes pass.
- [ ] A test shows the worst-case scalar call of each capped kernel is bounded (no hang), with the values that
  probes-numeric and ironcalc-calc already check unchanged.
- [ ] Each directive-5 cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
