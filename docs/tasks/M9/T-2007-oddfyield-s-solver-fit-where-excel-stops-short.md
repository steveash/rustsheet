---
id: T-2007
title: "ODDFYIELD's solver: fit where Excel stops short of ODDFPRICE's root (31 desktop cells with a known root, about 70 web cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
ODDFYIELD returns Excel's bits by reproducing where Excel's iteration stops: its start, step and stop rule. Excel's
results sit 1e-12 to 7e-11 away from ODDFPRICE's root, on both sides, while ours land on the root. If two attempts
do not raise the bit-exact count, the task records the candidates and is time-boxed under decision 0056.

## Context
Gap triage 2026-10-19. ODDFYIELD has 106 failing cells of 166 checked, and no task owns its solver: T-1374 lists only
its fin-argument-modes base row next to unrelated last-digit work.
- `ironcalc-calc/FINANCIAL/PRICE_YIELD.xlsx` sheet ODDFPRICE_ODDFYIELD, K2:K52: all 31 ODDFYIELD cells fail (28 close,
  3 mismatch). Each row prices a bond at a known yield F (0.05, 0.067, 0.06, 0.03) with `J = ODDFPRICE(..,F,..)`, then
  inverts it with `K = ODDFYIELD(..,J,..)`. So the true root is F to within the rounding of J, and Excel's stopping
  error can be read off directly. K2 is 0.04999999999851281 (ours 0.05), K8 is 0.0499999999321231 (ours
  0.049999999999999635), and K23 is 0.05000000000085884, above the root. Desktop cache (Mac 16.x). In 14 rows the
  engine's J is itself an ulp off (J11, J23, ... in eval/baseline/ironcalc-calc.txt, T-1374), so the lab should read
  Excel's cached J (`rustsheet-eval snapshot`), not ours.
- Web (excel-graph): probes/date-system-1900-odd-years (19), -1900-odd-threshold (13), -1900-odd-accrint-bound (11),
  -1904-odd-threshold (11), -1904-odd-years (7), -1900-oddf-issue-after-quasi (2), date-system-1904-financial S!A32,
  fin-remaining-later-argument-ranges I121 and its neighbours (8), fin-yield-odd-lift A5/I9, fin-argument-modes H200.
  Many repeat one bond, e.g. 0.06499377053016654 against our 0.06499377053015054.
Engine: `fn_oddfyield` (functions/financial/financial_bonds.rs) calls `solve_yield`, a central-difference Newton
(h = 1e-5) from the coupon rate that stops at |dy| < 1e-8, with a bisection fallback. YIELD uses the same solver, and its
fit is blocked (T-1592: Excel stops below PRICE's root on a residual test; no start formula found). ODDFYIELD's
errors fall on both sides of the root, so its rule may not be YIELD's. The known-root desktop rows give evidence YIELD's
fit never had. If a rule found here also explains YIELD, record it in T-1592's "Blocked because" section as a new
hypothesis, which reopens that task.
Lab: the solver kernels are in tools/src/rustsheet_tools/fitlab/kernels/solvers.py (`newton_back_chain`, YIELD's
candidates, T-1534's iterate traces). Add an `oddfyield` fit there.
This needs numeric fitting against a solver trace, hence opus.

## Acceptance criteria
- [ ] An `oddfyield` lab fit over the cells above (Excel's cached J as the price input) scores named candidates:
      Newton with an analytic or finite-difference derivative, secant, starts at the coupon rate / 0 / 0.1 / the
      one-period closed form, stop on |dy| or on |price residual|. Each candidate's bit-exact and within-4-ulp counts,
      rejected ones included, go into financial.md under a new "ODDFYIELD stops short of ODDFPRICE's root" entry that
      cites PRICE_YIELD K2, K8, K23 and the probe cells.
- [ ] Either: a model that raises the bit-exact count is implemented for ODDFYIELD alone, leaving YIELD on its current
      path unless the model also raises YIELD's count. ODDFYIELD F lines in probes.txt and ironcalc-calc.txt only
      shrink, and no baseline gains a line. Or: the task is set `blocked`, "Blocked because: time-boxed (decision
      0056)", with the candidates listed.
- [ ] The solver stays bounded within the lift weight `YIELD_SOLVER_WEIGHT` (financial.md "Yield and odd-coupon work cap").
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFPRICE's own last digits (the J column, T-1374) and YIELD (T-1592).
- ODDLYIELD, which has no failing cells.
