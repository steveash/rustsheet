---
id: T-3547
title: "RATE is #NUM! where its secant converges to the spurious root r = -1 of a type-1 annuity (13 T-3478 sweep cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
RATE returns Excel's `#NUM!` where its secant converges to the spurious root r = -1 of the FV residual, as on the 13
cells of probes-numeric/t3478-rate-r4-wide below, without losing a returning cell near -1.

## Context
- Found by T-3478's fresh sweep (financial.md "RATE is #NUM! at the spurious root r = -1"). With type 1 and fv = 0
  the FV residual pv(1+r)^n + pmt(1+r)((1+r)^n - 1)/r + fv is 0 at r = -1, whatever the real root. On these cells the
  secant (`compute_rate`, crates/rustsheet-engine/src/functions/financial/financial_util.rs) converges normally
  (step and |f| under 1e-7, 5 to 100 steps) to an iterate with 1 + r from 2.2E-16 to 3.7E-12; the engine returns it
  (e.g. -0.9999999999999378) and Excel is `#NUM!`.
- Cells (all type 1, fv 0): Calc!A60, A76, A85, A88, A89, A131, A146, A171, A213, A237, A243, A256, A265. The
  fit lab's T-2005 model (`fv_residual(TWICE, "pv,pmt,fv")` with T-3348's hold) reproduces the engine on all 300
  sweep cells, so the stop is the model's, not a different trajectory.
- What bounds the rule: Excel returns 1 + r = 1.19E-7 at `RATE(2,0,-100)` (probes/rate-pmt-zero-cap A2), and the
  cap's test is 1 + r > 1E-4 (T-2689). So Excel's threshold on a converged iterate lies between 3.7E-12 and 1.19E-7,
  or the rule is about r = -1 being a root of the residual (type 1, fv 0) and not about a threshold. A probe that
  separates the two (type 0 roots near -1; type 1, fv = 0 roots with 1 + r from 1E-11 to 1E-7) is the first step.
- Also from the sweep, not this rule: Calc!A285 `RATE(3,-27701521124.6443,83584699115.003,-1296.87658562967,0,
  228.952995156186)` is `#NUM!` in Excel; the engine returns -0.0028749021254309863 (math -0.00287490212542650...).
  It is a noise-floor cell of T-3424's kind (|pv| 8E+10); list it there if this task does not settle it.

## Acceptance criteria
- [ ] A probe case (probes/, tag M9, excel-graph) bracketing the rule, listed here before the fix.
- [ ] The 13 cells above are `#NUM!`; no gated cell regresses; the RATE fit set (960 cells) stays bit-exact.
- [ ] financial.md's entry states the evidenced rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3424's noise-floor cells (except A285 if the rule covers it).
