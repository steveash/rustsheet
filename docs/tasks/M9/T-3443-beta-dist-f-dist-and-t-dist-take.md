---
id: T-3443
title: "BETA.DIST, F.DIST and T.DIST take bratio's basym at two large shapes, with BETA.INV still inverting to Excel's root"
milestone: M9
lane: functions-stat
status: todo
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
