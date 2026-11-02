---
id: T-3801
title: "Pin RATE's 1 + r floor inside 1E-7 .. 1.0000034E-7 and give it its own constant"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3585]
---

## Goal
RATE's floor on 1 + r of a converged iterate is pinned by Excel evidence inside the bracket T-3585 measured, and it
is its own named constant, not a third use of the FITTED-tagged `TOLERANCE`.

## Context
- T-3585 (PR #2816) found that Excel returns a converged RATE root only when 1 + r is above a floor between 1E-7
  and 1.0000034E-7 (financial.md "RATE is #NUM! when 1 + r is 1e-7 or less"; probes/t3585-rate-num-near-minus-one
  A3 `RATE(1,100*1E-7,-100)` is `#NUM!`, C3 returns 1 + r = 1.0000034E-7). `compute_rate`
  (crates/rustsheet-engine/src/functions/financial/financial_util.rs) checks `1.0 + r2 > TOLERANCE`.
- Excel-fidelity review of #2816 (MINOR): the floor sits about one ulp above A3 (1 + r = 9.999999994736442e-08;
  the next double up, 1.0000000005838672e-07, would return), and `TOLERANCE` carries a FITTED tag for the
  residual stop, so an evidenced bound shares a fitted constant (T-3361's tag ratchet may miscount it).
- Probe idea: n = 1 roots `RATE(1,100*d,-100)` and `RATE(1,-50,-50,100*d,1)` for d = 1.0000001E-7, 1.000001E-7,
  1.00001E-7, 1.0001E-7 and 1.000003E-7, plus d at the doubles either side of 1E-7, to show whether the test is
  `1 + r > 1e-7`, `r > -1 + 1e-7` or another cut.

## Acceptance criteria
- [ ] A probe (probes/, tag M9, excel-graph) narrows the bracket; financial.md's entry records it.
- [ ] The floor is a named constant whose doc comment cites the probe (FITTED only if the value is a cut placed
      inside a bracket, with its range); `TOLERANCE`'s comment no longer counts it as one of its uses.
- [ ] probes/t3585-rate-num-near-minus-one A-F and the new probe pass; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The noise-floor holds (T-3424).
