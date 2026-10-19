# Financial functions

## Annuity evaluation order and `(1+r)^n` (PV/FV/PMT/NPER/IPMT/PPMT)
Rule: with `P = (1+r)^n` by right-to-left square-and-multiply (numbers.md, "Integer exponents"),
`k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, Excel computes PV = `-(fv+T)/P` and FV = `-(pv*P+T)`
(integer nper, rate ≠ 0), and NPER = `ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)`, every
operation in doubles and each logarithm a double logarithm of the double `1+r` (libm or correctly
rounded; the sweep does not separate them), not `log1p`. PMT, IPMT, PPMT and NPV are still
unexplained; 64-bit intermediates do not explain them either (below).
Evidence (Excel-cached values, `ironcalc-calc/FINANCIAL/PMT.xlsx`, `PV.xlsx`, `FV.xlsx`):
- `PMT.xlsx` Sheet1!F36 (rate 0.2, nper 5, pv -200): Excel's `(1+r)^n` behaves as exactly `2.48832`
  (square-and-multiply, or exp/log), not libm `powf` (`2.4883199999999994`). With that value
  `(fv*r+pv*r*P)/(1-P)` reproduces `66.87594065792302`.
- But `PMT.xlsx` Sheet1!J32 (rate 0.2, nper 3) matches only with libm `powf` (`1.7279999999999998`),
  so no single power routine plus one formula shape fits all cells.
- Fit experiment (2026-10-02, over 112 PV / 50 PMT / 53 FV cached cells): the best candidates
  (PV `-(fv + pmt*(1+r*t)*((P-1)/r))/P`, FV `-pv*P - pmt*(1+r*t)*((P-1)/r)`, powi) gain ~39 cells
  but regress 13 others (PMT F15/F33/I15/J15/J32/J35/L26, PV F14/G14, IPMT_PPMT G13/H31/H4,
  CUMPRINC_CUMIPMT G15), which the gate forbids.
- Related: gen-m2 shows `^` itself is not correctly rounded (numbers.md, T-0221).
- Oracle sweep (oracle: excel-graph, 2026-10-06, T-0942; probes-numeric/m9-fin-power, m9-fin-pv-fv, m9-fin-pmt,
  m9-fin-nper-npv): with `P = (1+r)^n` computed by right-to-left square-and-multiply (numbers.md,
  "Integer exponents"), `k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, PV = `-(fv+T)/P` matches
  308 of 308 cells and FV = `-(pv*P+T)` matches 308 of 308 (with and without fv/pv and type=1).
  The parenthesization of T matters: `(pmt*k)*((P-1)/r)` matches 270 of 308 PV cells. The same
  P and T do not give PMT: 16 candidate shapes and 4 power routines match at most 63 of 154.
  Desktop agreement: the same PV formula matches all 23 cells of `ironcalc-calc/FINANCIAL/PV.xlsx`
  (Mac Excel 16 cached values) with integer nper > 0 and rate ≠ 0.
  The engine passes 315/616 power, 208/616 PV/FV, 258/924 PMT/IPMT/PPMT and 119/220 NPER/NPV cells.
