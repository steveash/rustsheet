---
id: T-3617
title: "GAMMA.DIST off Excel at shapes 0.5-2.5 (x/a below 1E-6) and 20-1E3 (x/a below 0.5) on the T-3509 blind probe"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.DIST (cdf and density) matches Excel within half a unit of the 15th significant digit at the T-3509 regions no
other task owns: shape 0.5 to 2.5 with x/a below 1E-6, shape 20 to 1E3 with x/a below 0.5, and the single cells at
shape below 0.5 and 1E3 and up; or the Outcome lists the forms tried with their counts (decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, score-main-744815d06.out, "GAMMA.DIST cdf/pdf
  elsewhere" row): 37 visibly-off cells, 19 cdf and 18 density. The shape 2.5-20, x/a below 0.5 region is T-3219's
  (and T-3412's on the T-3311 branch), not this task's.
- Excel against the 50-digit math: 94 ulp (median) over the 37, up to about 1,450 ulp at the near-subnormal A166
  (4.3E-308). Ours is closer to the math at 25 of 37. All 37 are inside Gate A, so the gap is `visible`: Excel's own
  arithmetic.
- Shape 0.5-2.5, x/a below 1E-6 (cdf, 7 cells): the lower-tail series at a tiny x, e.g.
  `GAMMA.DIST(7.45986584495088E-11,2.05094603388257,1,TRUE)` Excel 8.088157212465994E-22, ours 8.088157212465939E-22,
  math 8.088157212465943E-22. Check whether this is T-3060's leading-factor form (T-3412 found it at shape above 1).
- Shape 20-1E3, x/a below 0.5 (cdf and density at the same 11 points): the prefactor x^a e^-x / Gamma(a+1) at a
  deep lower tail, e.g. `GAMMA.DIST(0.0590345556575017,85.4270502412247,1,FALSE)` Excel 7.564388379429032E-232, ours
  7.564388379428603E-232, math 7.564388379429098E-232. The cdf and the density at one point are off together, so
  the shared prefactor (its log or its Gamma) is the first suspect.
- Shape 1E3 and up (density, 5 cells; cdf 1): near the mode and at x/a 1.4 to 10 (Loader's saddle-point form,
  `loader.rs`).
- Code: `statistical/gamma.rs` (GAMMA.DIST), `statistical/incomplete_gamma/` (`density.rs`, `mod.rs`'s lower tail),
  `statistical/loader.rs`.
- Cells (excel-graph, 2026-10-31), all in probes-numeric/t3509-blind-gamma-dist.case.json, sheet S (A cdf, B density):
  - shape 0.5-2.5, cdf (7): A62, A156, A198, A222, A280, A386, A497
  - shape 20-1E3, cdf (11): A166, A254, A231, A291, A467, A28, A172, A182, A224, A258, A288
  - shape 20-1E3, density (11): B166, B254, B231, B291, B467, B28, B172, B182, B224, B258, B288
  - shape 1E3 and up: A33 (cdf); B33, B312, B458, B495, B40 (density)
  - single cells: B498 (shape 0.37), B333 (shape 1.29, x/a above 10)

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up. The probe cells are evidence for the region, not a fitting set: probe the band first
(the `excel-probe` skill).

## Acceptance criteria
- [ ] The 37 listed cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with
      their counts on a hold-out split, and files a follow-up per region left.
- [ ] Decision 0188 R4 guards: any candidate is chosen on a hold-out split of its own evidence (R4.1, train and test
      counts side by side, accepted only if it beats the correctly rounded candidate on test); it is never fitted to
      the T-3509 probe, which is scored before and after and both per-function tables go in the Outcome
      (docs/research/t3509/README.md); a fresh log-uniform sweep is filled after the model is frozen (R4.2); every fitted
      constant, cut-point or table carries `FITTED(T-XXXX, n cells, range)` and falls back to accurate math outside
      that range (R4.3); nothing is fitted below the 4-ulp oracle floor (R4.4); the Gate A envelope result is reported
      (R4.5).
- [ ] No gated cell regresses; no new known-failure line in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.DIST at shape 2.5 to 20, x/a below 0.5 (T-3219, T-3412); CHISQ.DIST.RT (T-3521, T-3310, T-3269).
