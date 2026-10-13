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
Engine today: 96 of 691 cells pass (IRR 19/216, RATE 36/368, XIRR 9/72, MIRR 32/35). RATE rejects
its sixth argument (guess) with `#ERROR!` (T-1050).
Status: VERIFIED (observations); the iteration (update rule, stopping test, cap) is UNVERIFIED (T-0916)
Implemented in: n/a (T-0916)


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
ulps off, CUMPRINC and EFFECT about 30-40 ulps, and those of ODDFYIELD, TBILLEQ and YIELD are off at 1e-10 to 1e-12 relative (T-1374). The
annuity, NPV and solver misses are T-0901's and T-0916's.
Status: VERIFIED (excel-graph)
Tasks: T-1084, T-1372, T-1373, T-1374

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
the maturity, and the issue is not the quasi-coupon date one period before the first coupon. An issue two or more
periods before is fine, and so is an issue a day off the quasi-coupon date. Examples (1900, semiannual, basis 0):
- `ODDFPRICE(DATE(2024,2,1),DATE(2025,7,2),DATE(2024,1,1),DATE(2024,7,1),0.05,0.06,100,2,0)` is #NUM!: the maturity
  is a day off the schedule. Maturity 2025-06-30 with first coupon 2024-07-01 is #NUM! too, as are basis 1 and
  quarterly twins.
- Maturity 2025-02-28 with first coupon 2024-08-31 gives 98.9344 (month-end rule); maturity 2025-02-27 is #NUM!.
- First coupon equal to the maturity is #NUM!.
- Issue 2024-01-01, first coupon 2024-07-01, maturity 2025-07-01 is #NUM! (a regular first period), for bases 0, 1
  and 4, quarterly as well, and for issue 2024-01-15 / first coupon 2024-07-15. Issue 2023-07-01 (two periods) gives
  98.5951, issue 2024-01-02 98.6562 and issue 2023-12-31 98.6556.
ODDFYIELD follows the same checks. Unexplained: ODDFPRICE/ODDFYIELD with first coupon 9998-12-31 and maturity
9999-06-30 (issue 9998-07-01, settlement 9998-08-01) are #NUM! in 1900 although the first coupon is on the schedule
(probes/date-system-1900-odd-accrint-bound C5, C6; T-1490).
Evidence: probes/date-system-1900-oddf-schedule, probes/date-system-1900-oddf-issue-boundary (excel-graph,
2026-10-13).
Status: VERIFIED (the C5/C6 cells above UNVERIFIED, T-1490). Tasks: T-1457.
Implemented in: `check_odd_first_coupon` (functions/financial/financial_bonds.rs).

## ODDLPRICE and ODDLYIELD: a quasi-coupon date past the calendar is #NUM!

ODDLPRICE and ODDLYIELD are #NUM! when a quasi-coupon date of the odd last period, including the one after the
maturity, is past 9999-12-31: `ODDLPRICE(DATE(9999,8,1),DATE(9999,12,31),DATE(9999,5,15),0.05,0.06,100,2,0)` (the
next quasi-coupon date after 9999-11-15 would be 10000-05-15), on bases 0 and 1.
Evidence: probes/date-system-1900-odd-accrint-bound D7-D10 (excel-graph, 2026-10-12).
Status: VERIFIED. Tasks: T-1447, T-1457.
Implemented in: `odd_l_func` (functions/financial/financial_bonds.rs).

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
182-day period, and bases 0, 2, 3 and 4 have fixed lengths.
Evidence: probes/coupdays-feb-month-end (296 cells, excel-graph, 2026-10-13, T-1493) and
probes/date-system-1900-financial-readers E15.
Ours: `coupdays_actual` (functions/financial/financial_bonds.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1489, T-1493

## YIELD stops short of PRICE's root
Rule: YIELD's last digits are not the exact root of PRICE: Excel stops early, below the root, by up to about 1e-9
relative (`YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,100,2,0)` is 0.05523587030251456; the root is
0.0552358703070220). Some price, rate and redemption values reach the root (price 101: 0.0474016398691), others stop
1e-11 to 1e-9 short. The same bond moved by whole four-year blocks (1 March of 2000, 2004, ..., 2040 and 2096, maturity
30 June four years later, basis 0) gives the same bits, but two bonds with equal coupon count, accrued days and period
length need not: `YIELD(DATE(2096,2,15),DATE(2100,2,28),0.05,98,100,2,0)` is 0.05558592876555125 and
`YIELD(DATE(2100,2,16),DATE(2104,3,1),0.05,98,100,2,0)` is 0.05558592884303248 (both 9 coupons, 165 of 180 days
accrued); their YEARFRAC differs (4.0361 against 4.0417), so the starting point may come from the dates.
Evidence: probes/yield-solver-sweep (66 cells), probes/date-system-1900-financial-readers row 8 and its -shifted twin
(excel-graph, 2026-10-13, T-1493).
Ours: Newton from the coupon rate to 1e-8, which lands on the root; the stopping rule is unfitted (T-1374).
Status: UNVERIFIED (the algorithm); the values are excel-graph
Tasks: T-1374, T-1493
