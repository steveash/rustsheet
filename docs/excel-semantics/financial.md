# Financial functions

## Annuity evaluation order and `(1+r)^n` (PV/FV/PMT/NPER/IPMT/PPMT)
Rule: with `P = (1+r)^n` by right-to-left square-and-multiply (numbers.md, "Integer exponents"),
`k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, Excel computes PV = `-(fv+T)/P` and FV = `-(pv*P+T)`
(integer nper, rate ≠ 0). PMT/IPMT/PPMT/NPER/NPV are still unexplained.
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
Status: PARTLY VERIFIED (PV/FV formulation fitted on the sweep; PMT/IPMT/PPMT/NPER/NPV open; T-0901)
Implemented in: n/a (T-0901). Salvaged from draft PR #13 by T-0031.

## IRR, RATE and XIRR stop early, and the guess and flow scaling change the result
Rule: Excel's iterative solvers (IRR, RATE, XIRR) return an un-converged iterate, not the root
rounded to double. The digits beyond the stopping tolerance depend on the starting guess (default
0.1), and for IRR also on the scale and sign of the cash flows. XIRR stops much earlier (about 1e-8
relative) than IRR and RATE (about 1e-12). Some guesses make Excel return `#NUM!` where another
guess converges. MIRR is closed form and is not affected.
Evidence (oracle: excel-graph, 2026-10-06, T-1009; `probes-numeric/m9-fin-irr-rate`, 691 cells;
labels in the case's `hypotheses`):
- The default guess is 0.1. Omitting the guess gives the same bits as passing 0.1 for every IRR,
  RATE and XIRR row in the sweep, for example `IRR({-20,5,5,5,5,5})` and `IRR(…,0.1)` both give
  0.07930826116039724, which matches the cached IRR.xlsx General!L2.
- IRR depends on the guess beyond 1e-12. For `{-20,5,5,5,5,5}`, guesses -0.99 … 100 give 0.079308261157…
  to 0.079308261165…, for example guess 17 gives 0.07930826116414913 (= IRR.xlsx General!N2) and guess
  0.0793, the nearest to the root, gives one of the furthest results (0.07930826114474532).
- IRR depends on the cash-flow scale and sign. Multiplying the flows by 1E6 or -1 changes the last
  digits in 11 of 20 families (`irr/flow=A/scale=1E6` 0.07930826116052869, `scale=-1`
  0.07930826116045497, against 0.07930826116039724). Multiplying by 1E-6 changes only 6 of 20 (E, F, J,
  K, N, P), so the stopping test is not purely absolute or purely relative on the NPV.
- An IRR whose root is exactly 0.1 (`{-100,110}`, `{-100,0,121}`, `{-100,10,10,10,10,110}`, leading
  or trailing zero flows) returns 0.10000000000000009, which is 0.1 plus 1 ULP. With guess 10 several
  of them return 0.09999999999999987 instead. A root of exactly 0 (`{-100,50,50}`) gives
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
  n ≥ 10 (and r=0.1, n=360, fv=-500, type=1). The values it does return differ from r by up to 1e-11
  relative (r=0.2, n=3: 0.19999999999843024).
- RATE does not snap a zero rate. `RATE(10,-100,1000)` gives -1.963351707292562e-10 and
  `RATE(10,-1E-6,1E-5)` gives -2.3981867563911566e-09, but `RATE(10,-1E6,1E7)` gives `#NUM!`, so
  RATE's stopping test also depends on scale. `RATE(1,0,-100,110)` gives exactly 0.1.
- XIRR is scale- and sign-invariant. All four scalings give identical bits for each of the 8 families.
  It is converged only to about 1e-8: `XIRR({-1000,1100},{45000,45365})` (exactly 10% a year) gives
  0.09999999403953552 (= 3355443/2^25), and with guesses 0, 1 and 10 it gives 0.0999999951171875,
  0.09999999776482582 and 0.1000000024214387. The cached XIRR.xlsx R2 (1.1859161972999572) is the
  default-guess value.
- XIRR guess failures: guess -0.5 gives `#NUM!` for positive-rate flows. Guesses 1 and 10 give `#NUM!` for
  the negative-rate flows (`flow=C`, `D`, `Byr`). XIRR with day-serial dates 1..6 (`flow=B`) gives
  `#NUM!` for every guess.
- MIRR: 32 of 35 cells already match the engine. `MIRR({-5,4},fr,rr)` is -0.19999999999999996 for
  every rate pair.
Engine today: 96 of 691 cells pass (IRR 19/216, RATE 36/368, XIRR 9/72, MIRR 32/35). RATE rejects
its sixth argument (guess) with `#ERROR!` (T-1050).
Status: VERIFIED (observations); the iteration (update rule, stopping test, cap) is UNVERIFIED (T-0916)
Implemented in: n/a (T-0916)
