---
id: T-3579
title: "Probe RATE's residual bits at the guess and near the root (n = 1)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Oracle evidence that pins down which of RATE's residual values Excel computes differently from the model, at the
guess points or near the root, so that T-3389's 4 noise-floor cells (and T-3424's type misses) can be fitted rather
than guessed at.

## Context
- T-3389 (financial.md "RATE holds on a flat secant and runs on below -1", "Not reproduced") found that
  Excel's value is not an iterate of the model's trace on any of probes/t3348-rate-fv-nonzero A112, A138, A153,
  A171, A175, A180 or A185, and no secant step between any two of the trace's iterates gives it either. So
  no stop or hold rule can fix them: Excel's path leaves the model's earlier, and some residual or step bits differ.
  110 residual and step variants did not reproduce any of the 4 cells beyond 15 digits (they are listed in
  financial.md).
- A171 `RATE(1,2560574.93,-2543613.41,-11634.76,0)` is the simplest case. At nper 1 the exact residual is linear,
  so the secant lands next to the root in one step and Excel's value depends only on f at the guess points (0.1,
  0.101) and on the residual's rounding near the root. With the model's residual near the root, Excel's value
  (0.002094170434484841) is reachable from first iterates 5.7e-14 or more from the model's
  (0.0020941704345913). Moving f(0.1) and f(0.101) by up to 600 ulps each gives it in 2 of 1.44 million pairs,
  both 400 to 600 ulps in opposite directions, so a small error in one guess residual does not explain it. The model's f(0.1) is -249034.58099999768, 2.3e-9 from
  the exact value, because of fl(1.1).
- Probe design ideas (excel-probe skill): A171's arguments with guesses set to chosen points, so that Excel's value
  after two steps is a known function of its f at those points; the same at nper 1 with pv, pmt and fv scaled by
  powers of 2, which leaves the rounding of 1+r unchanged and scales the rest; and guesses at the model's iterates
  of A185 (nper 5) and A180 (type 1).
- Code: `compute_rate`, `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs);
  fit lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values (excel-graph) whose cells determine, or bound,
      Excel's RATE residual at the guess points and near the root for nper 1.
- [ ] financial.md records which residual values differ from the model's and by how much, or that the probe rules
      out a difference in the residual (so the step differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `compute_rate`; that is T-3389's follow-up once the evidence exists.
