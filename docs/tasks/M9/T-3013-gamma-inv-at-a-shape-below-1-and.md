---
id: T-3013
title: "GAMMA.INV at a shape below 1 and a tiny p gives Excel's quantile, not the exact one (gen-m9 984cba67bbe6972b A43)"
milestone: M9
lane: functions-stat
status: todo
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
15th significant digit, plus the visibly-off cells of T-3012's probe. This is time-boxed (decision 0056).

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
  GAMMA.DIST (P is off by 6E-16 to 2.6E-14 relative at a = 0.1, 0.25, 0.75, 0.9, exact at 0.05 and 0.5), so crossing
  refinement alone will not reproduce A43; the leading term of its cdf matches `EXP(a*LN(x))/EXP(GAMMALN(a+1))`.
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
