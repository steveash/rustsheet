---
id: T-3013
title: "GAMMA.INV at a shape below 1 and a tiny p gives Excel's quantile, not the exact one (gen-m9 984cba67bbe6972b A43)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] gen-m9/984cba67bbe6972b.case.json Calc!A43 is within 15 digits of Excel's value (bit-exact or named in the
      Outcome with its ulp).
- [ ] The known-failure line for Calc!A43 disappears from eval/baseline/gen-m9.txt. T-3012's probe cells that the
      fix makes pass drop from eval/baseline/probes.txt. No new known-failure lines appear in any suite.
- [ ] The form used is cited in docs/excel-semantics/statistics.md with T-3012's cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Huge shapes (T-2982) and ordinary-shape last digits (T-0972, T-2367).
- Bit-level residue left within 15 digits: file it as a `defer: phase-1b` follow-up.

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- gen-m9/146b46e74f2dac3b Calc!A152 `GAMMA.INV(0.5,1E-300,ROUNDUP(2^53+1,4))`: Excel 0, ours 2.2250738585072014e-308 (the quantile (p·Gamma(a+1))^(1/a)·scale underflows; ours stops at the smallest normal double)
- gen-m9/60b0b6e69ad35a45 Calc!A210 `GAMMAINV(0.5,MINVERSE(INT(60)),1.5E+10)` (shape 1/60): Excel 7.404834485321526e-09, ours 7.404834485321458e-09
- gen-m9/c2974c061384fdd9 Calc!A179 `GAMMAINV(0.001,0.01,SUMPRODUCT(Data!C2:D9))` (shape 0.01): Excel 6.988576892175477e-293, ours 6.988576892175518e-293
At shape a the quantile's relative error is 1/a times that of p·Gamma(a+1), so these three show Excel's small-shape form strongly.
