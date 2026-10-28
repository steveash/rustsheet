---
id: T-3031
title: "ODDFYIELD's slope for a long odd first period at frequency 2 and 4 (T-2007 follow-up)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
ODDFYIELD's visibly-off cells left after T-2007 come within 15 digits of Excel: the long odd first periods at
frequency 2 (and 4), and the sweep cells near a zero yield.

## Context
T-2007 (2026-10-28) fitted ODDFYIELD as YIELD's Newton (`excel_yield_newton`, from `yield_start` over
`yield_start_years`, a step under 1e-10 not taken) with a slope that weighs the odd first coupon as a regular one
(`OddFirstSchedule::dprice`; financial.md "ODDFYIELD stops short of ODDFPRICE's root"). That fits the short first
periods and the annual long ones, but not:
- ironcalc-calc `FINANCIAL/PRICE_YIELD.xlsx` ODDFPRICE_ODDFYIELD!K8-K11, K29-K33, K49-K52: frequency 2, issue
  2020-01-15, first coupon 2021-03-01 (a long first period over three quasi-coupon periods), settlement on a
  quasi-coupon date. Ours is off by 1e-11 to 1e-10 on both sides. Tried and rejected in T-2007: YIELD's own slope over
  the regular `CouponSchedule::new(settlement, maturity)` (worse: 321 cells pass in that file against 325), and the
  odd schedule's slope with a regular coupon at every quasi-coupon date from settlement (325, no better). The committed
  slope (the lead weighed as a regular coupon at its own exponent `nq + y`) has no evidence for nq > 0.
- ODDFYIELD's #NUM! now comes from YIELD's Newton leaving ODDFPRICE's domain (no bisection fallback since T-2007);
  no ODDFYIELD oracle cell exercises it. Probe extreme prices (1, 5, 500, 5000, a zero coupon at a tiny price) on
  excel-graph and record the rule in financial.md.
- T-2007's lab was a temporary dump, not a committed `oddfyield` kernel in fitlab/kernels/solvers.py; commit one
  here so the scores can be reproduced.
- probes/oddfyield-solver-sweep S!A75-A83 and A9 (the 5% semiannual bond at prices near 124-132, its yield near 0),
  B1, C21, C35, C50, C52, C85, E1 and F1 (the inverted ODDFPRICE at a yield of 0), E32, E33, E40, F24, F32, F33, F46,
  F50 (NumericClose or NumericMismatch on the 2026-10-28 run).
- The rest of the sweep is within a few ulps; YIELD's slope operation order (`t * 100 * rate / m`) is worse (319
  sweep cells pass against 320).
Lab: the T-2007 method (dump `OddFirstSchedule`'s fields from `fn_oddfyield`, re-run Newton in Python with
`caret_pow`) scores a candidate in seconds; the long-period quasi-coupon count is `offset` (nq).

## Acceptance criteria
- [ ] The listed cells are within half a unit of the 15th significant digit of Excel, or the task is blocked
      time-boxed (decision 0056) with the candidates recorded in financial.md.
- [ ] ODDFYIELD lines in eval/baseline only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD (T-1592) and ODDFPRICE's own digits (T-1374).
