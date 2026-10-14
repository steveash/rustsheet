---
id: T-1533
title: "Fit IRR's and RATE's update and stop rule, XIRR's XNPV stop test on long spans and a zero first value, and multi-period YIELD"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1525]
---

## Goal
A fit-lab model that reproduces Excel's IRR and RATE bit for bit (update rule, stop test, `#NUM!`),
XIRR on the cells T-1525's model misses, and YIELD with more than one coupon left.

## Context
Split (2026-10-14): this task fitted IRR and recorded RATE; XIRR's long-span and zero-first-value cells
and multi-period YIELD moved to T-1583. The IRR notes below are the pre-fit state.
T-1525 (attempt 1 under decision 0056) fitted XIRR on its fit set and left these open. Run the
lab fits from `tools/`: `uv run rustsheet-fitlab irr`, `rate`, `xirr-oos`, `yield`
(`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`). Findings are in
`docs/excel-semantics/financial.md`: "IRR is a secant in v = 1/(1 + r)" (was "IRR returns x - 1 ..."), "RATE iterates on the rate itself",
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
- [x] Each solver's candidates and their bit-exact and within-4-ulp counts recorded (decision 0056), as lab fits (IRR and RATE; XIRR and YIELD moved to T-1583).
- [x] A model that raises the bit-exact count on at least one solver, or the task set `blocked` with the candidates tried.
- [x] Findings in `docs/excel-semantics/financial.md`; T-0916 updated with any fitted model.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0916, T-1532).

## Outcome
IRR fitted (attempt 1): IRR is a secant on NPV(v), v = 1/(1 + r), from v0 = 1/(1 + guess) and
v0 + 0.001*sign(NPV(v0)), stopping at |dv| < 1e-7 and |NPV| < 1e-7, retried once from 0.1 (`irr_secant_v`,
`uv run rustsheet-fitlab irr`): **189/263 bit-exact** (was 74 for the best Newton candidate; the engine 1),
207 within 4 ulp, 260 within 1e-9. Rejected variants are lab candidates (bit-exact / within 4 ulp): no
residual test 181/198, residual 1e-6 188/206 and 1e-8 180/199, |dv| < 1e-6 169/183 and 1e-8 146/157, offset
1e-4 89/99, no retry 187/205, 20-step cap 181/198, NPV by Horner 189/204; Newton in v converged 74/84. The evidence (the guess sweep's error law and signs, the step-test gap
9.84e-8 / 1.18e-7, the scaled flows, the far-guess retry) is in financial.md "IRR is a secant in v". T-0916
updated with the model.
RATE: IRR's secant on f(r) (12 lab variants) scores at most 19/136 bit-exact, 40 within 4 ulp (Newton 11,
engine 12) and hits no guess-sweep iterate; recorded in financial.md as tried. XIRR's long-span and zero-first-value
cells and multi-period YIELD were not attempted; they move to T-1583 (T-1534 traces would help RATE and
YIELD). RATE stays open: this was its second attempt (T-1525 the first) without a fitted model, but the
lab's best secant variant does raise RATE's bit-exact count (19/136 against Newton's 11 and the engine's
12), so it is not time-boxed under decision 0056; the next RATE attempt needs a new hypothesis
(per-iterate traces, T-1534).
Eval delta: none (lab and docs only; the engine change is T-0916).
