---
id: T-3397
title: "RATE's type at the noise floor: two Gate A sweep cells where one side errs"
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
RATE gives Excel's type, a number or `#NUM!`, on the two T-3348 Gate A sweep cells where the secant ends in the
residual's rounding noise and the engine and Excel disagree on the type. Decision 0188 R2 allows 0 such cells, so
Gate A fails until these are settled.

## Context
- Found by T-3348 (financial.md "RATE holds on a flat secant and runs on below -1"). A flat secant holds at its
  iterate, which returns when |f| < 1e-4 (the cap's looser test) and is `#NUM!` otherwise. Every other flat-hold
  cell of the oracle evidence fits that rule.
- probes-numeric/t3348-rate-gate-a-sweep Calc!A272: `=RATE(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)`.
  Excel 0.040657819686895275, engine `#NUM!`, math 0.040657819686896052522303945460859198781648510530346. The model
  converges in 5 steps and goes flat at step 8, holding r1 = 0.04065781968689529 with |f| = 1.286e-4. That is above
  the hold bound, and the bound is pinned by gated cells (Excel `#NUM!` at |f| 1.144e-4, gate-a-sweep A231; Excel
  returns at 7.17e-5, probes/t3348-rate-fv-nonzero A126). Excel's value is not the model's iterate but about 2 ulps
  away, so Excel's last steps differ from the model's in the noise: at its final iterate Excel's residual passes a test
  the model's fails.
- probes-numeric/t3348-rate-gate-a-sweep Calc!A296: `=RATE(351,-1684985188.67608,607019725001.509,-15796156082.0366,0)`.
  Excel `#NUM!`, engine 1.8814890660971136e-6 (2.4e-8 relative off the math), math
  0.0000018814890215042125897557712909148668011313466560856. From step 56 the model's iterates bounce in the noise
  (|f| 1 to 40 on terms of 6e11) until a flat secant at step 119 lands on |f| = 2.1e-5 and holds. Excel's walk
  through the same noise most likely does not go flat below 1e-4 before the 200-step cap.
- Rules that fit one cell regress gated cells or are fitted to one point:
  - A hold bound below 2.1e-5 turns 5 gated cells that Excel returns into `#NUM!` (probes/t3348-rate-fv-nonzero A113,
    A117, A126, A167, A183, held |f| 3.9e-5 to 7.2e-5).
  - A bound relative to the residual's terms (|f| / max(|pv·P|, |pmt·factor|, |fv|)) does not separate the cells:
    Excel errs at 5e-20 to 1.4e-16 and returns across the same range (T-3348 trace of all 860 literal RATE oracle
    cells).
  - A cut on the step count would sit between 104 (fv-nonzero A147 returns) and 119 (A296) on one point, against
    decision 0188 R4.
- The lead is the residual's bits near the root: the same cause as T-3389's 15-digit misses (A171's second iterate
  needs Excel's f about -2.71e-7 where the model has -1.67e-7). A residual form that reproduces Excel's noise there
  likely settles both.
- Code: `compute_rate` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; fit
  lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] Calc!A272 and Calc!A296 have Excel's type (A272 within 15 significant digits of Excel), or the task records,
      with the residual forms tried, why Excel's noise cannot be reproduced, and the owner accepts the Gate A
      exception under decision 0188.
- [ ] No gated cell regresses; the 5 fv-nonzero holds above stay numbers.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 15-digit misses with the right type (T-3389).
