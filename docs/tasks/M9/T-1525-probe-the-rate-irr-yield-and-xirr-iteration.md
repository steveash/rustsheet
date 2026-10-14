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
- [x] The rule for each solver recorded in `docs/excel-semantics/financial.md` with evidence; T-0916 (and a new YIELD task if needed) gets the fitted model in its Context; no engine change here.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bond pricing outside YIELD's solver.

## Outcome
2026-10-14, attempt 1 (decision 0056). New probe `probes/solver-iteration-traces` (353 cells,
excel-graph): sheet X 155 XIRR cells (two-flow closed-form roots at guesses 1, 0.1, -0.5, 0, one- and
ten-year spans, three-flow roots), sheet I 65 IRR cells (guess sweep 0.05 to 0.12 on `{-20,5,5,5,5,5}`,
far guesses on `{-1,0,...,0,2}`, two- and three-flow roots), sheet R 89 RATE cells (`RATE(1,0,-1,X)`,
`RATE(n,0,-1,2)` for n = 1 to 30, a guess sweep 0 to 0.03 on `RATE(48,-200,8000,0,1)`, annuities with
type 0/1 and fv), sheet Y 44 YIELD cells (one coupon period left; ten-year bonds priced 40 to 160 at
frequency 1, 2 and 4). Reused from T-1009: all 691 cells of `probes-numeric/m9-fin-irr-rate` (its 72
XIRR, 216 IRR and the literal RATE rows); YIELD also reused probes/yield-solver-sweep (T-1493). Scores
come from `docs/research/t1525-solver-fit.py`; findings are in `docs/excel-semantics/financial.md`.

- XIRR: fitted. A sign-change search on a chain from 0 (s, 2s, 4s, ... upward for a guess >= 0; -s then
  p -> (p - 1)/2 toward -1; s = guess, or 1e-5 for guess 0), then bisection with m = a + (b - a)/2 from
  the bracket's upper end, stopping when (b - a)/2 < 5e-9 + 1e-8|m| and |XNPV(m)| < 2e-8 max|value|.
  Bit-exact 227/227 on the fit set (155 new + 72 T-1009 cells, all 19 `#NUM!` included). Rejected stop
  rules: half-width test alone 214/227, h < 6e-9 100/227, h < 1e-8|m| 52/227, XNPV test alone 32/227;
  midpoint (a + b)/2 or starting from the lower end: 224/227. Out of sample 16/22 literal XIRR cells of
  older probes; the misses are recorded input checks (3), a zero first value (2) and a 124-year span
  (xirr-early-dates A8) that shows the XNPV part of the stop test is not normalised by max|value|.
- IRR: not fitted. Found: the result is x - 1 (263/263 cells), and the guess sweep's error grows as
  (guess - root)^8 (a third iterate of a quadratic method, constant between -1.19 and -1.25; Newton in
  v = 1/x has -1.17). Candidates (any iterate equals Excel): Newton in r 15/263, in x 57/263, in v 91/263,
  finite-difference Newton in v 78 to 82/263; the matches are the converged cells only.
- RATE: not fitted. Results are rates, not x - 1. `RATE(1,0,-1,X)` is two Newton steps from 0.1 in 9/10.
  Newton on the documented f contains Excel's value for 23/134 literal cells, FD Newton 18/134.
- YIELD: one coupon period left is the documented closed form, 20/20 bit-exact (follow-up T-1528, engine).
  More periods: Newton from the coupon rate, 0 or 0.1, secant (0, 1) and LibreOffice's regula falsi reach
  at most 6/24 (Y!C:E); unfitted.
Not changed: engine code (T-0916 owns the XIRR change; the model is in its Context). Remaining for a
second attempt (decision 0056): IRR's and RATE's update rule and stop test (hypotheses to try: a
safeguarded Newton in v with a residual test; Excel's PMT-style f for RATE), XIRR's XNPV test on long
spans and a zero first value, and the multi-period YIELD solver.
