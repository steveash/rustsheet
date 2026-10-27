---
id: T-2527
title: "RATE: the last-ulp misses of the secant model (f's evaluation), the root-0 #NUM! cell and a fitlab port of the scratch harness"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2005]
campaign: visible
---

## Goal
RATE's remaining visibly-off cells are within half a unit of the 15th significant digit of Excel, with the secant
model of T-2005 kept, or the cells that cannot be fitted are documented with the forms tried.

## Context
T-2005 replaced RATE's Newton with Excel's secant on the FV residual (financial.md "RATE is a secant on the FV
residual"): 240 of 421 numeric literal RATE cells bit-exact (Newton 16), cap 200 probed (probes/rate-step-cap). Left
after it (eval show on m9-fin-irr-rate, solver-iteration-traces, ironcalc-calc FINANCIAL/RATE.xlsx): 155 failing RATE
cells, 117 visibly off. Most are nested `RATE(n,PMT(r,n,10000[,-500,1]),...)` cells of probes-numeric/m9-fin-irr-rate,
e.g. A218 `RATE(1,PMT(0.2,1,10000,-500,1),10000,-500,1)` Excel 0.19999999999999138, ours 0.20000000000000448; the root
there depends on the inner PMT's bits and on f's rounding at the last secant step, so check first whether the inner
PMT matches Excel (`PMT` cells of the same rows) and then which operation of f differs (the scratch harness scored
the documented order 69/375 and FV's order 210/375; candidates in financial.md).
- Regressed by T-2005 (passed under Newton): m9-fin-irr-rate A278 `RATE(3,PMT(0.05,3,10000,-500,1),10000,-500,1)`
  68 ulp and visibly off (Excel 0.04999999999999943, ours 0.0499999999999999), A245 `RATE(1,PMT(0.1,1,10000),10000)`
  26 ulp, A367 `RATE(6,PMT(0.3,6,10000),10000)` 1 ulp.
- m9-fin-irr-rate A583 `RATE(10,-1E6,1E7)` (root exactly 0) is `#NUM!` in Excel; ours returns the 200th iterate
  (-2.6E-9): near 0 the residual is noise. Probe roots at 0 and cap-edge runs whose 200th residual is large to find
  Excel's rule (does the cap's return also test |f|?). Done by T-2689: the cap returns only when the step, |f| and
  1 + r pass 1e-4 (financial.md), so A583 is now `#NUM!` and passes.
- The T-2005 scoring ran as a scratch Rust test that evaluated each RATE argument with the engine (nested PMT
  included); port it to the fit lab (`uv run rustsheet-fitlab rate`, tools/src/rustsheet_tools/fitlab/kernels/solvers.py)
  so candidates are scored on all RATE cells with iterate hits, as T-2005's lab criterion asked.
Code: `compute_rate`, `rate_residual` (functions/financial/financial_util.rs).

## Acceptance criteria
- [ ] The fit lab parses the nested `RATE(n,PMT(..),..)` cells and scores the T-2005 model and candidates on every
      literal RATE cell, with iterate hits.
- [ ] Visible parity: fewer visibly-off RATE cells than the 117 above, with no new known-failure lines in any
      baseline, or the unfitted ones documented in financial.md with the forms tried.
- [x] A583's rule probed and implemented or documented (T-2689).
- [ ] A278, A245 and A367 (regressed by T-2005) pass again or are documented with the forms tried.
- [ ] Probe a guess whose residual is exactly 0 (ours runs the secant, the sign of the zero picking r1).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT's own bits (T-1528 and the PMT tasks), IRR, YIELD.
