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
- MIRR: 32 of 35 cells already match the engine. `MIRR({-5,4},fr,rr)` is -0.19999999999999996 for
  every rate pair.
Engine today (after T-0916): 312 of 691 cells pass (IRR 161/216, RATE 47/368, XIRR 72/72, MIRR 32/35;
before: 96, with IRR 19 and XIRR 9).
Status: VERIFIED (observations). XIRR's and IRR's iterations are fitted (next two sections, T-1525,
T-1533) and implemented (T-0916); RATE's update rule and stopping test are UNVERIFIED (partial findings
below); MIRR's last digits are T-1687
Implemented in: XIRR and IRR in functions/financial/financial_util.rs (T-0916, next two sections); RATE n/a

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
Tasks: T-1525, T-0916, T-1533

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
Ours: the engine solves PRICE for YIELD (`fn_yield`, functions/financial/financial_bonds.rs) and gets 0/60.
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
The zero-first-value result and the date-0-first root (the engine does not converge there, `#NUM!`)
are open (T-1155).
Tasks: T-1114, T-1124

## Financial function argument modes
Rule: every scalar argument of the M9 financial functions lifts elementwise over an array literal: all 256
`ROWS(F(..{v;v}..))` cells are 2, the dates, frequency, basis, type and guess arguments included. Over a two-cell
range the core worksheet functions lift as well (DB, DDB, FV, IPMT, ISPMT, MIRR, NPER, NPV's rate, PDURATION, PMT,
PPMT, PV, RATE, RRI, SLN, SYD, VDB and IRR's guess: `ROWS(F(G1:G2,..))` is 2). The Analysis ToolPak functions do not:
ACCRINT, ACCRINTM, AMORDEGRC, AMORLINC, the COUP* family, CUMIPMT, CUMPRINC, DISC, DOLLARDE, DOLLARFR, DURATION,
EFFECT, FVSCHEDULE's principal, INTRATE, MDURATION, NOMINAL, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, PRICE,
PRICEDISC, PRICEMAT, RECEIVED, TBILLEQ, TBILLPRICE, TBILLYIELD, XIRR's guess, XNPV's rate, YIELD, YIELDDISC and YIELDMAT
are `#VALUE!` when their first scalar argument is a multi-cell range. Only the first scalar was probed over a range (XIRR's guess
and XNPV's rate are the first scalars there). How the later scalars behave over a range is unverified.
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
Evidence: probes/fin-argument-modes (386 cells, excel-graph, 2026-10-11, T-1084). The cash flow is K1:K5 and its
dates are M1:M5. L1:L8 is the same flow with typed text, a boolean, text and a blank mixed in, and N1:N5 is a
schedule containing text.
Ours: no scalar argument lifts (`#N/IMPL!` for an array literal, `#VALUE!` for a range; T-1372). NPV of an array
literal is 0, and typed text and booleans are not coerced (T-1373). FVSCHEDULE ignores text in the schedule (T-1373).
The base values of CUMIPMT, DB, DDB, DURATION, INTRATE, MDURATION, ODDFPRICE, PRICE and YIELDDISC are 1 to a few
ulps off, CUMPRINC and EFFECT about 30-40 ulps, and those of ODDFYIELD and YIELD are off at 1e-10 to 1e-12 relative (T-1374; TBILLEQ and TBILLYIELD are exact since T-1475). The
annuity, NPV and solver misses are T-0901's and T-0916's.
Status: VERIFIED (excel-graph)
Tasks: T-1084, T-1372, T-1373, T-1374

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
Ours: the settlement-0 cells return #NUM! (T-1757); the other mismatches (42 of 45) are last-digit misses in the T-1374 class.
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
- AMORDEGRC with rate 0.20 or 0.25 over a 1.5 year life returns a number (1437, 1346), not #NUM!.
- FVSCHEDULE: any text, boolean or text-cell element in the schedule, a quoted-number element
  `{"0.1"}`, `TRUE` as principal and `""` as schedule are all `#VALUE!` (nothing is skipped).
- MDURATION and RRI(TRUE principal) differ from us only in the last ulp.
Evidence: probes/fin-vdb-modes, fin-amordegrc, fin-fvschedule, fin-mduration, fin-rri (excel-graph, 2026-10-17, T-1625).
Status: VERIFIED (excel-graph)
Tasks: T-1764 (incl. MDURATION, RRI ulp), T-1765
