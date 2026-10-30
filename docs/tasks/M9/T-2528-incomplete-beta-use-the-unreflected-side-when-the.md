---
id: T-2528
title: "Incomplete beta: accurate upper side at a huge shape where the reflected continued fraction does not converge (T-1538 split)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The regularized incomplete beta (`incomplete_beta.rs` `beta_tails`) is accurate above the mean at a huge
shape, where the reflected side's continued fraction does not converge within its 300-step cap.

## Context
Found during T-1538. `F.DIST(2,5,1E10,TRUE)` = `I_x(2.5, 5E9)` at x = 1E-9 is above the mean 5E-10, so
`beta_tails` evaluates the reflected `I_y(5E9, 2.5)` with y = 1 - 1E-9, whose `incbcf` fraction needs about
42,000 steps; truncated at 300 it gives 0.9247647476587724, where Excel's 0.9247647537543169 is the true value
(probes-numeric/t1530-f-df-cut A8; also A12, A14). Running that fraction to convergence does not help: it stays
about 1E-7 relative off at every cap from 300 to 300,000 (rounding piles up), for x from 2 to 20 on the F scale.

A first attempt (falling back to the unreflected side when it converges) was rejected in review (2026-10-23):
the unreflected fraction "converges" to garbage far above the mean, e.g. `F.DIST.RT(20,5,1E10)` = 1.0005 (true
5.285E-20), `F.DIST.RT(10,5,1E10)` = `#NUM!` (true 1.3858E-9), and it moved the excel-graph cells of
probes-numeric/t1538-beta-dist-at-excel-inv A15, A17, A18 by 5E-5 relative. The candidate is TOMS 708's large-b
expansion (`bgrat`, used by `bratio` for a huge b with a small a) or Temme's uniform expansion.

## Acceptance criteria
- [ ] t1530-f-df-cut A8, A12, A14 are within half a unit of the 15th digit of Excel's values.
- [ ] New probes-numeric cases (excel-graph) for F.DIST and F.DIST.RT at x = 2, 5, 10, 20, 50 with df (5,1E10),
      (5,1E9), (5,1E8), and the matching BETA.DIST reflections, pass at 15 digits; no cell of t1530-f-df-cut or
      t1538-* moves away from Excel (diff against main).
- [ ] A unit test sweeps x above the mean for a = 2.5, b = 5E9 and asserts 0 <= I <= 1 and finiteness.
- [ ] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
- Excel's BETA.INV search at a huge shape (T-1538).

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Outcome
- New `large_shape_beta.rs`: `DiDonato` and Morris' expansion of `I_x(a, b)` in Q(b, z) (TOMS 708 `bgrat`'s
  series, in accurate arithmetic: ln x and z in double-double, Legendre's fraction for Q/r evaluated from the bottom
  up, Gamma(a+b)/(Gamma(a) nu^b) without cancellation). `incomplete_beta.rs` `beta_tails` takes it after the
  reflection at a first shape of 1E6 or more with b/a at most 1E-6; Cephes' fraction did not converge there.
  Within 2E-15 of a 120-digit mpmath reference on the probe points (60 digits is itself 4E-14 off at x = 50).
- t1530-f-df-cut A8 bit-exact, A12 and A14 within half a unit of the 15th digit (were 8E-8 off).
- New probes-numeric/t2528-f-dist-huge-df2 (60 cells, excel-graph): 38 within 15 digits; the other 22 are where
  Excel itself is 1E-15 to 1.5E-14 off the true value (statistics.md "Incomplete beta at a huge first shape"),
  handed to T-3205.
- Eval against main: +16 passing cells, 44 cells closer to Excel, none further (t1530-f-df-cut +1,
  t1538-beta-dist-at-excel-inv +6, t3007-beta-dist-ln-x-overflow +3, t2528 +6; gen-m9 FDIST(7,10,1E+8),
  FDIST(37.5,...), BETADIST(1E-10,0.5,1E10), m9-legacy-fdist A86, t2435-dist-extremes A38 from 1E-7 to 1E-15 off).
- A first try at b/a <= 1E-6 alone also took tiny b at a moderate a (t2464-r2-bpser-small-tiny, 8 cells lost);
  the 1E6 floor on a keeps them.
- BETA.INV's search keeps the expansion's leading term (`inverse_beta.rs` `GAMMA_LIMIT_SHAPE_RATIO`), bit-exact on
  t1538-f-inv-huge-df's near-1 cells.
