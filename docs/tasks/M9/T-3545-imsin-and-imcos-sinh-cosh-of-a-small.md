---
id: T-3545
title: "IMSIN and IMCOS sinh/cosh of a small imaginary part (|y| < 0.01): fit Excel's kernel"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IMSIN and IMCOS give Excel's digits for an imaginary part with |y| below the `1/u` band (|y| <= 0.01), where the
engine's `(EXP(t) ∓ EXP(-t))/2` pair (complex/trig.rs `exp_sinh_cosh` via `trig_sinh_cosh`) cancels.

## Context
- Found by T-3525's fresh sweep (probes-numeric/t3525-im-hyperbolic-fresh-sweep, excel-graph): 36 of the 39 misses
  are IMSIN (C) and IMCOS (D) cells with 1e-4 < |y| < 0.0085, e.g. `IMSIN("0+0.0080407127044978i")` is
  0.00804079934755902i in Excel, ...908i in the engine; `IMCOS("-0.437695624558513-0.000131316980692306i")` has
  imaginary part -0.0000556591552727565 (engine ...7094). The correctly rounded value misses most of them too (Excel's
  kernel is neither the double EXP pair nor exact), so Excel has its own small-t form there.
- Constraint: `IMSIN("1E-200+1E-200i")` is "1E-200" (sinh 0 from equal EXPs) and `IMCOS("150-0.002i")` matches the EXP
  pair (docs/excel-semantics/engineering.md, T-2535). An extended-precision EXP pair is one candidate to try first.
- Also C117/D117 (|y| = 183, above the band): one 15th-digit miss of the EXP pair.
- T-3525's Outcome and engineering.md "IMSIN and IMCOS of a small imaginary part are neither the EXP pair nor exact
  (T-3525)" have the counts.

## Acceptance criteria
- [ ] A candidate kernel for |y| <= 0.01 is fitted through the fit lab (`rustsheet-fitlab im-hyperbolic-fresh` scores
      IM* text; decision 0188 R4: hold-out split, a fresh sweep after the freeze, `FITTED` tag if a constant is tuned).
- [ ] The t3525 sweep's small-|y| IMSIN/IMCOS cells pass, `IMSIN("1E-200+1E-200i")` and the 1,023 older IMSIN/IMCOS
      cells keep passing (or a loss is listed within decision 0083's allowance).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSINH/IMCOSH below |x| = 0.01 (3 fresh-sweep misses; T-3525 keeps the band as is).
