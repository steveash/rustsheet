---
id: T-0916
title: "IRR/RATE/XIRR/MIRR: match Excel's iteration and termination behavior"
milestone: M9
lane: functions
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The iterative financial solvers return results that differ from Excel at 1e-12 to 1e-8 relative because Excel stops its iteration at a looser tolerance than the engine, so the last digits are Excel's un-converged values. Emulate Excel's iteration (initial guess, update rule, tolerance, iteration cap) using the Excel-cached values as the spec. MIRR is a ulp-level formulation difference.

## Context
Evidence is Excel-cached values in `ironcalc-calc/FINANCIAL/*.xlsx`. Failing cells (report.md): IRR 74 (IRR.xlsx, mostly NumericClose), RATE 26 (RATE.xlsx), XIRR 17 (XIRR.xlsx 13, XNPV.xlsx 4; NumericMismatch up to 1e-8), MIRR 51 (MIRR.xlsx, NumericUlp). About 170 cells, the largest single group in the report.
Examples: IRR.xlsx General!L2 `IRR(A2:F2)` expected 0.07930826116039724 got 0.0793082611605286; General!N2 `IRR(A2:F2,I2)` expected 0.07930826116414913 got 0.0793082610836084 (the guess changes Excel's result, so the exact Newton path matters). RATE.xlsx Sheet1!F2 expected 0.008052981923906556 got 0.008052981923938503. XIRR.xlsx Sheet1!R2 expected 1.1859161972999572 got 1.1859162079821048. MIRR.xlsx `Two Parameters!E2` `-MIRR(A2:B2,C2,D2)` expected 0.19999999999999996 got 0.20000000000000007.
Approach: for IRR/RATE/XIRR, fit a Newton (or secant) iteration with candidate starting guesses and stopping rules to the cached results, e.g. Excel is documented to iterate up to 20 times to a 1e-5-ish tolerance; confirm against the data, do not trust the documentation. For MIRR, try the formulation `(-NPV(rr, positives)*(1+rr)^n / (NPV(fr, negatives)*(1+fr)))^(1/(n-1)) - 1` and variants until the cached bits match. Reuse any shared NPV/XNPV helper (see T-0901, which owns PV/FV/PMT/NPER/IPMT/PPMT/NPV; do not touch those functions here, only call them if they exist).

## Acceptance criteria
- [ ] At least 100 of the ~170 listed F lines for IRR, RATE, XIRR, MIRR leave eval/baseline/ironcalc-calc.txt with no new ones; the Outcome note lists the remainder.
- [ ] Algorithm parameters (guess, tolerance, iteration cap) documented in docs/excel-semantics/numbers.md with evidence cell ids.
- [ ] Error cases (#NUM! when no convergence) still pass.
- [ ] `make check` green.

## Out of scope
PV/FV/PMT/NPER/IPMT/PPMT/NPV (T-0901), bond PRICE/YIELD families, CUMIPMT/CUMPRINC.

## Findings (attempt 1, no oracle)
IRR.xlsx General!L2 (`IRR(-20,5,5,5,5,5)`, expected 0.07930826116039724): Excel's value is 1.3e-13 below the true root, and its own NPV residual there is 6.4e-12, so Excel stops un-converged. It is not any Newton iterate (from guess 0.1 the iterates are 0.0793082611540 then ...6052853), not a Newton iterate in 1/(1+r), and not a secant iterate for starts g*1.1, g±0.1, g+1e-3/1e-4/1e-5, 0. With guess 17 Excel still reaches 0.07930826116414913, where plain Newton diverges, so Excel uses a safeguarded method. Needs oracle probes with intermediate-iteration evidence before further fitting.
