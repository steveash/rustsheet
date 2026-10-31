---
id: T-3389
title: "RATE in the residual's noise floor: Excel's iterates where the model is beyond 15 digits"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
RATE returns Excel's value where the secant ends in the residual's rounding noise: the 4 cells T-3348 left inside
the Gate A envelope (within 1e-9 of the math) but beyond 15 digits of Excel. The Gate A sweep points where the noise
decides the type are T-3424 (P0), and decision 0192 records the trade.
P1 rather than P2: these 4 are visible misses, not bit residue, and visible parity is the accuracy stage's done
criterion (decisions 0083 and 0114). They rank below the Gate A failures (P0, decision 0188 R2) and above last-digit
work (R5).

## Context
- T-3348 made a flat secant hold and let iterates below -1 run on (financial.md "RATE holds on a flat secant and
  runs on below -1"). In the noise floor Excel's iterates differ from the fitted model's, though the model is
  bit-exact on 960 fit-set cells and on everything above the noise.
- Beyond 15 digits, right type: probes/t3348-rate-fv-nonzero Sheet1!A138, A171, A180 and A185 (up to 2e-11
  relative). The other 3 of T-3348's 7 known failures (A112, A153, A175) are within 15 digits, 1 to 2 ulps off; they
  are Phase 1b last-digit work (decision 0188 R5) and not in this task's criteria. A171 is `RATE(1,2560574.93,-2543613.41,-11634.76,0)`: Excel 0.002094170434484841, the
  model 0.0020941704345254896, root 0.002094170434492252. Excel's f at the model's second iterate would have to be
  about -2.71e-7 where the model has -1.67e-7 and the exact value is -2.52e-7, so the residual's rounding at n = 1
  (`(P-1)/r`, Excel's `+` snap) is the first lead.
- The type misses (16 Gate A sweep cells, decision 0188 R2) are T-3424 (P0), which has their traces. A residual
  form found here likely settles them too; it will be a structural model under reviewer check 7.
- Code: `compute_rate` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs;
  fit lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] A138, A171, A180 and A185 are within 15 significant digits of Excel, or the task records why
      Excel's noise cannot be reproduced (with the residual forms tried) and the cells stay known failures.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit parity on cells already within 15 digits (decision 0188 R5).
