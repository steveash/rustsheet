---
id: T-3013
title: "GAMMA.INV at a shape below 1 and a tiny p gives Excel's quantile, not the exact one (gen-m9 984cba67bbe6972b A43)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3012]
campaign: visible
---

## Goal
GAMMA.INV and GAMMAINV at a shape below 1 and a tiny p give Excel's value to 15 digits, including where Excel is
not the correctly rounded quantile.

## Visible-parity stage
`visible`: Excel is the inaccurate one here. Ours is the correctly rounded quantile, so the fix reproduces Excel's
form; it is not an accuracy fix. The target is gen-m9/984cba67bbe6972b.case.json Calc!A43 within half a unit of the
15th significant digit, plus the visibly-off cells of T-3012's probe
probes/t3012-gamma-inv-small-shape (all of it; its GAMMA.DIST companion, probes/t3012-gamma-dist-at-quantile, is
T-3060's). This is time-boxed (decision 0056).

## Context
- gen-m9/984cba67bbe6972b.case.json Calc!A43, `GAMMA.INV(1E-16,ABS(TIMEVALUE("6:00")),Data!A10)` =
  `GAMMA.INV(1E-16,0.25,1E+15)`. Excel gives 6.749697893111757E-50 and we give 6.74969789311173E-50. The exact value
  is 6.7496978931117296E-50, so Excel is 4.1E-15 relative high. Full numbers and the reasoning are in T-3012's
  Context.
- No open task owned A43 (gap triage 2026-10-28). T-1891 listed it, and T-2982 hands small-shape cells to T-0972,
  but T-0972, blocked on its branch `agent/functions-stat/T-0972-gamma-inv`, lists other cells of the case (A41, A85,
  A90, A104, ...), not A43.
- Needs an oracle sweep: one cell cannot pin Excel's form. T-3012 probes whether Excel's quantile is the crossing of
  its own GAMMA.DIST and which cdf form that is. If it is the crossing, matching Excel's lower-tail cdf form at a < 1
  and refining `gamma_quantile` to its crossing should reproduce it. Check against T-0972's branch, which refines
  gamma quantiles to the crossing, before starting, so the two do not collide.
- T-3012 result (docs/excel-semantics/statistics.md, GAMMA.INV bullet): Excel's quantile is NOT the crossing of its own
  GAMMA.DIST (P is off by up to 1E-15 at a = 0.05, 0.25 and 0.5, and by up to 8.5E-15 at a = 0.1, 2.6E-14 at
  a = 0.75 and 4.7E-15 at a = 0.9; the scale is a plain final product), so crossing refinement alone will not
  reproduce A43. At a = 0.25 (the only shape probed for this) the leading term of its cdf is closest to
  `EXP(a*LN(x))/EXP(GAMMALN(a+1))` (2 of 3 points equal,
  1 ulp off at x = 1E-65).
- Engine: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs `gamma_quantile`,
  `lower_series`.

## Acceptance criteria
- [x] gen-m9/984cba67bbe6972b.case.json Calc!A43 is within 15 digits of Excel's value (bit-exact or named in the
      Outcome with its ulp). Bit-exact.
- [x] The known-failure line for Calc!A43 disappears from eval/baseline/gen-m9.txt. T-3012's probe cells that the
      fix makes pass drop from eval/baseline/probes.txt. No new known-failure lines appear in any suite.
- [x] The form used is cited in docs/excel-semantics/statistics.md with T-3012's cells.
- [ ] `make check` green; reviewers approve. (`make check` green; review is run by the caller.)

## Out of scope
- Huge shapes (T-2982) and ordinary-shape last digits (T-0972, T-2367).
- Bit-level residue left within 15 digits: file it as a `defer: phase-1b` follow-up.

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- gen-m9/146b46e74f2dac3b Calc!A152 `GAMMA.INV(0.5,1E-300,ROUNDUP(2^53+1,4))`: Excel 0, ours 2.2250738585072014e-308 (the quantile (p·Gamma(a+1))^(1/a)·scale underflows; ours stops at the smallest normal double)
- gen-m9/60b0b6e69ad35a45 Calc!A210 `GAMMAINV(0.5,MINVERSE(INT(60)),1.5E+10)` (shape 1/60): Excel 7.404834485321526e-09, ours 7.404834485321458e-09
- gen-m9/c2974c061384fdd9 Calc!A179 `GAMMAINV(0.001,0.01,SUMPRODUCT(Data!C2:D9))` (shape 0.01): Excel 6.988576892175477e-293, ours 6.988576892175518e-293
At shape a the quantile's relative error is 1/a times that of p·Gamma(a+1), so these three show Excel's small-shape form strongly.

