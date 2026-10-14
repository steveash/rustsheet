---
id: T-1533
title: "Fit IRR's and RATE's update and stop rule, XIRR's XNPV stop test on long spans and a zero first value, and multi-period YIELD"
milestone: M9
lane: research
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1525]
---

## Goal
A fit-lab model that reproduces Excel's IRR and RATE bit for bit (update rule, stop test, `#NUM!`),
XIRR on the cells T-1525's model misses, and YIELD with more than one coupon left.

## Context
T-1525 (attempt 1 under decision 0056) fitted XIRR on its fit set and left these open. Run the
lab fits from `tools/`: `uv run rustsheet-fitlab irr`, `rate`, `xirr-oos`, `yield`
(`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`). Findings are in
`docs/excel-semantics/financial.md`: "IRR returns x - 1 ...", "RATE iterates on the rate itself",
"XIRR is a bracketing bisection" (out-of-sample misses), "YIELD stops short of PRICE's root".
- IRR (263 cells): the result is x - 1 with x = 1 + r a double; the guess sweep (solver-iteration-traces
  I!A) has error ~ (guess - root)^8, a third iterate of a quadratic method with f''/(2f') about -1.2.
  Best value-returning candidate: Newton in v = 1/(1+r) to convergence, 74/263 exact.
- RATE (136 literal cells): iterates on r; best candidate Newton in r, 8 steps, 11/136 (engine 12).
  Plain Newton crawls on RATE(360,-599.55,100000), which Excel solves.
- XIRR: the XNPV part of the stop test fails xirr-early-dates A8 (124-year span; Excel returns
  bisection step 25, the model step 28), and a zero first value (xirr-early-dates A10, A11 give
  0.1 * 2^-25).
- YIELD, basis 0, more than one coupon (90 cells): best candidate 2/90 (Newton from 0.1, LibreOffice's
  regula falsi).
Hypotheses not tried: a safeguarded Newton in v with a residual stop; Newton on NPV/x^k or on the
FV form; Excel's PMT-style f for RATE; secant on PRICE from the coupon rate; an XNPV test scaled by
the span or the derivative. Per-iterate traces need T-1534.

## Acceptance criteria
- [ ] Each solver's candidates and their bit-exact and within-4-ulp counts recorded (decision 0056), as lab fits.
- [ ] A model that raises the bit-exact count on at least one solver, or the task set `blocked` with the candidates tried.
- [ ] Findings in `docs/excel-semantics/financial.md`; T-0916 updated with any fitted model.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0916, T-1532).