- NPER and the x87 test (T-1524, fit lab `annuity-*` fits, same sweeps; numbers.md "x87 precision
  is per operator, not per kernel"): NPER as above matches 95 of 95 numeric cells
  (m9-fin-nper-npv; with libm or with correctly rounded double logarithms alike; the engine 61;
  `log1p(r)` in the denominator 11; the correctly rounded NPER 6). Keeping
  the intermediates at 64 bits (x87 registers, 1+r included) breaks the fitted forms: PV 174/308,
  FV 230/308, NPER 6/95. PMT (m9-fin-pmt, 308 cells; Excel lies up to 3.5 ulps from the exact
  value with the rate as a double): correctly rounded and every 64-bit model (P from the exact
  power, by square-and-multiply at 64 bits, or by fyl2x/f2xm1/fscale) 132, the best double
  formulation with `P-1 = expm1(n*log1p(r))` 129 (`-(pv*r+(pv+fv)*r/(P-1))/k`; `-(pv*P+fv)/((P-1)/r*k)`
  is 120 bit-exact and 308/308 within 4 ulp), double shapes on the square-and-multiply P at most 91, the
  engine 54 (176 within 4 ulp: its `(1+r)^n - 1` cancels for small r*n). Cell A7 `PMT(0.2,2,10000)`
  = -6545.454545454547 is 1.45 ulps from the exact value. IPMT (PMT then the FV recurrence):
  correctly rounded 219/308, 64-bit 218, doubles 195, engine 189. PPMT: 94, 85, 43, engine 15. NPV
  (88 cells): best 27 (a running factor `d *= 1+r`, or Horner), 64-bit 26, correctly rounded 18,
  engine 21; 11 further double orderings (`v*d` with `d /= 1+r`, `pow(1+r,-i)`, `exp(-i*log1p r)`,
  reverse sums, `fsum`) reach at most 27.
- Implementation check (T-0901): NPER's final division is Excel's `/`, rounded twice. With a
  plain double division the engine misses one cell, `NPER(0.2,-1000,-1000,20000)` (m9-fin-nper-npv
  A9, 7.8274691195894075), so the sweep does separate the two after all. Five PV and three NPER
  cells of `ironcalc-calc/FINANCIAL/PMT.xlsx` take their pmt from PMT cells; given Excel's cached
  PMT values as inputs, the fitted formulas reproduce all eight cached results (decision 0064).
- PMT with `P-1 = expm1(n*log1p(r))` (`-(pv*P+fv)/((P-1)/r*k)`, 120/308 bit-exact on the sweep)
  fixes 30 Excel-cached cells but regresses 22 others (PMT.xlsx, IPMT_PPMT.xlsx,
  CUMPRINC_CUMIPMT.xlsx), and IPMT's recurrence on the fitted FV regresses 4 more. Both stay with
  T-1528: Excel's PMT is not the accurate value.
- Extrapolated, not yet probed (T-1684): non-integer nper (where `^` is exp(y·ln x)), NPER with
  pmt = 0 (the general formula, not `-fv/pv`) or pv/fv = 0, and rates within a few ulps of -1 (`1+r`
  by Excel's `+`). The sweeps have integer nper and non-zero pmt only.
Status: PARTLY VERIFIED (PV/FV/NPER fitted on the sweep and implemented, for integer nper and pmt ≠ 0; PMT/IPMT/PPMT/NPV open, time-boxed per
decision 0056 after T-1524's two attempts, in T-1528)
Implemented in: `annuity_power`, `annuity_factor`, `compute_future_value` (`FutureValueForm::Fitted`),
`fn_pv` and `fn_nper` (crates/rustsheet-engine/src/functions/financial/mod.rs, T-0901). Salvaged from
draft PR #13 by T-0031.

## IRR, RATE and XIRR stop early, and the guess and flow scaling change the result
Rule: Excel's iterative solvers (IRR, RATE, XIRR) return an un-converged iterate, not the root
rounded to double. The digits beyond the stopping tolerance depend on the starting guess (default
0.1), and for IRR also on the scale and sign of the cash flows. XIRR stops much earlier (1e-8 to 1e-7
relative) than IRR (about 1e-12 relative). RATE's stopping error depends on scale (about 2e-10
absolute at a zero rate). Some guesses make Excel return `#NUM!` where another
guess converges. MIRR is closed form and is not affected.
Evidence (oracle: excel-graph, 2026-10-06, T-1009; `probes-numeric/m9-fin-irr-rate`, 691 cells;
labels in the case's `hypotheses`):
- The default guess is 0.1. Omitting the guess gives the same bits as passing 0.1 for every IRR,
  RATE and XIRR row in the sweep, for example `IRR({-20,5,5,5,5,5})` and `IRR(…,0.1)` both give
  0.07930826116039724, which matches the cached IRR.xlsx General!L2.
- IRR depends on the guess beyond 1e-12. For `{-20,5,5,5,5,5}`, guesses -0.99 … 100 give 0.079308261144…
  to 0.079308261165…, for example guess 17 gives 0.07930826116414913 (= IRR.xlsx General!N2) and guess
  0.0793, the nearest to the root, gives the furthest result (0.07930826114474532).
- IRR depends on the cash-flow scale and sign. Multiplying the flows by 1E6 or -1 changes the last
  digits in 11 of 20 families (`irr/flow=A/scale=1E6` 0.07930826116052869, `scale=-1`
  0.07930826116045497, against 0.07930826116039724). Multiplying by 1E-6 changes only 6 of 20 (E, F, J,
  K, N, P), so the stopping test is not purely absolute or purely relative on the NPV.
- An IRR whose root is exactly 0.1 (`{-100,110}`, `{-100,0,121}`, `{-100,10,10,10,10,110}`, leading
  or trailing zero flows) returns 0.10000000000000009, which is 0.1 + 6 ULP. With guess 10 several
  of them return 0.09999999999999987 instead (0.1 − 10 ULP). A root of exactly 0 (`{-100,50,50}`) gives
  -2.886801908630332e-12 by default, but exactly 0 with guess 0 or at scale 1E6.
- With multiple roots, the guess picks the root. `{-100,230,-132}` (roots 0.1 and 0.2) gives about 0.1
  for guesses -0.5, 0 and 0.1, and about 0.2 for guesses 1 and 10 (0.20000000000001728,
  0.20000000001634977).
- Flows with no sign change give `#NUM!` for every guess (`{20,5,5,5,5,5}`, `{-20,-5}`). The extreme
  roots `{-100,1}` and `{-1,100}` give -0.99 and 98.99999999999991 for every guess.
- RATE with guess -0.5 gives `#NUM!` on all 8 cached RATE.xlsx rows, and guess 0 gives `#NUM!` on 2 of
  them. `RATE(360,-599.55,100000)` converges (0.004999993193119262), but with guess 0.5 it gives `#NUM!`.
- The RATE round-trip `RATE(n,PMT(r,n,10000),10000)` from the default guess gives `#NUM!` when the
  true rate is far above 0.1 and nper is long: r=0.2 for n ≥ 24, r=0.125 for n ≥ 60, r=0.3 for
  n ≥ 10 (and r=0.1, n=360, fv=-500, type=1). The values it does return differ from r by up to 1.2e-9
  relative (47 of 277 numeric cells exceed 1e-11; worst `r=0.035/12/n=3/…/type=1`
  0.0029166666701373805). This includes the error of Excel's own PMT digits.
- RATE does not snap a zero rate. `RATE(10,-100,1000)` gives -1.963351707292562e-10 and
  `RATE(10,-1E-6,1E-5)` gives -2.3981867563911566e-09, but `RATE(10,-1E6,1E7)` gives `#NUM!`, so
  RATE's stopping test also depends on scale. `RATE(1,0,-100,110)` gives exactly 0.1.
- XIRR is scale- and sign-invariant. All four scalings give identical bits for each of the 8 families.
  It is converged only to about 1e-8 to 1e-7 relative: `XIRR({-1000,1100},{45000,45365})` (exactly 10% a year) gives
  0.09999999403953552 (= 3355443/2^25, 6e-8 relative below 0.1), and with guesses 0, 1 and 10 it gives 0.0999999951171875,
  0.09999999776482582 and 0.1000000024214387. The cached XIRR.xlsx R2 (1.1859161972999572) is the
  default-guess value.
- XIRR guess failures: guess -0.5 gives `#NUM!` for positive-rate flows. Guesses 1 and 10 give `#NUM!` for
  the negative-rate flows (`flow=C`, `D`, `Byr`). XIRR with day-serial dates 1..6 (`flow=B`) gives
  `#NUM!` for every guess.
- MIRR: 32 of 35 cells matched the engine before T-1687 (all 35 since, "MIRR's evaluation order" below).
  `MIRR({-5,4},fr,rr)` is -0.19999999999999996 for every rate pair.
Engine today (after T-0916): 312 of 691 cells pass (IRR 161/216, RATE 47/368, XIRR 72/72, MIRR 32/35;
before: 96, with IRR 19 and XIRR 9).
Status: VERIFIED (observations). XIRR's and IRR's iterations are fitted (next two sections, T-1525,
T-1533) and implemented (T-0916); RATE's update rule and stopping test are UNVERIFIED (partial findings
below); MIRR is fitted and implemented ("MIRR's evaluation order", T-1687)
Implemented in: XIRR and IRR in functions/financial/financial_util.rs (T-0916, next two sections); RATE n/a

## MIRR's evaluation order
Rule: MIRR is `q^(1/(n-1)) - 1` with `q = (P * (1+rr)^n / (1+rr)) / -N`, where P sums the positive flows
`x / (1+rr)^i` and N the negative flows `x / (1+fr)^i` (i = 0..n-1, in order), each power a running product
(`(1+r)^i = (1+r)^(i-1) * (1+r)` from 1, so `(1+rr)^n` is the loop's last factor), and the root is Excel's `^`
(SQRT for n = 3, else `exp(y * ln q)`; numbers.md "Non-integer exponents"). The `/ (1+rr)` step is not the
same as compounding to `(1+rr)^(n-1)`: on two flows (exponent 1, so the root is exact) `MIRR({-100,120},fr,0.03)`
is 0.19999999999999973 for every fr and `MIRR({-3,7},fr,rr)` is 1.333333333333333 at rr = 0.03, 0.07 and 0.12,
while `(120/1.03)*1.03` and `(7/A)*A` round back to the flow; of the products of x with A, A^2, 1/A tried, only `((x/A)*A^2)/A` gives Excel's digits there.
Microsoft's documented `-NPV(rr,pos)*(1+rr)^n / (NPV(fr,neg)*(1+fr))` is mathematically the same but scores 365.
Fit (fit lab `mirr`, 467 cells of m9-fin-mirr and m9-fin-irr-rate; bit-exact / within 4 ulp): this formulation with
Excel's `^` 467/467; with `exp(ln q/(n-1))` 456/467, libm `pow` 456/465; the same sums compounded as
`P * (1+rr)^(n-1)` 413/451 at best; FV and PV computed directly with libm powers 407/447 at best; the NPV form
365/437 at best (the engine before T-1687 was this NPV form, 207 of the first 252 sweep cells).
Evidence: probes-numeric/m9-fin-mirr Calc!A1:A432 (excel-graph, 2026-10-17; A253:A432 are the 6x6 rate grids for
{-100,120}, {-3,7}, {120,-100}, {-3,0,7} and {-3,7,0}); probes-numeric/m9-fin-irr-rate MIRR cells;
ironcalc-calc/FINANCIAL/MIRR.xlsx (225 cells, all passing since).
Status: VERIFIED (fit, 467/467 bit-exact)
Implemented in: `fn_mirr` (functions/financial/mirr.rs). A rate of -1 keeps its own limit cases, and a ratio that is
zero, negative or infinite keeps `powf`; neither is part of the fit.
Web vs desktop (T-0959): since T-0959 `^` takes `exp` from EXP's kernel on a twice-rounded product (numbers.md "Non-integer
exponents"), and MIRR's root does not follow it. `MIRR({40,-5,-5,-5,-5,-5},0.1,0.2)` is 0.3933280241536672 in Mac desktop Excel
16.0300 (ironcalc-calc MIRR.xlsx Sheet2!N16, cached) and 0.3933280241536674 in Excel for the web (probes-numeric/power-mirr-root A1,
A2, and A3 = the same closed form written with `^`). The pre-T-0959 root (libm `exp` of a once-rounded product,
`power_with_libm_exp`) gives the desktop value and still matches all 467 fit cells and 225 MIRR.xlsx cells, so MIRR keeps it
(decisions 0030 and 0039: passing desktop evidence stays). power-mirr-root A1/A2 are the web side of this conflict; the
Windows desktop value is a T-1075 (M14) question.
Tasks: T-1687, T-0959

## XIRR is a bracketing bisection, not Newton
Rule: XIRR finds a sign change of XNPV on a chain of trial rates, then bisects it. The chain starts at
0. With s = guess (s = 1e-5 when the guess is 0), it runs upward through s, 2s, 4s, ... (only when the
guess is not negative). If no upward point changes sign, it runs downward from -s, each point halving its
distance to -1 (p -> (p - 1) / 2); a guess of 1 or more has no downward chain, so a negative root is
`#NUM!`. A negative guess walks only the downward chain from the guess itself (g, (g - 1) / 2, ...), so a
positive root is `#NUM!`. The bracket is the first chain point whose XNPV sign differs from XNPV(0)'s,
with the point before it (0 included); no bracket is `#NUM!`. Bisection: `a` starts at the bracket's upper
end and `b` at its lower end; each step takes m = a + (b - a) / 2 and replaces the end whose XNPV has the
sign of XNPV(m). An exact XNPV(m) = 0 does not stop it. It returns m at the first step where both
h < 5e-9 + 1e-8 * |m| (h = (b - a) / 2, which equals |m - previous m|) and |XNPV(m)| < 2e-8 * max|value|.
So the result lies on the binary lattice of the bracket (for the default guess and a root of exactly 0.1
it is 3355443/2^25 = 0.09999999403953552), which explains why XIRR is scale- and sign-invariant,
converged only to about 1e-8, and why guess -0.5 or 1 gives `#NUM!` on one side.
Evidence (excel-graph, 2026-10-14, T-1525; from `tools/`: `uv run rustsheet-fitlab xirr` and `xirr-oos`,
`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`):
- Fit set: probes/solver-iteration-traces sheet X (155 cells: two-flow roots 0.013 to 0.94 and -0.013 to
  -0.918 at guesses 1, 0.1, -0.5 and 0, one- and ten-year spans, three-flow roots to 2.0) and the 72 XIRR
  cells of probes-numeric/m9-fin-irr-rate. The model is bit-exact on all 208 numeric cells, and it gives
  no result on all 19 `#NUM!` cells (the lab scores error cells since T-1534: `rustsheet-fitlab xirr` shows
  227/227; checked by tools/tests/test_fitlab_solvers.py). The lab's iterate report finds Excel's value among
  the model's bisection midpoints at index 21 to 30 (median 25) on all 208 numeric cells. The engine (Newton) gets 0/208.
- Rejected variants (fit set, numeric cells): the step test alone 195/208, h < 6e-9 81/208,
  h < 1e-8 * |m| 33/208, the XNPV test alone 13/208; m = (a + b) / 2 206/208 and starting `a` at the
  lower end 205/208 (each moves m by an ulp somewhere). The constants are tight: with B = 1e-8,
  A = 4.95e-9 gives 191/208, A = 5.05e-9 208/208 and A = 5.15e-9 203/208; with A = 5e-9, B = 9.9e-9 and
  1.03e-8 give 206/208 and B = 1.02e-8 208/208. |XNPV| < 1.5e-8 * sum|value| also fits all 208.
- The step test has no derivative in it, and the ten-year span (X!C, ten times the derivative) is
  reproduced with the same rule as the one-year span (X!A); the depth per cell comes from a scratch
  analysis, reproducible in the lab after T-1534.
- Out of sample (the literal XIRR cells of five older probes; `xirr-oos`): 8 of 11 numeric cells, and
  8 of 11 `#NUM!` cells. The `#NUM!` misses are input checks recorded elsewhere in this file
  (xirr-mismatched-empties X12, X13, xirr-early-dates A7), which the model does not apply. The numeric
  misses: a zero first value (xirr-early-dates A10, A11 give 2.9802322387695314e-09 = 0.1 * 2^-25; the model
  gives 0.09999999403953552), and the 124-year span of xirr-early-dates A8, where Excel returns bisection
  step 25 (|XNPV|/max|value| is 1.1e-7 there; the model goes to step 28). So the XNPV part of the stop test
  is not normalised by max|value| alone. The step test alone gets A8 but misses
  xirr-xnpv-whole-column-nogaps A4 (8/11, and 13 fitted cells); the sum|value| variant misses A4 too
  (7/11).
- Refinement (T-1583, `xirr-oos` 21/22 bit-exact or `#NUM!`-agreeing, fit set still 227/227; candidate
  "bisect, first-value sign, span-scaled |f| bound (T-1583)"): (1) the end replaced is chosen by the
  first value's sign: the upper end when XNPV(m) * values[0] >= 0, else the lower end. That is the same
  end as the XNPV(upper end) rule on every fitted cell, but with a zero first value it replaces the upper
  end every step, so the result is bracket-lower-end + (0.1 - 0) / 2^25 = 0.1/2^25 = 2.9802322387695314e-09,
  Excel's value on xirr-early-dates A10, A11 (and on xirr-mismatched-empties X3, X4, which the lab does not read);
  the XNPV-at-the-end rule converges to 0.1 instead. (2) The XNPV bound scales with the span:
  |XNPV(m)| < 2e-8 * max|value| * max(1, (latest date - first date)/365), and there is no XNPV bound
  when the first value is 0. Every fitted cell is consistent: on the 217 numeric cells (fit set and `xirr-oos`) other than the zero-first-value
  pair, the unscaled bound must lie in (1.93e-8, 2.05e-8] times max|value|, and A8 (124 years, stopped at
  1.1e-7 * max|value|) needs a span factor; the scaled bound fits all. The feasible-interval check also
  rejects |XNPV| / sum|value|, / |values[0]|, / the discounted sum of |value|, the Newton step
  |XNPV/XNPV'| and the XNPV change between steps as the normaliser. Rejected counts on `xirr-oos`:
  first-value sign with the fitted stop 18/22, the span-scaled bound without the first-value sign 17/22,
  the first-value sign with the XNPV bound skipped only for a zero first value 20/22. Two of the 21 are
  coincidences: the model reaches `#NUM!` on xirr-mismatched-empties X12, X13 (no date later than the
  first) by never stopping (the bisection walks away from the root), not by Excel's input check. The span
  factor's form rests on A8 alone; T-1591 probes it. Why a zero first value drops the XNPV bound is open:
  normalising XNPV by the first value (0/0, NaN) would explain both the end choice and the stop if the
  tests are written to pass on NaN, but nothing probes that.
- Unprobed: the cap on the chain walk (up to 17 doublings, X!F15, and 4 halvings are seen; day serials
  1..6, which need about 770 halvings, are `#NUM!`), the bisection cap, the order when both chains have
  a root, a guess at or below -1, and which end an exact XNPV(m) = 0 replaces (the first-value rule
  replaces the upper end, the XNPV(upper end) rule only when XNPV(upper end) <= 0; no cell tells them apart).
Ours: this model, T-1583's form (`xirr_bracket`, `compute_xirr`, functions/financial/financial_util.rs,
T-0916): 227/227 on the fit set and 22/22 on `xirr-oos` (the Newton code it replaced: 9 and 11). The walk
cap (100 points) and bisection cap (200 steps) are engine bounds (DEVIATION, "The IRR and XIRR solver
caps" below). A trial rate where a power overflows or underflows to 0 has no XNPV, as in the lab model.
Status: VERIFIED (excel-graph) for the bracket, the midpoint and the step test; the XNPV part of the stop
test is UNVERIFIED beyond the fit set (T-1533; refined by T-1583, the span factor UNVERIFIED until T-1591)
Tasks: T-1525, T-0916, T-1533, T-1583, T-1591

## IRR is a secant in v = 1/(1 + r) from v0 and v0 +- 0.001 (fitted, 189/263 bit-exact)
Rule: IRR runs the secant method on NPV as a function of v = 1/(1 + r):
- start: v0 = 1/(1 + guess) (guess 0.1 when omitted) and v1 = v0 + 0.001 when NPV(v0) > 0, else v0 - 0.001;
- step: v2 = v0 - NPV(v0)(v1 - v0)/(NPV(v1) - NPV(v0)), then shift (v0, v1) <- (v1, v2);
- stop: at the first v2 with |v2 - v1| < 1e-7 and |NPV(v2)| < 1e-7, returning 1/v2 - 1;
- a run that does not stop is retried once from guess 0.1. (The 18 `#NUM!` cells of the sweeps are all
  flows without a sign change; they are not scored.)
So the result is x - 1 with x = 1/v a double: for all 263 numeric IRR cells, value + 1 is exactly
representable (tools/tests/test_fitlab_solvers.py). Evidence for each part:
- The secant's error law, signs included: on `{-20,5,5,5,5,5}` the guesses 0.05 to 0.07 and 0.0875 to 0.115
  (I!A1:A9, I!A16:A27) return the fourth secant iterate, whose error grows as (guess - root)^8 (the
  Fibonacci orders 1, 1, 2, 3, 5, 8). Guesses just above the root give positive errors (I!A13, I!A14:
  +5.9e-15, +1.6e-12), which no quadratic method with f''/(2f') < 0 can produce and the secant's third
  iterate does. The offset is 0.001 in v, toward larger |NPV| (+ for guesses below this root, - above):
  with +0.001 on both sides the right-hand guesses miss by 3 to 11,410 ulp (121/263 overall); with -0.001
  they are exact.
- The step test: over the I sheet the returned iterate's |dv| is at most 9.84e-8 and every earlier one's
  at least 1.18e-7, so the bound is 1e-7 (Excel's documented "0.00001 percent"); 1e-6 scores 169, 1e-8
  146.
- The residual test makes IRR scale-dependent: `{-20,5,5,5,5,5}` gives 0.07930826116039724 and
  `{-20000000,5000000,...}` the converged root (m9-fin-irr-rate Calc!A1, A3). Without it 181/263, with
  1e-6 188, with 1e-8 180.
- The retry: guesses 0.7, 2, 3, 5 on `{-1,0,...,0,2}` (I!B13:B16) and 1, 10 (Calc!A134, A135, A159, A160)
  return exactly Excel's guess-0.1 value; from those guesses the secant creeps on a flat NPV and does not
  stop. Without the retry 187/263. (The model misses these cells by 1 ulp, as it misses the guess-0.1
  cells themselves, Calc!A132 and A157: the last-bit question below.)
- The step cap is at least 35 (the most steps an exact cell takes, I!B1) and below 1000 on this data
  (caps 35 to 100 score 189, 1000 scores 188: a slow run must fail and retry); 50 is used. A cap of 20
  (Excel's documented "20 tries") scores 181.
Open: 18 of the 74 misses are 1 to 4 ulp off, and most of the rest are within 1e-9 (NPV's evaluation
order matters: as sum c_i/x^i with x = 1/v the model gets 189, by Horner in v also 189 but on different
cells, and the two-flow cells prefer Horner), and `{0,0,-100,110}` with guess -0.5 (Calc!A173, 3 ulp)
and guess 10 (A175, where the model converges to another root, 6348552.78) are not reproduced.
Evidence: probes/solver-iteration-traces I!A1:A29 (guess sweep), I!B (far guesses on `{-1,0,...,0,2}`),
I!C, I!D (closed-form two- and three-flow roots), probes-numeric/m9-fin-irr-rate (scaled and sign-flipped
flows, guesses); excel-graph, 2026-10-14. `uv run rustsheet-fitlab irr` (263 cells): the fitted secant
189/263 bit-exact, 207 within 4 ulp, 260 within 1e-9; the best Newton candidate (in v, converged) 74; the
engine 1.
Ours: this secant (`irr_npv`, `irr_secant`, `compute_irr`, functions/financial/financial_util.rs, T-0916):
189/263 bit-exact and 207 within 4 ulp on the fit set, the same cells as the lab model (the Newton code it
replaced: 1). Three gated cells the Newton code matched by chance are known failures now (decision 0065).
Where both secant runs fail the engine falls back to the earlier Newton/bisection solver (not an Excel
fact). Excel returns `IRR({-400,1,1,1,1,1})` = -0.6737366751808405 (ironcalc-calc IRR.xlsx General!L17),
where the fitted secant does not stop, and raising its step cap does not explain it (1,000 steps give
-0.6737366751808431). Probing it is T-1686.
Status: VERIFIED for the method (secant in v, offset, both stop tests, retry; 189/263 bit-exact); UNVERIFIED
for the last bit (NPV's evaluation order), the step cap, and flows with leading zeros (A173, A175)
Tasks: T-1525, T-1533, T-0916

## The IRR and XIRR solver caps
Rule: Excel's caps on its IRR secant, XIRR's bracket walk and XIRR's bisection are unknown: no probe
reaches them. The engine bounds each loop itself, so an untrusted workbook cannot make one cell iterate
without end (CLAUDE.md directive 5). XIRR walks at most 100 points per chain and bisects at most 200
steps. IRR's secant takes at most 50 steps per run, with one retry from 0.1, and then the earlier
Newton/bisection solver at most 50 Newton steps per start and 50 bisection steps. A run that reaches its
cap gives `#NUM!` (or, for IRR, moves on to the next stage). Every oracle cell of the fit sets finishes
well inside the caps.
Evidence: none for Excel; this is a resource limit, not an Excel observation. Probing Excel's caps, the
fallback path and overflowing trial rates is T-1686.
Status: DEVIATION (decision 0014)
Implemented in: `xirr_bracket` (`WALK_CAP`), `compute_xirr` (`BISECTION_CAP`), `irr_secant` (`MAX_STEPS`),
`irr_fallback` and `irr_newton_raphson` (functions/financial/financial_util.rs)
Tasks: T-0916, T-1686

## RATE iterates on the rate itself
Rule: RATE's results are rates, not x - 1: only 7 of the 136 literal RATE cells pass the x - 1 test above
(tools/tests/test_fitlab_solvers.py). The update rule and the stop test are not fitted. `RATE(1,0,-1,X)` cannot separate
rules: f is linear in r, so every Newton step from the first on gives the same value, and 9 of the 10
cells are that value; the informative cell is X = 1.37 (R!A4), where Excel gives 0.3700000000000002 and
every Newton candidate 0.37. Plain Newton cannot be the method: from 0.1 it crawls on
RATE(360,-599.55,100000) (about 0.003 per step), which Excel solves.
Evidence: probes/solver-iteration-traces R!A1:A10, R!B1:B30 (RATE(n,0,-1,2)), R!C1:C31 (a guess sweep on
RATE(48,-200,8000,0,1)), R!D:F (annuities with type 0/1 and fv); excel-graph, 2026-10-14, T-1525.
`uv run rustsheet-fitlab rate` (136 literal cells; the 277 nested `RATE(n,PMT(...),...)` cells are not
parsed): Newton in r on the documented f(r) = pv(1+r)^n + pmt(1+r*type)((1+r)^n - 1)/r + fv gives at most
11/136 bit-exact (4 to 8 steps), FD Newton (h = 1e-8) 7/136; the engine gets 12/136.
T-1533 tried IRR's secant on RATE's documented f(r) (lab `rate`, family "secant": second point
r0 + 0.001, -0.001 or 0.0001 times the sign of f(r0); stop |dr| < 1e-7 or 1e-10, with and without
|f| < 1e-7): at most 19/136 bit-exact (-0.001, |dr| < 1e-7, |f| < 1e-7), 40 within 4 ulp. Excel's
errors on the guess sweep R!C1:C31 are not smooth in the guess (C8 -1.8e-13, C9 +2.6e-16, C11
+1.6e-13), unlike IRR's; between 0.019 and 0.028 they are positive and grow as about (guess - root)^26
(a secant's order 21 or 34; scratch analysis against a 60-digit root, not a lab fit, until T-1534's traces).
Ours: Newton, up to 50 iterations (`compute_rate`, functions/financial/financial_util.rs).
Status: UNVERIFIED (the update rule and the stop test)
Tasks: T-1525, T-0916, T-1533, T-2005

## YIELD with one coupon period left is closed form
Rule: when at most one coupon period is left, Excel evaluates the documented formula
`((red/100 + rate/f) - (pr/100 + A/E*rate/f)) / (pr/100 + (A/E)*(rate/f)) * (f*E/DSR)` in that order, with
DSR the basis' day count from settlement to maturity (DAYS360 US on basis 0, DAYS360 European on basis 4,
actual days on bases 1-3), not E - A: 2024-03-01 to 2024-08-31 on basis 0 is DSR 180 with A 181 and E
360. E is 360/f on bases 0 and 4 and the actual coupon-period length on bases 1, 2 and 3 (182 and 366
days for the probe bonds, not 360/f or 365/f). PRICE's one-period form differs: it discounts with
DSR = E - A and the nominal E (360/f on basis 2, 365/f on basis 3).
Evidence (excel-graph, 2026-10-14, T-1525): `uv run rustsheet-fitlab yield-one-period` is bit-exact on 60 of
60 cells: probes/solver-iteration-traces Y!A1:A10 (semiannual, maturity 2024-06-30) and Y!B1:B10 (annual,
maturity 2024-08-31), prices 90 to 110, basis 0, and probes/bond-one-period-bases sheet Y (the same two
bonds on bases 0 to 4, prices 95 to 105). E - A for DSR gives 30/60 and the nominal E on bases 2 and 3
44/60. PRICE: probes/bond-one-period-bases sheet P (40 cells, yields 0.02 to 0.3) all pass in the engine,
whose one-period PRICE uses E - A.
Ours: `yield_one_period` (functions/financial/financial_bonds/yield_closed.rs) evaluates the closed form and is bit-exact on 60/60 (T-1532); before it, the engine solved PRICE and got 0/60.
Where PRICE's E - A and E equal YIELD's (basis 0 on the semiannual bond, bases 1 and 4) it lands a few to
about 15 ulp off; elsewhere it solves a different equation: basis 0 annual (Y!B) is about 0.56% off (ratio
180/179, Y!B1 0.2807210017227244 against 0.2791614406020427), bases 2 and 3 up to 4%.
Status: VERIFIED (excel-graph)
Tasks: T-1525, T-1532

## RATE: an empty guess is guess 0, not the default 0.1
Rule: an empty guess argument (`RATE(48,-200,8000,0,1,)`) or a reference to an empty cell behaves as
guess 0, not the default 0.1: the result has the same bits as an explicit `0` (0.008052981923885287)
and differs from the 0.1 result in the last digits (…906556). Omitting the guess altogether uses 0.1
(A4 equals A2). An empty `fv` (`RATE(48,-200,8000,,1)`, A4) is 0 (the same bits as the explicit 0 and 0.1 guess
row A2), and an empty `type` is 0 (same bits as `type` 0, A5 = A6). A text guess (`""`) is `#VALUE!` and
`TRUE` is guess 1 (A9 = A10, same bits). Excel converges for guess 0 here; the engine's solver returns `#NUM!` (T-0916,
iteration fit).
Evidence: probes/rate-empty-arguments A1-A10 (excel-graph, T-1128)
Status: VERIFIED
Implemented in: n/a (empty argument already reads as 0; convergence is T-0916)

## XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty cells
Rule: XIRR reads an empty value or date cell as 0. It does not skip it. A zero value adds nothing, so
`XIRR(A1:A3,B1:B3)` with A3 and B3 empty is the same 0.09999999403953552 as `XIRR(A1:A2,B1:B2)`. So
are whole-column `XIRR(Data!A:A,Data!B:B)` on a sheet holding only A1:B2, whole-row
`XIRR(12:12,13:13)`, an empty value with a date (xirr-mismatched-empties X1) and a zero value with an
empty date (X11). Date 0 is a valid date: -1000 with an empty date, then 1100 on 45365, gives
1.1^(365/45365) - 1 = 0.000767144560813904 (X5, xirr-early-dates A8). A value with an empty date late
in the range puts that flow on date 0. 50 on date 0 after -1000 on 45000 and 1100 on 45365 has no root
and is `#NUM!` (X2). Text in either range is `#VALUE!` (X6, X7). Dates before the first are accepted:
`XIRR({-1000,1100,-1000},{45000,45365,1100})` = -0.01750176399946212 (XNPV of the same flows is
#NUM!), and -1000, 1100, -1000 on 45000, 45365, 44000 = -0.24668241366744048 (X8). But XIRR is `#NUM!`
when no date is later than the first, even where a root exists: -1000 on 45365 then 1100 on 45000 (X9,
xirr-early-dates is the same as an array: X12), and 1100, -1000, -1000 on 45365, 45000, 44000 (X13).
All dates equal is `#NUM!` (A3), while a repeat of the first date next to a later one is fine (A4,
A5). Positive flows on earlier dates with no root are `#NUM!` (A1, A2, A14, A15). A negative date or
one past 2958465 is `#NUM!` (A6, A7, A16). A zero first value gives 2.9802322387695314E-09 =
0.1 / 2^25 instead of the root, whatever its date (X3, X4, A10, A11); a zero value later is harmless
(A12).
XNPV is `#NUM!` when its ranges contain empty cells, and a whole column counts every cell, not the
used area: `XNPV(0.1,Data!A:A,Data!B:B)` is #NUM! although Data holds only A1:B2, where
`XNPV(0.1,Data!A1:A2,Data!B1:B2)` is finite.
Unprobed: which error XNPV gives for a whole column whose used area holds a cell error or a text date
(the engine returns that error, then #NUM! for the skipped cells); XIRR dates strictly between -1 and 0
(the engine truncates first, so -0.5 becomes -1 and is #NUM!).
Evidence: probes/xirr-xnpv-whole-column-nogaps A1-A8, probes/whole-column-empty-walkers E1, E5, E7,
F3, F4, F6, F7, F9 (excel-graph, 2026-10-07, T-1114); probes/xirr-mismatched-empties X1-X14,
probes/xirr-early-dates A1-A16 (excel-graph, 2026-10-08, T-1124).
Status: VERIFIED (these cells). XNPV matches since T-1114 (the used-area clip applies the empty-cell
error to the cells it skips, `get_array_of_numbers_generic`). XIRR follows the empty-as-0, date-range
and later-date rules since T-1124. Its values still miss in the last digits (the early stop above).
The zero-first-value result and the date-0-first root match since T-1583 and T-0916 (the solver entry
"XIRR is a bracketing bisection, not Newton" above; T-1155 done).
Tasks: T-1114, T-1124

## Financial function argument modes
Rule: every scalar argument of the M9 financial functions lifts elementwise over an array literal: all 256
`ROWS(F(..{v;v}..))` cells are 2, the dates, frequency, basis, type and guess arguments included. Over a two-cell
range the core worksheet functions lift as well (DB, DDB, FV, IPMT, ISPMT, MIRR, NPER, NPV's rate, PDURATION, PMT,
PPMT, PV, RATE, RRI, SLN, SYD, VDB and IRR's guess: `ROWS(F(G1:G2,..))` is 2). The Analysis ToolPak functions do not:
ACCRINT, ACCRINTM, AMORDEGRC, AMORLINC, the COUP* family, CUMIPMT, CUMPRINC, DISC, DOLLARDE, DOLLARFR, DURATION,
EFFECT, FVSCHEDULE's principal, INTRATE, MDURATION, NOMINAL, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PRICE,
PRICEDISC, PRICEMAT, RECEIVED, TBILLEQ, TBILLPRICE, TBILLYIELD, XIRR's guess, XNPV's rate, YIELD, YIELDDISC and YIELDMAT
are `#VALUE!` when their first scalar argument is a multi-cell range. Of the later scalars only EFFECT's npery was probed over a range: `ROWS(EFFECT(0.1,D1:D60))` is
`#VALUE!`, while the core `ROWS(PMT(0.1,D1:D60,100))` is 60 (probes/scalar-lift-shapes H12, H14, T-1767). The other
later ATP scalars sampled are refused as well (probes/scalar-lift-open-a Y1-Y7, Y16-Y18, T-1777, scalar controls Z1-Z7, Z16-Z18):
`ROWS(F(..))` is `#VALUE!` for PRICE's `frequency` and `basis`, COUPDAYS's and COUPNUM's `frequency`, COUPDAYS's `basis`,
ACCRINT's `frequency` and `basis`, and DOLLARDE's and DOLLARFR's `fraction` over a range. An error scalar beside a
range follows the function's own check order (ACCRINTM: `NA()` before the range is `#N/A`, after it `#VALUE!`; probes/scalar-lift-open-d W8-W15). Next to an array, the range
makes every probed element `#VALUE!` (`EFFECT({0.1;0.2},M1:M60)`). See arrays.md "Scalar arguments lift: shapes,
element errors, and the Analysis ToolPak range rule".
Every later argument of NOMINAL, ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT, TBILLEQ,
TBILLPRICE and TBILLYIELD is refused too, over a two-cell range on the formula's own rows, where the intersected
call (`@G3:G4`) is the base value (probes/fin-later-argument-ranges H1-I82, T-1372). The last argument of every core
function lifts over such a range (`ROWS(RATE(36,-300,8000,0,0,L7:L8))` is 2; K1-K25 for PV, FV, NPER, RATE, IPMT,
PPMT, ISPMT, RRI, PDURATION, SLN, SYD, DDB and PMT). ACCRINTM's basis lifts element by element like a scalar basis:
`{0;1;2;3;4}` gives 100, 100, 101.38888888888889, 100, 100; 5 and -1 are `#NUM!`, `TRUE` is `#VALUE!`, `"1"` is
100, 1.9 truncates to 1 and `"x"` is `#VALUE!`; beside `{39083;39100}` in the issue it broadcasts to a 2x2 result
(J1-J15). ACCRINTM reads every argument before it checks a value: `ACCRINTM(39083,39448,-0.1,1000,NA())` is `#N/A`,
not the rate's `#NUM!`, and the same per element (J17, J20).
Every other later argument of ACCRINT, AMORDEGRC, AMORLINC, the COUP* family, CUMIPMT, CUMPRINC, DURATION, MDURATION,
ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PRICE and YIELD is refused too, over a two-cell range on the formula's own
rows (93 cells, each with its intersected control giving the base value: probes/fin-remaining-later-argument-ranges
H1-I184, T-1822). Lifted elements are the scalar calls (`INDEX(COUPNUM(39448,{43831;44196},2,0),2)` = 26, K11-K13).
YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD lift the same way (probes/fin-yield-odd-lift, T-1842): an element
equals the scalar call (`INDEX(YIELD(39448,43831,0.0575,{95.04;98},100,2,0),2)=YIELD(..,98,..)` is TRUE, also for a
row against a column), an element error stays in its position (frequency 3, basis 5, a price of -1, a settlement past
the maturity, a text yield), a boolean element is `#VALUE!` like a typed boolean (`YIELD(..,{0;TRUE})`'s second
element and `YIELD(..,TRUE)`), the first argument over a range on the formula's own rows is `#VALUE!` (H5-H13, with
the intersected controls in I), and an error scalar beside a range follows the argument order:
`YIELD(NA(),G21:G22,..)` and `ODDFPRICE(..,NA(),..,G25:G26,0)` (the rate before the frequency) are `#N/A`,
`YIELD(G21:G22,NA(),..)` and `ODDLYIELD(G23:G24,..,NA(),0)` are `#VALUE!` (D1-D4). Read from a cell, 1E300 is `#NUM!` as
YIELD's settlement, ODDFPRICE's frequency and ODDFYIELD's basis, -1E300 as ODDLPRICE's settlement, and ODDLYIELD
with a price of 1E300 is -2 (L1-L8).
Excel computes CUMIPMT and CUMPRINC over very many periods (`CUMIPMT(0.01,1E6,100000,1,1E6,0)` = -999900000,
`CUMIPMT(0.01,1E15,100000,1E15-10,1E15,0)` = -632.3717517805198; K1-K3; `CUMIPMT(1E-9,1E9,1,1,1E9,0)` = -0.5819767073296636
and `CUMPRINC(1E-9,1E9,1,1,1E9,0)` = -1, probes/fin-cumipmt-period-bound A1, A2; "CUMIPMT and CUMPRINC over many
periods"), and AMORLINC and AMORDEGRC give 0 for a period of 1E9 (K4, K5; past the asset life, "AMORLINC and
AMORDEGRC over many periods").
NPV's values work like an aggregate. A range skips text, booleans and blanks: `NPV(0.1,L1:L8)` over
-10000, "2750", TRUE, 4250, "x", blank, 3250, 2750 is -1258.4522915101434, the NPV of -10000, 4250, 3250, 2750.
Typed arguments are coerced: `NPV(0.1,-10000,"2750",TRUE,4250)` = -3914.6233180793665, the NPV of
-10000, 2750, 1, 4250. Typed non-numeric text is `#VALUE!` (`NPV(0.1,-10000,"x",4250)`). An array literal is taken
whole (`NPV(0.1,{-10000,2750,4250,3250,2750})` = 302.233453999042, as for the range), and so is a values array,
which does not lift: `ROWS(NPV(0.1,{1;2}))` is 1.
IRR and MIRR skip text, booleans and blanks in a range, and IRR skips text inside an array literal too:
`IRR({-10000,"2750",4250,3250,2750})` = `IRR(L1:L8)` = 0.013442343703144344, the IRR of -10000, 4250, 3250, 2750.
With text and a boolean in the values, XNPV is `#NUM!` and XIRR is `#VALUE!`: `XNPV(0.09,L1:L5,M1:M5)` vs
`XIRR(L1:L5,M1:M5)`. FVSCHEDULE's schedule is `#VALUE!` with a text cell (`FVSCHEDULE(1000,N1:N2)` over 0.09, "0.11")
or text in an array literal (`{0.09,"0.11"}`). A plain number is a one-rate schedule: `FVSCHEDULE(1000,0.09)` = 1090.
These set the generator modes in eval/catalog/functions.yaml. Scalars are `value`, NPV's values are `aggregate`,
and IRR's and MIRR's values, XNPV's and XIRR's values and dates, and FVSCHEDULE's schedule are `array`.
Evidence: probes/fin-argument-modes (386 cells, excel-graph, 2026-10-11, T-1084), probes/fin-later-argument-ranges
(115 cells, excel-graph, 2026-10-17, T-1372), probes/fin-remaining-later-argument-ranges (199 cells, excel-graph,
2026-10-17, T-1822), probes/fin-yield-odd-lift (51 cells, excel-graph, 2026-10-18, T-1842). The cash flow is K1:K5 and its
dates are M1:M5. L1:L8 is the same flow with typed text, a boolean, text and a blank mixed in, and N1:N5 is a
schedule containing text.
Ours: since T-1372 the core functions but DB, VDB, NPV, IRR and MIRR are `Lift` and EFFECT, NOMINAL, DOLLARDE,
DOLLARFR, ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT and the T-bills `LiftArrays`
(SPEC-0011); since T-1822 so are ACCRINT, the COUP* family, DURATION, MDURATION and PRICE, each element charged for its
coupon schedule walks ("Coupon schedule work cap"), and since T-1842 YIELD and the ODD* functions, charged for their
solvers and walks ("Yield and odd-coupon work cap"), and since T-1845 CUMIPMT and
CUMPRINC, charged for their bounded period sums ("CUMIPMT and CUMPRINC work cap"), and since T-1841 AMORDEGRC and
AMORLINC, charged for their bounded period walks ("AMORLINC and AMORDEGRC work cap"). The rest do not lift yet (`#N/IMPL!` for an array literal; DB and
VDB `#VALUE!` for a range): DB and VDB need their period loops bounded first (T-1820), NPV's rate, IRR's guess and
MIRR's rates sit beside a whole cash flow (T-1821), and XNPV's rate,
XIRR's guess and FVSCHEDULE's principal sit beside whole data (T-1843). NPV of an array
literal is 0, and typed text and booleans are not coerced (T-1373). FVSCHEDULE gives `#VALUE!` for text in the schedule (T-1373) and for a boolean principal (T-1765).
The base values of CUMIPMT, DB, DDB, DURATION, INTRATE, MDURATION, ODDFPRICE, PRICE and YIELDDISC are 1 to a few
ulps off, CUMPRINC and EFFECT about 30-40 ulps, and those of ODDFYIELD and YIELD are off at 1e-10 to 1e-12 relative (T-1374; TBILLEQ and TBILLYIELD are exact since T-1475). The
annuity, NPV and solver misses are T-0901's and T-0916's.
Status: VERIFIED (excel-graph)
Tasks: T-1084, T-1372, T-1373, T-1374, T-1820, T-1821, T-1822, T-1841, T-1842, T-1843, T-1845, T-1870

## MIRR error and edge cases
Rule: MIRR divides each flow by its running power, so a zero power is `#DIV/0!`, and an overflowing power is an error:
- A single flow (negative or positive) is `#DIV/0!`; so are values with no negative flow. All-negative flows are -1 (`MIRR({-1,-2,-3},..)`).
- A reinvest rate of -1 is `#DIV/0!` whatever the flows (`{100,-50}`, `{-1,-2,-3}`, `{-100,0}`; A7, A19, A20, A25, A26). A finance rate of -1 is `#DIV/0!` when a negative flow
  follows the first (`{50,-100,80}`, `{-100,50,-80,120}`, `{-1,-2,-3}`); with the only negative flow first (`{-100,50,80}`, `{-50,100}`) it is a number (0.1618950038622251, 1).
- Rates below -1 are ordinary numbers: finance -2 with the first negative flow first is the same 0.1618950038622251 as -1 (A9), reinvest -2 gives -0.4522774424948339 (A10),
  both at -1.5 -0.2583801512904337 (A11), all-negative flows with reinvest -2 are -1 (A27); `#NUM!` when the ratio goes negative (`{50,-100,80}` with finance -2, A23).
- Power overflow (a rate of 1E+200, or 400 flows at rate 5): with positive flows a reinvest overflow is `#DIV/0!` (A16, A17) and a finance overflow `#NUM!` (A15, A24; negative 1E+200 too);
  both overflowing is `#DIV/0!` (A21). With no positive flow the two swap: reinvest overflow is `#NUM!` (A28), finance overflow `#DIV/0!` (A29). Both overflowing with no positive flow is unprobed.
Evidence: probes/fin-mirr-edges A1-A30 (excel-graph, 2026-10-19, T-1798).
Backend: excel-graph.
Status: VERIFIED (excel-graph)
Implemented in: `fn_mirr` in functions/financial/mirr.rs (T-1798); replaces the hand-taken limits at rate -1.
Tasks: T-1798

## Coupon settlement before the calendar
Rule: the COUP* functions, PRICE, YIELD, DURATION, MDURATION and ACCRINT are `#NUM!` for a settlement (ACCRINT: an
issue, first interest date or settlement) below serial 1 in a 1900 workbook. Probed values: -1 for every one of them
(A1, A4-A13; ACCRINT's three dates at -1 and -1E15, A14-A17); 0 for COUPNUM and PRICE (A3, A20); -0.5, 0.5 and -1E15
for COUPNUM only (A19, -b A11, A2). The other functions at 0 or a fraction follow the same check but are unprobed
(T-1844). In a 1904 workbook serial 0 is a date and computes
(`COUPPCD(0,400,2)` = -150), while -1 is `#NUM!` (probes/date-system-1904-financial A35-A41; only
`COUPDAYBS(-1,400,2)` backs the 1904 negatives, and whether -0.5 truncates to 0 is unprobed, T-1844).
Evidence: probes/fin-coupon-walk-bounds A1-A20 and -b A11 (excel-graph, 2026-10-17, T-1822).
Ours: `Model::check_settlement` (functions/financial/dates.rs) makes a settlement below 1900 serial 1, as
`financial_date` reads it, `#NUM!` for the COUP* functions, PRICE, YIELD and DURATION; ACCRINT already checked its
dates against the calendar. Before T-1822 the engine walked the coupon schedule back past 1900 and returned a number
(`COUPNUM(-1,43831,2)` was 241), and `COUPNUM(-1E15,43831,4)` stepped about 10^6 quarters back to chrono's first year
before it failed. The check keeps every schedule walk inside the calendar, at most about 32,400 quarters.
Status: VERIFIED (excel-graph)
Tasks: T-1822

## Coupon schedules at the calendar's edges
Rule: Excel is `#NUM!` (or a different count) for some valid schedules that reach the start or the end of the 1900
calendar, which the engine computes: `COUPNUM(1,43831,2)` is 240 (ours 241), `COUPNUM(1,1000,2)` and
`COUPNUM(DATE(1900,3,1),DATE(1910,3,31),4)` are `#NUM!`, `COUPPCD(1,DATE(9990,6,30),4)` is 0, every quarterly bond from
serial 1 or 1900-03-01 to a maturity in 9990-9999 is `#NUM!` for COUPNUM, PRICE, YIELD and DURATION, and
`ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)` is 0. The rule behind them is unknown.
Evidence: probes/fin-coupon-walk-bounds A21-A28, -b A1-A10, A12, A13 (excel-graph, 2026-10-17, T-1822).
Ours: computes these: `COUPNUM(1,43831,2)` 241, a number for the `#NUM!` cells (e.g. 32,362 quarterly coupons from
serial 1 to 9990, `PRICE` 404516.67); a known gap, the general rule still open.
Status: VERIFIED (excel-graph; the rule behind the cells is not found)
Tasks: T-1844

## Coupon schedule work cap
Rule: RustSheet charges a lifted COUP*, ACCRINT, DURATION, MDURATION or PRICE call for its coupon schedule walks, so one
call lifts at most 1,024 elements (COUP*, ACCRINT) or 512 (DURATION, MDURATION, PRICE):
`ROWS(COUPNUM(SEQUENCE(1025)+39000,43831,2))` and `ROWS(PRICE(SEQUENCE(513)+39000,43831,0.05,0.06,100,2))` are `#ERROR!`
where Excel lifts every element (probes/fin-argument-modes: they lift over arrays; Excel's own limit is unprobed). A
walk steps one coupon period at a time, up to about 32,400 quarters inside the calendar, about 1 ms in a release build;
each element is charged that worst case (`COUPON_WALK_WEIGHT`, 2^16 units per walk against SPEC-0011's 2^27 visit cap),
so a lifted call at the cap takes 1-2 s at the longest schedule. The scalar calls are not capped.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`COUPON_WALK_WEIGHT`, `COUPON`, `ACCRINT`, `BOND_5_OF_6`, `BOND_6_OF_7`)
Tasks: T-1822; T-1846 narrows the cap to a schedule-length budget.

## Yield and odd-coupon work cap
Rule: RustSheet charges a lifted YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE or ODDLYIELD call for its worst-case solver and
schedule work, so one call lifts at most 8 elements (YIELD, ODDFYIELD), 256 (ODDFPRICE) or 1,024 (ODDLPRICE, ODDLYIELD):
`ROWS(YIELD(SEQUENCE(9)+39000,43831,0.05,95,100,2))` is `#ERROR!` where Excel lifts every element (probes/fin-argument-modes,
fin-yield-odd-lift: they lift over arrays; Excel's own limit is unprobed). YIELD's and ODDFYIELD's kernels walk the coupon
schedule once (up to 4.4 and 6.3 ms at the calendar's longest schedule, release build), then the solver evaluates the price
at most 397 times (100 Newton steps of three prices, 97 bisection prices), each a sum over up to 32,400 quarterly coupons
(0.55 ms): about 0.23 s an element at that bound, charged 2^24 units against SPEC-0011's 2^27 visit cap (8 elements, about
1.8 s; the slowest inputs found, a price of 1E300 over 9990 years, take 54 ms an element). ODDFPRICE walks and sums its
odd first period once (up to 6.3 ms, 8 walks of `COUPON_WALK_WEIGHT`: 256 elements took 1.7 s), ODDLPRICE and ODDLYIELD
walk to the maturity once in closed form (up to 2.2 ms, 2 walks: 1,024 elements took 2.2 s). The scalar calls are not
capped. Before T-1842 YIELD and ODDFYIELD walked the schedule on every solver step (about 2.4 s a scalar call at the solver bound, 0.6 s measured); the walk
is now hoisted out of the solver, with the same values.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`YIELD_SOLVER_WEIGHT`, `YIELD`, `ODDFPRICE`, `ODDFYIELD`, `ODD_LAST`)
Tasks: T-1842; T-1869 narrows the cap to a schedule-length budget (after T-1846).

## CUMIPMT and CUMPRINC over many periods
Rule: Excel computes CUMIPMT and CUMPRINC in constant time whatever the period count, also where `(1+rate)^nper`
overflows a double: `CUMIPMT(0.01,1E308,1,1,1E308,0)` is -1E306 and `CUMIPMT(1E-300,1E300,1E300,1,1E300,0)` is
-5.819767068693264E299 (A26, A24), `CUMIPMT(1E-9,1E9,1,1,1E9,0)` is -0.5819767073296636 and the CUMPRINC -1 (A1, A2),
`CUMIPMT(0.01,1E6,100000,1,10,0)` is -10000 and its CUMPRINC 0 (A13, A14), and `CUMIPMT(1E10,100,1,1,100,0)` is
-999999999999 (A16). Its values are within a few ulps of the exact sums, so it does not round `1+rate` (a rate of 1E-300
is not lost) and does not add the periods one by one in doubles. Its form is close to `log1p`/`expm1`:
`CUMPRINC(1E10,100,1,1,100,0)` is -1.0000000000000013, exactly `-1E10/expm1(log1p(1E10))` (A17), where the exact value is
-1. An end period past nper is `#NUM!`, also below nper + 1 (`CUMIPMT(0.01,100.5,1000,1,101,0)`, A20-A22); the start
rounds up and the end down (`CUMIPMT(0.001,2000,1000,1.5,1999.7,0)` sums periods 2 to 1999, A23; CUMPRINC_CUMIPMT.xlsx
G6, G7). Lifted elements are the scalar calls (A29, A30).
Evidence: probes/fin-cumipmt-period-bound A1-A30 (excel-graph, 2026-10-18, T-1845), probes/fin-remaining-later-argument-
ranges K1-K3 (T-1822), `ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx`.
Ours: since T-1845 (functions/financial/cumulative.rs) up to 32 periods are main's per-period IPMT/PPMT sum, which matches
CUMPRINC_CUMIPMT.xlsx G3, G9, G15 and G26 where the closed form below is an ulp or two off; past 32 periods, or when a
period's `(1+rate)^nper` overflows, the closed form `pmt = -pv*r/(-expm1(-nper*L))` (over `1+r` for type 1),
`principal = pmt*((exp((end-nper)*L) - exp((first-1-nper)*L))/expm1(L))` with `L = log1p(r)` (type 1: period 1 adds the
whole payment and the sum starts at period 2), interest = `pmt*(end-start+1) - principal`. The sum is no closer to
Excel past 32 periods (83 ulps over 1,024 periods, A11; no correct digit over 10^5 periods of type 1, A5), while the
kernel passes 22 of the 30 probe cells and misses the others by at most 6 ulps, but for 48 to 64 near period 1E15 (A18,
K3). The fit tried `(1+r)^m` by libm and by Excel's `^` against `exp(m*log1p r)`, four payment and four balance
parenthesizations, and `r` against `expm1(log1p r)` as the denominator (72 cells): no single form matches both the short
sums and the long ones. Before T-1845 the engine looped over every period (10^9 steps for A1), was `#NUM!` once
`(1+rate)^nper` overflowed (A13-A19, K1-K3) and computed A21.
Status: PARTLY VERIFIED (excel-graph; the closed form's last digits open in T-1890, the short sums' in T-1374)
Tasks: T-1845, T-1890, T-1374

## CUMIPMT and CUMPRINC work cap
Rule: RustSheet charges each lifted CUMIPMT or CUMPRINC element for its kernel's bound, 2^8 units against SPEC-0011's
2^27 visit cap, so one call lifts at most 524,288 elements: `ROWS(CUMPRINC(SEQUENCE(524289)*0+0.004,1024,1000,1,32,1))`
is `#ERROR!` where Excel lifts every element (they lift over arrays: probes/fin-argument-modes H73-H87; Excel's own
limit is unprobed). The bound is 32 PPMT steps of three powers each (the per-period sum) and then the closed form when
the last step overflows, about 2.9 us an element in a release build: 2^19 elements took 1.53 s for
`CUMPRINC(..,1024,1E307*15.98,1,32,0)`, whose last step overflows, and 1.51 s for 32 full steps of type 1. Past 32
periods the kernel is the closed form (0.3 us), so the charge does not depend on the period count. The scalar calls are
not capped.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`CUMULATIVE_WEIGHT`, `CUMULATIVE`)
Tasks: T-1845

## AMORLINC and AMORDEGRC over many periods
Rule: Excel walks AMORLINC's and AMORDEGRC's periods one by one in doubles, so a long walk carries every rounding:
`AMORLINC(2400,39679,39813,300,875000,0.000001,1)` is 0.0015212859222804571, the remainder after 874,999 subtractions
of the charge 0.0024 (A6; the exact remainder, `cost - salvage - first - 874999*charge`, is 0.0015213114757168815),
the periods before charge 0.0024 (A5) and the ones after 0 (A7); at rate 0.001 period 875 charges 1.5213114753762325
(A2). A period between whole periods is the lower one (875.5, A4), and one below 1 charges a full period
(`AMORLINC(..,0.5,0.15,1)` is 360, A8, A9). AMORDEGRC's charge decays geometrically and rounds to a whole number:
`AMORDEGRC(1E15,39679,39813,1,P,0.000001,1)` is 2499997712, 2493761750, 1947004435, 716262008, 205213720 for P = 1, 1000,
1E5, 5E5, 999997, then half of what is left at the half period, 41042641307062 for P = 999998 and the same rest at
999999, then 0 (B6-B14); at 5E8 periods and rate 1E-9 it is 716262 (-b B1). At charges of 1E12 and more over 1.1E6
to 2.4E6 periods (rate 1E-7) Excel is 18989305437102, 16344246350030, 13720292047582 and 5154670166168 for
`AMORDEGRC(1E20,39679,39813,300,P,1E-7,1)` with P = 1100000, 1700000, 2400000 and `AMORDEGRC(3E19,..,1500000,..)`
(-c A1-A4): a walk in doubles gives A1, A3 and A4 but 16344246350031 for A2, so Excel's per-period arithmetic is
not exactly that walk at that size; `AMORDEGRC(1E300,..,2000000,1E-7,1)` is 1.51632679480347E293, 15 significant digits
(-c A5, as -b B2). Excel finishes 10^9 periods (`AMORLINC(2400,39679,39813,300,1E9,1E-9,1)` is 0, -b A1), but
`AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` did not finish within the excel-graph timeout: a fill of a workbook
holding only that cell failed after 8.5 minutes (createSession retries exhausted, 2026-10-18), as did, twice, a larger
one with periods of 1E12 to 1E308 at rates of 1E-13 to 1E-300; neither is in the suite. A period past the life,
`ceil(1/rate)`, is 0 at once (fin-remaining-later-argument-ranges K4, K5).
Evidence: probes/fin-amor-period-bound A1-A13, B1-B18, probes/fin-amor-period-bound-b A1-A3, B1, B2 and
probes/fin-amor-period-bound-c A1-A6 (excel-graph, 2026-10-18, T-1841).
Ours: since T-1841 (functions/financial/amortization.rs) both walks are bounded. AMORLINC keeps main's value for every
period main finished, and AMORDEGRC up to 2^20 periods. AMORLINC's walk is exact for any period: while the remainder stays in one binade, the subtraction rounds the
charge to that binade's grid the same way each time, so after two equal steps the next ones are jumped in one
multiplication, and a charge below half an ulp of the remainder stops it for ever (at most 616 single steps over every
double; `linear_charge`). AMORDEGRC walks up to 2^20 periods one by one, stopping early where the rest is fixed (below
the salvage, a charge that rounds to 0 before the half period, a remainder the charge no longer moves); past that it
finishes in closed form, `remain * exp(n * log1p(-rate))` ("AMORDEGRC past 2^20 periods"). Main's counter
`counted += 1.0` stopped moving past 2^53, so main never finished there; where Excel did not finish within the
excel-graph timeout, the kernel returns the value the walk reaches (a DEVIATION: `AMORLINC(2400,39679,39813,300,1E15,
1E-300,1)` is 2.4E-297, the charge, and `AMORDEGRC(1E300,39679,39813,300,1E15,1E-300,1)` is 3, the charge 2.5 rounded).
AMORDEGRC at a period below 1 (B17, 485 in Excel, ours 0) and at a cost of 1E308 (-b B2) are base values, T-1895.
Status: VERIFIED (excel-graph) for AMORLINC's walks and AMORDEGRC's up to 2^20 periods; DEVIATION (decision 0014) past
2^20 AMORDEGRC periods and where Excel did not finish
Tasks: T-1841, T-1895

## AMORDEGRC past 2^20 periods
Rule: past 2^20 periods RustSheet computes AMORDEGRC's decay in closed form, `remain * exp(n * log1p(-rate))` from the
state after 2^20 walked periods, instead of walking on. Excel and main walk every period, adding a rounding each time,
so the closed form drifts from the walk by up to about 1e-13 relative over 10^6 further periods: the same once rounded
for charges below about 1E11, but a larger charge can be a unit off (a review replay found one at about 3.1E11; at 1E12
or more it often is).
`AMORDEGRC(1E20,39679,39813,300,1100000,1E-7,1)` is 18989305437101 here and 18989305437102 in Excel (-c A1), and A3, A4
are a unit off the other way (13720292047583, 5154670166167); A2 is Excel's 16344246350030, where main's walk is ...031;
`AMORDEGRC(1E300,..,2000000,1E-7,1)` is 1.5163267948033552E293, Excel 1.51632679480347E293 and main
1.5163267948034761E293 (-c A5, also T-1895's 15-digit rounding). A review replay found 6 of 154 random cases (costs 1E6
to 1E20, 1.1 to 4 times 2^20 periods) a unit off main; the unit test `degressive_shortcuts_equal_the_walk_and_the_closed_form_stays_near_it`
bounds the drift at 1e-13 relative for costs up to 1E20. Main walked these in milliseconds (46 ms for the five above),
but a walk is unbounded in the period, and the 2^20 cut is the kernel's work cap ("AMORLINC and AMORDEGRC work cap").
Evidence: probes/fin-amor-period-bound-c A1-A6 (excel-graph, 2026-10-18, T-1841 review).
Status: DEVIATION (decision 0014)
Implemented in: functions/financial/amortization.rs (`DEGRESSIVE_LOOP_PERIODS`, `DegressiveWalk::closed_form`)
Tasks: T-1841, T-1895

## AMORLINC and AMORDEGRC work cap
Rule: RustSheet charges each lifted AMORLINC element 2^9 units and each AMORDEGRC element 2^18 against SPEC-0011's 2^27
visit cap, so one call lifts at most 262,144 AMORLINC or 512 AMORDEGRC elements:
`ROWS(AMORDEGRC(SEQUENCE(513)*0+2400,39679,39813,300,1,0.15,1))` is `#ERROR!` where Excel lifts every element (they lift
over arrays: probes/fin-argument-modes H19-H35; Excel's own limit is unprobed). The weights are the kernels' bounds in a
release build: AMORLINC's exact walk at most 1,024 single subtractions, its step cap (`LINEAR_STEP_CAP`; the longest
real walk takes 616, and 2^18 elements of `AMORLINC(..,9E15,1.08E-16,1)`, 613 steps each, took 1.23 s, so the cap is
about 7.9 us an element, 2.1 s per call; a walk at the cap would be `#NUM!`), AMORDEGRC's 2^20 walked periods and the closed
form (512 elements of `AMORDEGRC(1E15,..,1,5E6,1E-7,1)`, which takes no shortcut, 2.06 s). The scalar calls are not
capped beyond those bounds.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`AMORLINC_WEIGHT`, `AMORDEGRC_WEIGHT`, `AMORLINC`, `AMORDEGRC`)
Tasks: T-1841

## Coupon-bond functions: a negative basis is #NUM!
Rule: `YIELD(39448,43831,0.0575,95.04,100,2,-1E300)` (the basis read from a cell holding -1E300) is `#NUM!`. Basis -1 and a
fractional negative basis, and the other coupon-bond functions, are unprobed (the discount-security functions are
`#NUM!` at basis -1, "Discount-security functions").
Evidence: probes/fin-yield-odd-lift L5 (excel-graph, 2026-10-18, T-1842).
Ours: the bond kernels read the basis as `floor() as u32`, which saturates a negative number to 0, so YIELD computes basis
0 (0.063467439).
Status: VERIFIED (excel-graph) for the one cell
Tasks: T-1870

## Discount-security functions: settlement 0, basis and argument modes
Rule: INTRATE, PRICEDISC and YIELDDISC accept settlement serial 0 and compute with it (`INTRATE(0,43466,1000,1100)` =
0.0008403165192222404, `PRICEDISC(0,43466,0.05,100)` = -495.0138888888889, `YIELDDISC(0,43466,95,100)` =
0.00044227185222223174), whereas RECEIVED, PRICEMAT and YIELDMAT return #NUM! for settlement 0 (PRICEMAT and
YIELDMAT possibly because issue > settlement: inferred, the probe cannot tell it from a rejection of settlement 0 itself). All six: basis 5 and -1 are #NUM!, a fractional basis truncates (1.9 is 1), a text
basis "1" is coerced, a boolean basis is #VALUE!, text dates are parsed, fractional dates truncate, settlement >= maturity
is #NUM!, and a zero or negative investment, price or redemption (probed for INTRATE, PRICEDISC and YIELDDISC; RECEIVED's investment) is #NUM!, as is a zero or negative PRICEDISC discount; RECEIVED is #NUM! once 1 - discount*yearfrac = 0 (the "< 0" side is inferred, not probed) (`RECEIVED(...,1000,1)` is #NUM!, `...,0.99` is 99999.99999999991); a negative PRICEMAT/YIELDMAT rate or yield price is #NUM! (PRICEMAT and YIELDMAT accept a
zero rate; issue after settlement is #NUM!). Basis 2 is the only mode whose result differs on a one-year span
(actual/360); bases 0, 1, 3 and 4 agree there.
PRICEMAT and YIELDMAT count the settlement-to-maturity days as DSM = YEARFRAC-days(issue,maturity) - days(issue,settlement), not
a direct 30/360 count: on 2018-02-28 to 2018-03-30 basis 0 and basis 4 agree for them (A103 = A104 = 100.05774336283186,
A126 = A127 = 0.271557884707002, 32 days), whereas INTRATE, PRICEDISC, RECEIVED and YIELDDISC give different results for
the two bases (INTRATE A14 = 1.2000000000000002 for 30 days, A15 = 1.125 for 32). Ours already matches (`get_mat_factors`).
Evidence: probes/discount-security-modes (135 cells, excel-graph, 2026-10-17, T-1624).
Ours: the settlement-0 cells compute since T-1409 (YEARFRAC accepts serial 0 on basis 0; T-1757 added a unit test); all 45 remaining mismatches (including A21, A43 and A88) are last-digit misses in the T-1374 class.
Status: VERIFIED (excel-graph)
Tasks: T-1624, T-1757, T-1374

## PMT when (1+rate)^nper overflows
Rule: `PMT(100,184,439103)` is not an error, although 101^184 overflows a double: `FLOOR(PMT(100,184,439103),<#DIV/0!>)`
is `#DIV/0!` (inferred: FLOOR returns its first argument's error first). The value is unprobed (the limit is -pv*rate =
-43910300).
Evidence: gen-m9/3b8e9078261b5bd2 Calc!A34 (excel-graph, 2026-10-12, T-1289).
Ours: `#NUM!`.
Status: UNVERIFIED (inferred from one cell; T-1405 probes the value and the FLOOR error order)
Tasks: T-1405

## Analysis ToolPak securities functions check frequency and dates first
Rule: COUPDAYSNC with an invalid frequency is `#NUM!` even when its basis argument is an error:
`COUPDAYSNC(37342,46362,-0.0001,<#DIV/0!>)` is `#NUM!`. TBILLEQ with a negative maturity serial is `#NUM!` even when the
discount is a TEXTSPLIT result (text or an error): `TBILLEQ(38887,1e-15-2,TEXTSPLIT("a*",A1:D2))` is `#NUM!`
(gen-m9/4df1b22d3ed63f58 Calc!A20); that dates come before the discount is inferred from this one cell. The general order (which argument errors still win) is unprobed.
Evidence: gen-m9/0c7fc447bc2eebfd Calc!A24, gen-m9/4df1b22d3ed63f58 Calc!A20 (excel-graph, 2026-10-12, T-1289).
Ours: the basis error (`#DIV/0!`) and `#VALUE!` for the text discount.
Status: VERIFIED (web) for these two shapes; the general order UNVERIFIED
Tasks: T-1406

## ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule

ODDFPRICE and ODDFYIELD are #NUM! unless the first coupon is one of the maturity's coupon dates (stepping back from
the maturity by whole periods, with the month-end rule when the maturity is the last day of a month), strictly before
the maturity. They are also #NUM! when the first period is exactly one regular period long *as the basis counts it*:
when DFC, the issue-to-first-coupon day count, equals E, the length of the coupon period (360/frequency on bases 0, 2
and 4, 365/frequency on basis 3, the actual quasi-coupon period on basis 1). Excel prices DFC < E as a short first
period and DFC > E as a long one, and has no case for equality. Examples (1900, semiannual, basis 0):
- `ODDFPRICE(DATE(2024,2,1),DATE(2025,7,2),DATE(2024,1,1),DATE(2024,7,1),0.05,0.06,100,2,0)` is #NUM!: the maturity
  is a day off the schedule. Maturity 2025-06-30 with first coupon 2024-07-01 is #NUM! too, as are basis 1 and
  quarterly twins.
- Maturity 2025-02-28 with first coupon 2024-08-31 gives 98.9344 (month-end rule); maturity 2025-02-27 is #NUM!.
- First coupon equal to the maturity is #NUM!.
- Issue 2024-01-01, first coupon 2024-07-01, maturity 2025-07-01 is #NUM! (a regular first period), for bases 0, 1
  and 4, quarterly as well, and for issue 2024-01-15 / first coupon 2024-07-15. Issue 2023-07-01 (two periods) gives
  98.5951, issue 2024-01-02 98.6562 and issue 2023-12-31 98.6556.
- An issue a day *after* the quasi-coupon date can still be one 30/360 period: issue 2023-07-01, first coupon
  2023-12-31, maturity 2024-06-30, settlement 2023-08-01 is #NUM! on basis 0 (days360 counts 180) but 99.1193 on
  basis 1 (183 of 184 days) and 99.1185 on basis 4 (179). Likewise issue 2023-03-01 → first coupon 2023-08-31,
  issue 2023-08-01 → 2024-01-31, and quarterly 2023-10-01 → 2023-12-31 (90). Annual 2022-07-01 → 2023-06-30 counts
  359 and gives 98.2306. The same shape at the end of the calendar (9998-07-01 → 9998-12-31, maturity 9999-06-30)
  is #NUM! for the same reason, not because of the calendar (probes/date-system-1900-odd-accrint-bound C5, C6).
- Basis 2 counts actual days against 180: issue 2023-07-04 (180 days to 2023-12-31) is #NUM!, and the quasi-coupon
  date 2023-06-30 itself (184 days) is a value (96.2098). Basis 3 (E = 182.5) gives a value for the quasi-coupon date
  too (96.2158).
- On basis 1, E comes from the first coupon's own schedule: with first coupon 2023-11-30 (a month end) and maturity
  2024-05-30, issue 2023-05-31 is #NUM! (183 days, the period 2023-05-31 to 2023-11-30), as on basis 0.
ODDFYIELD follows the same checks. Unprobed: basis 3 annual with a 365-day first period (the rule predicts #NUM!).
Evidence: probes/date-system-1900-oddf-schedule, probes/date-system-1900-oddf-issue-boundary (excel-graph,
2026-10-13), probes/date-system-1900-oddf-issue-after-quasi (51 cells, excel-graph, 2026-10-13, T-1490).
Status: VERIFIED. Tasks: T-1457, T-1490.
Implemented in: `check_odd_first_coupon` (the schedule) and `compute_oddfprice` (DFC = E)
(functions/financial/financial_bonds.rs).

## ODDLPRICE and ODDLYIELD: a quasi-coupon date past the calendar is #NUM!

ODDLPRICE and ODDLYIELD are #NUM! when a quasi-coupon date of the odd last period, including the one after the
maturity, is past 9999-12-31: `ODDLPRICE(DATE(9999,8,1),DATE(9999,12,31),DATE(9999,5,15),0.05,0.06,100,2,0)` (the
next quasi-coupon date after 9999-11-15 would be 10000-05-15), on bases 0 and 1.
Evidence: probes/date-system-1900-odd-accrint-bound D7-D10 (excel-graph, 2026-10-12).
Status: VERIFIED. Tasks: T-1447, T-1457.
Implemented in: `odd_l_func` (functions/financial/financial_bonds.rs).

## ACCRINT with calc_method TRUE: a next coupon past the calendar is #NUM!
Rule: with calc_method TRUE and the settlement after first_interest, ACCRINT is #NUM! when the coupon date that ends
the settlement's period (first_interest stepped forward by 12/frequency months until it reaches the settlement) is
past 9999-12-31, on bases 0 and 1 and frequencies 1, 2 and 4, in both date systems (bases 2-4 unprobed; the engine
applies it to every basis, UNVERIFIED). With issue 2956276 (9994-01-02) and
first_interest 2956456 (9994-07-01; coupons on 1 January and 1 July), `ACCRINT(2956276,2956456,2958283,0.05,1000,2,0,TRUE)`
(settlement 9999-07-02, next coupon 10000-01-01) is #NUM!, and a settlement on 9999-07-01 itself (2958282) is a value
(274.86). With first_interest 9994-06-29 a settlement in November 9999 still accrues (next coupon 9999-12-29), one on
9999-12-30 does not. With the 9994-07-01 first_interest, a settlement on 9999-09-02 is #NUM! with frequency 1
(next coupon 10000-07-01) and accrues with frequency 4 (next coupon 9999-10-01). calc_method FALSE does not walk the schedule and is not affected (a settlement on 9999-12-30 accrues).
Evidence: probes/date-system-1900-accrint-settlement-bound A23:A53 and its 1904 twin
date-system-1904-accrint-settlement-bound (the same cells, every date 2 * 1462 serials lower; excel-graph,
2026-10-13, T-1494).
Ours: `compute_accrint` (functions/financial/accrint.rs).
Status: VERIFIED (excel-graph). Tasks: T-1494.

## COUPDAYS basis 1 around 29 February
Rule: COUPDAYS on basis 1 is the length of the coupon period around settlement, except that when that period (as
COUPPCD/COUPNCD give it, with the end-of-month rule) starts or ends on a 29 February that is not the maturity, it is
the length of the period of the schedule walked back from the maturity *without* the end-of-month rule (each date
the one after it moved back a period, its day clamped to the month). So with maturity 2025-02-28 the dates fall on the
28th: `COUPDAYS(DATE(2023,11,15),DATE(2025,2,28),2,1)` is 184 (2023-08-28 to 2024-02-28) while COUPPCD, COUPNCD,
COUPDAYBS (76) and COUPDAYSNC (106) describe 2023-08-31 to 2024-02-29 (182 days); settlement 2024-03-15 gives 182
(2024-02-28 to 2024-08-28, not 184), settlement 2024-02-28 gives 182 (2024-02-28 is then the period's start) and
2024-02-27 gives 184. A maturity of 2025-08-31 (frequency 2) and of 2025-05-31 (frequency 4) behave the same, since
the walk passes 2025-02-28: 184 and 92 for settlements 2023-11-15 and 2023-12-15. Frequency 1, maturity 2025-02-28,
settlement 2023-06-15: 365, not 366. A period ending on 28 February of a common year keeps the end-of-month rule
(2099-08-31 to 2100-02-28, 181), and so does one ending on a 29 February that is the maturity
(`COUPDAYS(DATE(2399,11,15),DATE(2400,2,29),2,1)` is 182). Only COUPDAYS shows it: PRICE and DURATION on basis 1 use the
182-day period, and bases 0, 2, 3 and 4 have fixed lengths. Unprobed: a walk that lands on a 29 February itself (maturity 2028-08-31, settlement 2027-11-15; the rule predicts 184, 2027-08-29 to 2028-02-29).
Evidence: probes/coupdays-feb-month-end (296 cells, excel-graph, 2026-10-13, T-1493) and
probes/date-system-1900-financial-readers E15.
Ours: `coupdays_actual` (functions/financial/financial_bonds/coupon_schedule.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1489, T-1493

## YIELD stops short of PRICE's root
Rule: YIELD's last digits are not the exact root of PRICE: Excel stops early, below the root, by up to about 1e-9
relative (`YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,100,2,0)` is 0.05523587030251456; the root, as we compute
it, is 0.0552358703070220). Some price, rate and redemption values reach the root (price 101: 0.0474016398691), others stop
1e-11 to 1e-9 short. The same bond moved by whole four-year blocks (1 March of 2000, 2004, ..., 2040 and 2096, maturity
30 June four years later, basis 0) gives the same bits, but two bonds with equal coupon count, accrued days and period
length need not: `YIELD(DATE(2096,2,15),DATE(2100,2,28),0.05,98,100,2,0)` is 0.05558592876555125 and
`YIELD(DATE(2100,2,16),DATE(2104,3,1),0.05,98,100,2,0)` is 0.05558592884303248 (both 9 coupons, 165 of 180 days
accrued); their YEARFRAC differs (4.0361 against 4.0417), so the starting point may come from the dates.
Evidence: probes/yield-solver-sweep (66 cells), probes/date-system-1900-financial-readers row 8 and its -shifted twin
(excel-graph, 2026-10-13, T-1493); probes/solver-iteration-traces Y!C:E (24 cells: prices 40 to 160 over ten years
at frequency 2, 4 and 1; excel-graph, 2026-10-14, T-1525). `uv run rustsheet-fitlab yield` scores the 90 basis-0
cells with more than one coupon left: Newton on PRICE from 0, the coupon rate or 0.1 (step tolerances 1e-6 to
1e-12), the secant from 0 and 1, and LibreOffice's regula falsi reach at most 2/90 bit-exact (5/90 for their
union); the engine gets 0/90. With one coupon period left YIELD is closed form (see "YIELD with one coupon period
left is closed form").
T-1583: on all 90 cells Excel's value is either PRICE's root (|PRICE(y) - pr| below 1e-11, 45 cells) or
below it (45 cells) with a residual from 1e-11 up to 5e-8 (yield-solver-sweep S!F1 2.1e-8, solver-iteration-traces
Y!D5 5.0e-8), which is what Newton on the convex, decreasing PRICE from below would leave if it stopped on a
residual test of about 5e-8 to 1e-7 and returned the iterate without its last step (a returned post-step
Newton iterate would be at the root). Stepping analytic Newton backwards from the below-root values gives a
shared earlier iterate that does not depend on frequency or redemption: 0.0552344627 for both S!A1
(frequency 2) and S!F1 (frequency 1), within 2e-9 of it for S!E10 (redemption 100.5), and about 0.133352 for
Y!C2, D2 and E2 (price 50, frequencies 2, 4 and 1). It is not 0, 0.1, the coupon rate, the current yield,
(C + (100 - P)/n)/((100 + P)/2) or an annual-coupon IRR. Newton with that residual stop from 0, the coupon rate
or the current yield (lab family "Newton, residual") reaches at most 3/90 bit-exact.
T-1592 (lab `newton_back_chain`): the shared iterate is 0.05523446268 for S!A1 and 0.05523446267 for S!F1,
the same to 1.1e-11 although the two bonds have different frequencies and coupon counts, and about 1.4e-6 below
S!A1's root. S!E10 (redemption 100.5, root 0.05628) reaches 0.0552344604 two steps back; three steps back,
Y!C2, D2 and E2 (price 50) reach 0.1333515 to 0.1333516, and Y!C3 and E3 (price 60) reach 0.1122589 and 0.1122579. So the start looks
like an iterate of an earlier phase that ends close to the redemption-100 root, not a closed form of the inputs.
Rejected as that start: Newton with the residual stop from the approximate yield
(C + (red - P)/T)/((red + P)/2), T = coupons left / f, 1/90 bit-exact and 1/90 within 4 ulp (lab fit). Rejected
analytically, not as lab fits (so with no per-cell counts): solving for the T that makes each
of these families give the shared iterates of S!A1 (4.33 years), Y!C2:E2 (price 50) and Y!C3, E3 (price 60,
both 10.33 years) gives no consistent T: annual or semi-annual annuities over a continuous T, continuous
compounding, and the approximate yield over (red + P)/2, P or (red + 2P)/3.
Ours: Newton from the coupon rate to 1e-8, which lands on the root; the stopping rule is unfitted.
Status: UNVERIFIED (the algorithm); the values are excel-graph
Tasks: T-1374, T-1493, T-1525, T-1533, T-1583, T-1592

## TBILLEQ and TBILLYIELD
Rule: Excel's TBILLYIELD is `(100 - pr)/pr * (360/DSM)`, evaluated in that order; the textbook
`(100 - pr)*360/(pr*DSM)` is 1 ulp off (`TBILLYIELD(45366,45731,2000)` is -0.9369863013698629, not
-0.936986301369863). TBILLEQ keeps `365*rate/(360 - rate*DSM)` up to 182 days; from 183 days it is
the root of the semi-annual equivalence in this form: `price = 100 - rate*DSM*100/360`, `t = DSM/Y`
(Y = 366 when DSM is 366, else 365), `(-t + sqrt(t*t - (2*t - 1)*(1 - 100/price)))/(t - 0.5)`.
Solving the same equation as a general quadratic `a y² + b y + c` is 1-2 ulps off, and up to
~7e-15 relative at 183 days (`TBILLEQ(2040,2223,0.8)` is 1.364497682211261). Writing the price as
`100*(1 - rate*DSM/360)` matches 5 of the 7 fitted cells but misses `rate` 0.1 and 1 at 318 days.
At the error boundary: a negative price whose square-root argument is negative is `#NUM!` (D29, D31),
and so is a price of exactly 0 from 183 days (`TBILLEQ(1000,1200,1.8)`, `TBILLEQ(1000,1360,1)`). Every
negative price is `#NUM!` in TBILLEQ, on both branches: `TBILLEQ(1000,1100,4)`, `TBILLEQ(1000,1182,2)`,
`TBILLEQ(1000,1183,2)` and `TBILLEQ(45332,45650,60)` (probes/tbilleq-nonpositive-price, excel-graph,
2026-10-14, T-1540). But up to 182 days a zero denominator gives **0**, not an error:
`TBILLEQ(1000,1100,3.6)` is 0 (probes/tbilleq-zero-price). TBILLPRICE gives 0 at price 0
(`TBILLPRICE(1000,1100,3.6)`) and `#NUM!` for every negative price (the four cases above).
Excel for the web also fails the oracle canary when that zero-price TBILLEQ is the only formula in a
workbook (the canary cell `SQRT(2)` comes back `#DIV/0!`); next to other formulas it computes normally.
Equal dates: TBILLEQ, TBILLPRICE and TBILLYIELD all give `#NUM!` when settlement >= maturity, so zero days never
reaches the division (`TBILLYIELD(45366,45366,98)`, `TBILLEQ(45366,45366,0.05)`, `TBILLPRICE(45366,45366,0.05)`,
and `TBILLYIELD(45366,45366,100)`, `TBILLEQ(45366,45366,0)`; one day later is a number: probes/tbill-equal-dates,
excel-graph, 2026-10-14, T-1539).
Evidence: ironcalc-calc/FINANCIAL/TBILLs.xlsx D2:D31, R2:R31 and L5:L26 (Excel-cached, Microsoft
Macintosh Excel; all 273 cells bit-exact, T-1475); the same formulation is bit-exact on the excel-graph
probes fin-argument-modes H313, date-system-1904-financial A29:A30 and -shift A32.
Status: VERIFIED (desktop-cached)
Tasks: T-1475

## T-1625 probes: VDB, AMORDEGRC, FVSCHEDULE, MDURATION, RRI
- `VDB(2400,3000,10,0,1)` (salvage above cost) is -600, not -60; `VDB(2400,300,10,9.5,10,2,FALSE)` is 0 (the
  last half period after the book value reaches salvage).
- AMORDEGRC with rate 0.20 or 0.25 (first period 1.5 years after purchase) returns a number (1437, 1346), not #NUM!. Observed thresholds: rate 0.3 gives 1616, rates 0.15 and 0.25 both give 1346, rate 0.5 (life 2) is #NUM!.
- FVSCHEDULE: any text, boolean or text-cell element in the schedule, a quoted-number element
  `{"0.1"}`, `TRUE` as principal and `""` as schedule are all `#VALUE!` (nothing is skipped).
- FVSCHEDULE: blank cells in the schedule are harmless (`B5:B6` = 1000) and a text-number principal is converted (`"1000"` -> 1100), but a boolean principal is `#VALUE!`.
- VDB differs from us by a few ulp (A6, A10, A13) and ~4e-13 (A7); Excel's result for the same inputs depends on no_switch (A6 FALSE = 203.31648000000007, A7 TRUE = 203.3164800000004).
- MDURATION and RRI(TRUE principal) differ from us only in the last ulp.
Evidence: probes/fin-vdb-modes, fin-amordegrc, fin-fvschedule, fin-mduration, fin-rri (excel-graph, 2026-10-17, T-1625).
Status: VERIFIED (excel-graph)
Implemented (T-1764): VDB with salvage above cost never switches to straight line (-600); a fractional end period
prorates the unclipped declining-balance charge and the total is capped at `cost - salvage` (A9 is 0; salvage <= cost only, salvage above cost keeps the clipped charge); AMORDEGRC
rejects only an asset life (1/rate) in [0, 3] or strictly between 4 and 5 (exactly 4 and 5 are valid, coefficients
1.5 and 2). The salvage-above-cost rule (A17) and the cap (A9) are each inferred from one probe cell. The AMORDEGRC bands at life 3 and strictly between 4 and 5 are documentation-only. Probes needed: T-1978. Still open: the ulp differences (VDB A6, A7, A10, A13, MDURATION, RRI), see the T-1764 Outcome.
Implemented (T-1765): a typed boolean principal is `#VALUE!` (A18); a text-number principal still converts. A boolean principal read through a reference is `#VALUE!` by inference, not probed (T-1992).
Tasks: T-1764 (incl. MDURATION, RRI ulp), T-1765

## NPV and FVSCHEDULE arguments
Rule: NPV reads an array literal whole (`NPV(0.1,{-10000,2750,4250,3250,2750})` is 302.233453999042) and skips
text, booleans and empty elements inside it (`{-10000,"2750",4250}`, `{-10000,TRUE,4250}`, `{-10000,"x"}`,
`{-10000,4250,""}` all ignore the element). A typed argument is coerced (`NPV(0.1,-10000,"2750",TRUE,4250)` is
-3914.6233180793665) and typed text that is no number is `#VALUE!` (`NPV(0.1,-10000,"x",4250)`). An argument
is read the way SUM reads one (`evaluate_reference_arg`): when it reads as a reference, the text and booleans of
its cells are skipped, however the reference is written: a range or single cell (`NPV(A5,B5,..)` with TRUE in B5;
ironcalc-calc FINANCIAL/NPV.xlsx K5, L5), in parentheses, after `@`, `B1:B1`, `IF(TRUE,B1)`, `IF(TRUE,B1:B2)`,
`INDEX(B1:B2,1)`, `CHOOSE(1,B1)`, `OFFSET(B1,0,0)`, `INDIRECT("B1")`, `LET(x,B1,x)`, a name `=Sheet1!$B$1`, a 3-D
`Sheet1:Sheet1!B1` (all -5578.512396694214 with "2750" or TRUE in B1/B2). A union is its areas as written:
`NPV(0.1,(B3,B4))` is -5578.512396694214 and `(B4,B3)` -4400.82644628099. Anything else is typed and coerces: a
function value (`T("2750")`, `"2750"&""`, `IF(TRUE,"2750")`, `VLOOKUP("2750",B1:B2,1,FALSE)` with a literal column
index), and a name whose body is a text or boolean constant (-3625.0939143501128, -5896.994740796395). A computed
array is read like a literal: `NPV(0.1,SEQUENCE(3))` is 4.815927873779113, `B3:B4*1` -5578.512396694214, and
`NPV(0.1,{1,#N/A})` is `#N/A`. A written empty argument is a cash flow of 0 (`NPV(0.1,-10000,,4250)` is
-5897.821187077386), and errors and typed text count left to right (`NPV(0.1,"x",#N/A)` is `#VALUE!`,
`NPV(0.1,#N/A,"x")` `#N/A`).
FVSCHEDULE is `#VALUE!` when the schedule holds text or a boolean, in a range (`N1:N2` with `"0.11"`, `C1:C2` with
TRUE) or in an array literal (`{0.09,"0.11"}`); typed text that reads as a number is a one-rate schedule
(`FVSCHEDULE(1000,"0.1")`, `("0.1")` and `T("0.1")` are 1100) while `"x"` and `TRUE` are `#VALUE!`.
Typed empty text is `#VALUE!` (`NPV(0.1,-10000,"",4250)`): `""` is no number. An argument that evaluates to an empty value (`IF(TRUE,)`, `LET(x,,x)`) is a cash flow
of 0, not skipped (`NPV(0.1,-10000,IF(TRUE,),4250)` is -5897.821187077386, equal to the written-empty form;
adding a trailing 2750 gives -4019.5341848234416). FVSCHEDULE walks the schedule left to right and returns the
first of text or error it meets in an array literal (`{"x",#N/A}` is `#VALUE!`, `{#N/A,"x"}` is `#N/A`); in a
range (`B1:B2` holding "x" then #N/A) the text wins as well (`#VALUE!`), and `#N/A` then "x" gives `#N/A`.
Evidence: probes/npv-fvschedule-unprobed A1-A8 (excel-graph, T-1878).
Evidence: probes/fin-argument-modes H368, H370, H371, H382-H384; probes/npv-fvschedule-typed A1-A16;
probes/npv-reference-forms A1-A20; probes/npv-fvschedule-edges A1-A6;
probes/fin-fvschedule A3, A4, A13, A14, A16, A19 (excel-graph, T-1084, T-1373, T-1625).
Backend: excel-graph.
Status: VERIFIED (excel-graph)
Implemented in: `fn_npv`, `fn_fvschedule` in functions/financial/mod.rs (T-1373)
Tasks: T-1084, T-1373
