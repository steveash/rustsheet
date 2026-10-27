---
id: T-2915
title: "IRR's fallback roots to Excel's digits: fit the Newton-from-v=0 run's steps and stop (probes/irr-fallback-roots, 7 visibly-off cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Where both of IRR's secant runs fail and the engine takes the fitted fallback (financial.md "IRR's fallback when the
secant fails"), IRR returns Excel's 15 significant digits, by reproducing how Excel's Newton-from-v = 0 run steps
and where it stops. Time-boxed under decision 0056: if two attempts do not bring the owned cells within 15 digits,
record the candidates tried and their scores in financial.md and here.

## Context
Gap triage 2026-10-27 (decision 0083 visibly-off cells with no open owner). T-2727 fitted which flows return through
the fallback (VERIFIED) but not its digits; its Outcome points at T-2131 and T-2565, which own the secant path
(T-2565 explicitly leaves the fallback out), and T-1686 is a probe task. No open task names these cells.

Owned cells (probes/irr-fallback-roots, excel-graph 2026-10-26; `make eval-show CASE=probes/irr-fallback-roots.case.json`),
all `NumericClose`, all visibly off:
- P!A12, P!A13, P!A14, P!A15: `IRR({42,1,1,-1},G)`, G = 0.5, 1, 0.01, 0: Excel -0.7467044766537371, ours
  -0.7467044766537408.
- P!A22, P!A23, P!A24: `IRR({-400,1,1,1,1,1},G)`, G = 0.5, 1, 0.01: Excel -0.6737366751808405, ours
  -0.6737366751808422.

Ours are the correctly rounded roots (exact rational bisection on the NPV polynomial: -0.74670447665374079...,
-0.67373667518084224...), so Excel is the inaccurate one (about 37 and 17 ulp off) and the stage is `visible`, not
`accuracy`: the fix reproduces Excel's run, it does not improve a kernel. P!A21 is in the same case but already within
15 digits (`NumericUlp`) and is not owned here. Further fallback evidence with digits: `IRR({0.3,-2.5,1.005},0.1)` is
6.909808543634389 in Excel and 6.909808543634611 from guess 0.01 (financial.md), and ironcalc-calc IRR.xlsx General!L17
(`IRR({-400,1,1,1,1,1})`); the gen-m9 cells listed in that financial.md section that return a number.

Engine: `irr_fallback`, `IRR_FALLBACK_STEPS` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).
Candidates to try in the fit lab (tools/src/rustsheet_tools/fitlab/kernels/solvers.py): the stop test (|dv| vs
relative, residual), returning v_k vs v_{k+1}, Horner vs power-sum evaluation of P and P', the r = 1/v - 1 conversion
order, and a finite-difference derivative as in `solve_yield`. The guess does not move Excel's result, so the guess
plays no part in the fallback.

Not owned here: the two-flow and zero-gap far roots in the same case (P!A25, A26, A28, A36-A39, e.g. `IRR({-1,1E+4})`
Excel 9999.0000000011, ours 9999.000000012204). They come from the secant path's early steps (financial.md "IRR's
far-root bound": "Excel's steps at these scales are not the fitted secant's"), which T-2565 probes and T-2131 fits.

## Visible-parity stage
Target: P!A12-A15 and P!A22-A24 of probes/irr-fallback-roots within half a unit of the 15th significant digit of
Excel's value, with no other cell of IRR newly visibly off.

## Acceptance criteria
- [ ] The 7 known-failure lines `probes/irr-fallback-roots.case.json` P!A12, A13, A14, A15, A22, A23, A24 disappear
      from eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure line in any suite.
- [ ] financial.md "IRR's fallback when the secant fails" states the fitted step and stop rule with these cells (or,
      if time-boxed, the candidates tried and their scores, and this task's bit-level remainder is filed as a
      follow-up with `defer: phase-1b`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which flows return or are `#NUM!` through the fallback (already fitted by T-2727).
- The far-root cells P!A25-A39 (T-2131, T-2565) and the `#DIV/0!` cells (T-2802).
- IRR's secant-path digits (T-2131).
