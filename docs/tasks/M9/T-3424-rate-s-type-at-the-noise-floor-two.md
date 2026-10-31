---
id: T-3424
title: "RATE's type at the noise floor: Gate A sweep cells where one side errs"
milestone: M9
lane: functions-financial
status: todo
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
RATE gives Excel's type, a number or `#NUM!`, on the 16 Gate A sweep cells of the T-3348 sweeps where the secant
ends in the residual's rounding noise and the engine and Excel disagree on the type. Decision 0188 R2 allows 0 such
cells, so RATE's Gate A fails until they are settled.

## Context
- T-3348 made a flat secant hold at its iterate (financial.md "RATE holds on a flat secant and runs on below -1").
  The iterate returns when |f| < 1e-7 or when it passes the cap's looser test (|f| < 1e-4); otherwise the run is
  `#NUM!`. Decision 0198 records the trade.
- **Cells T-3348 lost (13).** The old model's `#NUM!` matched Excel; the hold returns a number. Returning `#NUM!`
  on them is a goal of this task, through a model of Excel's noise, not through a rule on the hold (decision 0198):
  - probes-numeric/t3348-rate-gate-a-sweep Calc!A296:
    `=RATE(351,-1684985188.67608,607019725001.509,-15796156082.0366,0)`. Excel `#NUM!`; the engine gives
    1.8814890660971136e-6, 2.4e-8 relative off the math (0.0000018814890215042125897557712909148668011313466560856).
    From step 56 the iterates bounce in the noise (|f| 1 to 40 on terms of 6e11) until a flat secant at step 119
    lands on |f| = 2.1e-5 and holds.
  - probes-numeric/t3348-rate-gate-a-fresh (the sweep drawn after the freeze) Calc!A59, A70, A74, A77, A89, A90,
    A95, A157, A162, A173, A275 and A292. The cells, Excel's `#NUM!`, the engine's values and the 50-digit roots are
    listed in T-3348's Outcome. They hold at steps 11 to 89 on |f| 2.4e-7 to 5.5e-5.
- **Cells neither model gets right (3).** Excel returns a number; the engine gives `#NUM!`:
  - t3348-rate-gate-a-sweep Calc!A272: `=RATE(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)`, Excel
    0.040657819686895275, math 0.040657819686896052522303945460859198781648510530346. It holds at step 8 on
    |f| = 1.286e-4, above the bound. Excel's value is about 2 ulps from the model's held iterate
    0.04065781968689529, so Excel's last steps differ from the model's.
  - t3348-rate-gate-a-fresh Calc!A251 and A272 (in T-3348's Outcome). They hold at |f| 2.4e-4 and 9.5e-4.
- All 16 have |pv| and |fv| from 1e6 to 1e12. Across the 1,796 RATE oracle cells, Excel returns 133 holds and errs
  on 13 below the 1e-4 bound, and errs on 95 and returns 3 above it. No rule on the hold gives them all Excel's type
  (decision 0198):
  - an absolute bound: Excel errs at 2.4e-7 and returns up to 9.5e-4;
  - a relative bound: errors and returns overlap from 5e-20 to 1.4e-16;
  - zero step against distinct iterates: Excel errs on 12 of 125 zero-step holds and 1 of 21 distinct ones;
  - a step cut: the losses run from step 11 to step 119.

  A bound below 2.1e-5 alone would turn 5 gated cells Excel returns into `#NUM!` (probes/t3348-rate-fv-nonzero
  A113, A117, A126, A167, A183, held |f| 3.9e-5 to 7.2e-5).
- The lead is the residual's bits near the root, the same cause as T-3389's misses. A171's second iterate needs
  Excel's f to be about -2.71e-7 where the model has -1.67e-7. A residual form that reproduces Excel's noise likely
  settles both tasks; it will be a structural model under reviewer check 7, with its own hold-out and fresh sweep.
- Code: `compute_rate` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; fit
  lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] The 16 cells above have Excel's type, and the numbers are within 15 significant digits of Excel. If Excel's
      noise cannot be reproduced, the task records the residual forms tried and the cells stay known misses;
      RATE's Gate A then stays failed (no exception exists for it).
- [ ] No gated cell regresses, and the 5 fv-nonzero holds above stay numbers.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells of the right type that are not Excel's bits (T-3389).