## Outcome
- Finding: at a shape below 1 (not 1/2) and a tiny quantile, Excel's GAMMA.INV is not a search. It is the closed
  form `EXP((LN(p)+GAMMALN(a)+LN(a))/a)` (that summation order; `GAMMALN(a+1)` is an exponent ulp off at A9 and
  A16), plus one Newton step on T-3060's leading-factor series form, ungated, where that step is more than
  3.5E-14 of x. 3.5E-14 is a point inside the evidence bracket (2.78E-14, 4.1E-14), not an Excel constant. The step
  uses that form even where the engine's GAMMA.DIST is the pair (a·LN(x) above -16), and gen-m9 A254 and A396
  need it. A subnormal step counts as no step (single-cell evidence, A9 and B9). The 1E-21 cut is a choice inside
  (4.15E-22, 6.75E-21), and T-3252 probes all three choices. A subnormal quantile is `#DIV/0!` (A4, B4), one that
  underflows is 0, and the scale is a plain final product. This is why T-3012 found the quantile off the crossing.
  Implemented as `incomplete_gamma::excel_small_shape_quantile` (no loop: one closed form, at most one step), called
  from `gamma.rs` `gamma_inv_of` for unit quantiles below 1E-21. Unit test
  `small_shape_quantile_is_excels_closed_form`. Recorded in docs/excel-semantics/statistics.md (T-3013 bullets).
- Eval, per cell over all suites (full `rustsheet-eval run` of each binary, cells compared by actual value):
  - Against origin/main 9b5fc0e7b (T-3060 merged as #2418; the main merged in by 7307f2222), which isolates this
    change: fixed (failing to bit-exact) 36. gen-m9 A43 (984cba67bbe6972b, -29 ulps before), A152 (146b46e74f2dac3b,
    2.2E-308 before, Excel 0), A396 (146b46e74f2dac3b `GAMMA.INV(0.05,0.01,..)`, -8), A254 (3a25a03eb94fcfbd
    `GAMMAINV(0.5,0.01,100)`, -46), A317 (a612e89321a6df9c `GAMMAINV(..,0.3,99999)`, -100) and A179
    (c2974c061384fdd9, +37). probes/t3012-gamma-inv-small-shape A4-A6, A9-A12, A15-A17, A26-A28, A32, A33 and the
    same rows of column B (30, 6 to 487 ulps before; A4 and B4 were 0 and 5.8E-306 against Excel's `#DIV/0!`).
    Lost bit-exact: 0. Newly visibly off: 0. Visibly fixed: 35 (all of these but A396). Type or error changes among
    cells that were passing: 0. No other cell changed value. Review round 1 merged origin/main 0a8b2cfb8 (no gamma
    changes since 9b5fc0e7b) and changes no behavior: the removed product-overflow check could not fire.
    `make check`'s eval gate passes on it.
  - Against the T-3060 branch head 9bb9ed4f7: the 3,137 cases common to both runs differ in 118,792 cells. 118,756
    of these are origin/main's own changes since T-3060 branched (the same cells, same values, as a run of main:
    30,938 fixed, 5,713 lost bit-exact, 3,002 newly visibly off, 40,115 visibly fixed). The other 36 are this
    task's cells above, and no cell changed by main changes again here. Restricted to GAMMA.INV, GAMMAINV,
    CHISQ.INV and CHIINV formulas: 112 fixed (76 from main's T-0972/T-3016, 36 here), 0 lost bit-exact, 0 newly
    visibly off, 36 visibly fixed (1 from main).
  - `make eval-baseline`: gen-m9 3155 to 3149 known failures, probes 2315 to 2285, 0 regressions in every suite.
- Two fitting rounds (decision 0056). First round: the closed form with a cdf-tolerance test fitted the probe, but
  it made gen-m9 A254 worse (-46 to -354 ulps), so it was not kept. Second round: a step-size test, bracketed by
  A396 (kept at 2.78E-14) and A26/A254 (taken at 4.1E-14 and 4.4E-14). A unit-quantile-times-scale rule for every
  shape below 1 would fix B20-B22 and B29 too, but it loses three bit-exact cells (statistics.md), so it was not
  kept.
- Open, filed as T-3252 (`campaign: visible`): unit quantiles from 1E-21 up. Probe A18, A29, A30, A34-A36 and
  column B (B29, A35 and B35 visibly off) and gen-m9/60b0b6e69ad35a45 A210 (shape 1/60, -83 ulps). One step from the
  closed form on our cdf model is 1 to 34 ulps off Excel there. Shape-1/2 column B (B20-B24) is 1 to 4 ulps off,
  within 15 digits. The task names no CHISQ.INV cell; CHISQ.INV (chisq.rs) is unchanged and unprobed at df other
  than 1 below 2.
