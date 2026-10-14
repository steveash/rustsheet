---
id: T-0916
title: "IRR/RATE/XIRR/MIRR: match Excel's iteration and termination behavior"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1009, T-1525]
---

## Goal
The iterative financial solvers return results that differ from Excel at 1e-12 to 1e-8 relative because Excel stops its iteration at a looser tolerance than the engine, so the last digits are Excel's un-converged values. Emulate Excel's iteration (initial guess, update rule, tolerance, iteration cap) using the Excel-cached values as the spec. MIRR is a ulp-level formulation difference.

## Context
Kernel map (T-1522, decision 0056): rank 4 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Evidence is Excel-cached values in `ironcalc-calc/FINANCIAL/*.xlsx`. Failing cells (report.md): IRR 74 (IRR.xlsx, mostly NumericClose), RATE 26 (RATE.xlsx), XIRR 17 (XIRR.xlsx 13, XNPV.xlsx 4; NumericMismatch up to 1e-8), MIRR 51 (MIRR.xlsx, NumericUlp). About 170 cells, the largest single group in the report.
Examples: IRR.xlsx General!L2 `IRR(A2:F2)` expected 0.07930826116039724 got 0.0793082611605286; General!N2 `IRR(A2:F2,I2)` expected 0.07930826116414913 got 0.0793082610836084 (the guess changes Excel's result, so the exact Newton path matters). RATE.xlsx Sheet1!F2 expected 0.008052981923906556 got 0.008052981923938503. XIRR.xlsx Sheet1!R2 expected 1.1859161972999572 got 1.1859162079821048. MIRR.xlsx `Two Parameters!E2` `-MIRR(A2:B2,C2,D2)` expected 0.19999999999999996 got 0.20000000000000007.
Approach: for IRR/RATE/XIRR, fit a Newton (or secant) iteration with candidate starting guesses and stopping rules to the cached results, e.g. Excel is documented to iterate up to 20 times to a 1e-5-ish tolerance; confirm against the data, do not trust the documentation. For MIRR, try the formulation `(-NPV(rr, positives)*(1+rr)^n / (NPV(fr, negatives)*(1+fr)))^(1/(n-1)) - 1` and variants until the cached bits match. Reuse any shared NPV/XNPV helper (see T-0901, which owns PV/FV/PMT/NPER/IPMT/PPMT/NPV; do not touch those functions here, only call them if they exist).

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1009 sweep (`probes-numeric/m9-fin-irr-rate`), never blind.
The sweep has landed (691 cells, oracle-filled). Its observations are in
`docs/excel-semantics/financial.md`, "IRR, RATE and XIRR stop early": the default guess is 0.1, IRR's
last digits depend on the guess and on the flow scale and sign, XIRR stops at about 1e-8 to 1e-7 relative and is
scale-invariant, and some guesses give `#NUM!`. RATE's guess argument is unreachable today (T-1050).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/675763979c1defb6 Calc!A39 `=(42+MIRR(Data!A2:C11,0.5,STDEV.S(Data!C11:C12,Data!D7)))` (reinvest rate 5.8E+14): Excel 636207158862729.4, engine 636207158862728.5.
- gen-m9/eefcc3989f31de76 Calc!A5 `=IFERROR((RATE(199,-805,61378)/Data!E3),..)` (E3 = -1): Excel -0.011859926889590918, engine -0.011859926889595479.

**XIRR model fitted by T-1525** (from `tools/`: `uv run rustsheet-fitlab xirr`, 208/208 numeric cells of
probes/solver-iteration-traces X and m9-fin-irr-rate bit-exact, plus 19/19 `#NUM!` via
`solvers.xirr_error_agreement`; `xirr-oos` 8/11; model `xirr_model` in
`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`). XIRR is not Newton. With s = guess (1e-5 when the
guess is 0) and f0 = XNPV(0): for a guess >= 0 walk 0, s, 2s, 4s, ...; the bracket is the first point whose
XNPV sign differs from f0's, with the point before it; if none and s < 1, walk 0, -s, then p -> (p - 1)/2;
a negative guess walks only 0, g, (g - 1)/2, ...; no bracket is `#NUM!`. Then with a = upper end, b = lower
end: m = a + (b - a)/2, replace the end whose XNPV has XNPV(m)'s sign (an exact 0 does not stop), and
return m once (b - a)/2 < 5e-9 + 1e-8*|m| and |XNPV(m)| < 2e-8*max|value|. The XNPV part fails one
124-year span (xirr-early-dates A8: Excel returns bisection step 25, the model step 28) and a zero first
value (A10, A11); T-1533 continues that fit. The walk and bisection caps are unprobed.

**Scope.** The XIRR part can be implemented now. IRR and RATE are not fitted (see
`docs/excel-semantics/financial.md`, "IRR returns x - 1 ..." and "RATE iterates on the rate itself"); their
part of this task waits for T-1533.

## Acceptance criteria
- [ ] At least 100 of the ~170 listed F lines for IRR, RATE, XIRR, MIRR leave eval/baseline/ironcalc-calc.txt with no new ones; the Outcome note lists the remainder.
- [ ] Algorithm parameters (guess, tolerance, iteration cap) documented in docs/excel-semantics/numbers.md with evidence cell ids.
- [ ] Error cases (#NUM! when no convergence) still pass.
- [ ] XIRR follows the T-1525 model (bracket chain, bisection, stop test); `uv run rustsheet-fitlab xirr` shows the engine at 208/208.
- [ ] The chain-walk and bisection caps (unprobed; 100 points per walk and 200 bisection steps are enough on all data) are either probed or recorded as a DEVIATION candidate under decision 0052 and directive 5 (a bound the engine needs whatever Excel's cap is).
- [ ] `make check` green.

## Out of scope
PV/FV/PMT/NPER/IPMT/PPMT/NPV (T-0901), bond PRICE/YIELD families, CUMIPMT/CUMPRINC.

## Findings (attempt 1, no oracle)
IRR.xlsx General!L2 (`IRR(-20,5,5,5,5,5)`, expected 0.07930826116039724): Excel's value is 1.3e-13 below the true root, and its own NPV residual there is 6.4e-12, so Excel stops un-converged. It is not any Newton iterate (from guess 0.1 the iterates are 0.0793082611540 then ...6052853), not a Newton iterate in 1/(1+r), and not a secant iterate for starts g*1.1, g±0.1, g+1e-3/1e-4/1e-5, 0. With guess 17 Excel still reaches 0.07930826116414913, where plain Newton diverges, so Excel uses a safeguarded method. Needs oracle probes with intermediate-iteration evidence before further fitting.
