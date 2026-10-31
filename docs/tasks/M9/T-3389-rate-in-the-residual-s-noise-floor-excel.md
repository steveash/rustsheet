---
id: T-3389
title: "RATE in the residual's noise floor: Excel's iterates and type at a huge balance"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
RATE returns Excel's value and type where the secant ends in the residual's rounding noise: the cells T-3348 left
within the Gate A envelope but not within 15 digits of Excel, and the two Gate A sweep points where the noise decides
the type.

## Context
- T-3348 made a flat secant hold and let iterates below -1 run on (financial.md "RATE holds on a flat secant and
  runs on below -1"). In the noise floor Excel's iterates differ from the fitted model's, though the model is
  bit-exact on 960 fit-set cells and on everything above the noise.
- 15-digit misses, right type: probes/t3348-rate-fv-nonzero Sheet1!A112, A138, A153, A171, A175, A180, A185
  (1 ulp to 2e-11 relative). A171 is `RATE(1,2560574.93,-2543613.41,-11634.76,0)`: Excel 0.002094170434484841, the
  model 0.0020941704345254896, root 0.002094170434492252. Excel's f at the model's second iterate would have to be
  about -2.71e-7 where the model has -1.67e-7 and the exact value is -2.52e-7, so the residual's rounding at n = 1
  (`(P-1)/r`, Excel's `+` snap) is the first lead.
- Type misses (Gate A sweep, decision 0188 R2): probes-numeric/t3348-rate-gate-a-sweep Calc!A272
  (`RATE(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)`, Excel 0.040657819686895275, the model holds at
  |f| 1.29e-4 and is `#NUM!`) and A296 (Excel `#NUM!`, the model holds at |f| 2.1e-5).
- Code: `compute_rate` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs;
  fit lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] The listed cells are within 15 significant digits of Excel with the right type, or the task records why
      Excel's noise cannot be reproduced (with the residual forms tried) and the cells stay known failures.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit parity on cells already within 15 digits (decision 0188 R5).
