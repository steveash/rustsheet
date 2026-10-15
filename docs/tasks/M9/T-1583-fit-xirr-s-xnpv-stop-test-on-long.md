---
id: T-1583
title: "Fit XIRR's XNPV stop test on long spans and a zero first value, and multi-period YIELD"
milestone: M9
lane: research
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1533]
---

## Goal
A fit-lab model that reproduces XIRR on the cells T-1525's model misses and YIELD with more than one
coupon left, bit for bit (split from T-1533, which fitted IRR and recorded RATE).

## Context
`uv run rustsheet-fitlab xirr-oos` and `yield` (`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`);
docs/excel-semantics/financial.md "XIRR is a bracketing bisection" and "YIELD stops short of PRICE's root".
- XIRR: the XNPV part of the stop test fails xirr-early-dates A8 (124-year span; Excel returns bisection
  step 25, the model step 28), and a zero first value (A10, A11 give 0.1 * 2^-25). Hypotheses: an XNPV
  test scaled by the span or the derivative.
- YIELD, basis 0, more than one coupon (90 cells): best candidate 2/90 (Newton from 0.1, LibreOffice's
  regula falsi). Hypotheses: a secant like IRR's (T-1533: secant in v = 1/(1 + r) from v0 and v0 +- 0.001,
  stop |dv| < 1e-7 and a residual test), secant on PRICE from the coupon rate, the yield-solver-sweep guess
  structure. Per-iterate traces need T-1534.

## Acceptance criteria
- [ ] Each candidate's bit-exact and within-4-ulp counts recorded as lab fits (decision 0056).
- [ ] A model that raises the bit-exact count on XIRR-oos or YIELD, or the task set `blocked` with the candidates tried.
- [ ] Findings in `docs/excel-semantics/financial.md`; T-0916 updated with any fitted model.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0916, T-1532).
