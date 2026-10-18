---
id: T-1583
title: "Fit XIRR's XNPV stop test on long spans and a zero first value, and multi-period YIELD"
milestone: M9
lane: research
status: done
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
- [x] Each candidate's bit-exact and within-4-ulp counts recorded as lab fits (decision 0056).
- [x] A model that raises the bit-exact count on XIRR-oos or YIELD, or the task set `blocked` with the candidates tried.
- [x] Findings in `docs/excel-semantics/financial.md`; T-0916 updated with any fitted model.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-0916, T-1532).

## Outcome
XIRR (lab only): two refinements of the T-1525 model, candidate "bisect, first-value sign, span-scaled
|f| bound (T-1583)" (`span_stop`, `xirr_model(first_value_sign=True)`): the end replaced is chosen by
XNPV(m) * values[0] >= 0, and the XNPV stop bound is 2e-8 * max|value| * max(1, span in years), absent
for a zero first value. `xirr-oos` 16/22 -> 21/22 (all 11 numbers; the zero-first-value cells A10, A11 and
the 124-year span A8 now match), fit set unchanged at 227/227. The span factor rests on A8 alone (follow-up
T-1591 probes it). YIELD: no fit. Excel's values are either at PRICE's root (45/90) or below it with a
residual up to 5e-8 (45/90), consistent with Newton stopping on a residual test; stepping Newton backwards
gives a frequency- and redemption-independent earlier iterate (0.0552344627 for S!A1 and S!F1). Newton with
a residual stop from 0, the coupon rate or the current yield reaches 3/90 bit-exact (recorded as lab
family "Newton, residual"); follow-up T-1592 fits the start. Findings in financial.md; T-0916 updated.
No engine change, so no eval delta.
