---
id: T-2527
title: "RATE: the last-ulp misses of the secant model (f's evaluation), the root-0 #NUM! cell and a fitlab port of the scratch harness"
milestone: M9
lane: functions-financial
status: done
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
- [x] The fit lab parses the nested `RATE(n,PMT(..),..)` cells and scores the T-2005 model and candidates on every
      literal RATE cell, with iterate hits.
- [x] Visible parity: fewer visibly-off RATE cells than the 117 above, with no new known-failure lines in any
      baseline, or the unfitted ones documented in financial.md with the forms tried.
- [x] A583's rule probed and implemented or documented (T-2689).
- [x] A278, A245 and A367 (regressed by T-2005) pass again or are documented with the forms tried.
- [x] Probe a guess whose residual is exactly 0 (ours runs the secant, the sign of the zero picking r1).
- [x] `make check` green; reviewers approve.

## Out of scope
- PMT's own bits (T-1528 and the PMT tasks), IRR, YIELD.

## Outcome
- Fit lab `rate` (`uv run rustsheet-fitlab rate`): the sweep reads a nested call whose text is the whole formula
  of a cell of the fit's new `calls` cases as Excel's value of that cell (`Fit.calls`, `sweep.call_values`), so
  `RATE(n,PMT(..),..)` takes Excel's own PMT bits from probes-numeric/m9-fin-pmt (all 308 inner calls are
  there). `rate_secant_candidates` ports the T-2005 model with traces; the fit scores 474 cells (errors included).
- Finding: with Excel's PMT bits the secant is bit-exact on 474/474 when the residual's `+`, `*` and `/` are
  Excel's twice-rounded operators, 472/474 with FV's once-rounded `annuity_factor` (A487 2,183 ulp, A218 157
  ulp), 457-468 with the sum in another order; Excel's value is an iterate of the trace on 428/428 numeric
  cells. `rate_residual` now uses the twice-rounded operators (financial.md "RATE is a secant on the FV residual").
- The RATE cells still off are off by the engine's PMT, which differs from Excel on 176 of the 308 inner calls
  (PMT's tasks, T-1528): against the engine's own PMT bits both residuals give 320/474 exact and 116 visibly off
  (117 before T-2689's A583). A218, A278, A245 and A367 are among them, documented in financial.md.
- Probe probes/rate-zero-residual-guess (12 cells, excel-graph): a guess with residual exactly 0 runs the secant
  (`RATE(1,0,0,0)` is `#NUM!`, the flat secant, not 0.1); the model passes all 12.
- Eval delta: gated baselines unchanged (0 fixed, 0 regressions; probes +1 case, 12 cells, all pass); visible
  parity 3225 in-scope cells visibly off (unchanged). Follow-up: T-2922 (norm-s-inv onto `calls`).
