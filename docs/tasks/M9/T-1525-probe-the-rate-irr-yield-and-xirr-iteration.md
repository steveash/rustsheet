---
id: T-1525
title: "Probe the RATE, IRR, YIELD and XIRR iteration traces: Excel's start, step, stop and failure rule"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1009]
---

## Goal
Know exactly how Excel's RATE, IRR, YIELD and XIRR iterate (starting point, update rule, stop test, iteration cap, and when #NUM! is returned), so the engine reproduces the same iterate and its last digits.

## Context
Owner request 2026-10-14 (decision 0056); the near-miss figures and their source are in T-1522. The `numeric_close` misses in these solvers are far beyond rounding: the engine stops at a different iterate. Documented behavior: RATE and IRR iterate up to 20 times to 0.00001%. T-1009 already landed a 691-cell IRR/RATE/XIRR sweep (probes-numeric/m9-fin-irr-rate), and financial.md "IRR, RATE and XIRR stop early" records the default guess, early stop and #NUM! findings. This task reuses those cells and only probes what they lack: which update rule, derivative and stop test produce Excel's last digits, plus YIELD, which has no sweep. T-1050: RATE's guess argument cannot be reached today. The engine fix stays in T-0916, which now depends on this task.

## Acceptance criteria
- [x] Oracle probes (excel-probe skill), on top of the T-1009 cells, that separate solver models by their last digits: Newton versus secant versus other update rules, the derivative form, the exact stop test and iteration count; YIELD across price, coupon count and basis.
- [x] The rule for each solver, or partial findings where it is unfitted, recorded in `docs/excel-semantics/financial.md` with evidence; T-0916 (and a new YIELD task if needed) gets the fitted model in its Context; no engine change here.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bond pricing outside YIELD's solver.

## Outcome
2026-10-14, attempt 1 (decision 0056). New probes (excel-graph): `probes/solver-iteration-traces` (353 cells)
- sheet X, 155 XIRR cells (two-flow closed-form roots at guesses 1, 0.1, -0.5, 0, one- and ten-year spans,
  three-flow roots);
- sheet I, 65 IRR cells (guess sweep 0.05 to 0.12 on `{-20,5,5,5,5,5}`, far guesses on `{-1,0,...,0,2}`,
  two- and three-flow roots);
- sheet R, 89 RATE cells (`RATE(1,0,-1,X)`, `RATE(n,0,-1,2)` for n = 1 to 30, a guess sweep 0 to 0.03 on
  `RATE(48,-200,8000,0,1)`, annuities with type 0/1 and fv);
- sheet Y, 44 YIELD cells (one coupon period left; ten-year bonds priced 40 to 160 at frequency 1, 2, 4);
and, from review, `probes/bond-one-period-bases` (80 cells: YIELD and PRICE with one period left on
bases 0 to 4). Reused from T-1009: all 691 cells of `probes-numeric/m9-fin-irr-rate` (its 72 XIRR, 216
IRR and literal RATE cells). YIELD also reuses probes/yield-solver-sweep (T-1493).

Scores are fit-lab fits (`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`; from `tools/`:
`uv run rustsheet-fitlab <fit>`), bit-exact counts on numeric Excel values:
- `xirr` (fit set, 208 cells): the bracket-and-bisection model 208/208 (and 19/19 `#NUM!`,
  `xirr_error_agreement`); |XNPV| < 1.5e-8 sum|value| 208; step test alone 195; m = (a+b)/2 206; `a` from
  the lower end 205; A = 4.95e-9 191, 5.05e-9 208, 5.15e-9 203; B = 9.9e-9 and 1.03e-8 206; h < 6e-9 81;
  h < 1e-8|m| 33; XNPV test alone 13; engine 0.
- `xirr-oos` (11 cells): model 8/11 (and 8/11 `#NUM!`); misses A8 (124-year span) and A10, A11 (zero first
  value) of xirr-early-dates; the sum|value| variant 7/11.
- `irr` (263 cells): not fitted. Best Newton in v = 1/(1+r) converged 74; FD Newton in v 73-74; Newton in
  x 47; Newton in r 5; union 96; engine 1. Found: results are x - 1 (263/263), and the guess-sweep error
  grows as (guess - root)^8.
- `rate` (136 literal cells): not fitted. Newton in r 11 (4-8 steps), FD Newton 7, engine 12. Results are
  rates (7/136 pass the x - 1 test).
- `yield-one-period` (60 cells, bases 0-4): closed form with the basis' DSR and the period E on bases 1-3
  60/60; E - A for DSR 30; nominal E on bases 2, 3 44; engine 0. Follow-up T-1532 (engine). PRICE's
  one-period form uses E - A (40/40 cells, engine already passes).
- `yield` (90 basis-0 multi-coupon cells): not fitted; best 2/90 (Newton from 0.1; LibreOffice's regula
  falsi), union 5/90, engine 0.
Outside the lab (T-1534 extends it): the `#NUM!` agreement (a helper plus tests), and the per-iterate
trace analysis (bisection depth per cell, which candidate iterate equals Excel, the ^8 law and the
effective constant), done in scratch scripts.
Not changed: engine code. The XIRR model is in T-0916's Context; T-0916's IRR/RATE part waits for
T-1533, the follow-up fit (IRR's and RATE's update and stop rule, XIRR's XNPV test on long spans and a
zero first value, multi-period YIELD).
