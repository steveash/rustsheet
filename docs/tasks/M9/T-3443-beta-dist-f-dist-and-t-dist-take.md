---
id: T-3443
title: "BETA.DIST, F.DIST and T.DIST take bratio's basym at two large shapes, with BETA.INV still inverting to Excel's root"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.DIST, F.DIST and T.DIST (and their legacy and right-tail forms) take bratio's basym branch where Excel does (two
shapes above 100, λ within 3% of the smaller one), as BINOM.DIST and NEGBINOM.DIST do since T-3352, without moving
BETA.INV or F.INV off Excel's root.

## Context
- T-3352 ported TOMS 708 `basym` (`bratio/basym.rs`, `bratio::basym_tails`) for the binomial family only.
  statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n: basym and Loader, within 1E-11 of the math (T-3352)".
- With basym under BETA.DIST (`bratio::kernel_tails_with`), ironcalc-calc BETADIST_BETAINV.xlsx G29
  `BETA.DIST(0.555,1000,1000,TRUE,0.1,1)` becomes Excel's 0.6903551301985376 bit for bit (a known failure today), but H29
  `BETA.INV(G29,1000,1000,0.1,1)` becomes 0.5550000000000002 instead of Excel's 0.555 (a pass today). It stays
  0.5550000000000002 whether BETA.INV's search (`beta.rs` `beta_quantile`, `incomplete_beta::search_tails`) crosses
  Cephes' fraction or basym. The cause found: for p > 1/2 the search crosses the upper tail at the target fl(1 - p);
  basym's upper tail at the root double is w = 0.30964486980146244, one ulp above 1 - G29, because Excel's lower tail is
  (0.5 - w) + 0.5, rounded; so the bracket is (root, next) and its midpoint rounds to the next double.
- Tried in T-3352 and rejected (each regresses gated cells): `nearest_crossing_from` instead of `refine_increasing` in
  `beta_quantile` (8 regressions: BETADIST_BETAINV H2, H16, H63, BETA_GAMMA J3, t2435-dist-extremes A70, A71, B70, B71);
  AS 109's flip 1 - Q(1 - p, b, a) for p > 1/2 at every shape (4: gen-m9 787ba8d801105a60 A259, BETA_GAMMA J3,
  t2435-dist-extremes B73, B111; it fixed H29 and two others).
- Find Excel's BETA.INV crossing rule at these points first (excel-probe: round trips at shapes 100 to 1E5 near the
  mean), then move the callers.

## Acceptance criteria
- [ ] BETA.DIST, F.DIST and T.DIST take basym at bratio's basym points; ironcalc-calc BETADIST_BETAINV.xlsx G29 passes
      and H29 still passes.
- [ ] A probe of BETA.DIST, F.DIST and BETA.INV round trips at basym points (oracle-filled) is within 15 digits of Excel.
- [ ] No gated cell regresses; the rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's arithmetic inside basym (last digits; Phase 1b).

## Outcome
BETA.DIST, F.DIST and F.DIST.RT (and BETA.INV's and F.INV's searches) take bratio's basym at its points:
`Branch::Basym` is now one of `bratio::ported_tails`' branches, so `bratio::basym_tails` (the binomial family's
separate fallback) is gone. T.DIST never reaches basym (its kernel call has b = 1/2). BETA.INV
(`beta.rs` `beta_crossing`) returns the bracket end whose tail is nearer the target where the crossing dispatches to
basym, and the even end elsewhere. That keeps H29 at 0.555: at the root basym's upper tail is one ulp above fl(1 - p).
- Excel evidence: probes-numeric/t3443-basym-a (in-sample), t3443-basym-hold (hold-out, scored once on the frozen
  rule) and t3443-basym-sweep (fresh sweep, filled after the freeze), 220 cells each, excel-graph; generator
  docs/research/t3443/gen_probe.py. Counts (exact / within 15 digits / visibly off, main then T-3443) are in
  statistics.md "BETA.DIST and F.DIST at basym points; BETA.INV takes the nearest end there (T-3443)". Hold-out:
  BETA.DIST 1/12/37 to 7/23/20, F 0/5/35 to 2/15/23, BETA.INV 30/49/11 to 46/36/8, F.INV 5/21/14 to 3/24/13.
- R4 (decision 0188): the crossing rule is a structural choice between two candidates (nearest end, even end), scoped
  by bratio's own basym criterion, so no `FITTED` tag and no per-shape or keep-matched branch. Isolated on the same
  basym kernel (BETA.INV + BETAINV, 90 cells, exact / within 4 ulp / within 15 digits / visibly off, even then
  nearest): in-sample 39/46/1/4 to 49/35/2/4 (+10 exact), hold-out (scored once) 35/45/3/7 to 46/34/3/7 (+11), fresh
  sweep (filled after the freeze) 43/36/1/10 to 48/30/3/9 (+5). R4.1 margin: hold-out exact rate at most 5 points
  below in-sample; 54.4% in-sample, 51.1% hold-out, 53.3% sweep (the margin was named after scoring; the sweep came
  after the freeze). Gate A has no table for these functions, so the envelope was checked on the sweep: all 220 cells
  within 3.2E-13 relative of Excel, the cdf cells within 1.4E-13 of 50-digit math, the inverse cells that resolve in
  mpmath (148) within 1.9E-13; none outside 1E-9. The cells the nearest end loses are listed in statistics.md.
- Acceptance 2 is partly met: the round trips' remaining visibly-off cells (in-sample BETA.INV 4, F.INV 13) follow the
  cdf's last digits, which are Excel's arithmetic inside basym (this task's out of scope). T-3597 owns them, with the
  cdf cells still visibly off (BETA.DIST 33 and F 17 in-sample).
- Eval delta (main to HEAD): ironcalc-calc BETADIST_BETAINV.xlsx G29 fixed and H29 still passes (known failures
  335 to 334); probes-numeric t1543-inverse-cdf-tiny-p 2 cells within 15 digits to exact, t3354-f-dist-sweep 1 visibly
  off to within 15 digits; no other cell of probes, probes-numeric, gen-m9 or ironcalc-calc moved; no gated
  regression; in-scope visibly-off cells 2621 to 2600.
