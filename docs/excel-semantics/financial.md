# Financial functions

## Annuity evaluation order and `(1+r)^n` (PV/FV/PMT/NPER/IPMT/PPMT)
Rule: with `P = (1+r)^n` by right-to-left square-and-multiply (numbers.md, "Integer exponents"),
`k = 1+r*type` and `T = pmt*(k*((P-1)/r))`, Excel computes PV = `-(fv+T)/P` and FV = `-(pv*P+T)`
(integer nper, rate ≠ 0), and NPER = `ln((pmt*k - fv*r)/(pmt*k + pv*r)) / ln(1+r)`, every
operation in doubles and each logarithm a double logarithm of the double `1+r` (libm or correctly
rounded; the sweep does not separate them), not `log1p`. PMT, IPMT and PPMT are the exact payment
to 15 digits but not to the bit ("PMT, IPMT and PPMT are the exact payment" below); NPV is still
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
Status: PARTLY VERIFIED (PV/FV/NPER fitted on the sweep and implemented, for integer nper and pmt ≠ 0; PMT/IPMT/PPMT/NPV bits open, time-boxed per
decision 0056 after T-1524's two attempts, in T-1528)
Implemented in: `annuity_power`, `annuity_factor`, `compute_future_value`,
`fn_pv` and `fn_nper` (crates/rustsheet-engine/src/functions/financial/mod.rs, T-0901). Salvaged from
draft PR #13 by T-0031.

## PMT, IPMT and PPMT are the exact payment
Rule: Excel's PMT, IPMT and PPMT are within a few ulp of the exact value of the payment on the
double arguments, with `1+rate` exact (not the double `1+rate` of PV and FV), for integer and
fractional nper and period alike. With `P_x = (1+rate)^x`, `e` = period-1 (period-2 for type 1),
`k = 1+rate*type` and the balance `B = (pv*P_e*(P_(n-e)-1) - fv*(P_e-1))/(P_n-1)`: PMT =
`-(fv+pv*P_n)*rate/(k*(P_n-1))`, IPMT = `-rate*B` (`-rate*(B+PMT)` for type 1, 0 in period 1) and
PPMT = `-(pv+fv)*rate*P_e/(P_n-1)` (PMT in period 1 of type 1). So IPMT is not the FV recurrence
on the rounded PMT, and PPMT is not PMT − IPMT of rounded doubles: `PPMT(0.2,1,360,10000)` is
-6.248581209907205E-26, where both PMT and IPMT are about -2000 (A82).
Evidence (oracle: excel-graph, probes-numeric/m9-fin-pmt, 924 cells, 2026-10-06; Excel-cached
values of ironcalc-calc FINANCIAL/IPMT_PPMT.xlsx), each model computed exactly in rationals or at
200 bits and rounded once (T-2130):
- The exact model is within half a unit of the 15th significant digit on 308/308 PMT, 308/308
  IPMT and 304/308 PPMT cells of m9-fin-pmt. With `1+rate` rounded to a double first it is 221,
  305 and 194; IPMT on the PMT rounded to a double is 294 (`IPMT(0.2,180,360,10000,0,1)`: Excel
  -1666.6666666666588, that model -1666.8616136989206, A83).
- The same closed forms with real exponents match the fractional cells of IPMT_PPMT.xlsx (`PPMT(0.1,
  3.3,30,8000)`, `PPMT(0.1,3,29.9,8000,10,1)`, `IPMT(0.0083,12.5,36,8000,1234,1)`: J13, I15,
  G30) to 15 digits.
- Excel is not the exact value to the bit: 3.5 ulp at most for PMT ("Annuity evaluation order").
  Even at nper 1, where the exact payment is a plain quotient, Excel is off it: `PMT(0.05,1,10000)` is
  -10499.999999999998 and `PMT(0.1,1,10000)` -11000.000000000002 (m9-fin-pmt A169, A85; exact -10500 and
  -11000). Gap triage 2026-10-28: PPMT's discount below (`Q(-n)` by EXP of `-n*L`, `1-Q(-n)` by Kahan's
  expm1) in the form `-(pv+fv*Q)/(k*((1-Q)/rate))` is bit-exact on 175 of the 308 PMT cells with libm EXP
  and LN (docs/research/t3014_pmt_discount_score.py), above every form T-1528 tried (at most 133). UNVERIFIED
  as Excel's PMT until it is fitted with Excel's EXP and LN (T-3014). These ulps matter: RATE amplifies them,
  so 113 `RATE(n,PMT(..))` cells of m9-fin-irr-rate are visibly off on the engine's PMT and bit-exact on
  Excel's (probes/rate-excel-pmt-bits, T-2527).
  Where PPMT cancels, Excel's error grows: the four period-1 cells of long terms A82, A166, A412,
  A418 are 7 to 41 ulp from the exact value and visibly different from it. For the 154 fv-0,
  type-0 PPMT cells, `PMT*exp((per-1-nper)*log1p(rate))` (the exact PMT rounded to a double, libm
  `exp` and `log1p`) is within 15 digits on 153, all but A166 (Excel -1.2549699967849914E-12,
  that model -1.254969996784983E-12), and bit-exact on 63; with `ln` of the double `1+rate` it is
  131. Explained by Excel's own log and EXP ("PPMT discounts with an atanh-series log", T-2242).
- Past P_n of about 1E+270 and where IPMT's balance cancels on fv: "IPMT cancels on fv" (T-2490).
  Not probed: |nper| above 2^32.
Status: VERIFIED to 15 digits (m9-fin-pmt and IPMT_PPMT.xlsx), except the four PPMT cells above (T-2242);
bits open (T-1528, T-2243)
Implemented in: `accurate_growth`, `accurate_payment` and `accurate_period_split`
(crates/rustsheet-engine/src/functions/financial/annuity.rs, T-2130).

## PPMT discounts with an atanh-series log
Rule: Excel's PPMT is its PMT discounted with Excel's EXP and its own `L = ln(1+rate)`:
`PPMT(rate, 1, n, pv)` = `PMT(rate, n, pv) * EXP(-n*L)`, the product `-n*L` a double and the
multiplication rounded once. L is not `log1p`: for |rate| up to at least 0.37 it is
`2*(s + s^3/3 + s^5/5 + ...)` with `s = rate/(2+rate)` a double and `s^2`, the running power, each
term `power/(2k+1)` and each partial sum held at 64 bits (x87 registers) until a term leaves the
sum unchanged, then stored and doubled; from 0.38 on it is Excel's LN of the double `1+rate`
(FLDLN2/FYL2X, numbers.md "LN is FLDLN2 then FYL2X"). Either way L is up to two ulp from the
correctly rounded ln(1+rate) (-1 at 0.1, +1 at 0.05, -2 at 0.38, +2 at 0.6), and for long terms
`n*L` carries that into PPMT: `PPMT(0.1,1,360,10000)` = -1.2549699967849914E-12, 6.5E-15
relative from the exact -1.254969996784983E-12 (m9-fin-pmt A166).
- Evidence: probes-numeric/ppmt-first-period-sweep (n = 1..400 at 0.1 and 0.05 with the PMT of
  each term, excel-graph, 2026-10-24): with Excel's own PMT cell, this L is bit-exact on 282/400
  (0.1) and 271/400 (0.05) and within 2 ulp on all 800; `log1p` is bit-exact on 38 and 44 and up
  to 64 ulp off. probes-numeric/ppmt-log-rate-sweep (126 rates from 0.001 to 2, six terms each
  with `n*L` near 30, excel-graph, 2026-10-24, T-2242): at every rate one double L puts all six
  cells within 2 ulp, and this rule gives that L at all 126 (the series alone 113, all misses at
  0.38 and above; LN(1+rate) alone fits every rate from 0.38 on; `log1p` 69; `2*atanh(s)` in
  libm 85; Goldberg's `ln(u)*rate/(u-1)` 53 or 56 by the order of its operations).
- Other periods, fv and type 1: PPMT = `-(pv+fv)*rate*Q(per-1-n)/(1-Q(-n))` with
  `Q(x) = EXP(x*L)`, `1-Q(-n)` Excel's expm1 of `-n*L` (Kahan's formula, numbers.md "Excel's expm1 is
  Kahan's formula") and the whole divided by `1+rate` for type 1, rounded once: within 15 digits on
  316 of 318 cells (all 308 PPMT cells of m9-fin-pmt and 8 gen-m9 fv and type-1 cells; 192
  bit-exact) where the PMT-based form above with an exact fv share gives 308 on m9 but misses the
  large-rate fv cells by 10 to 94 ulp (`PPMT(10,2,85,-1,733)` = -2.440762531497786E-84), `1-Q(-n)`
  taken directly misses short terms (279; it cancels), and `Q(per-2-n)` for type 1 instead of the
  division misses `PPMT(2.5,5,214,1E-300,589,1)` by 108 ulp. In period 1 of type 1 PPMT is PMT.
- Negative rates (T-2588): the same discount holds for rates down to at least -0.9 (the lowest probed), not
  only above -0.375; the engine extends it to every rate above -1, from -0.9 down an extrapolation.
  `PPMT(-0.5,5,294,171132)` is -5347.875000000112 and `PPMT(-0.5,12,158,1.9)` -0.00046386718749999705,
  where the exact values are -5347.875 and -0.0004638671875 (gen-m9 79d87fc527927626 A46, 38e4fc309fe199ea
  A73). probes-numeric/ppmt-negative-rate-sweep (excel-graph, 2026-10-27: rates -0.9 to -0.3 in four shapes
  with fv and type 1, a fine grid at -0.38 to -0.37, a -0.5 grid over period, term and fv): over the 62
  cells at or below -0.375 this form, with a correctly rounded LN and EXP, is within 15 digits on 61 and
  bit-exact on 31, where the exact form is within 15 digits on 32 (8 bit-exact; `PPMT(-0.9,5,60,100000)`
  is Excel's -8.999999999999833, exact -8.999999999999993). The engine's form is within 15 digits on all
  79 cells (43 bit-exact). The earlier note that `PPMT(-0.5,1,277,73630,609)` = -37119.5 ruled it out is
  wrong: this form gives -37119.5 too.
- Past the double range: Excel's value is the form's exact value where a double product of
  `(pv+fv)*rate*Q` would overflow, consistent with an extended-range (x87) intermediate: `PPMT(-0.5,4,230,1E+300)` is -6.249999999999966E+298 (exact
  -6.25E+298; gen-m9 67a4c03afd63f988 A180, and -0.25 and -0.75 in the sweep). The engine rescales
  `pv+fv` by 2^-600 and back when the double product overflows, which keeps every bit.
- Fractional periods take the same form (`PPMT(-0.5,1.999,97,226365)` is -56630.48966324223, the
  engine an ulp below; gen-m9 d0854366830dcdee A335).
- Where |n*L| < 1E-6 L's error cannot move the power by an ulp, and the exact form matches Excel's
  last bit where this one is an ulp off (`PPMT(1E-10,2,124,191996,822,1)` = -1554.9838614045912).
- Open: the series/LN switch between 0.37 and 0.38 is not pinned (the engine takes 0.375), nor
  whether it is on |rate| (the engine) for negative rates: the sweep is within 15 digits either side
  of -0.375 but was not scored by L candidate. The last-bit misses of the sweeps (a 64-bit `-n*L`, the
  x87 EXP on it, and a divided form were each worse) are open too.
Status: VERIFIED to 15 digits (excel-graph); bits open.
Implemented in: `growth_log` and `discounted_principal`, on `x87_exp::excel_expm1`
(crates/rustsheet-engine/src/functions/financial/annuity.rs, T-2242, T-2588).
Conflicting evidence: 35 cells of desktop-Excel-saved Calipers models fed by PPMT and IPMT
matched Excel bit for bit before T-2242 and are now off by a few ulps. They are in pv_goldens
pv23_007 ("Pro Forma Cash Flow Statement" AF13, AV13, BG15, G15, G24, N13, …), pv23_008 ("Debt
Financing"!E231, "Est Property Value"!B47:C47, IRR!B26), pv23_009 and pv23_056 (found merging
T-0455, 2026-10-24). Until T-0458 settles which form Excel uses for their arguments, the calipers
baseline cannot record its fixes.
Tasks: T-2242, T-0458

## IPMT cancels on fv
Rule: Excel's IPMT is `-rate*B`, divided by `1+rate` for type 1 (0 in period 1 of type 1), with
the balance after period-1 periods `B = (pv+fv)*Y - fv` and `Y = (1-(1+rate)^(per-1-nper))/(1-(1+rate)^-nper)`.
Where that balance is smaller than fv (`|B| < |fv|`) it is computed in doubles on the rounded `Y`, so
IPMT carries fv's rounding instead of the exact balance: `IPMT(0.5,2,124,0,315)` is 0 (exact
1.15E-20), `IPMT(50,2,10,0,315)` is 5.6843418860808015E-12 = 25*2^-42 (exact 6.615E-12, B is two
ulp of 315), `IPMT(1E+10,10,10,0,315)` is 315.00007935392205 (exact 314.9999999685) and
`IPMT(1E+300,10,10,0,315)` 0 (exact 315). Elsewhere B is the exact balance, as "PMT, IPMT and PPMT
are the exact payment" has it. The form needs no `(1+rate)^nper`, so past its overflow IPMT keeps
its value, about `-pv*rate` for huge rates: `IPMT(1E+300,3,2000,452017,905)` is -4.52017E+305,
`IPMT(123456789.123,3,156,64486)` -7961234503385.777, `IPMT(2,1100,1100,1,5)` 2. PPMT past the
overflow is the discounted form of "PPMT discounts with an atanh-series log", with a discount
`Q(per-1-nper)` below the normal range read as 0 (so PPMT is 0): `PPMT(r,1,300,1)` keeps a discount of
2^-1022 and is 0 from 2^-1022.5 on, `PPMT(10,1,296,1)` is 0 though its value, 5.6E-308, is normal, `PPMT(42,10,303,8292,1,1)` and `PPMT(1E+300,1.5,180,194733,-1E+300)`
0, `PPMT(10,11,304,44626,826)` -3.0768304255991313E-301, `PPMT(2,1100,1100,1,5)` -4. Neither needs
PMT's value: `PPMT(1E+307,7,292,176644,89)` is 0 where `pv*rate` overflows, and
`IPMT(-0.5,1,2000,1,0,1)` is 0 where PMT is `#NUM!` (PPMT of it `#NUM!`, as PMT).
Evidence: probes/fin-ipmt-ppmt-huge-rate (excel-graph, 2026-10-25, T-2490): column A, 1152 cells
over rates 0.5 to 1E+300, nper 10 to 2000, period 2, nper/2 and nper, pv/fv (1000,0), (0,315),
(1000,315), (1000,-1000) and both types; column B the edges above. probes/fin-ppmt-discount-underflow
(excel-graph, 2026-10-25): `PPMT(r,1,300,1)` with `(1+r)^300 = 2^k`, k from 1021.5 to 1074, and power-of-two bases. Fit (exact rationals, then the
rounding named): on its 595 IPMT cells the cancelling form is within 15 digits on 590, the exact
balance on 521 (all its misses pv-0 cells); on m9-fin-pmt's 286 IPMT cells the doubles-everywhere
form misses 38 to the bit and 2 to 15 digits (`IPMT(0.05/12,2,3,10000,0,1)`), so the switch on
`|B| < |fv|` (any threshold from 0.1 to 1 fits; 2 loses 5 m9 cells) keeps every exact match.
`Y` from `1-EXP(x*L)` instead of the exact quotient is 592 of 595 but loses short terms at small
rates; `-expm1` 586. The gen-m9 cells of T-2257 that motivated it: 31474c20fc1dafd6 A235,
6376e27be7a05871 A32, abb7d565af89b849 A112, b6a02893a8d6b151 A280, 81bcdc0bec0c8e6b A249 (rate
50, pv 0: 0) and the rest of T-2490's list.
- Open: a fractional nper past the range. `IPMT(1.005,7,~1.85E+7,92160,674,1)` is
  -46194.91271820449, `-rate*(B/(1+rate))` and the FV recurrence on the rounded PMT, not the exact
  balance's ...448, while on the integer-nper type-1 cells past the range the exact balance fits 91
  of 118 and `-rate*(B/(1+rate))` 59; the engine keeps the FV recurrence for a fractional nper
  there (gen-m9 0a957119475760c5 A153; T-2665).
- Open: `PPMT(7,10,350,58827)` is -4.581304695604488E-303, its discount 8^-341 = 2^-1023 kept, where
  `PPMT(7,1,341,1)` with the same discount is 0 (probes/fin-ppmt-discount-underflow F1, F8; gen-m9
  a91bad4e84f49739 A158; T-2666).
- Open: the fit's rational model is within 15 digits on 590 of 595, the engine on 588: 7 pv-0 cells keep
  their cancellation's last bits unexplained: `IPMT(10,2,10,0,315)` (Excel 1.2144607808295405E-6, ours
  1.214461349263729E-6), `IPMT(10,5,10,0,315)` and `IPMT(10,10,10,0,315)` (one ulp of 315 in B; Excel
  286.3636362532304, ours 286.363636253231), each with its type 1, and `IPMT(0.1,2,360,0,1)` (Excel 0,
  ours 1.11E-17: Excel's Y rounds to 1 where the exact quotient rounds below it). Excel's own Y is a few
  ulp from the exact one; which formulation it is was not separated (T-2667). The `|B| < |fv|` switch is
  a fitted boundary, not an identified formulation.
Status: VERIFIED to 15 digits on 590/595 by the model, 588/595 by the engine (excel-graph); bits open.
Implemented in: `balance_interest`, `accurate_discount` and `discounted_principal`'s `past_range`
(crates/rustsheet-engine/src/functions/financial/annuity.rs, T-2490).
Tasks: T-2490, T-2665, T-2666, T-2667

## IPMT follows Excel's own discount where it moves the value
Rule: where IPMT's balance does not cancel on fv ("IPMT cancels on fv"), Excel's IPMT is the exact balance of "PMT,
IPMT and PPMT are the exact payment" only to a few ulp. Where Excel's own discount moves the value further, Excel has
`-rate*((pv+fv)*Y - fv)` in doubles (divided by `1+rate` for type 1) with `Y = expm1((per-1-nper)*L)/expm1(-nper*L)`,
Excel's expm1 and PPMT's `L` ("PPMT discounts with an atanh-series log"). That happens for long terms at negative
rates, where `(1+rate)^-nper` is huge and carries L's error, and for a fractional period:
`IPMT(-0.05,10,336,1)` = 0.03151246988087522 (exact 0.03151246988087528), `IPMT(-0.5,3,232,259859,333,1)` =
64714.99999999975 (exact 64715), `IPMT(-0.05,4,341,208149,115,1)` = 9391.86032881097 (exact ...977),
`IPMT(3.14159,1.999,167,0.375)` = -1.1780962499999998 (the engine's exact form had ...500000083).
Evidence (excel-graph; gen-m9 e168be7f6d3f84c2 A385, c0681a8b150ce301 A104, b6a02893a8d6b151 A35, 1cd3096e3e87c965
A227; T-2587 fit over the 1403 IPMT and 441 CUMIPMT cells of gen-m9, probes, probes-numeric and ironcalc-calc):
- The discounted form is bit-exact on all four cells; the exact balance misses them by 4 to 37 ulp. `1-EXP(x*L)` in
  place of expm1 fits the same four but not `IPMT(1,6,50,272273,1371742112.8222222)`, and the same form taken in
  double-double arithmetic misses A35.
- Taken wherever the balance does not cancel, the discounted form leaves no cell visibly off but trades bits both
  ways (76 cells better, 88 worse); with `1-EXP(x*L)` it is visibly off at tiny rates (`IPMT(1E-10,11,198,114508)`,
  gen-m9 89f0aee953a55a45 A26). Taken only where it differs from the exact balance by
  more than 6E-16 relative (about 3 ulp), it fixes the four cells and moves no other cell; 5E-16 to 7E-16 fit
  the same, 3E-16 moves 4 cells the wrong way and 1E-15 loses A35. The switch is a fitted boundary, not an identified
  formulation.
- Open: `IPMT(1,6,50,272273,AVERAGE(..))` (fv about 1.37E+9, gen-m9 0a957119475760c5 A202) cancels on fv. Excel's
  -272272.99996209145 is the discounted form with expm1 (the cancelling form's exact `Y` gives ...232986), but taking
  that form for cancelling balances makes `IPMT(0.1,8,291,1,638)` (gen-m9 6376e27be7a05871 A207) and four
  probes/fin-ipmt-ppmt-huge-rate pv-0 cells (`IPMT(2,2,10,0,315)`) visibly off (T-2943).
Status: VERIFIED to the bit on the four cells (excel-graph); the switch is fitted.
Implemented in: `discounted_interest`, `DISCOUNTED_INTEREST_FROM` and `period_payments`
(crates/rustsheet-engine/src/functions/financial/annuity.rs, T-2587).
Tasks: T-2587, T-2943

## DB and SYD argument domains
Rule: SYD with a negative salvage is `#NUM!` (`SYD(100,-1E-300,10,1)`; salvage 0 is a value), and `life*(life+1)` past the
double range is `#DIV/0!` (`SYD(100,10,1.3E+154,1)` 1.3846153846153848E-152, `SYD(100,10,1.4E+154,1)` and
`SYD(1E+300,1,1E+200,1)` `#DIV/0!`); an overflowing numerator is `#NUM!` (`SYD(9E+307,0,10,1)`, `SYD(9E+307,1E+307,3,1)`). DB
reads life and period as 32-bit integers: a life above 2147483647 is `#NUM!` (`DB(100,10,2147483648,1)`,
`DB(6912,0,2147483648,2,6)`, `DB(100,10,2147483648,0.5)`; 2147483647 is a value), with one odd exception: from a life of
4294967295 (u32::MAX) on, with a month other than 12, a period in (0, 1) is the partial last year straight after the first,
`(cost - cost*rate*month/12) * (rate*(12-month)/12)` (`DB(100,0,4294967296,0.5,8)` 11.111111111111114,
`DB(100,0,1E+10,0.5,6)` 25, `DB(18057,894,FACT(13),0.5,8)` 0, gen-m9 762cd0e4ec9a7aa9 A200), as if the life saturated in an
unsigned 32-bit integer and life+1 wrapped to period 0. 4294967294 and 4E+9 are still `#NUM!` there, a month of 12 is
`#NUM!` (`DB(100,10,4294967296,0.5,12)`), and period 1 is `#NUM!` (`DB(100,10,4294967296,1,8)`); a 64-bit-to-32-bit wrap of
the life is ruled out (`DB(100,10,6442450944,0.5,8)` is 0), and the threshold is on the untruncated life
(`DB(100,10,4294967294.5,0.5,8)` is `#NUM!`). Only salvage-0 cells test the formula's value (L7, L8): for any other
salvage the rate rounds to 0 at such lives. A period above 2147483647 is period 1's value whatever the life
(`DB(100,10,4,2147483648)` and `DB(100,10,4,1E+200)` are 43.8, `DB(0.01,488,32,1E+200,10)` -0.0033416666666666673,
`DB(1,0,1E15,1E15)` 1)
while 2147483647 is past life+1 and `#NUM!`; the life+1 bound is on the period's whole part (`DB(0.01,488,32,33.5,10)` is
period 33's value; C1 and C2 are one ulp off ours, T-2697). A negative salvage is `#NUM!` for DB too, even at cost 0 (`DB(0,-1,4,1)`, `DB(100,-0.0001,4,1)`), while
`DB(0,0,4,1)` is 0. So a huge period needs no schedule walk and is not under "DB and VDB period cap".
Evidence: probes/fin-syd-db-domain (excel-graph, 2026-10-25, T-2491); gen-m9 cells of T-2491.
Status: VERIFIED (excel-graph)
Implemented in: `fn_db`, `fn_syd`, `DB_INTEGER_MAX`, `DB_WRAPPED_LIFE`, `db_wrapped_last_year` (crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs, T-2491).
Tasks: T-2491

## ISPMT's operation order
Rule: Excel computes `ISPMT(rate, per, nper, pv)` as `(per - nper) * pv * rate / nper`, left to right in doubles. Each
other order tried misses cells where the orders differ in the last bit: `pv*rate*(per/nper-1)` (the old engine form)
fits 8 of 24, `pv*rate*(per-nper)/nper` 16, `pv*(rate*(per-nper))/nper` 17, `pv*rate/nper*(per-nper)` 6. It also gives
the Mac cache's `ISPMT(0.2,1,3,100)` = -13.333333333333334, where the old form gave ...336.
Unprobed: inputs where the three-factor product overflows before the division (`ISPMT(1,0,1E300,1E300)`: ours
`#NUM!`, where dividing first stays finite).
Evidence: probes/fin-ispmt-order A1-A24 (inputs chosen where the candidate orders differ, excel-graph, 2026-10-20,
T-2048); ironcalc-calc/FINANCIAL/ISPMT.xlsx Sheet1!E2 (Mac desktop cache).
Status: VERIFIED (excel-graph)
Implemented in: `ispmt_of` (functions/financial/annuity.rs)
Tasks: T-2048

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
MIRR power overflow and underflow (T-1995): with no positive flow, both powers overflowing (`MIRR({-1,-2,-3},1E+200,1E+200)`) is `#DIV/0!`, the
finance power's error winning over the reinvest power's `#NUM!` (each alone keeps its own: finance 1E+200 `#DIV/0!`, reinvest 1E+200 `#NUM!`).
A power underflowing to 0 over 400 flows at a rate of -0.999999 is `#DIV/0!` for the reinvest rate or both rates and a number for the finance rate alone
(0.004041825210995276, the same as at 0.1). Both overflowing with a positive flow is `#DIV/0!` too (`MIRR({-1,2,3},1E+200,1E+200)`, positive flow first or last), the reinvest power's error;
with a positive flow finance 1E+200 alone is `#NUM!` and reinvest 1E+200 alone `#DIV/0!`.
Evidence: probes/fin-mirr-overflow-underflow A1-A7 (T-1995), fin-mirr-overflow-positive A1-A4 (T-2076) (excel-graph, 2026-10-19).
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
`tools/src/rustsheet_tools/fitlab/kernels/xirr.py`):
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
  factor's form rests on A8 alone; T-1591 probed it and the bullet after this one records the result (it does not hold). Why a zero first value drops the XNPV bound is open:
  normalising XNPV by the first value (0/0, NaN) would explain both the end choice and the stop if the
  tests are written to pass on NaN, but nothing probes that.
- Unprobed: the cap on the chain walk (up to 17 doublings, X!F15, and 4 halvings are seen; day serials
  1..6, which need about 770 halvings, are `#NUM!`), the bisection cap, the order when both chains have
  a root, a guess at or below -1, and which end an exact XNPV(m) = 0 replaces (the first-value rule
  replaces the upper end, the XNPV(upper end) rule only when XNPV(upper end) <= 0; no cell tells them apart).
- T-1591 probe (probes/xirr-span-zero-first, excel-graph, 2026-10-21): 21 two-flow cells
  `XIRR({-1, (1+r)^T}, {d0, d0 + 365*T})` for T = 2, 5, 10, 20, 40, 80, 120 and r = 0.013, 0.1, 0.5, and nine
  zero-first-value cells `{0,-1000,1100}` with the first date before, equal to and after the second at guesses
  omitted, 0.2 and 1. Zero first value: Excel returns 2.9802322387695314e-09 (0.1 * 2^-25) at the default guess
  and at 0.2, and 3.725290298461914e-09 (= 0.125 / 2^25, i.e. the guess-1 bracket walk ends at 0.125, which the T-1583 model reproduces) at guess 1,
  for every date order (also the first date after the second, where the
  model gives the same). Span: r = 0.013 and 0.5 return the step-25/26 lattice point the model computes at every
  span, but r = 0.1 returns 0.10000000596046449 (one lattice point above the model's 0.09999999403953552) at
  T = 20, 40 and 120 and the model's value at T = 2, 5, 10 and 80. So the T-1583 span factor
  max(1, years) is NOT confirmed: the bound is not monotone in the span (80 years stops where 40 and 120 do not),
  and |XNPV(m)| / max|value| at the model's stop is 1.6e-8 (T=20), 4.8e-9 (40), 7.0e-12 (120) for the cells Excel
  continued past, against 2.1e-10 (80) and up to 7e-8 (r = 0.013, T >= 40) for cells where it stopped. Resolved
  by T-2328 (next bullet): the stop bound is not at fault. Counts on `xirr-oos` (52 cells with
  the probe): T-1583 candidate 48/52 (misses: xirr-early-dates A7 input check, and span-zero-first A11, A14, A20),
  engine (Newton) 46/52, "bisect, span-scaled |f| bound" alone 35/52, first-value sign with the bound skipped for a
  zero first value 41/52, fitted stop 28/52.
- T-2328 (probes/xirr-exact-root-sign, excel-graph, 2026-10-24): the three misses are the bracket, not the stop.
  The flows of those cells make 0.1, the first point of the default chain, an exact root, so the sign of
  XNPV(0.1) picks the bracket: [0, 0.1] (Excel's value 0.09999999403953552) or [0.1, 0.2] (0.10000000596046449).
  Excel's XNPV(0.1) is negative at T = 2, 5, 10 and 80 and positive at 20, 40 and 120 (A1:A7, B1:B7), and its XIRR
  lands on the matching side every time (G1:G7). The lab's XNPV gave the wrong signs because it discounted with
  libm's `pow` and read the 16- and 17-digit literals in full. Excel's XNPV divides each value by Excel's own `^`
  (see "XNPV discounts with Excel's `^`" below), and Excel cuts the literals to 15 digits. With both, T-1583's
  rule is bit-exact on every numeric cell: candidate "bisect, first-value sign, span-scaled |f| bound, Excel ^
  (T-2328)" scores 227/227 on the fit set and 51/52 on `xirr-oos`, where the one miss is the A7 input check
  (`xirr-early-dates`). The span factor stays: with Excel's `^`, the unscaled fitted stop scores 33/52 and the
  step test alone 48/52. The A8 124-year cell still needs the scaled bound.
- T-2329 (probes/xirr-zero-first-bracket, excel-graph, 2026-10-24): with a zero first value the flows do
  not matter. The bracket is [0, s] (s is the guess itself for any positive guess, 1 or more included, and 1e-5 for a guess of 0), and the
  bisection replaces the upper end every step (the first-value sign rule above), so XIRR is s/2^k at the first k
  with s/2^k < 5e-9 + 1e-8·s/2^k: 2.9802322387695314e-09 at the default guess for a root at 1.0 (A1), 3 (A4) or
  -0.1 (A5), on all-equal dates where XNPV is a flat 10 (A8) or a flat 0 (A12), and on a first date later than
  equal others (A13); 4.470348358154297e-09 = 0.3/2^26 at guess 0.3 (A3), 4.8828125e-09 = 1e-5/2^11 at guess 0
  (A9), 3.725290298461914e-09 = 2/2^29 at guess 2 (A10). A negative guess is `#NUM!` (A6, A7, A11), even
  where the downward chain from the guess brackets the root (A6: root -0.1, guess -0.2). So no XNPV sign is
  read on the walk; a NaN from normalising XNPV by the first value would explain it (unconfirmed). (A nonzero first value with XNPV(0)
  exactly 0 does walk from a negative guess: the T-2728 bullet below.)
- T-2728 (probes/irr-guess-edges A41-A47, probes/t2728-irr-guess-ref-mark A73-A81, excel-graph, 2026-10-25/26): when
  XNPV(0) is exactly 0 (flows summing to 0, a root at 0), any nonzero XNPV differs from it, so the first chain point
  brackets and the bisection halves the guess down to 0: `XIRR({-100,10,20,30,40},{0,1,59,60,61},G)` is G/2^k at the
  first k past the step test, 2.9802322387695314E-09 (0.1/2^25) with no guess, 3.637978807091713E-09 (1000/2^38),
  3.4266142457661796E-09 (123456789.123/2^55), 4.336808689942018E-09 (1E+10/2^61), 3.3087224502121107E-09 (1E+15/2^78),
  2.5243548967072378E-09 (1E+20/2^95), 4.8828125e-09 at guess 0 (1E-5/2^11) and -3.725290298461914e-09 at guess -0.5
  (-0.5/2^27, the downward chain's first point); the same flows negated give 0.1/2^25 too. At guess 1E+300 it is `#NUM!`
  (about 1000 halvings), so Excel's bisection cap lies between 95 and about 1000 steps (ours 200). Before T-2728 the
  engine read XNPV(0) = 0 as negative, skipped the upward chain and returned the negative mirror or `#NUM!`.
Ours: this model, T-1583's form (`xirr_bracket`, `compute_xirr`, functions/financial/financial_util.rs,
T-0916): 227/227 on the fit set and 22/22 on the pre-T-1591 `xirr-oos` cells (the Newton code it replaced: 9 and 11); 48/52 with T-1591's cells. The walk
cap (100 points) and bisection cap (200 steps) are engine bounds (DEVIATION, "The IRR and XIRR solver
caps" below). A trial rate where a power overflows or underflows to 0 has no XNPV, as in the lab model.
Status: VERIFIED (excel-graph) for the bracket, the midpoint and the step test. The XNPV part of the stop test
(T-1583's span-scaled form) fits every numeric cell probed once XNPV uses Excel's `^` (T-2328). Its form rests on the
fit set, `xirr-oos` and A8 alone, so it is a fitted rule, not a known mechanism.
Tasks: T-1525, T-0916, T-1533, T-1583, T-1591, T-2328, T-2329, T-2728

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
Where both secant runs fail the engine runs the fitted fallback of "IRR's fallback when the secant fails" (T-2727). Excel returns `IRR({-400,1,1,1,1,1})` = -0.6737366751808405 (ironcalc-calc IRR.xlsx General!L17),
where the fitted secant does not stop, and raising its step cap does not explain it (1,000 steps give
-0.6737366751808431). T-2727 fitted that fallback ("IRR's fallback when the secant fails"); its digits and caps
are left to T-1686.
Open, the 2026-10-24 re-fit over every IRR cell of the eval (T-2131; 388 numeric cells with
distinct flows and guess, gen-m9 included, of which the model matches 219): no variant of the
arithmetic closes the gap. NPV as c_i/x^i, Horner in v or c_i*v^i, each with double or 64-bit
terms and sums in either order, and the step anchored on v0 or v1 with or without twice-rounded
operators score 194 to 222 bit-exact and move only last-bit cells. The visibly-off cells are of
two other kinds:
- Roots far above the guess. Excel returns the model's stopping step but with its error 0.1 to
  0.4 % different: `IRR({-0.0001,2,42})` is 20019.97799564059 (gen-m9/c0681a8b150ce301
  Calc!A218; the model 20019.977995638386, the root 20019.9779961838) and
  `IRR({-2.5,-1,123456789.123})` 7026.083662734896 (gen-m9/67a4c03afd63f988 Calc!A346; the model
  7026.0836627358585), likewise A3, A199, A285 and the gen-m9 cells of T-2131. The model's value
  does not move under a 1e6-ulp change of v1 but moves continuously with the offset, which would
  have to be 0.00079 to 0.00124 depending on the cell. Neither a constant offset, an offset of
  0.001 in r, nor a relative offset v0(1 +- 0.001) fits, and neither does a Newton first step. So
  Excel's steps far from the root differ from the secant's, by a rule not yet identified (T-2565
  probes it).
- Runs where both secant runs fail. For `{-400,1,1,1,1,1}` (IRR.xlsx L17), `{42,1,1,-1}`
  (gen-m9/31474c20fc1dafd6 A271) and `{0.3,-2.5,1.005}` (gen-m9/a91bad4e84f49739 A369), no
  secant run from any guess in -0.99 to 100 gives Excel's bits (the nearest is 2.6e-15 off), so
  Excel's fallback is not a retry of the same secant from another fixed guess (T-1686).
Status: VERIFIED for the method (secant in v, offset, both stop tests, retry; 189/263 bit-exact); UNVERIFIED
for the last bit (NPV's evaluation order), the step cap, and flows with leading zeros (A173, A175)
Tasks: T-1525, T-1533, T-0916, T-2131, T-2565

## The IRR and XIRR solver caps
Rule: Excel's caps on its IRR secant, XIRR's bracket walk and XIRR's bisection are unknown: no probe
reaches them. The engine bounds each loop itself, so an untrusted workbook cannot make one cell iterate
without end (CLAUDE.md directive 5). XIRR walks at most 100 points per chain and bisects at most 200
steps. IRR's secant takes at most 50 steps per run, with one retry from 0.1, and then the fallback's Newton search
at most 50 steps ("IRR's fallback when the secant fails"; Excel's own cap there lies between 29 and 119). A run that reaches its
cap gives `#NUM!` (or, for IRR, moves on to the next stage). Every oracle cell of the fit sets finishes
well inside the caps.
Evidence: none for Excel; this is a resource limit, not an Excel observation. Probing Excel's caps, the
fallback path and overflowing trial rates is T-1686.
Status: DEVIATION (decision 0014)
Implemented in: `xirr_bracket` (`WALK_CAP`), `compute_xirr` (`BISECTION_CAP`), `irr_secant` (`MAX_STEPS`),
`irr_fallback` (`IRR_FALLBACK_STEPS`) (functions/financial/financial_util.rs)
Tasks: T-0916, T-1686

## IRR's guess range
Rule: IRR's guess must be above -1 and below 2^53, else the call is `#VALUE!`, ahead of the no-sign-change `#NUM!`:
`IRR({-100,60,60},G)` converges for G = 1E+15, 2^52, 8E+15, 9E+15 and 2^53-1, and is `#VALUE!` for 2^53, 2^60, 2^100,
1E+16, 1E+20, 2.5E+29 and 1E+300 (as for -1, -2 and -1E+300); `IRR(COUNTBLANK(..),1E+300)` (a scalar flow, otherwise
`#NUM!`) is `#VALUE!` too. (guess >= 2^53 and 1 + guess = guess are the same test on doubles.) An error argument in
the flows comes before the guess bound (`IRR(1/0,1E+300)` is `#DIV/0!`); the full order is "IRR's error order" below.
Other findings of the same probes: `IRR({-1,X})` is `#NUM!` for X from 1E+13 to 1E+16 except `#DIV/0!` at X = 1E+15
("IRR's far-root bound"); `IRR(CELL("row",D2))`, `IRR(CELL("col",D2))` and
`IRR(LET(x,CELL("row",D2),x))` are `#VALUE!` while `IRR(CELL("row",D2)+0)`, `IRR(CELL("contents",D2))` and `IRR(ROW(D2))`
are `#NUM!` (followed since T-2728, "IRR's error order"); `XIRR({-100,10,20,30,40},{0,1,59,60,61},123456789.123)` is
3.4266142457661796E-09 and with no guess 2.9802322387695314E-09 (followed since T-2728, "XIRR is a bracketing bisection").
Evidence: probes/irr-guess-edges, probes/irr-guess-edges-2 (excel-graph, 2026-10-25).
Status: VERIFIED (excel-graph)
Implemented in: functions/financial/financial_util.rs (`compute_irr`, `IRR_GUESS_BOUND`)
Tasks: T-2494, T-2727, T-2728

## IRR's error order
Rule: IRR reads its errors in this order: (1) an error that the flows argument itself evaluates to (not a single
cell's), as is; (2) the guess: an error guess, as is, then the guess range (`#VALUE!`, "IRR's guess range"); (3) the
`#VALUE!` of an error inside the flows (an array element or a referenced cell, "IRR over an error element") and the
`#VALUE!` of a flow that is a number straight from `CELL("row"|"col",..)` or ERROR.TYPE; (4) the solver's `#NUM!`.
- (1) before (2): `IRR(1/0,NA())`, `IRR(1/0,"x")`, `IRR(1/0,1E+300)` and `IRR(1/0,{0.1,0.2})` are `#DIV/0!` (A41, A33,
  A29, A27).
- (2) before (3): `IRR({1,"a"}+0,1/0)` is `#DIV/0!` (A39) where `IRR({1,"a"}+0,0.1)` is `#VALUE!` (A23);
  `IRR(CELL("row",Data!D2),1/0)` is `#DIV/0!` and `IRR(ERROR.TYPE(#REF!),NA())` `#N/A` (-2 A1, A3). The gen-m9 cell
  `IRR((YEAR(Data!D$11)+UNIQUE(Data!B2:E10,TRUE,TRUE)),INDEX(Data!A3:B9,2))` is `#REF!` this way: its flows hold `#VALUE!`
  elements (A5) and its guess is INDEX's own `#REF!` for one index into a 2-D range (lookup.md "INDEX with one index into a
  2-D range is #REF!"; `ABS(INDEX(Data!A3:B9,2))`, `PMT(..)`, `NPV(..)`, `XIRR(..)`, `MIRR(..)`, `RATE(..)` of it are
  `#REF!` too, -2 A29-A39), not an IRR quirk: a plain 1x2 range guess lifts (`Data!A4:B4`, `(Data!A4:B4)`,
  `OFFSET(Data!A4,0,0,1,2)`, `INDIRECT(..)`, `IF(TRUE,..)`, `CHOOSE(1,..)`, `LET`, an intersection and INDEX with an area
  all spill two results, A9, -2 A13-A27).
- (2) before (4): `IRR(Data!B9:B10,1E+300)` (two text cells, no number) is `#VALUE!`, `IRR(Data!B9:B10,1/0)` `#DIV/0!`
  (A45, -2 A5).
- The mark of (3): `IRR(CELL("row",D2))`, `IRR(CELL("col",D2))`, `IRR(ERROR.TYPE(#REF!))` and `IRR(CELL("row",D2),0.5)`
  are `#VALUE!` where `IRR(5)`, `IRR(ROW(D2))`, `IRR(CELL("contents",D2))`, `IRR(CELL("row",D2)+0)` and
  `IRR(@CELL("row",D2))` are `#NUM!` (irr-guess-edges A20-A26, irr-guess-edges-2 A35-A44; A51, A63). It is the mark of
  statistics.md "ERROR.TYPE and CELL results are #NUM! in the paired functions", and it passes through the same wrappers:
  `LET(x,..,x)`, `IF(TRUE,..)`, `CHOOSE(1,..)`, `INDEX(..,1)`, `IFERROR(..,0)` and `LAMBDA(x,x)(..)` keep the `#VALUE!`
  (A53-A61).
- A single error cell as the flows is (3): with A1 `=1/0`, `IRR(A1,NA())`, `IRR(A1:A1,NA())` and `IRR((A1),NA())` are `#N/A`
  and `IRR(A1,0.1)` `#VALUE!` (probes/t2728-irr-guess-ref-mark-3 A2-A5).
Evidence: probes/t2728-irr-guess-ref-mark (cells A1-A81, "-2" is probes/t2728-irr-guess-ref-mark-2), -3, excel-graph,
2026-10-26; probes/irr-guess-edges, irr-guess-edges-2 (2026-10-25); gen-m9 5422bdbf173fc233 A63, 5c6a23c4f6597f1c A153.
Status: VERIFIED (excel-graph)
Implemented in: `fn_irr` (functions/financial/mod.rs), `is_marked_number` (functions/util.rs); the wrapped forms are
TODO(T-2766), unscored in the probe until then.
Tasks: T-2728, T-2766

## IRR's far-root bound
Rule: IRR is `#NUM!` at a root with 1 + r at or above about 1E+7 (v = 1/(1 + r) at or below 1E-7), whether the
secant or the fallback finds it: `IRR({-1,X})` returns for X = 1E+4, 1E+5, 1E+6 and 3E+6 and is `#NUM!` for X = 1E+7,
3E+7, 1E+8 to 1E+14 and 2E+15 (also with guess 1E+8), and `IRR({-1,0,X})` returns for X = 1E+8 to 1E+14 (the last
9999998.994161328, 1 + r = 9999999.994, just below 1E+7). Three-flow and longer sets agree: `IRR({-0.0001,2,42})` = 20019.98 returns, and
the gen-m9 roots 3.16E+7 and 2E+7 (9a1fa5366bc50918 A400, abc2939ebd1e2365 A280's inner IRR), `IRR({-1E-15,1})` and
`IRR({-1E-13,1})` are `#NUM!`. The exact threshold between 9999998.99 and 1E+7 is not pinned; the engine uses 1E+7.
The two-flow returns are not the exact root X - 1 (`IRR({-1,1E+6})` is 999998.9999712444), so Excel's steps at
these scales are not the fitted secant's (T-2131, T-2565).
Not explained: `IRR({-1,1E+15})` is `#DIV/0!` from guesses 0.05, 0.1, 5E+14 and 1E+15, and so are `IRR({-1,1E+300})`
and IRR over gen-m9 bc04c1f478c9896c's Data!A1:D12 (first flow 0) from guesses 0.05, 0.1 and 0.5; ours is `#NUM!`
(T-2802). `IRR({0,1E+15,2,42,1E+15,-0.0001})`, also a zero first flow, is `#NUM!` as ours.
Evidence: probes/irr-fallback-roots A25-A40, probes/irr-guess-edges-2 A15-A34, probes/irr-guess-edges A27-A39
(excel-graph, 2026-10-25 and 2026-10-26).
Status: VERIFIED (excel-graph) for the bound; the `#DIV/0!` cases are UNVERIFIED
Implemented in: `IRR_FAR_ROOT_BOUND`, `irr_from_discount` (functions/financial/financial_util.rs)
Tasks: T-2727, T-2802

## IRR's fallback when the secant fails
Rule: where both secant runs fail (an iterate at v <= 0, a flat secant, no stop in the cap), Excel finds some roots and
not others, and the split is that of Newton's method on the NPV polynomial in v, P(v) = sum c_i v^i, started at
v = 0 (r = +inf): it converges to the real root nearest v = 0 and the call is `#NUM!` when that root is not positive or
the run needs more than about 50 steps. Leading zero flows are dropped first (P(v)/v^k), else v = 0 would be a root:
`IRR({0,2,-1})` returns about -0.5 (gen-m9/3c06f9298f65a3ba A394, under COSH(100)) and
`IRR({0,1E+15,2,42,1E+15,-0.0001})` is `#NUM!` (probes/irr-guess-edges A34, A35). The guess does not change which flows return (it can change the digits;
Sweep below). Probes:
- `IRR({42,1,1,-1},G)` = -0.7467044766537371 for G = 0, 0.01, 0.1, 0.5, 1 and `IRR({-400,1,1,1,1,1},G)` =
  -0.6737366751808405 for G = 0.01, 0.1, 0.5, 1 (P has one real root; Newton from 0 takes 29 and 26 steps);
- `IRR({1,3,-1})`, `IRR({1,7,-1})` (guesses 0.01 to 1) and `IRR({1,20,-1})` are `#NUM!`, though each has a root at
  r = -0.70, -0.86 and -0.95 that a secant from a guess near it finds: their P has a negative root nearer 0;
- `IRR({0.3,-2.5,1.005},G)` = 6.909808543634611 for G = 1E-6 to 0.01 (the root at v = 0.126, nearer 0 than v = 2.36);
- gen-m9: 31474c20fc1dafd6 A173 returns 661.41 (v = 0.0015, a negative root at -0.005 is farther); A22 c0681a8b150ce301,
  A104 and A349 5c6a23c4f6597f1c, A117 1b60bf99d71424e7, A277 1cd3096e3e87c965, A293 0a957119475760c5 and A251
  3c06f9298f65a3ba are `#NUM!` (negative nearest roots); abb7d565af89b849 A254 `{1E+15,-0.0001,1,-2.5,-0.0001}` is
  `#NUM!` although its nearest root v = 50887 is positive: Newton from 0 needs 119 steps there. In 1b60bf99d71424e7
  A117 and 1cd3096e3e87c965 A277 Newton from 0 jumps past a nearer negative root (-4.6E-5, -4.01) to a positive one
  (8.41, 4.06); Excel is `#NUM!`, so the engine also rejects a root when P changes sign on [-v, 0).
Neither an XIRR-style bracket walk from the guess (it finds `{1,3,-1}`'s root) nor a secant retry from any fixed guess
in -0.99 to 1000 (they find the `{1,k,-1}` roots) fits. The returned digits are 15 to 35 ulp from Excel's
(`{42,1,1,-1}` -0.7467044766537408 against -0.7467044766537371), not bit-exact (`{0.3,-2.5,1.005}` with guess 0.1 is 6.909808543634389 in Excel, 6.909808543634611 from guess 0.01).
Against the exact roots (mpmath, 50 digits; gap triage 2026-10-27), Excel is the inaccurate one. `{42,1,1,-1}`'s
root is -0.74670447665374079 and `{-400,1,1,1,1,1}`'s is -0.67373667518084225. Excel is 3.7E-15 and 1.8E-15 above
them, while ours is within 1E-16, correctly rounded. Matching Excel means reproducing where its iteration stops
(T-2928, after the T-2927 sweep).
Sweep (T-2927; probes-numeric/t2927-irr-fallback-sweep, 150 flows where the fitted secant fails from guess 0.1 and
its retry, excel-graph 2026-10-27; exact roots of the flows' binary values and the signed offset of Excel's double in ulps in
t2927-irr-fallback-sweep.roots.tsv, decimal bisection to 30 digits; both files are written by tools'
rustsheet_tools.fitlab.irr_fallback_sweep). Excel's offset from the exact root has no fixed sign or size, by family:
- `{k,1,1,-1}`, k = 2..100: -59 to +113 ulps (median +1; 0 to 2 ulps for many k); scaled by 1E6: -2.0 to +8.9;
  negated (same root): -572 to +1251 (k = 25: +1 ulp, negated +1251).
- `{-N,1,1,1,1,1}`, N = 163..375: within 1 ulp except N = 284 (A24), +1865 ulps, whose 1E6-scaled and negated
  forms are within 1 ulp; negated up to +53.
- `{0.3,-2.5,1.005}` scaled by 1E-3..1E+6 (A32-A41): -251 to -252 ulps at every scale. Scaled by a further 1E6
  (A113-A131): -251 while the first flow is at most 3E6 (A113-A121), +7 from 3E7 on (A123-A131): the split goes with
  the size of the flows. Negated: -282 to -291.
- `{0.3,-b,1.005}`, b = 2.2..4.0: -9168 (b = 2.4) to +8395 ulps (b = 3.0); scaled by 1E6: -35 to +14; negated:
  -10524 (A138) to +9036 (A144).
- `{0,2,-1}`: exact in every form.
Hypotheses for T-2928, not findings: the size dependence (1E6 scaling shrinks the offsets, and the flows' size splits
A113-A131) suggests a stop test on |P| in the flows' units; the negation dependence says the iteration is not
symmetric in the sign of P. The secant model that chose the flows is itself a fit, so some rows may be secant results
rather than fallback ones, the guess-dependent rows below first. The guess column (column B, every tenth row) changes
the digits on 5 of its 15 rows: `{2,1,1,-1}` (A1) and its 1E6 scaling (A51), -0.5000000000000002 against
-0.5000000000000036 with guess 0.5; `{0.3E6,-2.5E6,1.005E6}` (A41) and `{3E6,-25E6,10.05E6}` (A121),
6.909808543634389 against 6.909808543634618; `{7E6,1E6,1E6,-1E6}` (A61) by one ulp. This conflicts with
irr-fallback-roots, where `{42,1,1,-1}` and `{-400,1,1,1,1,1}` give the same digits for every guess from 0 to 1:
whether the fallback starts from the guess (or from the secant's last iterate) is UNVERIFIED and left to T-2928.
Which flows return does not depend on the guess.
`#NUM!` where the fallback rule returns: `{-N,1,1,1,1,1}` for N = 496, 656, 866, 1145, 1513 and 2000 (rows 26-31),
and the same scaled by 1E6 and negated (rows 101-112), while N = 163 to 375 and 400 return. The engine's Newton from v = 0
converges there within its cap, so Excel's fallback gives up on a condition the rule lacks (a step cap below ours, or a stop on |P|
that the large first flow defeats).
Fit (T-2928 attempt 1, `uv run rustsheet-fitlab irr-fallback`, kernels/irr_fallback.py `irr_fallback_candidates`; 212
cells of the sweep and irr-fallback-roots, the `#NUM!` cells included; bit-exact / within 4 ulp / within 1E-9, each
candidate run as the fallback alone, the engine with its secant first): the engine 72/108/191; Newton in v from 0 as
the engine 63/98/179, with |P(next)| < 1E-7 added 69/104/183, |dv| < 1E-10 67/101/179, < 1E-8 65/101/179, < 1E-7
63/101/179, < 1E-5 51/74/179, < 1E-7·|v| 63/98/179, P summed in ascending powers 65/98/179, returning v instead of
next 28/28/113 (relative stop 21/21/106), from v = 1 36/49/104, from the guess 54/91/178; Newton in r from the guess
2/9/40, from 0 1/9/37, in x = 1 + r 2/10/42; the fitted secant from the guess with a 1000-step cap 48/50/63 (by Horner
49/51/63). In a scratch harness (not shipped) these also fail: the secant continuing past an iterate at v <= 0 with
the iterate kept, -v, v1/2, v1/10, (v0 + v1)/2, 0.9·v1, v1 + 1 or 1E-3 (at most 29 of the 152 numeric cells
bit-exact, the engine's fallback 38); the secant in v, v by Horner, r and x = 1 + r from every fixed start on a grid of
1,200 (-0.99 to 10, offsets 1E-5 to 0.1, signed or not, step tolerance 1E-6 to 1E-8, with or without |f| < 1E-7):
only r0 = -0.65, r1 = r0 + 0.001 gives both `{42,1,1,-1}` and `{-400,1,1,1,1,1}`, and it scores 24/152 overall, so
the hit is chance. Findings that bound the next attempt: (1) Excel's error is not what a plain Newton step leaves: on
`{42,1,1,-1}`, `{-400,1,1,1,1,1}` and `{0.3,-2.5,1.005}` a last Newton step in v or in r would leave an error of the
sign of P''/P', and Excel's has the other sign (only Newton in r on `{0.3,-2.5,1.005}` fits, from an iterate
1.4E-6 off). A secant's error C·e1·e0 has no fixed sign, and its signed first offset explains the negation
dependence. (2) `{0.3,-2.5,1.005}` at -251 ulps from 1E-3 to 1E+6 scale and +7 from 3E7 on fits a stop test
|P| < 1E-7 in the flows' units: at Excel's iterate |P| is about 7.9E-15 times the scale, so it passes up to the
1E7 scale and fails from 1E8. (3) That test also turns the engine's Newton `#NUM!` on the 1E6-scaled
`{-N,1,1,1,1,1}` for N = 656 to 2000 (rows A103-A111, as Excel) because |P| cannot fall below 1E-7 at that scale, but
not on N = 496 scaled (A101) or on the unscaled rows A26-A31. (4) Excel's value is an iterate of Newton in v from 0,
Newton in r from the guess or the secant in v or r from the guess on only 53 of the 152 numeric cells (Newton in v
on 40, the secant in v on 19, Newton and the secant in r on 2 each). (5) No step cap on the engine's Newton from
v = 0 separates the `#NUM!` rows: `{-N,1,1,1,1,1}` takes 24 to 27 steps for N = 163 to 400 (returns) and 28 to 33 for
N = 496 to 2000 (`#NUM!`), but `{40,1,1,-1}` takes 28 and `{42,1,1,-1}` 30 and both return; neither does a bound on
the root (v = 3.21 is `#NUM!` for N = 496 while `{100,1,1,-1}` returns at v = 5.08).
Evidence: probes/irr-fallback-roots A1-A24 (excel-graph, 2026-10-26); the gen-m9 cells above;
probes-numeric/t2927-irr-fallback-sweep (excel-graph, 2026-10-27).
Status: VERIFIED (excel-graph) for which flows return; UNVERIFIED for the method and the step cap (between 29 and 119)
Implemented in: `irr_fallback`, `IRR_FALLBACK_STEPS` (functions/financial/financial_util.rs)
Tasks: T-2727, T-1686, T-2131, T-2927 (sweep), T-2928 (returned digits)

## RATE is a secant on the FV residual
Rule: RATE is a secant, not Newton's method, on the residual f(r) = ((pv·P) + (pmt·(k·((P-1)/r)))) + fv:
P = (1+r)^n by Excel's `^` on Excel's `1+r`, k = 1+r for type 1, in exactly that parenthesization, each
`+`, `-`, `*` and `/` Excel's twice-rounded operator (not FV's `annuity_factor`, which rounds once), and
f(0) = pv + pmt·n + fv. It starts from r0 = guess and
r1 = r0 - 0.001·sign(f(r0)), steps r2 = r1 - f1·(r1-r0)/(f1-f0) with Excel's twice-rounded `-`, `*` and `/`,
and stops at the first iterate with |r2 - r1| < 1e-7 and |f(r2)| < 1e-7, returning r2. The cap is 200
steps, not the documented 20: guesses whose run needs 172 to 200 steps converge, those needing 201 to 216 are
`#NUM!`. The 200th iterate, unconverged, is returned when it passes a looser test: |r200 - r199| < 1e-4,
|f(r200)| < 1e-4 and 1 + r200 > 1e-4, all three absolute (T-2689, below). Otherwise the run is `#NUM!`. An
iterate at or below -1 is `#NUM!` (m9-fin-irr-rate A236, A352, A371-A378: the secant's root is
below -1 there). Guess 0 therefore converges (rate-empty-arguments A1, A3, A8), and Excel's guess-sweep
errors on R!C1:C31 are the secant's last step, whose sign and size depend on where the run lands.
Evidence: scratch fit harness (T-2005; the RATE arguments evaluated by the engine, nested
`RATE(n,PMT(..),..)` cells included) over the 464 literal RATE cells of probes-numeric/m9-fin-irr-rate,
probes/solver-iteration-traces sheet R, probes/rate-empty-arguments and probes/fin-argument-modes
(421 numeric): the model is bit-exact on 240 and gives Excel's value-or-`#NUM!` on 463 of 464 with the
200-step cap (not A583, below). Rejected candidates (bit-exact / numeric cells, cap 20 unless stated):
the engine's Newton 16/405; the secant with the naive double f (`(1+r)^n` by `powf`, documented order)
39/373; with `^` and `+` but the documented order `pmt·(1+r·t)·(P-1)/r` 69/375; with every operation
twice-rounded in the documented order 71/375; FV's order 210/375, 212 with the twice-rounded step; stop
tests |dr| < 1e-7 alone 166, |f| < 1e-6 188, < 1e-8 181, |dr| < 1e-6 191, < 1e-8 181; step forms
`r1 - f1/((f1-f0)/(r1-r0))` 137, `(r0·f1 - r1·f0)/(f1-f0)` 29, `r0 - f0·(r1-r0)/(f1-f0)` 114; cap 50
217/394, 100 232/414. In exact arithmetic Newton (in r, v = 1/(1+r), ln(1+r), on f, f/(1+r)^n,
r·f and f·(1+r)^a for a in -100..100), Halley and finite-difference Newton (steps ±1E-5 to ±1E-10,
absolute and relative) never reproduce R!C's large-error cells; the secant reproduces R!B's to a few ulps.
The 181 numeric cells still off were f's rounding and the nested PMT's bits (T-2527): the fit lab
(`uv run rustsheet-fitlab rate`, kernels/solvers.py `rate_secant_candidates`) reads a nested `PMT(..)` as
Excel's value of the same call in probes-numeric/m9-fin-pmt (all 308 inner calls are there). Excel agrees
that the nested call adds nothing: probes/rate-excel-pmt-bits repeats the 308 nested cells with the PMT
replaced by a cell holding Excel's PMT bits (its 15-digit literal plus an exact m·2^-e), and its 308 values
are the nested cells' bits. On the 782 RATE cells of the fit set (errors included; the probes above,
rate-zero-residual-guess and rate-excel-pmt-bits) the secant is bit-exact on all 782 with the residual's
operators twice-rounded, 778 with them rounded once (FV's evaluation; m9-fin-irr-rate A487 2,183 ulp, A218
157 ulp, and the same two inputs again as rows A2 and A271 of rate-excel-pmt-bits), and 751 to 774 with the
three-term sum in another order (pv,fv,pmt; fv,pv,pmt; pmt,fv,pv; either rounding). Excel's value is an
iterate of the model's trace on 705 of 705 numeric cells. So the RATE cells still off
in the eval (m9-fin-irr-rate A218, A278, A245, A367 among them) are off by the engine's PMT bits, not by
RATE's: the engine's PMT differs from Excel on 176 of the 308 inner calls (PMT's own tasks, T-1528).
A guess whose residual is exactly 0 runs the secant, it is not returned (probes/rate-zero-residual-guess,
T-2527): `RATE(1,0,0,0)` and `RATE(1,0,0,0,0,0.3)`, residual 0 everywhere, are `#NUM!` (the flat secant),
not the guess; `RATE(10,-1E6,1E7,0,0,0)` is 0 (from guess 0 the first step lands on 0 exactly and the
next one does not move), where the default guess is `#NUM!` (A583 below), and the guess-0 roots A8 and A10
are 0 too; exact roots as guesses (`RATE(2,0,-1,1.5625,0,0.25)`, `RATE(2,-1,-1,4.75,0,0.5)`; A3-A6, A12)
return the guess's bits, as the model. A7 (default guess) and A11 (type 1, residual -1.25 at the guess) are
controls, not zero residuals. One cell disagrees on the value type:
m9-fin-irr-rate A583 `RATE(10,-1E6,1E7)`, root exactly 0, is `#NUM!` in Excel; near 0 the residual is
rounding noise (|f| about 0.1 to 0.4 against the 1E-7 test), the run wanders until the cap and the
model returned its 200th iterate, -2.6E-9, before the cap's |f| test below (now `#NUM!`, as in Excel).
The cap's return test (T-2689; probes/rate-cap-return-rule, rate-pmt-zero-cap, rate-step-cap, excel-graph):
- Step, absolute: the 200th steps 7.3e-5 and 7.8e-5 return (rate-cap-return-rule A68 at r 0.05, A72 at r 0.2, where it is 4e-4
  of the rate, so the test is not relative) and 1.09e-4 to 1.3e-4 is `#NUM!` (A70; rate-pmt-zero-cap A25, A55).
- |f|, absolute: the same loan run returns at |f| 7.3e-5 (rate-step-cap A30) and is `#NUM!` at 1.06e-4 to 6e-3
  (rate-cap-return-rule A53-A64, A20-A30, steps below 1e-7 included: A31 of rate-step-cap is this, not its step).
  Scaling pmt and pv by 2^-10 to 2^-40 keeps the trajectory and shrinks f, and those runs return (A65-A74).
- 1 + r: `RATE(n,0,pv)` creeps to its root -1, and returns at 1 + r200 = 1.02e-4 (rate-cap-return-rule A1, A7)
  but not at 9.4e-5 (A16) or below (A5, A6, A8, A10, A11, A13, A15; rate-pmt-zero-cap n 11-15). The guesses
  move 1 + r200 while the last step's ratio to 1 + r stays a function of n alone, so the edge is on 1 + r. With
  pmt 0 and fv 0 the bands follow: n 2-10 converge, n 11-15 end below 1 + r = 1e-4, n 16-24 return (calipers
  formulas_financial F142 = rate-pmt-zero-cap A24, -0.9970141220397587), n 25 and up have a step over 1e-4.
Whether a test is `<` or `<=` and whether the last one is 1 + r or r + 1 are not separated by any cell.
Not probed, kept from the model: `#NUM!` for a residual whose `^` overflows (a flat secant is probed:
rate-zero-residual-guess A1, A2); only the iterates r2 are tested against -1 (not r1 = guess - 0.001); a -0
residual at the guess (the zero's sign picks r1; the probe's residuals are all +0); f(0) is pv + pmt·n + fv in plain doubles (the three
guess-0 cells are bit-exact, but no cell separates the order); the cap probe's guesses were chosen by the model's own
step count, so the cap's position is measured in model steps (A30's exact unconverged bits confirm it).
Ours: `compute_rate` (functions/financial/financial_util.rs), this model.
Status: VERIFIED (method, f's evaluation to the last ulp, stop rule, cap and its return test, a zero residual at the guess; excel-graph) / UNVERIFIED (the rules above)
Tasks: T-1525, T-0916, T-1533, T-2005, T-2689, T-2527

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
DSR 0: with settlement the 30th and maturity the 31st of one month, DAYS360 gives DSR 0 on bases 0 and 4 although
settlement < maturity. The closed form then divides by 0, and Excel returns **0**, not #NUM!: all 12 probe bonds
(frequencies 1, 2 and 4, maturities 2024-01-31 and 2023-03-31, bases 0 and 4, price 99, so a nonzero numerator) are 0,
and so are bonds whose numerator is 0 (C1:C2 rate 0 with pr = redemption; C3 rate 0.06 with pr = redemption, where A = E
on basis 0 cancels it). A DSR 2 control is ordinary
(1.794410851446313). PRICE at the same dates is 100.
Evidence (excel-graph, 2026-10-19, T-1949): probes/yield-one-period-dsr-zero Sheet1!A1:A12, B1:B3, C1:C3.
Ours: `yield_one_period` returns 0 when DSR is 0 (T-1949).
Status: VERIFIED (excel-graph)
Tasks: T-1525, T-1532

## RATE: an empty guess is guess 0, not the default 0.1
Rule: an empty guess argument (`RATE(48,-200,8000,0,1,)`) or a reference to an empty cell behaves as
guess 0, not the default 0.1: the result has the same bits as an explicit `0` (0.008052981923885287)
and differs from the 0.1 result in the last digits (…906556). Omitting the guess altogether uses 0.1
(A4 equals A2). An empty `fv` (`RATE(48,-200,8000,,1)`, A4) is 0 (the same bits as the explicit 0 and 0.1 guess
row A2), and an empty `type` is 0 (same bits as `type` 0, A5 = A6). A text guess (`""`) is `#VALUE!` and
`TRUE` is guess 1 (A9 = A10, same bits). Excel converges for guess 0 here: RATE is a secant, so no step divides
by the guess ("RATE is a secant on the FV residual", T-2005).
Evidence: probes/rate-empty-arguments A1-A10 (excel-graph, T-1128)
Status: VERIFIED
Implemented in: `compute_rate` (empty argument reads as 0; guess 0 converges, T-2005)

## XIRR (and IRR): an empty guess argument is the default 0.1 for XIRR, guess 0 for IRR
Rule: XIRR reads a written empty guess (`XIRR(v,d,)`) as the default 0.1, the opposite of RATE
("RATE: an empty guess is guess 0, not the default 0.1"). A reference to an empty cell is still guess 0
(same bits as an explicit `0`), so only the empty argument itself is special. IRR reads both as guess 0,
like RATE. So the rule is per function and neither side may be generalised from the other.
- XIRR, Excel's doc flows -10000, 2750, 4250, 3250, 2750 on 39448, 39508, 39751, 39859, 39904: `XIRR(v,d,)`
  = 0.3733625352382659, the same bits as no guess and as `,0.1` (A1, A2, A4), over ranges and array
  constants (A7, A8). `XIRR(v,d,0)`, `XIRR(v,d,E1)` with E1 empty and `XIRR(v,d,0.1-0.1)` give
  0.3733625341796877 (A3, A5, A22; A9 for constants). A `""` guess is `#VALUE!` (A6). The bisection's
  bracket starts from the guess ("XIRR is a bracketing bisection"), so the two runs differ in the 9th digit.
- IRR, -70000, 12000, 15000, 18000, 21000, 26000: `IRR(v,)`, `IRR(v,E1)` and `IRR(v,0)` all give
  0.08663094803653149 (A11, A15, A13; A17, A19 for constants), and no guess or `,0.1` gives
  0.08663094803652216 (A12, A14, A18). A `""` guess is `#VALUE!` (A16).
- Desktop agrees: calipers pv23_056 Summary!F23 `=XIRR(...,...,)` (Microsoft Macintosh Excel 16.x) caches
  0.11376532912254333, the fit lab's XIRR model with guess 0.1; guess 0 gives 0.11376532714843753.
Evidence: probes/xirr-irr-empty-guess A1-A22 (excel-graph, T-2688); calipers pv23_056 Summary!F23
Status: VERIFIED
Implemented in: `fn_xirr` (an `EmptyArg` guess is dropped, so the default applies); `fn_irr` reads it as 0.
Our IRR from guess 0 is a few ulps off Excel's on these flows (…3171 for …3149), a separate kernel gap.

## XIRR: a mismatched value/date size is #NUM! whatever the guess, a written empty guess included
Rule: `XIRR(v,d,)`, `XIRR(v,d)`, `XIRR(v,d,E1)` (E1 empty) and `XIRR(v,d,0.1)` all give `#NUM!` when the
values and dates differ in length (3 vs 2, 2 vs 3, 3 vs 4), over ranges and array constants, and also when
the longer flow holds a text or boolean element (G21-G36). A written empty guess and an empty-cell guess
take the same error path as an absent or number guess; a text, boolean or error guess keeps `#VALUE!`
(see "XIRR and XNPV under the gen-m9 statistical cells").
Evidence: probes/xirr-size-mismatch-empty-guess G1-G36 (excel-graph, T-2773)
Status: VERIFIED
Implemented in: `fn_xirr` (`number_guess` counts `EmptyArg` and `EmptyCell`; before T-2773 they gave
`#VALUE!` over text or boolean flows).

## XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty cells
(One empty values cell on its own is the exception: see "XNPV reads one empty values cell as a single 0 cash flow".)
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
when no date is later than the first and the first value is nonzero, even where a root exists (with a zero first value
it is not: probes/xirr-span-zero-first A28-A30, `{0,-1000,1100}` on 46000, 45000, 45365, give 2.98e-9 / 3.73e-9, T-1591;
T-2329 skips the engine's later-date check for a zero first value; equal dates with a zero first value are 2.5849043512437796E-09 in Excel
(gen-m9/e168be7f6d3f84c2 A125, a computed guess), because a zero first value fixes the bracket whatever the dates, see the T-2329 bullet under
"XIRR is a bracketing bisection"): -1000 on 45365 then 1100 on 45000 (X9,
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
Tasks: T-1114, T-1124, T-2329

## DB and VDB period cap
Rule: RustSheet returns `#NUM!` for DB with a `period`, and VDB with an `end_period`, whose integer part is above
100,000, after Excel's own argument checks, to bound per-call work on untrusted input (CLAUDE.md directive 5). Both
schedules step one period at a time (`VDB(1000,0,1E9,0,1E9)` would walk 1E9 periods). Excel's values above the cap are
unprobed, except that a DB period above 2147483647 is period 1's value and walks nothing ("DB and SYD argument domains";
`DB(1,0,1E15,1E15)` is 1), so DB's cap covers periods 100,001 to 2147483647. A lifted DB or VDB call is also charged by each element's period
(SPEC-0011 `ValueWork::Loop`, T-1820): 2^17 units at the cap, so at most 1,024 such elements per call, while small
periods lift freely (`ROWS(DB(G25:G26,1000,6,2,7))` is 2 in Excel, probes/fin-argument-modes). Within the cap the values are unchanged.
Evidence: the cap is a resource limit, not an Excel observation; probes/fin-argument-modes shows Excel lifting DB and VDB.
Status: DEVIATION (decision 0014)
Tasks: T-1820, T-2491
Implemented in: functions/financial/financial_depreciation.rs (`MAX_DEPRECIATION_PERIODS`),
functions/lift/kinds/financial.rs (`DB`, `VDB`)

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
not the rate's `#NUM!`, and the same per element (J17, J20); a date off the calendar is the exception, `#NUM!` as it
is read ("Discount securities check order").
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
Ours: since T-1372 the core functions but DB, VDB, NPV, IRR and MIRR are `Lift`, since T-1820 DB and VDB too,
charged by their period up to their own cap ("DB and VDB period cap"), since T-1821 NPV's rate, IRR's guess and
MIRR's rates too, beside cash flows read once, whole, and charged per element for their walks over the flows ("NPV,
IRR and MIRR cash-flow work cap", a DEVIATION); and EFFECT, NOMINAL, DOLLARDE,
DOLLARFR, ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT and the T-bills `LiftArrays`
(SPEC-0011); since T-1822 so are ACCRINT, the COUP* family, DURATION, MDURATION and PRICE, each element charged for its
coupon schedule walks ("Coupon schedule work cap"), and since T-1842 YIELD and the ODD* functions, charged for their
solvers and walks ("Yield and odd-coupon work cap"), and since T-1845 CUMIPMT and
CUMPRINC, charged for their bounded period sums ("CUMIPMT and CUMPRINC work cap"), and since T-1841 AMORDEGRC and
AMORLINC, charged for their bounded period walks ("AMORLINC and AMORDEGRC work cap"), and since T-1843 XNPV's rate,
XIRR's guess and FVSCHEDULE's principal, beside cash flows, dates and a schedule read once, whole ("XNPV, XIRR and
FVSCHEDULE beside their whole data", a DEVIATION for the work cap). NPV of an array
literal is 0, and typed text and booleans are not coerced (T-1373). FVSCHEDULE gives `#VALUE!` for text in the schedule (T-1373) and for a boolean principal (T-1765).
The base values of CUMIPMT, DB, DDB, DURATION, MDURATION, ODDFPRICE and PRICE are 1 to a few
ulps off, CUMPRINC about 30-40 ulps (EFFECT is exact since T-2034), and those of ODDFYIELD and YIELD are off at 1e-10 to 1e-12 relative (T-1374; TBILLEQ and TBILLYIELD are exact since T-1475, INTRATE and YIELDDISC since T-2032). The
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
Rule: PRICE, YIELD, DURATION, MDURATION and ACCRINT are `#NUM!` for a settlement (ACCRINT: an issue, first interest
date or settlement) below serial 1 in a 1900 workbook; the COUP* functions only for one below serial 0. Probed values:
-1 for every one of them (A1, A4-A13; ACCRINT's three dates at -1 and -1E15, A14-A17); -0.5 and -1E15 for COUPNUM
(A19, A2); 0 and 0.5 for PRICE, YIELD, DURATION, MDURATION and ACCRINT (A20, -c M3-N7). At 0 and 0.5 the COUP*
functions read the settlement as serial 0: `COUPPCD(0,43831,2)` and `COUPPCD(0.5,43831,2)` are 0 and
`COUPDAYS(0,43831,2)` is 180 (-c M1, M2, N1, N2, N9), while COUPNUM is `#NUM!` (A3, -b A11, -c M9) because its
previous coupon date falls before the calendar ("Coupon schedules at the calendar's edges"). In a 1904 workbook serial
0 is a date and computes (`COUPPCD(0,400,2)` = -150, `PRICE(0,43831,0.05,0.06,100,2)` = 83.347), while -1 is `#NUM!`
for COUPPCD, PRICE, YIELD, DURATION and MDURATION (probes/date-system-1904-financial A35-A41,
fin-coupon-walk-bounds-1904 A1-A3, A8, A12-A14); `COUPNUM(-0.5,400,2)` is `#NUM!` (-1904 A4).
Evidence: probes/fin-coupon-walk-bounds A1-A20, -b A11 (excel-graph, 2026-10-17, T-1822), fin-coupon-walk-bounds-c
M1-N9 and -1904 (excel-graph, 2026-10-20, T-1844).
Ours: `Model::check_settlement` (functions/financial/dates.rs) makes a settlement below 1900 serial 1, as
`financial_date` reads it, `#NUM!` for PRICE, YIELD and DURATION; `Model::coupon_args` one below 0 for the COUP*
functions; ACCRINT already checked its dates against the calendar. Before T-1822 the engine walked the coupon
schedule back past 1900 and returned a number (`COUPNUM(-1,43831,2)` was 241), and `COUPNUM(-1E15,43831,4)` stepped
about 10^6 quarters back to chrono's first year before it failed. The checks keep every schedule walk inside the
calendar, at most about 32,400 quarters.
Status: VERIFIED (excel-graph)
Tasks: T-1822, T-1844

## Coupon schedules at the calendar's edges
Rule: the coupon schedule walk reads its dates as Excel serials of the 1900 calendar: 1900-01-01 is serial 1, a date
from 1 March 1900 on counts Excel's fictitious 1900-02-29, and a coupon date before 1900-01-01 is serial 0. So:
- COUPPCD is 0 when the previous coupon date falls before 1900-01-01 (`COUPPCD(1,1000,2)` is 0, not 1899-09-26;
  `COUPPCD(1,DATE(9990,6,30),4)` is 0), and COUPNUM, PRICE, YIELD, DURATION and MDURATION are `#NUM!` there
  (`COUPNUM(1,1000,2)`, `COUPNUM(DATE(1900,3,1),DATE(1910,3,31),4)`, every quarterly bond from serial 1 or 1900-03-01
  to a maturity in 9990-9999). That, not a coupon count cap, is the rule behind the calendar-spanning `#NUM!` cells.
- A schedule date on 1900-01-01 is serial 1: `COUPNUM(1,43831,2)` is 240, `COUPPCD(1,43831,2)` 1 and `COUPNCD` 183
  (1900-07-01, counting 1900-02-29).
- COUPDAYBS counts from the previous coupon date as read, 0 before the calendar: `COUPDAYBS(1,1000,2)` is 1 on bases
  0 and 1. The 30/360 bases read serial 0 as 1900-01-00 and serial 60 as 1900-02-29, so `COUPDAYBS(60,36585,4)` is -1
  (from 1900-02-28, the month's last day); the actual bases count serials (`COUPDAYBS(61,36585,4,1)` is 2).
- COUPDAYSNC on basis 0 is the period's 30/360 length less COUPDAYBS (`COUPDAYSNC(1,1000,2)` is 85), on basis 1 the
  serials to the next coupon date.
- COUPDAYS on basis 1 counts its own period ("COUPDAYS basis 1 counts from the chained next coupon"): one that starts
  before 1900-01-01 in the real calendar (`COUPDAYS(1,43830,4,1)` is 90, 1899-12-30 to 1900-03-30;
  `COUPDAYS(1,1000,2,1)` 181), one inside it by serials (`COUPDAYS(1,43831,2,1)` is 182), and 1900-02-28 is not a
  month end there: `COUPDAYS(59,36585,4,1)` and `COUPDAYS(1,43890,4,1)` are 90 and 92.
- A month-end schedule's February 1900 ends on the 28th (`COUPPCD(59,36585,4)` is 59).
Evidence: probes/fin-coupon-walk-bounds A21-A28, -b A1-A13 (excel-graph, 2026-10-17, T-1822), fin-coupon-walk-bounds-c
A1-L14 (excel-graph, 2026-10-20, T-1844).
Ours: matches (functions/financial/financial_bonds/coupon_schedule.rs: `coupon_period`, `coupon_count`,
`coupdays_actual`, `coupon_days_nc`). In a 1904 workbook
`COUPNUM(0,400,2)` and `COUPNUM(1,400,2)` are `#NUM!` (previous coupon 1903-08-04, before the 1904 calendar) while
`COUPPCD(1,400,2)` is -150 (-1904 A5-A7); see "COUPNUM in a 1904 workbook" (ours matches since T-2337).
Status: VERIFIED (excel-graph)
Tasks: T-1822, T-1844, T-2124, T-2125

## ACCRINT at the calendar's start
Rule: ACCRINT reads its dates as Excel 1900 serials too, and is `#NUM!` when a coupon date it accrues over falls before
1900-01-01 (serial 0): the coupon date before settlement (the regular one, `first_interest` less a period, when
settlement is not past `first_interest` or calc_method is FALSE) or any period start on the walk back to the issue.
- `ACCRINT(DATE(1900,3,1),DATE(1900,6,30),S,0.05,1000,4,0,TRUE)` is `#NUM!` for every settlement S from 1900-05-01 to
  9999-12-31 (the issue's period starts 1899-12-31), with calc_method FALSE, on bases 0, 1, 3 and 4 and with semiannual
  or annual coupons; `ACCRINT(DATE(1900,3,1),DATE(1900,3,31),DATE(1900,3,15),...)` too (the regular period starts
  1899-12-31). `ACCRINT(DATE(1900,4,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)` is 404512.3611111111.
- A regular period that starts before 1900 does not matter when the settlement's period is later:
  `ACCRINT(33448,1,37714,1E+300,0.3,1,2,TRUE)` is a number (gen-m9 3b7990bd4e3307ba Calc!A317).
  On basis 1 it does: basis 1 reads the length of first_interest's regular period, so that period must start on
  1900-01-01 or later however late the settlement is. `ACCRINT(33853,FI,37554,1,9888,freq,1)` is `#NUM!` for FI 1, 1.005
  and 2 at frequencies 1, 2 and 4, and a number for FI 367 (regular period from 1900-01-01 when annual); bases 0, 2, 3
  and 4 give numbers for all of them (probes/accrint-calendar-start-1900 A1:A60; gen-m9 932b33b1ef8b7608 Calc!A118 is
  FI 1.005, annual, basis 1; T-2562).
- A first_interest of serial 59 (1900-02-28) or 60 (Excel's 1900-02-29) with an issue of 1, 30 or 45 is `#NUM!`,
  semiannual and quarterly, on bases 0 and 1, settlement before or after it: the regular period starts in 1899
  (probes/accrint-calendar-start-1900 B1:B32). Those cells did not show how serial 59 and 60 act in the schedule;
  probes/accrint-eom-1900 (T-2837) does. With calc_method TRUE and a settlement past first_interest (semiannual
  settlement 1900-08-30, quarterly 1900-05-30):
  - An issue of 58.5 is `#NUM!` for first_interest 59 and 60 on bases 0 and 1.
  - When the issue is not before first_interest (issue 59 with first_interest 59, issue 60 with first_interest 59
    or 60), basis 0 is a number (`ACCRINT(59,59,242,...,2,0,TRUE)` is 24.86111111111111, `ACCRINT(59,59,150,...,
    4,0,TRUE)` 12.36111111111111, issue 60 gives 25 and 12.5) and basis 1 is `#NUM!` (the regular period starts in
    1899).
  - Issue 59 with first_interest 60 is `#NUM!` on basis 0 as well: serial 60 stays serial 60 as the coupon date, so
    the issue (59) lies before it and the walk back reaches 1899. This does not show whether serial 59 counts as
    month-end: no cell reached a coupon date after 1900-02-28/29 that depends on it (the settlement slice uses the
    regular period's 360/frequency days on basis 0), so the question has no observable effect in these cells.
- Basis 1's cut on first_interest's regular period is exactly 1900-01-01: ACCRINT(33853,FI,37554,1,9888,freq,1) is
  `#NUM!` for FI 365 and 366 annual, 182 (1900-06-30, whose period starts before serial 1 with or without the month-end rule) semiannual and 91
  (1900-03-31) quarterly, and a number for FI 367 annual (period from 1900-01-01), 183 and 184 semiannual and 92 and
  93 quarterly (probes/accrint-eom-1900 B1:B9; the same cells on basis 0 are all numbers, C1:C9; T-2837).
- In a 1904 workbook the cut moves to 1904-01-01: with first_interest 1904-03-31 or 1904-06-30 (quarterly), an issue of
  1904-01-01, 1904-01-02, serial 0 or serial 1 is `#NUM!` for settlements 1904-02-15 and 1950-06-30, calc_method TRUE or
  FALSE, and `ACCRINT(DATE(1904,1,1),DATE(1904,3,31),DATE(1950,6,30),0.05,1000,freq,basis,TRUE)` is `#NUM!` on every basis,
  semiannual and quarterly. An issue of 1904-04-01 computes (2312.361111111111 for first_interest 1904-03-31, calc_method
  TRUE, or 1904-06-30), except with first_interest 1904-03-31 and calc_method FALSE (regular period from 1903-12-31),
  which is `#NUM!` (probes/accrint-calendar-start-1904, 47 of its 50 cells `#NUM!`; first filled 2026-10-26, re-filled
  2026-10-27 with the same values). ACCRINT reads a 1904 date as the date itself (dates.md "The 1904 date system:
  ACCRINT, COUPNUM and the TBILL functions read the date itself"), so this is the 1900 rule with the calendar starting on
  1904-01-01. An issue on serial 0 is `#NUM!` in 1904 too, also when its period starts on 1904-01-01
  (`ACCRINT(0,DATE(1904,7,1),DATE(1904,3,1),0.06,1000,2,0)`, fin-coupnum-1904-maturity-sweep C2). Ours since T-2337
  (`DateSystem::first_coupon_1900_serial` passed to `compute_accrint`).
- The day counts read the serials as Excel shows them: the actual bases count serials, so the fictitious 1900-02-29
  counts, and the 30/360 bases read serial 1 as 1900-01-01: `ACCRINT(DATE(1900,2,15),DATE(1900,5,10),DATE(1900,4,1),
  0.05,1000,4,0,TRUE)` is 6.388888888888888 (46/90) on bases 0, 1, 2 and 4, and 6.301369863013699 on basis 3;
  `ACCRINT(1,DATE(1900,7,1),DATE(1950,6,30),0.05,1000,2,0,TRUE)` is 2524.861111111111. On basis 0 serial 60
  (1900-02-29) is not February's last day (1900 is not a leap year there): `ACCRINT(60,DATE(1900,5,10),DATE(1900,4,1),
  0.05,1000,4,0,TRUE)` is 4.444444444444445 (32 days), while 1900-02-28 is (`ACCRINT(DATE(1900,2,28),DATE(1900,5,31),
  DATE(1900,4,1),...)` 4.305555555555555, 31 days).
- Excel for the web returned 0 for `ACCRINT(DATE(1900,3,1),DATE(1900,6,30),DATE(9990,6,30),0.05,1000,4,0,TRUE)`
  (fin-coupon-walk-bounds-b A9) on 2026-10-17 and `#NUM!` on 2026-10-24, in a re-fill of -b and in
  fin-accrint-1900-calendar D1; the oracle now records `#NUM!`.
Evidence: probes/fin-accrint-1900-calendar (52 cells), fin-coupon-walk-bounds-c O1-O14, -b A9 (excel-graph, 2026-10-24);
accrint-calendar-start-1900 (92 cells) (excel-graph, 2026-10-26); accrint-calendar-start-1904 (50 cells, excel-graph,
2026-10-27, T-2835, landed by T-2337).
Status: VERIFIED (excel-graph)
Implemented (T-2124): `compute_accrint` in functions/financial/accrint.rs.
Tasks: T-2124, T-2562, T-2835, T-2337

## COUPNUM in a 1904 workbook
Rule: in a 1904 workbook COUPNUM is `#NUM!` exactly when the previous coupon date is before 1904-01-01 (a negative
1904 serial); a previous coupon on 1904-01-01 (serial 0) counts. This is the 1900 rule ("Coupon schedules at the
calendar's edges") in the 1904 calendar: since 2026-10-21 COUPNUM reads a 1904 date as the date itself (dates.md "The
1904 date system: ACCRINT, COUPNUM and the TBILL functions read the date itself"). Nothing else takes this cut: COUPPCD returns the
negative serial, and COUPNCD, COUPDAYBS, COUPDAYS, COUPDAYSNC, PRICE, YIELD, DURATION and MDURATION compute with that
period. The cut is the same at frequency 1, 2 and 4; bases 1 and 3 and fractional dates are probed on the `#NUM!` side
only (A37-A40).
- Previous coupon -1: `COUPNUM(0,181,2)`, `COUPNUM(0,365,1)`, `COUPNUM(0,90,4)` are `#NUM!` while `COUPPCD` is -1
  (A1, A7, A13); `PRICE(0,181,0.05,0.06,100,2)` is 99.51677950405495 and `COUPDAYBS(0,181,2)` 1 (A23, A20).
- Previous coupon 0: `COUPNUM(0,182,2)` is 1 and `COUPPCD` 0 (A3, A4); likewise frequencies 1 and 4 (A9, A15).
- Also `#NUM!`: bases 1 and 3, settlement 0.5, maturity 181.5, a previous coupon a year back (A37-A41).
- Every maturity tried whose previous coupon is 1903-12-31 is `#NUM!`, quarterly, semiannual and annual, out to
  1912 (fin-coupnum-1904-maturity-sweep A1-A72, A77, A78), including `COUPNUM(0,DATE(1910,3,31),4)`.
  `COUPNUM(0,DATE(1908,1,1),4)` (previous coupon 1904-01-01) is 16 (A73).
- ACCRINT has its own cut: `#NUM!` when the issue is serial 0 (C1, C2, C4-C9, C11, C12, C17) or when the issue's
  previous coupon is before 1904-01-01 (`ACCRINT(10,DATE(1904,6,30),DATE(1904,3,1),0.06,1000,2,0)`, C10); an issue of 1
  or later whose previous coupon is on or after 1904-01-01 computes (C3, C15, C16, C18, C19).
- `ACCRINT(200,DATE(1904,12,31),DATE(1905,3,1),0.06,1000,2,0)` is 37.16666666666667 (C14; ours 37 until T-2337); with
  calc_method FALSE it is 37 (C21).
Before 2026-10-21 Excel for the web read the dates 2·1462 days on here too and gave 25 for `COUPNUM(0,DATE(1910,3,31),4)`
(probes/date-system-1904-financial S!A38, 2026-10-12); a re-fill on 2026-10-27 confirms the values above (T-2337).
Evidence: probes/fin-coupnum-1904-epoch A1-A41, fin-coupnum-1904-maturity-sweep A1-A79, C1-C21 (excel-graph,
2026-10-21, re-filled 2026-10-27; T-2125, T-2337), fin-coupon-walk-bounds-1904 A5-A7.
Ours: COUPNUM reads `DateReading::Calendar` and checks the previous coupon against
`DateSystem::first_coupon_1900_serial` (`fn_coupnum`, financial_bonds.rs; T-2337); ACCRINT's cut is
`accrint_coupon_serial` (accrint.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2125, T-2337

## Coupon schedule work cap
Rule: RustSheet charges a lifted COUP*, ACCRINT, DURATION, MDURATION or PRICE call for its coupon schedule walks, in
proportion to each element's schedule length, so one call lifts at most 1,024 elements (COUP*, ACCRINT) or 512
(DURATION, MDURATION, PRICE) at the calendar's longest schedule, and more over shorter ones:
`ROWS(COUPNUM(SEQUENCE(1025)*0+1,DATE(9999,12,31),4))` and
`ROWS(PRICE(SEQUENCE(513)*0+1,DATE(9999,12,31),0.05,0.06,100,4))` are `#ERROR!` where Excel lifts every element
(probes/fin-argument-modes: they lift over arrays; Excel's own limit is unprobed), while
`ROWS(PRICE(SEQUENCE(513)+39000,43831,0.05,0.04,100,2))` is 513. A walk steps one coupon period at a time, up to about
32,400 quarters inside the calendar, about 1 ms in a release build. Each walk is charged 2^16 units at that longest
schedule (`COUPON_WALK_WEIGHT`, against SPEC-0011's 2^27 visit cap) and in proportion to its period count below it:
the days between the element's earliest and latest date times its frequency (`ValueWork::CouponWalks`, T-1846), plus
one period (T-1869), so a 10-year semiannual walk is 43 units. A text date is charged as the longest schedule. A lifted call
at the cap takes 1-2 s at the longest schedule. The scalar calls are not capped.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`COUPON_WALK_WEIGHT`, `COUPON`, `ACCRINT`, `BOND_5_OF_6`,
`BOND_6_OF_7`), functions/lift/kinds/mod.rs (`ValueWork::CouponWalks`)
Tasks: T-1822, T-1846

## Yield and odd-coupon work cap
Rule: RustSheet charges a lifted YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE or ODDLYIELD call for its solver and schedule work,
in proportion to each element's coupon count plus one, so one call lifts at most 2 elements (YIELD, ODDFYIELD), 256
(ODDFPRICE) or 1,024 (ODDLPRICE, ODDLYIELD) at the calendar's longest schedule, and more over shorter ones:
`ROWS(YIELD(SEQUENCE(9)*0+1,2958465,0.05,1E300,100,4,1))` is `#ERROR!` where Excel lifts every element
(probes/fin-argument-modes, fin-yield-odd-lift: they lift over arrays; Excel's own limit is unprobed), while
`ROWS(YIELD(SEQUENCE(9)+39000,43831,0.05,95,100,2))` is 9. YIELD's and ODDFYIELD's kernels walk the coupon schedule once
(up to 4.4 and 6.3 ms at the calendar's longest schedule, release build), then the solver evaluates the price at most
397 times (ODDFYIELD: 100 Newton steps of three prices, 97 bisection prices; YIELD's Newton, since T-2391, at most 100
steps of a price and a slope), each a sum over up to 32,400 quarterly coupons
(0.55 ms): about 0.23 s an element at that bound, charged 2^26 units against SPEC-0011's 2^27 visit cap (2 elements).
Where the discount's power takes its slow path near overflow (a price of 1E-100) a price term costs about 340 ns, not
17 ns, so the weight is fitted to those inputs (T-2879, release build, one call at the cap): 161 elements of
`YIELD(43831,80355,0.05,1E-100,100,4)` take 4.3 s, 535 of `YIELD(43831,54788,0.05,1E-100,100,4)` 0.75 s, 3,085 of
`YIELD(43831,47484,0.05,1E-100,100,2)` 2.6 s and 89 of `ODDFYIELD(43831,DATE(2200,1,1),43800,DATE(2020,4,1),0,1E-100,100,4)`
4.2 s; one element more is `#ERROR!`. At 2^24 those took 10.9 s to 17.3 s. ODDFPRICE walks and sums
its odd first period once (up to 6.3 ms, 8 walks of `COUPON_WALK_WEIGHT`: 256 elements took 1.7 s), ODDLPRICE and
ODDLYIELD walk to the maturity once in closed form (up to 2.2 ms, 2 walks: 1,024 elements took 2.2 s). Below the longest
schedule each element is charged in proportion to its coupon count plus one, the extra period covering a pass's fixed cost
(`ValueWork::CouponWalks`, T-1869): a 10-year semiannual YIELD is 43,501 units (3,085 elements), and a one-period one
2,093. An ODD* or YIELD element its kernel refuses before any walk is charged 1 unit (T-2815, T-2879 for YIELD): every ODD*
kernel reads all its arguments first (`Model::odd_args`), as YIELD does (`Model::bond_args`), returning on an error
or a logical in any of them and on a date outside [0, 2958466) as it reads it, so `ODDFYIELD(45535,45661,X,38502,{22 errors},82,110,4)` with an issue X near -1.2E14
lifts its 22 elements (gen-m9/ba4ddb817a8e700e Calc!A323, Excel `#SPILL!` there) where it was charged 22
calendar-long solvers. A text argument of any of these kernels, and of the COUP* functions, PRICE, DURATION,
MDURATION and ACCRINT, is charged as the engine reads it (`Model::cast_number`, T-2879): a date text as its date, a text that reads
as no number 1, as it is `#VALUE!` before any walk (gen-m9/0a957119475760c5 Calc!A116, whose settlement texts `a*`,
`ABC`, `x~*`, `abc` were each charged a calendar-long walk).
The scalar calls are not capped. Before T-1842 YIELD and ODDFYIELD walked the schedule on every solver step (about
2.4 s a scalar call at the solver bound, 0.6 s measured); the walk is now hoisted out of the solver, with the same values.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`YIELD_SOLVER_WEIGHT`, `YIELD`, `ODDFPRICE`, `ODDFYIELD`, `ODD_LAST`),
functions/lift/kinds/mod.rs (`ValueWork::CouponWalks`)
Tasks: T-1842, T-1869, T-2815, T-2879

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
G6, G7), but a start below 1 is `#NUM!` as given, before it rounds up (0.3, 0.5, 1E-15; T-2489, gen-m9 sweep below). Lifted elements are the scalar calls (A29, A30).
Evidence: probes/fin-cumipmt-period-bound A1-A30 (excel-graph, 2026-10-18, T-1845), probes/fin-remaining-later-argument-
ranges K1-K3 (T-1822), `ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx`.
Ours: since T-1845 (functions/financial/cumulative.rs) up to 32 periods are main's per-period IPMT/PPMT sum, which matches
CUMPRINC_CUMIPMT.xlsx G3, G9, G15 and G26 where the closed form below is an ulp or two off; past 32 periods, or when PMT's
fitted `pv*(1+rate)^nper` overflows (T-1405) or a period fails, the closed form `pmt = -pv*r/(-expm1(-nper*L))` (over `1+r` for type 1),
`principal = pmt*((exp((end-nper)*L) - exp((first-1-nper)*L))/expm1(L))` with `L = log1p(r)` (type 1: period 1 adds the
whole payment and the sum starts at period 2), interest = `pmt*(end-start+1) - principal`. The sum is no closer to
Excel past 32 periods (83 ulps over 1,024 periods, A11; no correct digit over 10^5 periods of type 1, A5), while the
kernel passes 22 of the 30 probe cells and misses the others by at most 6 ulps, but for 48 to 64 near period 1E15 (A18,
K3). The fit tried `(1+r)^m` by libm and by Excel's `^` against `exp(m*log1p r)`, four payment and four balance
parenthesizations, and `r` against `expm1(log1p r)` as the denominator (72 cells): no single form matches both the short
sums and the long ones. Before T-1845 the engine looped over every period (10^9 steps for A1), was `#NUM!` once
`(1+rate)^nper` overflowed (A13-A19, K1-K3) and computed A21.
T-1890 (probes/fin-cumipmt-overflow-and-forms A1-A14, excel-graph 2026-10-20):
- A payment that overflows is `#NUM!` for CUMIPMT (`CUMIPMT(1E300,1E308,1E300,1,1E308,0)`, A1;
  `CUMIPMT(0.5,10,1E308,1,10,0)`, A3) but not always for CUMPRINC: `CUMPRINC(1E300,10,1E300,1,10,1)`
  is -1E300 (A2, ours `#NUM!`) and `CUMPRINC(0.5,10,1E308,1,10,0)` -1E308 (A4).
- Excel's interest is not the exact sum: one period, `CUMIPMT(0.01,100,100000,100,100,0)`, is
  -15.708656559792416 where the exact value is ...592 (A12), and `CUMIPMT(0.01,1E15,100000,1E15,1E15,0)`
  -9.900990099009732 where it is ...901 (A11).
- A 1E-15 rate past period 2^53 has its own digits: `CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)` is
  -8.999591419181679E-15 and the CUMPRINC -4.085993687045525E-19 (A5, A6).
- Its principal is within a few ulps of the exact sum, also where it is a tiny part of the payments
  (CUMPRINC_CUMIPMT.xlsx H5 -0.0047565423422487076).
Ours since T-1890: the principal is the closed form at any length. Over up to 32 periods its
difference of powers is written `exp((first-1-nper)*L)*expm1((end-first+1)*L)`, which does not cancel.
The plain difference of powers is kept in two places. One is where that `expm1` overflows (a rate of
1E10 over 31 periods: A15-A19). The other is where the first power is subnormal or underflows, because
the product would lose its digits there. For example, `CUMPRINC(1E10,32,1,1,25,0)` is
-9.99999999300011E-71 in Excel and ...3000182E-71 here, where the product gave -9.999888696826292E-71.
The other cells of that kind are `(1E10,32,1,1,10,0)` -9.999999978000387E-221, `(1E10,31.5,1,1,20,0)`
-9.999999988500136E-116, and `CUMPRINC(1E10,40,1,1,20,0)` and `(…,15,20,0)` -9.999999980000501E-201
(ours ...0004E-201 for A23 and ...0572E-201 for A24, exact ...0000E-201): probes/fin-cumipmt-overflow-and-forms A20-A24, excel-graph
2026-10-21. Their last digits are T-2192's.
The closed form gets H6, H22 and H25 to the bit ( `CUMPRINC(0.01,360,100000,1,12,0)` -362.8793467231003, Excel
-362.8793467231; superseded for type 0 by T-2192 below, now ...0024). The PPMT sum it replaces lost a small principal's digits (H5 8 right, H7 7). The
interest keeps the IPMT sum up to 32 periods.
T-2192 (float64 fit against the cells above; excel-graph and excel-cached):
- Excel's errors on the 1E10-rate cells follow the rounding of `m·ln(1+r)`: the relative error is
  up to half an ulp of that product (5E-14 for A23, where `21·L` is about 483). Type 0's principal
  as the per-period sum of `pmt·exp((k-1-nper)·L)`, `pmt = -pv·r/(1-v^nper)`, is within an ulp of
  Excel on A16 and A20-A24 (bit-exact on A20, A21), and 0 to 4 ulps off on the other type 0
  CUMPRINC cells probed: fin-argument-modes H80 and fin-remaining-later-argument-ranges I75-I83
  4 (the closed form was 5), CUMPRINC_CUMIPMT.xlsx H29 3 (the closed form was 1), H21 1 (was 2).
- Type 0's interest over a short range is the balance form `-(pv·q·r - pmt·(q-1))`, `q = exp((k-1)·L)`,
  summed by period: A12 and A14 to the bit, H72, A13 and G29 within 4 ulps. It overflows past a
  rate of 1E10 over 40 periods and near period 1E15.
- Excel's CUMIPMT equals `-pmt·m - CUMPRINC` in doubles (`m = end-start+1`) on every cell probed
  with both functions, to the bit on K3, A9, G2, G4, G5, G6, G21 and G22 (type 0 with
  `pmt = (pv/(1-v^nper))·r`: all 13 pairs). So its digits are its principal's.
- `CUMPRINC(1E300,10,1E300,1,10,1)` is -1E300 although `pv·r` overflows: the payment is
  `pv·(r/(1+r))/(1-v^nper)` there (A2). As type 1's payment everywhere, that form gets H3, H16 and
  H31 to the bit but misses H25 by an ulp.
Ours since T-2192: over up to 32 periods of type 0, the principal is that per-period sum and the
interest the balance form (the closed form where it overflows), and type 1's principal takes the
`r/(1+r)` payment where `pv·r` overflows. Type 1's interest keeps the IPMT sum, and its principal
the closed form: the per-period form misses H22 and H25, which pass.
Past 2^53 Excel stays accurate: `CUMPRINC(1E-15,2E16,1,1E16,1E16+8,0)` (A6) is within an ulp of
the correctly rounded -4.0859936870455272E-19, and A5's CUMIPMT equals that value's
`-pmt·9 - P` to the bit, so its sums keep digits that a double `n·L` exponent with n near 1E16
loses (60-digit reference, T-2363 notes). Type 1's G2 is `-pmt·10 - H2` to the bit, with
`pmt = pv·r/(-expm1(-nper·L))/(1+r)` and Excel's own H2.
T-2363 (float64 fit against the 52 type 0 CUMIPMT cells of the probes, CUMPRINC_CUMIPMT.xlsx and the gen-m9
sweep):
- Excel's type 0 interest carries the cancellation of `-pmt·m - CUMPRINC` also where the exact sum is easy:
  `CUMIPMT(1E-9,55,157761,1,18,0)` is -0.002400835590378847 to the bit with that form, where the exact sum is
  -0.0024008355913268677 (gen-m9 5c6a23c4f6597f1c A392), `CUMIPMT(1E-10,296,197540,13,19,1)` is 6.7E-9 relative
  off the exact sum the same way (b72d952c5f52cb4d A257, type 1), and at huge rates the interest is the payments
  `-pmt·m` with no trace of the balance: `CUMIPMT(10,89,267388,3,13,0)` is -29412680, `CUMIPMT(1E10,235,99106,6,18,0)`
  -1.288378E16, `CUMIPMT(100,167,7,6,16,0)` -7700 (1cd3096e3e87c965 A316, 3c06f9298f65a3ba A140, e168be7f6d3f84c2 A330).
- Where `1 + rate` is 1 the principal is the drop in the balance `(pv/(1-v^nper))·v^(nper-start+1)·expm1(m·L)`,
  not the payment added m times: `CUMIPMT(1E-300,243,80923,12,22,0)` is -4.547473508864641E-13 and
  `CUMIPMT(1E-300,83,156759,5,17,0)` 0 to the bit with it (81bcdc0bec0c8e6b A274, abb7d565af89b849 A84); the same
  principal gets A5 (`CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)`) to the bit.
- The principals of integer rates (`CUMPRINC(10,261,484823,4,24,0)` -7.507871520276804E-242, ours 15 ulps off;
  `CUMPRINC(1,200,480533,5,18,0)` -7.838581871529754E-50, 1 ulp below the exact ...755, ours 11 ulps off) are no
  `exp(m·log1p r)`, libm `pow`, `exp2(m·log2(1+r))`, repeated product or squaring form tried, with any of six
  payment forms, nor `log1p r` moved by up to six ulps: Excel's power is 1 ulp off the exact value for base 2 and
  311 ulps for base 11 (T-2514).
Ours since T-2363: type 0's interest is `-pmt·m - principal` with `pmt = (pv/(1-v^nper))·r`; the principal is the
per-period sum over up to 32 periods (the balance drop above where `1 + rate` is 1, and past 32 periods). Over up
to 32 periods the balance form is kept while the growth `(1+r)^(end-1) - 1` stays within [2^-10, 2^10], where it is
bit-exact on A12, A14 and G21 and our principal is an ulp off Excel's (A12 would be 256 ulps off, G29 12). 18 visibly
off cells are right (gen-m9 A316, A81 twice, A140, A189, A343, A392, A49, A365, A274, A372, A5, A83, A84, A162, A169,
A330; probes A5); none regressed.
T-2514 (float64 fit against the CUMPRINC cells of the gen-m9 sweep, the probes and CUMPRINC_CUMIPMT.xlsx):
- Excel's short principal is one geometric sum from the last period back,
  `-((pmt·((1-v^m)/r·(1+r)))·v^(nper-end+1))` with `pmt = (pv/(1-v^nper))·r`, `v^j = exp(-j·L)`,
  `1-v^m = -expm1(-m·L)`, in that operation order. Type 1 sums from period 2 and divides that by `1+r`.
  - It is within an ulp of Excel on the 5 integer-rate type 0 principals, which no power form fitted
    (T-2363). `CUMPRINC(10,261,484823,4,24,0)` is -7.507871520276804E-242 to the bit, 311 ulps from the exact
    sum (gen-m9 9a1fa5366bc50918 A361; A18, 67a4c03afd63f988 A110, 1cd3096e3e87c965 A147, d0854366830dcdee A98).
    So Excel's "power" is not a power at all: the one `exp` of the smallest discount carries the error.
  - It is bit-exact on these short type 1 principals (every one probed but A372 below), where the closed form was 11 to 104 ulps off:
    `CUMPRINC(10,121,308352,2,16,1)` -1.263054652346343E-105 (0a957119475760c5 A383; 5ded7594e597e31e A22,
    A267, a91bad4e84f49739 A193, b6a02893a8d6b151 A398, CUMPRINC_CUMIPMT.xlsx H26 `CUMPRINC(7,36,8000,3,10,1)`).
- At small discounts the other forms are closer. `CUMPRINC(1E-9,192,2707,8,24,0)` is -239.68227237224258 to the
  bit with the per-period sum and 2 ulps off with the geometric sum (gen-m9 d912671baa1364aa A300). H25
  (`0.1234` over 36 periods, discount exponent `(nper-first+1)·L` 3.96) is to the bit with the closed form and
  3 ulps off with the geometric sum. At exponent 8.9 and above every probed cell is as close or closer with it;
  between 3.96 and 8.9 nothing is probed.
- Type 1's interest at a tiny rate is the payments less the principal, `-pmt·m - CUMPRINC` with
  `pmt = pv·r/(1-v^nper)/(1+r)`. `CUMIPMT(1E-10,296,197540,13,19,1)` is -1.3127066813467536E-4 to the bit
  (b72d952c5f52cb4d A257, and the STDEVA over one at 6e18550e9fc9c1b9 A329). At ordinary rates the IPMT sum stays
  bit-exact (G3, G9, G15, G12, G16, G17, G25; T-2363 notes).
- `CUMPRINC(1E-300,129,257125,1,17,0)`, a rate lost in `1 + rate`, is the balance drop that the interest already
  used: `DOLLARDE` of it, -33888.31201550391, is to the bit (01523d247594952e A322).
- Not fitted:
  - `CUMPRINC(2,151,319288,3,42,1)` -1.049166891970377E-47, a 40-period type 1 range (b73c6cc96e093f03 A62). The
    closed form is 103 ulps off and the geometric sum 245.
  - `CUMPRINC(0.1,1023,478581,7,24,1)` -1.589128180790285E-36 (9d9ceba257f8001b A372). The geometric sum is 67
    ulps off; it is to the bit only with `log1p(0.1)` one ulp low, which no log1p rewrite tried gives.
  - `CUMIPMT(1E-15,283,(1.005+3.14159+1E15)%,2,16,0)` -0.1456298828125 (79d87fc527927626 A352; ours
    -0.14581298828125, 3 ulps of `pmt·m`). No pairing of 10 payment forms with 5 principal forms gives it.
  - `CUMIPMT(1E-9,235,379138,7,14,1)` -0.0029104891709721414 (e168be7f6d3f84c2 A379): neither the IPMT sum nor the
    payments less the principal fits.
Ours since T-2514: a short principal (CUMPRINC, and the principal of type 0's interest) is that geometric sum where
the discount exponent is 8 or more and its power is normal (`GEOMETRIC_MIN_DISCOUNT`). Below that it is the
per-period sum (type 0) or the closed form (type 1), and the balance drop where `1 + rate` is 1. Type 1's short
interest is the payments less the closed-form principal where `nper·L` is below 2^-10 (`TINY_RATE_GROWTH`).
19 known failures are fixed, none regressed, and visibly-off cells go from 8281 to 8267. The gates have no
oracle cell near their edges (T-2521).
Status: PARTLY VERIFIED (excel-graph; visibly right but for A6, A7 (= G2), A9-A11 and K3 near period 1E15,
fin-cumipmt-period-bound A18, and the gen-m9 cells A62, A372, A352, A379 above: T-2521; the short sums' last digits
in T-1374)
Tasks: T-1845, T-1890, T-2192, T-2363, T-2514, T-2521, T-1374

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

## NPV, IRR and MIRR cash-flow work cap
Rule: RustSheet charges each lifted NPV, IRR or MIRR element for its walks over the cash flows, against SPEC-0011's
2^27 visit cap. NPV and MIRR elements cost one unit per flow. An IRR element costs 512 units per flow (`IRR_WEIGHT`),
because `compute_irr` makes at most 455 NPV passes per call: two secant runs of 52, then the fallback's Newton from the
guess (100 with its derivative passes), its bisection's 51, and its Newtons from 200 and -2 (200). A call over too
many flows times elements is `#ERROR!` before its first element, where Excel lifts every element (the lifts:
probes/fin-argument-modes H155-H156, H172-H174, H187-H188; Excel's own limit is unprobed).
`ROWS(IRR(A1:A300000,{0.1;0.1}))`, `ROWS(NPV(SEQUENCE(500),A1:A300000))` and
`ROWS(MIRR(A1:A300000,SEQUENCE(500),0.1))` are `#ERROR!`. `ROWS(NPV(SEQUENCE(100),A1:A300000))` is 100. The scalar
calls are not capped.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`NPV`, `IRR_WEIGHT`, `IRR`, `MIRR`); functions/financial/mod.rs
(`flow_count`)
Tasks: T-1821

## XNPV discounts with Excel's `^`
Rule: XNPV(r, values, dates) is the sum of value_i / (1 + r)^((date_i - date_0) / 365). The power is Excel's own `^`
(numbers.md): right-to-left square-and-multiply for a whole number of years, and exp(y·ln x) otherwise. It is
not libm's `pow`. `XNPV(r,{0,1},{45000,45000+365T})` equals `1/(1+r)^T` with Excel's `^` for all 390 cells
with r = 0.1, 0.5 and 0.013 and T = 1 to 130, where libm misses 120, 8 and 128 of them. It also holds for
130 cells 100 days past each year (t not an integer). On two-flow cells at an exact root the result is a few
ulps from 0, and its sign is Excel's: `XNPV(0.1,{-1,6.727499949325611},{45000,52300})` is 2.220446049250313E-16,
from the literal cut to 6.72749994932561 (numbers.md, "Formula number literals are read at 15 significant
digits"). That sign decides XIRR's bracket (see "XIRR is a bracketing bisection"). The division is the
quotient `/`. Twice-rounded and plain division both fit, and multiplying by (1+r)^-t misses 7 of the 21 cells
with nontrivial values. A date at serial 0 makes XNPV `#NUM!` (J1:J5), although XIRR accepts it
(xirr-early-dates A8).
Evidence: probes/xirr-exact-root-sign: 541 numeric cells, all matched by the lab's `xirr.xnpv` (tools/.../kernels/xirr.py) with
`caret_pow` and 15-digit literals (excel-graph, 2026-10-24, T-2328).
Ours: `compute_xnpv` and XIRR's `xirr_xnpv` use `arithmetic::power`, Excel's `^` (T-2566).
Unprobed: where a power overflows (`XNPV(1E+300,{-100,100},{39448,40000})`), ours adds that flow as 0 (value/inf), as before
T-2566; Excel's result is not evidenced.
Status: VERIFIED (excel-graph)
Implemented in: functions/financial/financial_util.rs (`compute_xnpv`, `xirr_xnpv`)
Tasks: T-2328, T-2566

## XNPV, XIRR and FVSCHEDULE beside their whole data
Rule: XNPV's rate, XIRR's guess and FVSCHEDULE's principal lift over an array and refuse a multi-cell range, as the other
Analysis ToolPak scalars ("Financial function argument modes"); each element is the scalar call over the same values,
dates and schedule. Ours: those are read once per call, before the elements, and each element checks its arguments in
the scalar order: XNPV's rate and FVSCHEDULE's principal before the flows or schedule, XIRR's values and dates before
its guess (unprobed, as "NPV, IRR and MIRR error order"). Each lifted element is charged for its walks over the flows
against SPEC-0011's 2^27 visit cap: one unit per value for XNPV, one per rate for FVSCHEDULE, and 512 per value for
XIRR (`XIRR_WEIGHT`: `compute_xirr` makes at most 401 XNPV passes, XNPV(0), two bracket walks of 100 points and 200
bisection steps; "The IRR and XIRR solver caps"). `ROWS(XIRR(A1:A300000,B1:B300000,{0.1;0.1}))`,
`ROWS(XNPV(SEQUENCE(500),A1:A300000,B1:B300000))` and `ROWS(FVSCHEDULE(SEQUENCE(500),A1:A300000))` are `#ERROR!` before
their first element, where Excel's own limit is unprobed. The scalar calls are not capped.
Evidence: probes/fin-argument-modes H137-H138, H338-H339, H341-H342 (the lifts and the refused ranges, excel-graph,
2026-10-11, T-1084); none for the work cap, which is a resource limit (CLAUDE.md directive 5).
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`XNPV`, `XIRR_WEIGHT`, `XIRR`, `FVSCHEDULE`);
functions/financial/mod.rs (`fn_xnpv`, `fn_xirr`, `fn_fvschedule`, `flow_count`)
Tasks: T-1843

## NPV, IRR and MIRR error order
Rule (ours, unprobed for IRR and MIRR): NPV reads its rate before its cash flows, so a rate error wins over an error in
the flows (probed for NPV: `NPV(NA(),"x")` is #N/A, `NPV("x",NA())` #VALUE!, probes/npv-power-edges-2 A41-A43). IRR
and MIRR read their flows first, so a flows error wins over a guess or rate error. This is the order the scalar calls
had before T-1821. When the rate lifts, an error in the flows is every element's error, after that element's own
earlier arguments. The flows are evaluated once per call, before the elements, even when NPV's rate is an error. The
values are the same, but it is not Excel's evaluation order, which is unprobed. Among NPV's flows, see "NPV's error
order".
Evidence: probes/npv-power-edges-2 A41-A43 (NPV); none yet for IRR and MIRR (T-2103).
Status: VERIFIED for NPV's rate-first order; UNVERIFIED for IRR and MIRR
Implemented in: functions/financial/mod.rs (`fn_npv`, `fn_irr`), functions/financial/mirr.rs (`fn_mirr`)
Tasks: T-1821, T-2103, T-2493

## NPV's error order
Rule: among NPV's cash-flow arguments, an error that is a direct argument (an error value, or text that is no number,
#VALUE!) is the result in argument order, and an error inside a range or an array (a constant one, a computed one or a
single cell) is the result only when no direct argument errs, whatever the order. Among ranges and arrays the first
error in argument order wins. Any flow error beats NPV's own `#DIV/0!` ("NPV's discount past the double range").
`NPV(1.0049,0,{1,2}/0,NA())` is #N/A, `NPV(0.1,{1,#DIV/0!},"x")` #VALUE!, `NPV(0.1,Q1:Q2,NA())` (Q2 `=1/0`) #N/A,
`NPV(0.1,Q3,NA())` (Q3 `=1/0`) #N/A, `NPV(0.1,{#N/A},#DIV/0!)` #DIV/0!, `NPV(1.0049,0,1/0,NA())` #DIV/0!,
`NPV(0.1,{1,#DIV/0!},{#N/A})` #DIV/0!, `NPV(0.1,Q1:Q2,{1,#N/A})` #DIV/0!, `NPV(0.1,{1,#N/A},Q1:Q2)` #N/A,
`NPV(1E+300,1,1,{1,#N/A})` #N/A, `NPV(-1,{1,#N/A},NA())` #N/A. This explains gen-m9 0a957119475760c5 Calc!A255, whose
third argument is a `TOROW(..)/0` array of #DIV/0! and fourth a RANK.EQ #N/A (Excel #N/A).
Evidence: probes/npv-power-edges A35-A51, probes/npv-power-edges-2 A31-A48 (excel-graph, 2026-10-25); also
probes/t2493-npv-rate-edges A26:A29, A48:A59, A71:A77 (an error flow before a failing discount, `NPV(-1,NA())` #N/A; error
cells in B3:B4 against `NA()`) and probes/t2493-npv-error-order-inner-calls (gen-m9/0a957119475760c5 A157, A255; T-2734).
Status: VERIFIED (excel-graph)
Implemented in: functions/financial/mod.rs (`npv_cash_flows`)
Tasks: T-2493

## NPV's discount past the double range
Rule: NPV divides each value v_i by d(i-1), the running product (1+rate)^(i-1) kept as a double, and the sum once by
1+rate (the next entry, "NPV's residue at IRR's root is not the double sum"). It is
`#DIV/0!` when 1+rate is 0 (a rate of -1, or one that rounds to -1 such as -1+1E-100, with any flows, 0s included:
`NPV(-1,0)`, `NPV(-1,{1,2})`) and when the power some later period builds on overflows or falls below the smallest
normal double. The last period's own power is never formed, so its overflow loses nothing: `NPV(1E+300,1,1E+300)` is
2E-300, `NPV(1E+300,0,0)` 0 and `NPV(1E+200,1,1)` 1E-200, but `NPV(1E+200,1,1,1)`, `NPV(1E+300,0,0,0)`,
`NPV(1E+300,1,,)` and `NPV(9E+307,1,1,1)` are #DIV/0!; `NPV(1E+154,1,1,1)` is 1E-154 (1E+308 is in range).
`NPV(1,SEQUENCE(1024))` is 1.9999999999999998 and 1025 flows are #DIV/0!; `NPV(-0.5,<1023 zeros>)` is 0 and
`NPV(-0.5,<1074 zeros>)` #DIV/0! (consistent with a subnormal power read as 0; 1024 to 1073 zeros unprobed). A
subnormal quotient is 0 (`NPV(9E+307,1)` is 0), and an overflowing sum is #NUM! (`NPV(-0.5,9E+307,1)`). Unprobed: a partial sum that overflows before
the final division brings it back in range (`NPV(1,1.5E+308,1.5E+308)`: #NUM! in doubles, about 1.1E+308 if Excel
sums in a wider format), and a subnormal quotient that the final division would scale up (T-2953). A rate below
-1 is fine (`NPV(-2,1,1)` 0, `NPV(-3,1,1,1)` -0.375). The literal 1E+308 in a probe formula makes Excel Online
reject the workbook ("unsupported features"), so these probes use 9E+307.
Evidence: probes/npv-power-edges A1-A34, probes/npv-power-edges-2 A1-A30 (excel-graph, 2026-10-25); probes/t2493-npv-rate-edges
A1:A14, A30:A47 and A60:A70 (a typed -0.9999999999999999 is cut to 15 digits, numbers.md "Formula number literals are read at
15 significant digits"; 20 flows of 1 give 1.016120504184532E+300, 21 give #NUM!,
22 and more #DIV/0!; T-2734); the T-2436 and
T-2496 sub-expression probes (NPV(-1,..) and NPV(1E+300,..) cells).
Ours: matches since T-2493. Since T-2672 the power is Excel's running product d(i) = d(i-1)*(1+r) (financial.md "NPV's
residue at IRR's root is not the double sum"), and the last period's power is never formed: the sum is divided by
1+r once, at the end. probes/npv-power-edges and npv-power-edges-2 stay green under it.
Status: VERIFIED (excel-graph)
Implemented in: functions/financial/financial_util.rs (`excel_npv`)
Tasks: T-2493

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
not exactly that walk at that size (it subtracts in x87 extended precision, a walk that does gives all of A1-A5:
"AMORDEGRC rounds its charge like ROUND"); `AMORDEGRC(1E300,..,2000000,1E-7,1)` is 1.51632679480347E293, 15 significant digits
(-c A5, as -b B2). Excel finishes 10^9 periods (`AMORLINC(2400,39679,39813,300,1E9,1E-9,1)` is 0, -b A1), but
`AMORLINC(2400,39679,39813,300,1E15,1E-300,1)` did not finish within the excel-graph timeout: a fill of a workbook
holding only that cell failed after 8.5 minutes (createSession retries exhausted, 2026-10-18), as did, twice, a larger
one with periods of 1E12 to 1E308 at rates of 1E-13 to 1E-300; neither is in the suite. A period past the life,
`ceil(1/rate)`, is 0 at once (fin-remaining-later-argument-ranges K4, K5).
Evidence: probes/fin-amor-period-bound A1-A13, B1-B18, probes/fin-amor-period-bound-b A1-A3, B1, B2 and
probes/fin-amor-period-bound-c A1-A6 (excel-graph, 2026-10-18, T-1841).
Ours: since T-1841 (functions/financial/amortization.rs) both walks are bounded. AMORLINC keeps main's value for every
period main finished, and AMORDEGRC up to 2^22 periods (2^20 before T-2918). AMORLINC's walk is exact for any period: while the remainder stays in one binade, the subtraction rounds the
charge to that binade's grid the same way each time, so after two equal steps the next ones are jumped in one
multiplication, and a charge below half an ulp of the remainder stops it for ever (at most 616 single steps over every
double; `linear_charge`). AMORDEGRC walks up to 2^22 periods one by one, stopping early where the rest is fixed (below
the salvage, a charge that rounds to 0 before the half period, a remainder the charge no longer moves); past that it
finishes in closed form, `remain * exp(n * log1p(-rate))` ("AMORDEGRC past 2^22 periods"). Main's counter
`counted += 1.0` stopped moving past 2^53, so main never finished there; where Excel did not finish within the
excel-graph timeout, the kernel returns the value the walk reaches (a DEVIATION: `AMORLINC(2400,39679,39813,300,1E15,
1E-300,1)` is 2.4E-297, the charge, and `AMORDEGRC(1E300,39679,39813,300,1E15,1E-300,1)` is 3, the charge 2.5 rounded).
AMORDEGRC at a period below 1 (B17, 485: the value left over from B18, "AMORDEGRC below period 1"; ours since T-2825) and at a cost of 1E308 (-b B2) are base values, T-1895.
Status: VERIFIED (excel-graph) for AMORLINC's walks and AMORDEGRC's up to 2^22 periods; DEVIATION (decision 0014) past
2^22 AMORDEGRC periods and where Excel did not finish
Tasks: T-1841, T-1895

## AMORLINC and AMORDEGRC check order
Rule: AMORLINC and AMORDEGRC read cost, date_purchased, first_period, salvage, period and rate in order, each a number (an
error, text or boolean is returned; text "1" is 1), and a date that does not floor to 0-2958465 (9999-12-31) is `#NUM!` as it
is read (`AMORLINC(2400,-1,NA(),..)` and `AMORLINC(2400,39679,2958466,NA(),..)` are `#NUM!`, `AMORLINC(NA(),-1,..)` `#N/A`;
dates 0 and 0.5 pass, `AMORLINC(2400,0.5,NA(),..)` is `#N/A`). Then the values are checked, all `#NUM!`: a negative cost or
salvage, a salvage not below the cost, a negative period, a rate of 0 or less, a purchase after the first period (one on
it is valid: `AMORLINC(2400,1,1,300,1,0.15,1)` is 360). Only then the basis is read: its error, text or boolean, then
`#NUM!` for any basis but 0, 1, 3 or 4 (`AMORLINC(-1,39679,39813,300,1,0.15,NA())` is `#NUM!`). AMORDEGRC's asset-life check
comes last (`AMORDEGRC(2400,39679,39813,300,1,0.5,NA())` is `#N/A`). A purchase date below 1 is serial 0, the day before
serial 1: `AMORDEGRC(2400,0,366,300,1,0.15,1)` is 562 (366 days), `AMORDEGRC(2400,0,1,..)` 899, `AMORDEGRC(2400,1,366,..)` 563,
`AMORLINC(46557,0,39881,440,0,0.1)` 46117.
AMORDEGRC with a period strictly between 0 and 1 returns a value the previous AMORDEGRC or AMORLINC left behind
("AMORDEGRC below period 1" below).
From 1 on a fractional period
truncates (1.5 is period 1, 2.9 period 2), and AMORLINC reads any period above 0 and up to the full periods as a full charge
(`AMORLINC(..,0.0001,0.15,1)` is 360).
Evidence: probes/amor-argument-check-order (866 cells: each out-of-domain argument against `NA()`, text and `TRUE` in every
other position, `NA()` and `1/0` in every pair, booleans and text alone, purchase dates below 1, fractional periods;
excel-graph, 2026-10-24, T-2485) and probes/amordegrc-fractional-period (32 cells). Every order cell fits the rule.
Dates -1, -0.5, 0, 1E-15, 0.5, 0.99, 1 and 2958466 were probed (-0.5 is `#NUM!`, A410-A412, A843-A845); 2958465.5 is
inferred (the floor is taken from the shared reader's other functions).
Ours: since T-2485 (`amortization_of` and `amor_date` in functions/financial/amortization.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2485, T-1895

## AMORDEGRC below period 1
Rule: AMORDEGRC with a period strictly between 0 and 1 computes no charge of its own: it returns a value left behind by
the last AMORDEGRC or AMORLINC that Excel evaluated before it in the workbook, and 0 when none did. The value is
workbook-wide state written in Excel's calculation order:
- Alone in a fresh workbook every such formula is 0: `AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)`,
  `AMORDEGRC(2400,39679,39813,300,0.5,0.15,1)`, `(..,0.0001,..)` and `AMORDEGRC(98882,37287,38647,1E-13,0.5625,0.1,1)`
  (probes/amordegrc-fraction-alone-a to -d).
- An AMORDEGRC from period 1 on leaves its own value: 485 from `AMORDEGRC(2400,39679,39813,300,2,0.15,1)`, 776 from
  `(..,1,..)` (alone-f, fraction-state-l to -q). A period-0 AMORDEGRC leaves the value as it was, not its own
  46117: `AMORDEGRC(46557,0,39881,440,0,0.1)` passes the 485 below it through (fraction-state-z; alone, -t, 0).
- AMORLINC leaves its first-period charge rounded to a whole number, whatever its period:
  `AMORLINC(2400,39679,39813,300,p,0.15,1)` is 360 for p 1 and 3 and 131.803… for p 0, and leaves 132 for each
  (fraction-state-j, -w, -s); bought on its first period's end, `AMORLINC(2400,39813,39813,300,1,0.15,1)` leaves 360
  (-y). Other functions leave the value as it was: `EFFECT(1,1)` (1) and `SUM(1,2)` pass the 485 below them through
  (-aa, -ab), and alone `EFFECT(1,1)` and a constant 7 below the fraction leave it 0 (-i, -k).
- An AMORDEGRC or AMORLINC that is an error, or past the asset life (period 20 of a 0.15 rate, value 0), leaves the
  value as it was: an early `#NUM!` (negative salvage, -m), one from the asset-life check (rate 3.1E26, -l) and both
  functions at period 20 (-u, -v) pass the 485 below them through, and alone an asset-life error (rate 1) leaves
  0 (-h). An error after the arguments are checked (a NaN AMORLINC charge) is unprobed.
- Lifted over an array, every fractional element is 0: `SUM(AMORDEGRC(..,{0.3,0.5},..))` above the 485 charge is 0
  (-x). What a lifted call leaves is unprobed.
- The order is Excel's fresh-load order (recalc.md "The order Excel calculates a freshly loaded workbook in"): the
  value comes from the cell below (-p: A1 is 776 from A2, not A3's 485), the cell to the right (-q: A1 is B1's 485)
  or a later sheet (-n: Sheet1!A1 is Sheet2!A1's 485); a precedent is evaluated first wherever it sits (-o: A2's
  period reads A1, so A2 is A1's 485, where without the reference it is 0, alone-e).
Every earlier cell fits this reading: probes/amordegrc-fractional-period A1-A11 are 776, the value of A12 (period
1.0001) below them; A23-A26 are 1693, A24's and A26's from A25 and A27 (a different asset, whose own period 1 is 2284,
A28); fin-amor-period-bound B17 (period 0.5) is 485, B18's value; amor-argument-check-order A846-A851 are 776 from A852
(period 1.5), and A862-A864 are 0: A866's period-0 AMORDEGRC, the first cell Excel runs, and the error in A865
leave the state empty. gen-m9 d912671baa1364aa A76
(period 0.5625) is 1, which the table does not explain: the last AMORDEGRC or AMORLINC after it, A96
(`AMORLINC(92598,36663,39379,343,6,1E-300,3)`), leaves 0 (fraction-state-r), and its other AMORDEGRCs and AMORLINCs are
errors.
Evidence: probes/amordegrc-fraction-alone-a to -g (excel-graph, 2026-10-26, T-1895) and
probes/amordegrc-fraction-state-h to -ab (excel-graph, 2026-10-27, T-2825), with the earlier cells above.
Ours: since T-2825 (decision 0131), `OrderState::amor_left_over` in evaluation.rs, set and read by
`Model::amortization_call` (functions/financial/amortization.rs, `LeftOver`); a read runs the pass in Excel's order
(`excel_order.rs`, SPEC-0017), whose move budget and its DEVIATION (dates.md) apply here too. gen-m9 d912671baa1364aa
Calc!A76 stays a known failure.
Status: VERIFIED (excel-graph) on freshly loaded workbooks; after an edit in a live session the order is not
reproducible from the file (recalc.md), what a lifted call and a dynamic-array formula's place in the order do to the
value are unprobed (ours: the last element's value; left as it is), and gen-m9 A76 is UNEXPLAINED
Tasks: T-1895, T-2825

## AMORDEGRC past 2^22 periods
Rule: past 2^22 periods (4,194,304) RustSheet computes AMORDEGRC's decay in closed form, `remain * exp(n *
log1p(-rate))` from the state after 2^22 walked periods, instead of walking on. Excel walks every period, adding a
rounding each time (twice rounded at each subtraction, "AMORDEGRC rounds its charge like ROUND", T-2849), so the closed
form drifts from the walk by up to about 1e-13 relative over 10^6 further periods: the same once rounded for charges
below about 1E11, but a larger charge can be a unit off (a review replay found one at about 3.1E11; at 1E12 or more it
often is). Up to 2^22 periods the kernel is Excel's walk.
Until T-2918 the bound was 2^20, and the closed form made 4 of the 6 probes/fin-amor-period-bound-c cells (1.1E6 to
2.4E6 periods) miss Excel: `AMORDEGRC(1E20,39679,39813,300,1100000,1E-7,1)` was 18989305437101 (Excel 18989305437102,
-c A1), A3 and A4 were a unit off the other way (13720292047583, 5154670166167, Excel ...582 and ...168), and
`AMORDEGRC(1E300,..,2000000,1E-7,1)` was 1.5163267948033552E293 (Excel 1.51632679480347E293, A5); A2 was Excel's
16344246350030 by chance. The walk to 2^22 gives all six. A review replay (T-1841) found 6 of 154 random cases (costs
1E6 to 1E20, 1.1 to 4 times 2^20 periods) a unit off main's walk under the closed form; the unit test
`degressive_shortcuts_equal_the_walk_and_the_closed_form_stays_near_it` bounds the drift at 1e-13 relative for costs
up to 1E20. No probe has a charge of 1E11 or more past 2^22 periods, so no evidence cell is known to miss.
Why 2^22 (T-2918): a walk is unbounded in the period (a life is `ceil(1/rate)`), so some bound is the kernel's work
cap ("AMORLINC and AMORDEGRC work cap"). A walked period costs about 7.6 ns in release, so 2^22 periods are about 29 ms
a scalar call (7 ms at 2^20) and a lifted call keeps its 2^27-unit cap at about 4 s by charging 2^20 units an element
(128 elements: 3.96 to 4.35 s measured, 2026-10-27), in line with the other weighted kernels of SPEC-0011 (YIELD's
solver cap is 4.3 s a call). The next step, 2^23, would
be about 58 ms a scalar call and 64 lifted elements for no known cell. AMORDEGRC's lifted element limit fell from 512
to 128; no eval case lifts more than 128 AMORDEGRC elements that reach the walk.
Evidence: probes/fin-amor-period-bound-c A1-A6 (excel-graph, 2026-10-18, T-1841 review).
Status: DEVIATION (decision 0014)
Implemented in: functions/financial/amortization.rs (`DEGRESSIVE_LOOP_PERIODS`, `DegressiveWalk::closed_form`)
Tasks: T-1841, T-1895, T-2826, T-2918, T-2907

T-1895: AMORDEGRC's charge past about 1.8E295 stays finite: `AMORDEGRC(1E308,39679,39813,300,1E5,1E-6,1)` is
1.94700443465919E302 in Excel (probes/fin-amor-period-bound-b B2), main's walk cut to 15 digits ("AMORDEGRC rounds
its charge like ROUND", T-2826). Main's rounding scaled the value by 1E13 first and overflowed to 0.
Excel does not round each period's charge before subtracting it from what is left: a walk that does gives -c A1-A4
but probes/fin-amor-period-bound B4 and B5 become 1000 and 999 (Excel 0), B11 41042641306981 (Excel
41042641307062) and fin-amordegrc A7 155 (Excel 154) (T-1895, a scratch run). With Excel's final rounding, main's walk
gives -c A1, A3 and A4 past 2^20 periods (the closed form, which took over there before T-2918, does not), but not A2 (main ...030.504 cuts to ...030.5 and
rounds to ...031) nor A5; Excel's own walk, which subtracts in x87 extended precision, gives all five ("AMORDEGRC rounds
its charge like ROUND", T-2849).

## AMORDEGRC rounds its charge like ROUND
Rule: AMORDEGRC rounds the charge it returns the way ROUND(x,0) does ("ROUND family rounds in two decimal stages",
numbers.md): the exact value of the walk's charge is cut to 15 significant digits, then rounded half away from zero. A
charge of 1E15 or more comes back as its 15-digit cut: `AMORDEGRC(1E300,39679,39813,300,1,1E-7,1)` is
2.49999977117486E293 (the walk's 2.4999997711748632E293), and a charge whose 15-digit cut is a half rounds up although
the charge is below it: `AMORDEGRC(1.72531591826348E20,..,1,1E-7,1)` (charge 43132894008630.49) is 43132894008631.
Main pre-rounded at 13 decimal places (`x * 1E13`), which kept such a charge below the half and a huge charge unrounded.
Excel's per-period walk subtracts each charge from what is left in x87 extended precision and stores the difference
as a double, so the remaining value is rounded twice, first to a 64-bit significand and then to 53 bits ("Division
rounds twice", numbers.md): `remain = RN53(RN64(remain - rate * remain))`, the charge `rate * remain` a plain double
product. About one step in 4,000 then lands an ulp from main's plain `remain -= rate * remain`, so the two walks agree at
1 and 2 periods (every charge within one ulp of a cut boundary: probes/amordegrc-walk-drift D1-D12, -ties A1-A12, C1-C3,
C7) and drift apart from about 1000 periods: main's walk misses 1 of 6 near-boundary cells at 1000 periods (-drift D16:
Excel 1.54281774366471E294, main's charge 1.5428177436647047E294), 2 of 6 at 10^4 (E5, E6) and about half at 5x10^4 to
10^6 (E7-E45; -ties A51, A53, C5, C8, C9), 28 of the 135 cells, where the twice-rounded walk gives all 135. It also gives
all five of probes/fin-amor-period-bound-c A1-A5 (1.1E6 to 2.4E6 periods, not used in the fit), where main's walk
misses A2 and A5. Not Excel's (each scored against the 135 cells in a scratch fit, T-2826 and T-2849): the walk in exact
or extended arithmetic throughout (86), an FMA update, `remain * (1 - rate)` (38), the rate and coefficient multiplied
in another order, a first charge computed in another order (107 each), a charge cut to 8 to 16 significant digits or
rounded to a whole number before it is subtracted, and an accumulated depreciation subtracted from the cost. The
product `rate * remain` kept in extended precision too (rounded twice when it is the charge) also gives all 135 and
differs from the double product in 2 raw charges of 135, none visible once rounded; the subtraction of the first
period's rounded charge from the cost fits the probes plain or rounded twice (no cell tells them apart).
The first period's charge (period 0) rounds the same way: `AMORDEGRC(3.47925977217322E18,39679,39813,300,0,1E-7,1)`
(318456837070.49963) is 318456837071, where main's pre-round gave ...070 (probes/amordegrc-round-mid F1-F6; G1-G6 at
period 1, charges 1.2E12 to 2.3E12). Smaller charges, where the two rules still differ for a fraction within 0.5*10^(d-15) below a half (d integer digits;
about 5E-5 at 1E10, down to charges of about 1E3), and the value AMORLINC leaves behind follow ROUND by analogy, unprobed; so does a charge whose 15-digit cut
overflows, which ours leaves as it is where ROUND is `#NUM!` (T-2849).
Evidence: probes/amordegrc-walk-ties (69 cells: costs near 1E20 at 1 to 10^6 periods whose charge in main's walk is
within 0.01 of a half, exact halves, costs 1E300 and 1.23456789012345E300) and probes/amordegrc-walk-drift (66 cells:
charges within one ulp of a 15-digit cut boundary at 1, 2 and 1000 periods, and near x.45 at 10^4 to 10^6 periods),
excel-graph, 2026-10-27, T-2826; probes/amordegrc-round-mid (12 cells, excel-graph, 2026-10-27, T-2826 review);
probes/fin-amor-period-bound-b B2; probes/fin-amor-period-bound-c A1-A5 (excel-graph, 2026-10-18, T-1841 review).
Ours: since T-2826 (`amor_round` in functions/financial/amortization.rs, through `decimal_round::round_decimal`); the
twice-rounded walk since T-2849 (`DegressiveWalk::charge` through `arithmetic::extended_sum`, which rounds the first
period's subtraction twice too, by analogy). Since T-2918 the walk goes on to 2^22 periods, so -c A1-A5 are Excel's;
past that the kernel still finishes in closed form ("AMORDEGRC past 2^22 periods", a DEVIATION).
Status: VERIFIED (excel-graph) for the rounding of charges from about 3E11 up and for the walk; smaller charges and the
first subtraction inferred
Tasks: T-2826, T-2849, T-2917

## AMORLINC and AMORDEGRC work cap
Rule: RustSheet charges each lifted AMORLINC element 2^9 units and each AMORDEGRC element 2^20 against SPEC-0011's 2^27
visit cap, so one call lifts at most 262,144 AMORLINC or 128 AMORDEGRC elements (512 before T-2918):
`ROWS(AMORDEGRC(SEQUENCE(129)*0+2400,39679,39813,300,1,0.15,1))` is `#ERROR!` where Excel lifts every element (they lift
over arrays: probes/fin-argument-modes H19-H35; Excel's own limit is unprobed). The weights are the kernels' bounds in a
release build: AMORLINC's exact walk at most 1,024 single subtractions, its step cap (`LINEAR_STEP_CAP`; the longest
real walk takes 616, and 2^18 elements of `AMORLINC(..,9E15,1.08E-16,1)`, 613 steps each, took 1.23 s, so the cap is
about 7.9 us an element, 2.1 s per call; a walk at the cap would be `#NUM!`), AMORDEGRC's 2^22 walked periods and the closed
form (128 elements of `AMORDEGRC(1E15,..,1,5E6,1E-7,1)`, which takes no shortcut, 3.96 to 4.35 s at T-2918's 2^22
periods, each subtraction rounded twice since T-2849; at 2^20 periods 512 elements took 3.9 s, 2.06 s before T-2849).
The scalar calls are not
capped beyond those bounds.
Since T-2840 an element the kernel refuses before its walk is charged 1, not the weight, so a broadcast of such elements
lifts in full: an error or a logical in any argument (each is read, the basis too, before the walk), a cost, salvage,
period and rate, all numbers, that fail the value checks (`0 <= salvage < cost`, `period >= 0`, `rate > 0`, NaN failing
them), or for AMORDEGRC a rate whose asset life is refused (`1/rate` in [0, 3] or (4, 5)); the charge calls the
kernel's own predicates (`amortization_values_valid`, `asset_life_refused`). A text or a blank is not judged (it may
read as any number), nor is a date's calendar check, which depends on the date system: such an element is charged the
weight. gen-m9/e8d55ec393460bb7
Calc!A10, about 1,250 PIVOTBY purchase dates (most of them texts) at a rate of `PMT(100,271,417889,43,0)` (negative),
spills like Excel's instead of `#ERROR!`.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5, SPEC-0011 "Security rule for kernel work").
Status: DEVIATION (decision 0014)
Implemented in: functions/lift/kinds/financial.rs (`AMORLINC_WEIGHT`, `AMORDEGRC_WEIGHT`, `AMORLINC`, `AMORDEGRC`),
functions/lift/kinds/mod.rs (`ValueWork::Amortization`, `amortization_refused`)
Tasks: T-1841, T-2840, T-2918

## Day-count basis: a negative basis is #NUM!
Rule: every function with a day-count basis argument rejects any basis below 0 with `#NUM!`: -1, -0.5, -0.9999999999,
-1E-300, and -1E300 or -0.5 read from a cell, are all `#NUM!`. A basis in [0, 5) truncates (4.9 is basis 4). A negative
basis is not truncated toward 0. Probed for 22 functions: YIELD, PRICE, DURATION, MDURATION, COUPDAYS, COUPNUM,
COUPDAYBS, COUPNCD, ACCRINT, ACCRINTM, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, YEARFRAC, DISC, INTRATE, RECEIVED,
PRICEMAT, YIELDMAT, AMORLINC and AMORDEGRC. COUPDAYSNC, COUPPCD, PRICEDISC and YIELDDISC are inferred, not probed: they
share the readers of probed ones. Also `YIELD(39448,43831,0.0575,95.04,100,2,-1E300)` with
-1E300 read from a cell (fin-yield-odd-lift L5).
Evidence: probes/fin-bond-negative-basis (176 cells, excel-graph, 2026-10-20, T-1870); probes/fin-yield-odd-lift L5
(excel-graph, 2026-10-18, T-1842).
Ours: one reader, `day_count_basis` (functions/financial/dates.rs), serves the coupon-bond and ODD* kernels, ACCRINT,
AMORLINC, AMORDEGRC (through `day_count_basis_of`, read after the value checks, since T-2485) and YEARFRAC. ACCRINTM and the discount-security functions go through YEARFRAC (its parts since T-2032). Before T-1870,
`floor() as u32` saturated a negative basis to 0, and the `as i32` of YEARFRAC and the `trunc()` of AMOR* turned -0.5
into 0. The probe's eight failing cells, rows 1, 2, 9 and 17 in columns E and G (YIELD, PRICE,
ODDFPRICE and ODDFYIELD at bases 0 and 4.9), are last-digit misses of the T-1374 and T-2007 class, not basis reads;
row 21 (INTRATE) matches since T-2032.
Status: VERIFIED (excel-graph)
Tasks: T-1842, T-1870

## Discount-security functions: settlement 0, basis and argument modes
Rule: INTRATE, PRICEDISC and YIELDDISC accept settlement serial 0 and compute with it (`INTRATE(0,43466,1000,1100)` =
0.0008403165192222404, `PRICEDISC(0,43466,0.05,100)` = -495.0138888888889, `YIELDDISC(0,43466,95,100)` =
0.00044227185222223174), whereas RECEIVED, PRICEMAT and YIELDMAT return #NUM! for settlement 0 (PRICEMAT and
YIELDMAT possibly because issue > settlement: inferred, the probe cannot tell it from a rejection of settlement 0 itself). All six: basis 5 and -1 are #NUM!, a fractional basis truncates (1.9 is 1) and any negative basis is #NUM! ("Day-count basis: a negative basis is #NUM!"), a text
basis "1" is coerced, a boolean basis is #VALUE!, text dates are parsed, fractional dates truncate, settlement >= maturity
is #NUM!, and a zero or negative investment, price or redemption (probed for INTRATE, PRICEDISC and YIELDDISC; RECEIVED's investment) is #NUM!, as is a zero or negative PRICEDISC discount; RECEIVED is #NUM! once 1 - discount*yearfrac = 0 (the "< 0" side is inferred, not probed) (`RECEIVED(...,1000,1)` is #NUM!, `...,0.99` is 99999.99999999991); a negative PRICEMAT/YIELDMAT rate or yield price is #NUM! (PRICEMAT and YIELDMAT accept a
zero rate; issue on or after settlement is #NUM!, issue = settlement included: probes/matdisc-serial-0-60 A9, A14, A25, A30). Basis 2 is the only mode whose result differs on a one-year span
(actual/360); bases 0, 1, 3 and 4 agree there.
PRICEMAT and YIELDMAT count the settlement-to-maturity days as DSM = YEARFRAC-days(issue,maturity) - days(issue,settlement), not
a direct 30/360 count: on 2018-02-28 to 2018-03-30 basis 0 and basis 4 agree for them (A103 = A104 = 100.05774336283186,
A126 = A127 = 0.271557884707002, 32 days), whereas INTRATE, PRICEDISC, RECEIVED and YIELDDISC give different results for
the two bases (INTRATE A14 = 1.2000000000000002 for 30 days, A15 = 1.125 for 32). Ours already matches (`get_mat_factors`).
Evidence: probes/discount-security-modes (135 cells, excel-graph, 2026-10-17, T-1624); probes/matdisc-serial-0-60 (160 cells, excel-graph, 2026-10-22, T-2283: PRICEMAT and YIELDMAT on bases 0-4 with issue 0, 1, 59 and 60, settlement 59, 60, 61 and 366, maturity 400, all matching).
Ours: the settlement-0 cells compute since T-1409 (YEARFRAC accepts serial 0, on every basis since T-2237, dates.md "YEARFRAC accepts serial 0"; T-1757 added a unit test); the last-digit misses are matched since T-2032 ("Discount securities: day count and year length").
Status: VERIFIED (excel-graph)
Tasks: T-1624, T-1757, T-1374, T-2032, T-2283

## Discount securities: day count and year length
Rule: DISC, INTRATE, PRICEDISC, RECEIVED, YIELDDISC, PRICEMAT and YIELDMAT do not divide by a year fraction. They
take YEARFRAC's two parts, the basis day count D (30/360 for bases 0 and 4, actual days otherwise) and the basis year
length B (360, 365, or basis 1's YEARFRAC year), and round in this order, with `b/d` computed first where it appears:
- DISC = `(1 - pr/redemption) * (B/DSM)`; INTRATE = `(redemption - investment)/investment * (B/DIM)`;
  YIELDDISC = `(redemption - pr)/pr * (B/DSM)`; PRICEDISC = `redemption - discount*redemption*DSM/B`;
  RECEIVED = `investment / (1 - discount*DIM/B)`, left to right (T-2484: "Discount securities check order").
- PRICEMAT and YIELDMAT read one B, that of issue to settlement, with A = days(issue, settlement), DIM =
  days(issue, maturity) and DSM = DIM - A: PRICEMAT = `(100 + DIM/B*rate*100) / (1 + DSM/B*yld) - A/B*rate*100`;
  YIELDMAT = `((1 + DIM/B*rate) - T) / T * (B/DSM)` with T = `pr/100 + A/B*rate`.
The forms differ from `(...)/YEARFRAC` in the last digit: INTRATE.xlsx F2 (90 days, basis 0, 2000000 to 2040000) is
exactly 0.08 (the year-fraction form gives 0.08000000000000007), `INTRATE(43159,43189,1000,1100,0)` is
1.2000000000000002 = 0.1*(360/30), not 1.2, and `YIELDMAT(43101,43466,42736,0.06,98,0)` is 0.07692307692307698 (the
expanded numerator `DIM/B*rate + 1 - pr/100 - A/B*rate` gives ...704). The grouping `x*(B/D)` is the only one of ten
candidates matching all eight distinct DISC.xlsx cells (F2:F5 and F15:F18, bases 0-3 on two spans; F6:F10 and F19
repeat them with basis 4, 2.4 or omitted). RECEIVED's form was the year
fraction (`DIM/B` is YEARFRAC) until probes/discount-argument-check-order A758 separated it (T-2484).
Evidence: ironcalc-calc DISC.xlsx F2:F19 and INTRATE.xlsx F2:F6 (Excel-cached); probes/discount-security-modes,
probes/date-system-1900-financial-daycount, probes/date-system-1904-financial-daycount,
probes/date-system-1904-financial A25, A27, probes/fin-argument-modes H139, H352 (excel-graph). 73 cells, all now bit-exact.
Ours: `Model::year_fraction_parts` (date_and_time/yearfrac.rs) gives the parts; the kernels are in
functions/financial/discount.rs and `get_mat_factors` (financial/mod.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2032

## EFFECT and NOMINAL raise to their power with `^`
Rule: EFFECT is `(1 + r/n)^n - 1` and NOMINAL is `((1 + e)^(1/n) - 1) * n`, with n floored, and both powers are
Excel's `^` operator, not libm `pow`. EFFECT's integer n goes through right-to-left square-and-multiply (numbers.md
"Integer exponents: right-to-left square-and-multiply"). For NOMINAL, n = 1 is an exact integer power, n = 2 is the
square root, and every other n is `exp(ln(1+e) * (1/n))` (numbers.md "Non-integer exponents: exp(y·ln x)").
`EFFECT(0.5,10)` is 0.6288946267774416 (libm pow gives ...442), `NOMINAL(0.5,4.5)` is 0.4267276788012868 (...859), and
`EFFECT(0.05,4)` is 0.050945336914062445 (...222). Columns E and F of NOMINAL_EFFECT.xlsx, which convert the other
function's result back, match once C and D do.
Evidence: ironcalc-calc FINANCIAL/NOMINAL_EFFECT.xlsx C8, D2 and the 11 other cells of its baseline lines (Excel-cached;
all 90 cells now pass), probes/fin-argument-modes H125 (excel-graph).
Overflow (T-2241): an overflowing power is `#NUM!`, not `#DIV/0!`: `EFFECT(1E300,2)`, `EFFECT(1E200,3)`,
`EFFECT(1E10,100)`, `EFFECT(1E300,11)`, `EFFECT(1E13,365)`, `EFFECT(1E15,1E15)`, `EFFECT(123456789.123,365)` and
`NOMINAL(1E300,0.5)` are all `#NUM!`. Exact power of ten: `EFFECT(270,30)` is 1E30 (1 + r/n = 10), as are `EFFECT(9,1)` = 9,
`EFFECT(18,2)` = 99 and `EFFECT(990,99)` = 1.2527829399838425E+103. `NOMINAL(1E300,2)` is 2E+150 and `NOMINAL(1E300,1)` 1E+300.
Evidence: probes/effect-overflow-power-of-ten A1-A14 (excel-graph).
Ours: `rate_power` in functions/financial/rate_conversions.rs calls `arithmetic::power` and maps its overflow to `#NUM!`.
Status: VERIFIED (excel-graph)
Tasks: T-2034, T-2241

## PRICE and ODDFPRICE discount with Excel's `^`
Rule: PRICE and ODDFPRICE discount each coupon and the redemption by `(1 + yld/frequency)^t` with Excel's `^`
operator (numbers.md "Integer exponents: right-to-left square-and-multiply" and "Non-integer exponents:
exp(y·ln x)"), not libm `pow`. `PRICE(DATE(2020,3,1),DATE(2030,3,1),0.06,0.07,130,1,0)` is 108.22689722310892 (libm
gives ...889), `PRICE(1,43831,0.05,0.06,100,2)` is 83.34716628375435 (240 coupons; libm ...423) and
`ODDFPRICE(39448,43831,39083,39630,0.08,0.09,100,2,0)` is 92.4077632093273 (libm 92.40776320932721).
Integer exponents (settlement on a coupon date) and the fractional ones of a mid-period settlement and of
ODDFPRICE's offsets all come out bit-exact with `^`: every PRICE and ODDFPRICE cell in eval/ then passed, except
where both the coupon rate and the yield are far above 1 (the observation below).
Evidence: ironcalc-calc FINANCIAL/PRICE_YIELD.xlsx PRICE_YIELD!H2-H6, H8-H11, H28, ODDFPRICE_ODDFYIELD!J5, J8, J9,
J11, J23, J25, J27, J29-J33, J44, J45 and ODDFPRICE_SHORT!J9, J10, J13 (Excel-cached); probes/fin-coupon-walk-bounds-c
C10, C11, G10, G11, J10, J11, fin-argument-modes H189, H251, fin-bond-negative-basis E2, E9, G2, G9,
fin-remaining-later-argument-ranges I105-I119 and I165-I171 (odd rows), fin-yield-odd-lift A3, I7 (excel-graph).
YIELD's solver does not follow: priced with `^`, YIELD fixes 4 more cells but moves 2 that pass with libm prices,
ironcalc-calc PRICE_YIELD!I27 by 4 ulp (Excel 0.07000000000000012, `^`-priced 0.07000000000000017, visibly
0.0700000000000002) and probes/yield-start-rate-term S!B45 by 2 ulp, so the engine's YIELD solver keeps libm prices.
ODDFYIELD passes the same cells on either kernel and uses ODDFPRICE's `^`. In the fit lab (178 basis-0 cells with more than one coupon left),
`uv run rustsheet-fitlab yield-caret` (kernels/yield_caret.py) scores `^`-priced copies of every `yield` candidate:
they reach at most 4/178
bit-exact, as the libm-priced ones do: Newton from 0.1 to a 1e-12 step 4/178 (10 within 4 ulp) libm against
4/178 (7) `^`; Newton from a continuous annuity root 3/178 (8) against 2/178 (7); from PRICE's root at frequency 2
and redemption 100 0/178 (5) against 1/178 (14). The union of both fits' candidates is 15/178, against 12/178 for `yield` alone. So the
PRICE kernel is not what keeps the YIELD fits (T-1592, T-2007) from Excel's bits.
Ours: `discount_growth` in functions/financial/financial_bonds.rs (`arithmetic::power`, libm where `^` is an error);
DURATION and MDURATION use it too (T-1975), and so does YIELD's Newton (T-1592); ODDFYIELD's `solve_yield` prices with
`f64::powf`.
Observation (T-2732): where both the coupon rate and the yield are far above 1, PRICE, DURATION and MDURATION leave
Excel's bits, and in 62 cells its 15 digits. probes/bond-extreme-coupon-yield (579 cells, excel-graph) crosses coupon
rates 10..1E+300 with yields 1..44270 at frequency 1, 2 and 4, basis 0 and 1, mid-period (DSC = 26 days) and on a
coupon date, plus short stubs (43830-45293 at frequency 4, basis 1: DSC/E = 2/92; 39697-44449 at frequency 2,
basis 0: 4/180). The engine passes 165 cells, 319 are within 4 ulp and 95 beyond, 62 of them visibly off: PRICE 43,
DURATION 8, MDURATION 11. The pieces pass: COUPDAYSNC, COUPDAYS, COUPNUM (Parts!A1-A3), `(1+yld/2)^(DSC/E)` and
`(1+yld/2)^(DSC/E+N-1)` with Excel's `^` (Parts!C1-D6), and the redemption term alone at rate 0 (Parts!B1-B6), so
the gap is in how the coupon flows are summed and combined: the coupon term alone (redemption 1E-300, Parts!E1-J6)
already misses, 5 cells visibly (Parts!F1, F2, F5, F6, G2), with the same digits as PRICE at redemption 100. Against
a 50-digit mpmath value of the documented sums (DSC/E, N and A/E from the coupon dates; q = 1+yld/f, c = 100·rate/f;
price = sum over k = 1..N of c/q^(k-1+DSC/E), plus redemption/q^(N-1+DSC/E), minus c·A/E; DURATION = the Macaulay sum
of the same flows ÷ f; MDURATION = DURATION/q; each cell's distance from it in ulps of the exact value):
- Short stubs: Excel is close to correctly rounded and the engine is not. All 34 failing DURATION and MDURATION
  cells are within 3.1 ulp in Excel and 5 to 30 ulp off in the engine; the 16 visibly-off PRICE cells are 0.6 to 23
  ulp off in Excel (median 6) and 16 to 55 in the engine (median 33). All 33 visibly-off short-stub cells are closer
  in Excel, including gen-m9 a91bad4e84f49739 A142's twin `PRICE(43830,45293,100,278,50,4,1)` (Calc!A182: Excel +15.8 ulp,
  ours -16.2) and 30e20ba3302fea8b A90's `MDURATION(39697,44449,EDATE(58318,22),44270,2,0)` (Calc!C183: Excel
  -0.8 ulp, ours -17.8).
- The main grid (DSC = 26 days): neither is correctly rounded. On the 29 visibly-off cells (27 of them PRICE, 5 of
  those on Parts) Excel is 0 to 32 ulp off (median 5) and the engine 0.6 to 23 (median 9); Excel is closer in 16, the
  engine in 13. Excel's own error reaches
  32 ulp (Calc!A20 `PRICE(38136,44737,1E+10,7,100,1,0)`: Excel 55709579072.88489, -32 ulp; ours ...88501, -16).
  gen-m9 0a957119475760c5 A279's `PRICE(38136,44737,1E+300,7,100,2)` (Calc!A181) is Excel +0.5 ulp, ours +8.5 (
against the double that 1E+300 parses to).
So a more accurate kernel would fix the short-stub DURATION and MDURATION cells but not the grid's PRICE cells,
where only Excel's own operation order gives its digits: T-2733 keeps the `visible` stage.
Status: VERIFIED (excel-graph)
Tasks: T-2207, T-2732, T-2733

## PMT when (1+rate)^nper overflows or underflows
Rule: when `pv*P` overflows (`P = (1+rate)^nper`, overflowing or not), PMT is not an error but the annuity quotient
divided through by P, whose limit is `-pv*rate` at the end of the period and `-pv*rate/(1+rate)` at the start; the fv
term vanishes. `PMT(100,184,439103)` is -43910300, `PMT(100,184,-439103)` 43910300, `PMT(100,184,439103,0,1)`
-434755.44554455444 (43910300/101), `PMT(100,153,1)` -100 (P = 101^153 is finite; only `pv*rate*P` overflows),
`PMT(1,2000,1)` -1, `PMT(1,2000,1,0,1)` -0.5, `PMT(1,2000,1,5)` -1, `PMT(1,2000,0,5)` 0, `PMT(0.5,2000,3,7)` -1.5,
and `PMT(1,1023,1)`, `PMT(1,1024,1)`, `PMT(1,1100,1)` are all -1. When P is finite but P-1 rounds to P
(`PMT(1,60,1)`, P = 2^60) the ordinary quotient gives -1 (-0.5 for type 1). When P is so small that 1/P
overflows (0, or the subnormal 2^-1070) PMT is `#NUM!` whatever fv and type are, and so are IPMT, PPMT and CUMIPMT:
`PMT(-0.5,2000,1)`, `PMT(1,-2000,1)`, `PMT(-0.5,2000,0,5)`, `PMT(-0.5,2000,1,0,1)`, `PMT(-0.5,2000,1,5)`,
`PMT(-0.5,1070,1)`, `PMT(-0.5,1070,0,5)`, `IPMT(-0.5,1,2000,1)`, `PPMT(-0.5,1,2000,1)`, `CUMIPMT(-0.5,2000,1,1,1,0)`
(CUMPRINC, which shares the kernel, and a subnormal P whose reciprocal is finite are unprobed), except IPMT in period 1 of type 1, which is 0 ("IPMT cancels on fv"). IPMT and PPMT follow PMT: `IPMT(100,1,184,439103)` is
-43910300 and `PPMT` of it 0; `IPMT(1,1,2000,1)`, `IPMT(1,2,2000,1)` are -1 and their PPMT 0;
`PPMT(1,1,60,1)` is -8.67361737988405E-19 (ours 0, unscored: a last-digit question, T-1528). PV, FV and NPER stay `#NUM!` there (`PV(100,184,1)`, `FV(100,184,1)`,
`NPER(100,-1,1)`). The gen-m9 inference holds: `FLOOR(NA(),1/0)` is `#N/A`, FLOOR returns its number's error first.
Evidence: probes/pmt-overflow Sheet1!A1:A18, B1:B8, C1:C3, D1:D3, E1:E9 (excel-graph, 2026-10-19, T-1405);
gen-m9/3b8e9078261b5bd2 Calc!A34 (excel-graph, 2026-10-12, T-1289).
Status: VERIFIED (excel-graph)
Ours: matches since T-1405 except `PPMT(1,1,60,1)` (0, the T-1528 last-digit case) (`compute_payment` keeps the fitted quotient whenever it is finite and falls back to the
divided-through form, so no finite result changes digits; a P whose reciprocal overflows is `#NUM!`).
Tasks: T-1405

## Analysis ToolPak securities functions check frequency and dates first
Rule: the COUP* functions (COUPDAYSNC and COUPNUM probed; they share one argument reader) check each argument as they read
it, left to right, so a bad value is `#NUM!` before a later argument's error or text:
1. settlement: an error or text is that error (`#VALUE!` for text); a serial that does not floor to 0-2958465
   (9999-12-31) is `#NUM!` at once (`COUPDAYSNC(-1,1/0,2,0)`, `COUPDAYSNC(-1,"x",2,0)` and `COUPDAYS(2958466,NA(),2)` are
   `#NUM!`; `COUPNCD(2958465.5,NA(),2)` is `#N/A`);
2. maturity: its error or text, then a serial past 2958465 is `#NUM!` (`COUPNUM(39000,2958466,2,NA())`,
   `COUPDAYBS(39000,1E15,3,NA())`; `COUPPCD(39000,2958465.9,2)` is 38898), then settlement >= maturity is `#NUM!`
   (`COUPDAYSNC(40000,39000,1/0,0)`,
   `COUPDAYSNC(39000,39000,"x",0)`, `COUPNUM(40000,39000,NA(),1/0)` are `#NUM!`);
3. frequency: its error or text, then a frequency other than 1, 2 or 4 is `#NUM!` (`COUPDAYSNC(39000,40000,3,1/0)` and
   `COUPDAYSNC(37342,46362,-0.0001,<#DIV/0!>)` are `#NUM!`);
4. basis: its error or text, then a basis outside 0-4 is `#NUM!`.
Two errors give the first one's (`COUPDAYSNC(NA(),40000,3,1/0)` and `COUPDAYSNC(39000,40000,NA(),1/0)` are `#N/A`).
TBILLEQ, TBILLPRICE and TBILLYIELD read settlement, maturity and the discount (price) in order, and check only that each date
floors to a serial from 0 to 2958465 (9999-12-31) as they read it: `TBILLEQ(39000,-1,1/0)`, `TBILLEQ(-1,39100,"x")`,
`TBILLEQ(39000,2958466,1/0)` and `TBILLPRICE(-1,1/0,"x")` are `#NUM!`, and `TBILLEQ(38887,1e-15-2,TEXTSPLIT("a*",A1:D2))` is
`#NUM!`. Serial 0 passes this check (`TBILLEQ(0,39100,1/0)` is `#DIV/0!`, `TBILLYIELD(0.5,39100,"x")` `#VALUE!`). A reversed
or equal date pair, or a maturity more than a year after settlement, comes after the discount (`TBILLEQ(39100,39000,1/0)`
is `#DIV/0!`, `TBILLPRICE(39000,39500,"x")` `#VALUE!`). An error settlement beats an invalid maturity
(`TBILLYIELD(1/0,-1,0.05)` is `#DIV/0!`).
Evidence: probes/securities-argument-error-order (192 cells, excel-graph, 2026-10-20, T-1406); probes/coup-calendar-edges
A1-A8 (the 9999-12-31 bound, excel-graph, 2026-10-25, T-2486); gen-m9/0c7fc447bc2eebfd
Calc!A24, gen-m9/4df1b22d3ed63f58 Calc!A20 (excel-graph, 2026-10-12, T-1289).
In a 1904 workbook the T-bills read the date itself, so their range is the 1904 calendar's own, 1904-01-01 to
9999-12-31: `TBILLPRICE(DATE(9996,1,1),DATE(9996,6,30),0.05)` is a number while DISC of the same dates is `#NUM!`, and
`TBILLPRICE(-1,100,0.05)` is `#NUM!` (probes/fin-1904-calendar-readers A1, A10, A16; dates.md "The 1904 date system:
ACCRINT, COUPNUM and the TBILL functions read the date itself"). Near the end of the calendar see "T-bills settled in 9999".
Ours: matches since T-1406 (`coupon_args` in financial_bonds.rs, `tbill_arguments` in discount.rs); the COUP* upper
bound since T-2486 (`coupon_date_of` in financial/dates.rs). In a 1904 workbook the bound is the 1904 calendar's own
last day, 2957003, though the financial reading moves it past 9999-12-31: `COUPPCD(39000,2957003,2)` = -1462,
`COUPPCD(39000,2957004,2)` and `COUPPCD(2957004,NA(),2)` are `#NUM!`, `COUPPCD(2957003,NA(),2)` is `#N/A`
(probes/coup-calendar-edges-1904, excel-graph, 2026-10-25) (before it, a maturity past 9999-12-31 read on to a later error, and `COUPDAYS(39123,2958466,1,0)` was
360).
Status: VERIFIED (web; the COUP* upper bound excel-graph)
Tasks: T-1406, T-2486

## PRICE, YIELD, DURATION and MDURATION check order
Rule: these four do not read their arguments as the COUP* functions do, and PRICE and YIELD differ from DURATION and
MDURATION:
1. PRICE and YIELD read the basis first: its error or text is returned, and a basis outside 0-4 is `#NUM!`, before
   anything else (`PRICE(-1,43831,0.0575,0.065,100,2,NA())` is `#N/A`, `PRICE(NA(),43831,0.0575,0.065,100,2,5)` and
   `YIELD(39448,43831,0.0575,95.04,100,"x",5)` are `#NUM!`). DURATION and MDURATION read it last (step 4).
2. settlement, then maturity: an error or text is returned, and a serial that does not floor to 0-2958465 is `#NUM!`
   as it is read (`PRICE(-1,NA(),..)`, `PRICE(2958466,"x",..)`, `DURATION(39448,-1,NA(),..)` are `#NUM!`,
   `DURATION(NA(),-1,..)` is `#N/A`). Serial 0, 0.5 and 2958465 pass this read (`PRICE(0,43831,NA(),..)` is `#N/A`),
   though a settlement below 1 is `#NUM!` later (step 5).
3. rate, yld (YIELD's price), PRICE's and YIELD's redemption, then frequency: an error, text or boolean is returned.
4. DURATION and MDURATION check the frequency as they read it, then read the basis: `DURATION(39448,43831,0.08,0.09,3,NA())`
   is `#NUM!`, `DURATION(39448,43831,0.08,"x",3,1)` `#VALUE!`. PRICE and YIELD check the frequency after every read
   (`PRICE(..,100,3,NA())` is `#N/A`, but step 1 already read the basis).
5. Only then the value checks, all `#NUM!`: settlement below 1 or not before the maturity, a negative rate, a negative
   yld (YIELD: a price of 0 or less), a redemption of 0 or less. So `PRICE(39448,39000,"x",..)` and
   `DURATION(39448,43831,-0.05,NA(),2,1)` are the later argument's `#VALUE!` and `#N/A`.
The reason the basis comes first only for PRICE and YIELD is unknown; it holds for every position probed.
Evidence: probes/bond-argument-check-order (298 cells: each out-of-domain argument against an error, and text next to
it, in every other position; excel-graph, 2026-10-24, T-2482).
Unprobed: a maturity past 9999-12-31 against a later error (the read check is taken from the settlement's and the
T-bills'); a negative fraction such as -0.5 as a date (floored to -1 like the T-bills').
Ours: matches since T-2482 (`bond_args` in financial_bonds.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2482

## PRICE, DURATION and MDURATION at a huge yield or coupon
Rule: PRICE, DURATION and MDURATION are `#NUM!` when `(1+yld/frequency)^t` overflows a double at the redemption's term
`t = DSC/E + N - 1`, the longest one, although the overflowing flows would just discount to 0. A sum that overflows (a
huge coupon) is `#NUM!` too. There is no separate cap on the yield or the coupon.
- The boundary is the power's overflow: with N = 20 and settlement on a coupon date (t = 20), a yield of 5.12154E+15
  gives `PRICE` 9.76268856632966E-16 and 5.22501E+15 `#NUM!` (the overflow is at 5.1733E+15); with settlement half a
  period in (t = 19.5) 1.27233E+16 is a number and 1.29803E+16 `#NUM!`; with N = 40 1.00701E+08 is a number and
  1.02735E+08 `#NUM!`. A zero coupon has the same boundary. DURATION and MDURATION switch at the same yields.
- A single remaining coupon (t at most 1) never overflows: `PRICE(47300,47484,0.05,1E+300,100,2,0)` is 2.05E-298,
  DURATION 0.5, MDURATION 1E-300.
- At a 5% yield a coupon rate of 1E+300 gives a price of 7.794581142823408E+302 and DURATION 4.841148506210218; 1E+306
  and 9.99999999999999E+307, whose coupon sums overflow, are `#NUM!` for all three.
- `1E+308` cannot be typed into Excel (9.99999999999999E+307 is the largest literal): a workbook with it fails to load
  in Excel for the web (`FileCorruptTryRepair`).
Evidence: probes/bond-huge-yield-overflow (78 cells, excel-graph, 2026-10-25, T-2488); gen-m9 cells such as
`PRICE(39050,44044,0.5,123456789.123,110,4,2)` and `MDURATION(38368,44642,1%,1E+300,4,3)`, both `#NUM!`.
Unprobed: ODDFPRICE and ODDLPRICE at an overflowing yield; YIELD's solver near the boundary (T-2391).
Ours: matches since T-2488 (`CouponSchedule::discount_overflows` in financial_bonds.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2488

## ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order
Rule: the four odd-coupon functions read their arguments in order and check few of them as they read:
1. The dates (settlement, maturity, then issue and first_coupon, or last_interest): an error or text is returned, and a
   serial below 0 (-0.5 too) or past 2958465 (9999-12-31) is `#NUM!` as it is read
   (`ODDFPRICE(2958466,NA(),..)`, `ODDLYIELD(43000,43200,-1,NA(),..)` are `#NUM!`, `ODDFPRICE(NA(),-1,..)` is `#N/A`). Serial
   0, 0.5 and 2958465 pass this read (`ODDFPRICE(0.5,43831,39300,39630,NA(),..)` is `#N/A`).
2. rate, yld or pr, redemption, then frequency: an error, text or boolean is returned; the frequency is `#NUM!` unless 1, 2
   or 4 as it is read (`ODDLPRICE(..,100,3,NA())` is `#NUM!`).
3. basis: an error, text or boolean is returned, then outside 0-4 `#NUM!`.
4. Only then the date order and value checks, all `#NUM!`: a settlement not before the maturity, an issue not before the
   first coupon, a coupon date not before the maturity, ODDLPRICE's and ODDLYIELD's settlement on or before `last_interest`
   (`ODDLPRICE(42917,43200,42917,..)` is `#NUM!`, `ODDLPRICE(42918,43200,42917,0.0575,0.065,100,2,0)` = 99.44775142683748),
   a negative rate, a negative yld (ODDFYIELD and ODDLYIELD: a price of 0 or less), a redemption of 0 or less. So
   `ODDFPRICE(43832,43831,..,NA(),2,0)` is `#N/A` and `ODDLPRICE(43000,43200,42917,-0.05,..,TRUE)` `#VALUE!`.
Unlike PRICE and YIELD, the basis is read last (step 3).
Evidence: probes/odd-bond-argument-check-order (1091 cells: each out-of-domain argument against `NA()`, text and `TRUE` in
every other position, the ODDL* settlement around `last_interest`, dates 0, 0.5 and 2958465; excel-graph, 2026-10-24,
T-2483); probes/fin-odd-date-before-error-arg for the negative dates.
Unprobed: a 1904 workbook, whose reading moves a date 2 * 1462 days on (`DateSystem::odd_period_1900_serial`): a 1904 serial
that only the move takes past 9999-12-31 is `#NUM!` after the reads, as before T-2483.
Ours: matches since T-2483 (`odd_args` and `odd_last_of` in financial_bonds.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2483

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
2026-10-13, T-1494). Since 2026-10-21 a 1904 workbook's ACCRINT reads those serials as the dates themselves, eight
years before the 1900 cells, so its twin no longer reaches the cut and gives the 1900 values of its own dates
(re-filled 2026-10-27; dates.md "The 1904 date system: ACCRINT, COUPNUM and the TBILL functions read the date itself").
Ours: `compute_accrint` (functions/financial/accrint.rs).
Status: VERIFIED (excel-graph). Tasks: T-1494.

## COUPDAYS basis 1 counts from the chained next coupon
Rule: COUPDAYS on basis 1 is not COUPNCD - COUPPCD. Excel walks the schedule back from the maturity *without* the
end-of-month rule (each date the one after it moved back a period, its day clamped to the month, so the day can drift),
takes the first date of that walk after settlement, B, and counts from A, B moved back one period: to the last day of
A's month when B is the last day of its month, else B's day clamped. Excel's 1900 has a 29 February, so 1900-02-28 is
not a month end. If A is after settlement, Excel steps back once more (B := A, A from it the same way). A date before
1900-01-01 compares one higher than its serial in that test (1899-12-31 as 1). The count is the serial difference,
except: real-calendar days when B is before 1900-01-01, or when A is and there was no extra step; B's serial (A reads
as serial 0) when A is before 1900-01-01 after the extra step.
- Month-end schedules inside the calendar (maturity 2019-03-31 semiannual): 181 for settlement 1950-01-15 (1949-09-30
  to 1950-03-30; COUPPCD and COUPNCD describe 1949-09-30 to 1950-03-31, 182), 183 for 1950-06-15 (1950-03-31 to
  1950-09-30), and 182 for 2018-12-15 (the walk's first date is the maturity). Maturity 2019-04-30 semiannual: 183 for
  2018-06-15 (2018-04-30 to 2018-10-30, not 184). Maturity 2019-01-31 quarterly: 92 for 1950-01-15 (1949-10-30 to
  1950-01-30), 91 for 1950-06-15 (1950-04-30 to 1950-07-30), but 92 for 2018-06-15 (the walk is still on the 31st:
  2018-04-30 to 2018-07-31). A maturity on the 30th that is not a month end: 2019-12-30 semiannual gives 181 for
  2018-01-15 (B = 2018-06-30, a month end, so A = 2017-12-31), 183 for 2018-08-15.
- Around 29 February (T-1493): maturity 2025-02-28 semiannual, settlement 2023-11-15: 184 (2023-08-28 to 2024-02-28,
  since 2024-02-28 is not a month end), while COUPPCD, COUPNCD, COUPDAYBS (76) and COUPDAYSNC (106) describe 2023-08-31
  to 2024-02-29; settlement 2024-03-15 gives 182 (2024-02-28 to 2024-08-28), 2024-02-28 gives 182 and 2024-02-27 184.
  Frequency 1, maturity 2025-02-28, settlement 2023-06-15: 365. A maturity on a 29 February is a month end
  (`COUPDAYS(DATE(2399,11,15),DATE(2400,2,29),2,1)` is 182), and so is 28 February of a common year (2099-08-31 to
  2100-02-28, 181). `COUPDAYS(59,36585,4,1)` is 90 (1900-02-28 is not a month end). A walk that lands on a 29 February
  counts from the month end: `COUPDAYS(DATE(2027,11,15),DATE(2028,8,31),2,1)` is 182 (2027-08-31 to 2028-02-29; T-1493
  had predicted 184), and `COUPDAYS(DATE(2027,11,15),DATE(2028,5,31),4,1)` is 92 (2027-08-29 to 2027-11-29).
- Before the calendar (T-2640): maturity 2019-03-31 semiannual is 181 for settlements 0 to 89 (1899-09-30 to
  1900-03-30, real days), 91 for 90 (1900-03-30: A = 1900-03-31 is after it, so 1899-09-30 to 1900-03-31 counted as
  serial 91 - 0) and 183 from 91; maturity 2019-05-31 is 152 at settlement 151. Maturity 2019-12-31 or 9999-12-31
  semiannual (also 2019-12-30 and 2019-06-30): 184 for a settlement in [0, 1) (A = 1899-12-31 compares as 1, so
  1899-06-30 to 1899-12-31) and 181 from 1 (1899-12-31 to 1900-06-30); maturity 2019-12-29 is 182. Quarterly
  maturity 2019-01-31 is 31 at settlement 30. Annual month-end schedules are 365 throughout 1900 except the February
  one, 366 from settlement 59.
Only COUPDAYS shows it: COUPPCD, COUPNCD, COUPDAYBS and COUPDAYSNC keep the end-of-month schedule
(`COUPNCD(0,2958465,2)` = 182, `COUPDAYBS(0,2958465,2,1)` = 0, `COUPDAYSNC(0,2958465,2,1)` = 182), PRICE and DURATION
on basis 1 use COUPNCD - COUPPCD, and bases 0, 2, 3 and 4 have fixed lengths.
Evidence: probes/coupdays-pre1900-control (1,728 cells: settlements on the 15th of every month of 1900, 1901, 1904, 1950,
2000 and 2018 against 12 schedules), coupdays-pre1900-sweep (6,316 cells: every settlement 0 to 370 on 38 schedules),
coupdays-pre1900-period (148 cells) (excel-graph, 2026-10-25, T-2640); coup-calendar-edges C1-C21, D1-I12 (T-2486);
coupdays-feb-month-end (296 cells, T-1493) and date-system-1900-financial-readers E15; gen-m9/b72d952c5f52cb4d Calc!A88
(`COUPDAYS(5E-05,2958465,2,1)` = 184). The rule reproduces all 7,422 basis-1 COUPDAYS cells of the 1900-system probes.
Ours: `coupdays_actual` (functions/financial/financial_bonds/coupon_schedule.rs). Before T-2640 it counted
COUPNCD - COUPPCD (the real calendar before 1900) and walked the chained schedule only around a 29 February.
Status: VERIFIED (excel-graph)
Tasks: T-1489, T-1493, T-2486, T-2640

## Coupon dates keep the maturity's day
Rule: every coupon date of the COUP* schedule is the maturity moved back a whole number of periods, its day clamped to
the month (or the month's last day under the end-of-month rule), so a maturity on the 29th or 30th that is not a month
end keeps that day after a short month: maturity 2027-11-29 quarterly has coupons on 2003-08-29 and 2003-11-29
(`COUPPCD(37913,46720,4)` = 37862, `COUPNCD` 37954, `COUPDAYBS(37913,46720,4,4)` and on basis 0 = 50,
`COUPDAYSNC(..,1)` = 41), and maturity 2027-05-30 quarterly has 2019-11-30 and 2020-02-29 (`COUPPCD(DATE(2020,1,1),
DATE(2027,5,30),4)` = 43799, `COUPNCD` 43890); maturity 2027-08-29 semiannual has 2025-02-28 (common year) but 2025-05-29
quarterly. PRICE, YIELD, DURATION, MDURATION, ODDFPRICE and ODDFYIELD use the same schedule:
`PRICE(37913,46720,0.05,0.06,100,4,0)` = 87.29595935878451, `DURATION(37913,46720,0.05,0.06,4,0)` = 13.296825458940603,
and with maturity 2027-11-29 an ODDFPRICE or ODDFYIELD first coupon of 2003-11-29 is on the schedule (87.29680647881554)
while 2003-11-28 is `#NUM!`. Only COUPDAYS on basis 1 (and, below, ODDLPRICE, ODDLYIELD and ACCRINT, each in its own walk direction) walks the chained schedule, each date clamped from its neighbour
in the walk ("COUPDAYS basis 1 counts from the chained next coupon").
Evidence: probes/coup-calendar-edges B1-B14, J1-J14 (excel-graph, 2026-10-25, T-2486); gen-m9/7d2979fba2c95d64 Calc!A304.
Ours: matches since T-2486 (`coupon_period` in financial_bonds/coupon_schedule.rs, which the COUP*, PRICE, YIELD,
DURATION, MDURATION and ODDF* schedules read); before it the walk stepped from each date to the next, so the day drifted
to the 28th after the first February (COUPDAYBS was 51, the PRICE above 87.29662784091713, and the ODDF* first coupon
had to be the 28th). The ODDL* and ACCRINT walks (`change_month` in financial_bonds.rs, accrint.rs) step date to
date, and Excel does the same: EXCEPTION to this rule, Excel's ODDLPRICE, ODDLYIELD and ACCRINT coupon walks DRIFT. Each
date is clamped from the previous one, so after a February the 29th/30th becomes the 28th/29th and stays there. Do not
"unify" them with `coupon_period`. Evidence: probes/oddl-accrint-february-day (excel-graph, T-2639, 49 cells, ours matches
all). ODDLPRICE and ODDLYIELD give the same value for a last interest on the 29th and on the 30th (A1/A4, A2/A5, A3/A6,
A7/A10, A32/A35, A44/A47 and the other pairs, frequencies 2 and 4, bases 0 and 1): after the first February both schedules
sit on the same day, which only a drifting walk does (a walk anchored on the last interest keeps the 29th and 30th
apart). Only A13-A15 (Oct 30 -> Jan 30 -> Apr 30, never a February) do not discriminate. ACCRINT's backward walk from the
first interest drifts too: A16 (issue 2023-08-15, first interest 2024-08-30, frequency 2, basis 0, calc_method TRUE) =
56.26086281812539 = 25 * (31/180 + 2 + 14/179), whose issue stub ends on 2023-08-29 (14 days of a 179-day period
from 2023-02-28), not on 2023-08-30; and the quarterly first interests 2024-05-30 and 2024-05-29 differ (A24 =
35.833..., A28 = 35.972...), as the drifting stub ends differ. ACCRINT basis 0 with calc_method FALSE is equal for the
two days (A18 = A22).
Ours: matches (the date-to-date `change_month` walk). Keep it: it is deliberate Excel behaviour.
Status: VERIFIED (excel-graph)
Tasks: T-2486, T-2639

## YIELD stops short of PRICE's root
Rule: YIELD's last digits are not the exact root of PRICE: Excel stops early, below the root, by up to about 1e-9
relative (about 6e-9 on the T-2304 grid, B6) (`YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,100,2,0)` is 0.05523587030251456; the root, as we compute
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
T-2187 (probes/yield-start-sweep, excel-graph, 2026-10-21; the base bond `YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,R,F,B)`):
analytic Newton run *forward* from the fixed start S = 0.0552344627 reproduces Excel on all 63 cells of columns A, B
and C (redemption 100 to 101 by 0.05 at frequency 2, 1 and 4, basis 0), every one to within 1.2e-14 relative, when it
returns the iterate after the first step smaller than about 7e-6 (the lab test uses 6.9e-6). The step count changes with the inputs: one step at
R = 100 for F = 1 and 2 (A1, B1, a 1.4e-6 step), two steps up to A17 (R = 100.8, last step 6.37e-6), B16 (6.26e-6) and
C18 (6.66e-6), and three steps from A18 (7.19e-6), B17 (7.11e-6) and C19 (7.47e-6). C1 (F = 4, R = 100) takes two
steps, 1.52e-5 then 5.0e-10. So the step-size cut lies between 6.66e-6 and 7.11e-6 absolute, or between 1.17e-4 and
1.25e-4 relative to the iterate; these cells do not separate the two. This replaces the residual-stop reading of
T-1583. The start S does not depend on redemption or frequency; it does depend on price and on the day-count basis:
- Back-chain starts by price (column E): about 0.10913448 at price 80 (E4) and 0.2427821 at price 50 (E1).
- Back-chain starts by basis (column D, R = 100): 0.0552334228 on bases 1 and 3 (D1, D2, D8) against 0.0552344627
  on bases 0 and 4 (D10, D11; these dates count the same on 30/360 and 30E/360).
- Column F moves the bond to 1 March 2000 to 30 June 2004 (same 30/360 day counts) and gives the same bits as the 2096
  bond: F6 = A1 = E8 = 0.05523587030251456, F7 = A6, F8 = A11, F9 = A16, F1 = D10 = B1.
- Lab: `uv run python` on `solvers.newton_back_chain`, pinned in
  tools/tests/test_fitlab_solvers.py::test_yield_start_sweep_newton_from_a_shared_start.
T-1592 (second round, lab fit `yield`, now 178 cells with yield-start-sweep):
- The basis moves S as the term T = settlement-to-maturity days in the basis over 360 (basis 0, 1559/360) or 365
  (basis 1, 1581/365) moves a frequency-free annuity's root: D2's start minus D11's (1.04e-6) is matched to 1e-8
  whatever the compounding, against 1e-6 to 1.3e-5 off for 1581/366 or 1581/365.25 (pinned in
  tools/tests/test_fitlab_solvers.py::test_yield_start_moves_with_the_basis_days_over_its_year). So S
  uses the term in years, not the coupon schedule, and actual days over 365 on basis 1 (not YEARFRAC's
  average year).
- S is not the root of such a model. Newton with the step cut from the root of an annuity of 100 * rate a year
  over T, paid m times a year, with redemption 100: m = 1 0/178 bit-exact (2 within 4 ulp), m = 2 1/178 (3),
  m = 3 2/178 (6), m = 4 0/178 (11), continuous 3/178 (8); from PRICE's own root at frequency 2 and
  redemption 100: 0/178 (5). The best candidate overall stays Newton from 0.1 to a 1e-12 step, 4/178 (10).
  No compounding fits more than one price: an annuity's root minus S at price 98, 80 and 50 is +2.2e-5,
  +5.2e-5 and +1.9e-4 for m = 2, and +4e-7, -4.1e-4 and -3.0e-3 for m = 3.
- S is not an iterate of Newton, the secant, regula falsi, Illinois or bisection run on PRICE at a fixed
  frequency 1, 2 or 4 with redemption 100 from 0, 0.01, 0.02, 0.05, 0.1, 0.5, 1 or 1e-6 (pairs for the
  bracketing methods): no iterate comes within 3e-9 relative of the starts of A1, D2, E1, E4 or E5.
- On price alone the S values are ambiguous: only price 98 (frequency independence) pins the step count. With
  one or two steps back, price 80 (E4) gives 0.109134476 or 0.10825933, price 90 (E5) 0.0776002232 or
  0.0774089333, price 50 (E1) 0.242782115 or 0.2424219411.
T-2304 (probes/yield-start-rate-term, excel-graph 2026-10-21; 50 bonds x frequency 2, 1, 4; basis 0, settlement
2096-03-01, redemption 100.5, maturities 30 June 2098 to 2126, rates 0 to 0.12, prices 98 and 90): 122 of 150 cells
stop below PRICE's root (PRICE(y) - price from +3e-10 to +7.7e-8, so they show S) and 28 sit at the root
(|residual| 2e-11 or less): B1, B26:B30, B33:B35, B39, A40:C40, A44:C45 and A48:C50 (column A is frequency 2, B 1,
C 4). Residuals mostly fall with the rate and the term at price 98, with a rise at the highest rates and longest terms
(2110 at rate 0.12 in columns A and C; 2126 column A: 7.9e-9, 2.3e-9, 3.2e-9, 1.1e-8, 3.7e-8); the largest is 7.7e-8 (B43, price 90, rate 0.05, 2110). Stepping Newton
back two steps (lab arithmetic): 98 / 0.05 / 2100 (A8) reaches 0.05523446, the S of T-1592, but 98 / 0.05 / 2098 (A3)
reaches 0.0593019, 98 / 0.05 / 2110 (A18) 0.0519331, 90 / 0 / 2098 (A26) 0.0456824 and 90 / 0.05 / 2110 (A43)
0.0600873, so S moves with term, rate and price and is not one constant (Newton forward from S = 0.0552344627 reproduces only A8:C8
and the root cells). The cells at the root need no S.
These cells also contradict the stop rule above (first step below about 7e-6): B2 stops with PRICE 2.0e-8 above the
target after a last step of 7.54e-6, above T-2187's 7.11e-6 upper bound (B17), so the absolute cut is ruled out; B2
(2.4e-4 relative) and A26 (last step 6.87e-6, 1.43e-4 relative) rule out the relative cut. The stop rule needs
re-probing (T-2330).
T-1592 (third round, 2026-10-24): a numerical derivative does not explain S either. Newton with a finite-difference
derivative (forward or central, step 1e-8 to 1e-2 absolute or relative to y, also -1e-4 and -1e-3; from 0, 0.1, the coupon rate
or the current yield; stops on the step at 1e-10 to 1e-5 or on |PRICE - pr| at 1e-8 to 1e-6; 560 variants in a scratch grid)
reaches at most 5/178 bit-exact, against 4/178 for analytic Newton from 0.1 (lab family "Newton, finite difference", six
variants: 5/178 bit-exact and 14 within 4 ulp at best; the family union 14/178, the union of all candidates 23/178). So the
back-chained S is not an artefact of assuming an analytic derivative. Nor is S an iterate of Newton on the frequency-free annuity
model (redemption 100, compounding 1, 2, 3, 4, 6, 12 or 365 times a year, or continuous, over the basis term T) from 0, 0.01,
0.05, 0.1, 0.5 or 1: no iterate comes within 5e-4 relative of the starts of A1, D2, E1, E4 or E5 (scratch arithmetic).
Next (T-1592, T-2330): fit S on the T-2304 grid (it moves with term, rate and price) and re-probe the stop rule.
T-2740 (probes/yield-start-one-input, excel-graph 2026-10-25; T-2304's A8 bond `YIELD(DATE(2096,3,1),M,R,P,100.5,F,0)`,
M = DATE(2100,6,30), R = 0.05, P = 98 unless varied; column A frequency 2, B frequency 1; rows 1-41 P 80 to 100 by 0.5,
42-66 R 0 to 0.12 by 0.005, 67-96 M on 30 June 2097 to 2126). The lab's `newton_back_chain` gives:
- Step count: on every row with both twins below the root (rows 22-96 but 67) A and B step back exactly **two**
  iterates to one S, to 1.3e-8 (the back-chain's own precision); one step back they differ by 2.2e-7 to 3.7e-4, three
  steps back by 3.6e-5 to 2.8e-3. So S is two Newton steps before Excel's value on this bond, at frequency 2 and 1 alike.
- At the root (|PRICE(y) - P| below 1e-11, so no S): both twins on rows 1-5 (P 80 to 82) and row 67 (maturity 2097);
  only B on rows 6-21 (P 82.5 to 90), whose A cells alone leave the step count open.
- Rate: S = 0.0046704412 + 1.01128042·R over rows 42-66, to 6e-9, i.e. linear in the coupon rate within the
  back-chain's precision (row 52, R = 0.05, is 0.0552344615, the T-2187 start 0.0552344627).
- Price: at P = 100 S is the coupon rate (row 41: 0.0499999981 and 0.0500000009). Below, S - R grows faster than
  linearly in 100 - P: (S - R)/(100 - P) is 0.0025953 at P 99.5, 0.0026172 at 98, 0.0027328 at 90.5. No simple form
  fitted yet (time-box).
- Term: S falls toward the coupon rate as the maturity grows (0.0593019 for 2098, 0.0552345 for 2100, 0.0526987 for
  2105, 0.0516858 for 2113, 0.0511796 for 2126); (S - R)·T, with T the years from settlement, is not constant (about
  0.0232 at 2101, 0.0353 at 2125), so not a plain 1/T. No form fitted yet.
Read together: a start that equals the coupon at par and moves linearly with the coupon looks like a current-yield or
yield-approximation formula; T-1592 is the place to fit it. Lab test: `test_yield_start_one_input_two_steps_back`.
T-1592 (fourth round, 2026-10-26): the start and the stop rule are fitted.
- Start: S = (100·R + (100 - P)/T) / ((100 + P)/2 - (100 - P)/(4T)), with T the basis's settlement-to-maturity days
  over 360 (over 365 on bases 1 and 3; T-2777 below replaces this term on every basis). On yield-start-one-input
  rows 22-96 (but 67) the two-steps-back start fits a denominator P + w(100 - P) where (1/2 - w)·T is 0.25000 on every
  row (prices 90.5 to 99.5, rates 0 to 0.12, terms 2.3 to 30.3 years). At price 98 and T = 1559/360 that gives
  0.0552344627, the T-2187 start. It does not use the redemption, the frequency or the coupon schedule. Rows 1-21 and 67 sit
  at the root, so the back-chain cannot reach their start.
- Stop: an analytic-Newton step on PRICE (the `^`-priced PRICE that PRICE itself computes) that is below 1e-10 is not
  taken, so the value is the iterate before it. That is why the values sit short of the root. The cut lies between 9.67e-11
  (a step yield-start-sweep C18 does not take) and 1.01e-10 (one yield-start-one-input B21 takes). This replaces the step-size
  cuts tried in T-2187 and T-2304: their 6e-6 to 7e-6 "last steps" are the step before the one that is not taken.
- Lab fit `yield-caret` (family "Newton, fitted start"): 240/370 bit-exact and 361/370 within 4 ulp (`yield`, libm-priced:
  212 and 340). Cut 1e-9: 194/370; 1e-11: 145/370. The previous best was 4/178. The 9 cells left are 7 to 33 ulp off
  (solver-iteration-traces Y!E2; yield-solver-sweep C7 and C11, where the rate is 0.4 or 0.2; yield-start-one-input B11,
  B17, B31, B54, B62 and B69), within 15 significant digits. Lab test: `test_yield_fitted_start_and_short_stop`.
T-2777 (probes/yield-start-term-basis, excel-graph 2026-10-26/27; 159 cells `YIELD(S,M,0.05,P,100.5,F,B)`, columns P 98
F 2, P 98 F 4 and P 110 F 2; 12 settlement-maturity pairs on bases 1, 2 and 3, and rows 37-53 on basis 0 and 4 with 28
and 29 February settlements, plus sub-year terms): the start's term
T is not the basis' days over a year. It is the whole years from settlement, then the basis' days from the last
anniversary of the settlement to maturity over 360 on bases 0, 2 and 4, over 365 on basis 3, and over the length of
the maturity's calendar year (365 or 366) on basis 1. A 29 February settlement has its anniversaries on 1 March in
common years. Examples: 31 January 2010 to 30 January 2020 is 9 + 364/366 on basis 1 (2020 is a leap year), not
3651/365; 5 July 2004 to 15 May 2022 is 17 + 314/360 on basis 2 (gen-m9 A189), not 6523/360; 29 February 2012 to
15 August 2019 is 7 + 167/365 on basis 1 and 7 + DAYS360(1 March 2019, 15 August 2019)/360 = 7 + 164/360 on basis 0,
not DAYS360 over the whole term (2685/360). The 30/360 count starts at the anniversary, so its end-of-February rule looks
at the anniversary, not the settlement: 28 February 2015 to 15 August 2016 on basis 0 is 1 + DAYS360(28 February 2016,
15 August 2016)/360 = 1 + 167/360, not DAYS360's 525/360 (row 46), and 29 February 2012 to 15 August 2019 on basis 4
is 7 + 164/360 (row 41). Counted back from maturity instead, or with 29 February anniversaries on 28 February,
or with the anniversary year's or the settlement year's length on basis 1, the fit is worse (lab arithmetic,
cells bit-exact / within 4 ulp, of rows 1-40: this rule 59/115; forward with 28 February 46/103 at best; back from maturity
48/92 at best; the days over a year 8/14; lab `solvers.start_years` on all 159 cells 79/153, the days over a year
18/34). In the lab, rows 41-53 are all within 2 ulp under this rule, where the old term misses rows 41, 46, 47, 49, 51 and 53 by
3e-15 to 1.4e-9. The 5 cells left (A9, C11, C20, A26, A38) sit at PRICE's root (next step 1e-12 to
1e-16) and are 4e-15 to 1e-14 off, the root-level last digits of T-2788; the engine also leaves the sub-year cells
B50 and A52 10 to 12 ulp off (their term is 344/365 under either rule, so the miss is in the Newton or PRICE), with their twins in the other columns
within 2 ulp. gen-m9/ba4ddb817a8e700e A17 (basis 1, 20 + 284/365) becomes bit-exact; gen-m9/81bcdc0bec0c8e6b A189
(basis 2) moves from 3e-11 off to 8 ulp. The dates are the ones Excel shows: serial 60 is 29 February 1900, so
`YIELD(60,36585,0.05,90,100,4)` (to 29 February 2000) starts from 100 years and keeps Excel's bits
(probes/fin-coupon-walk-bounds-c H12), where 28 February 1900 would give 100 + 1/360. Lab: `solvers.start_years`, pinned in
tools/tests/test_fitlab_solvers.py::test_yield_start_term_whole_years_then_leftover_days.
Ours: since T-1592 the fitted start and stop on the `^`-priced PRICE with an analytic slope (`excel_yield_newton`), with
T-2777's term (`yield_start_years`). Since
T-2391 there is no fallback: an iterate off PRICE's domain is `#NUM!` and the 100th iterate is the value ("YIELD's #NUM!
cells and its 100-step cap").
Off the fit (T-2741, `make eval` on de48145d): of the 370 probe and ironcalc-calc YIELD cells that were visibly off
before T-1592, 367 are now within Excel's 15 digits. Those cells come from yield-start-rate-term, yield-start-sweep,
yield-solver-sweep, PRICE_YIELD.xlsx's I column, solver-iteration-traces, the 1900 date-system readers and the
fin-* probes. PRICE_YIELD!I30 (0.06499999999159671, short of PRICE's own 0.065) is bit-exact. Both
date-system-1900-financial-readers S!E8 twins and yield-start-rate-term S!B20 are 1 ulp off. The twins are the same bond
four years apart, and their start's term in the basis's years differs, so Excel's values differ by 8e-11 and ours
follow (`fn_yield_matches_excel_off_the_fit`). Still visibly off, though only 5 to 27 ulp from Excel (a gap larger than half a
unit in the 15th digit): yield-start-rate-term S!B39 and S!B49 (basis 1, 8 and 30 years) and yield-solver-sweep S!C7
(T-2974).
Status: UNVERIFIED (the algorithm is a fit: 240/370 bit-exact in the lab; 367/370 of T-2741's cells within 15 digits); the values are excel-graph
Tasks: T-1374, T-1493, T-1525, T-1533, T-1583, T-1592, T-2187, T-2304, T-2740, T-2741, T-2777 (the start's term), T-2788 (the last ulps), T-2974 (the three visibly-off cells)

## ODDFYIELD stops short of ODDFPRICE's root
Rule: ODDFYIELD's last digits are not the root of ODDFPRICE's price at the given price, and they fall on both sides
of it, so they are not analytic Newton on that price stopped short (Newton on a convex, decreasing price stays below
the root after its first step; "YIELD stops short of PRICE's root").
Fitted (T-2007 attempt 2): ODDFYIELD is YIELD's Newton ("YIELD stops short of PRICE's root": YIELD's closed-form
start over the term in years from settlement to maturity, a step under 1e-10 not taken, 100 steps at most, #NUM!
when a step is not finite) on ODDFPRICE, but its slope is not ODDFPRICE's: the odd first coupon weighs in the slope
as a regular coupon (100 × rate / frequency at the odd coupon's exponent, not the odd amount DFC/E or DCNL of a
coupon). That wrong slope is what lands Excel above the root. On probes/oddfyield-solver-sweep (a short-first 5%
semiannual bond, the same bond at a 0% coupon, and a short-first annual 6% bond at redemption 120, over prices 50
to 150, and two ODDFPRICE inversions over yields 0 to 0.3) this model puts 98 of 101 zero-coupon cells within 4 ulp
(39 bit-exact; there the slope is exact) and 323 of 324 coupon-bond cells within 1e-13 relative on libm prices,
against 15 for ODDFPRICE's exact slope; in the engine 320 of the 547 sweep cells and every probe ODDFYIELD cell of
the date-system cases pass bit-exact or within the harness' ulp tolerance. Still off: the long first periods at
frequency 2 of PRICE_YIELD.xlsx (K8-K11, K29-K33, K49-K52, issue 2020-01-15 to first coupon 2021-03-01 with the
settlement on a quasi-coupon date), and sweep cells whose yield is near 0 (T-3031). For a long first period the
lead-at-its-exponent form is unverified: no cell that fits has one (the sweep's first periods are short, and the
annual PRICE_YIELD rows that fit have the settlement in the last quasi period). Tried there with no better score
(PRICE_YIELD.xlsx cells passing, of 352; the committed slope 325): a regular coupon in the slope at every
quasi-coupon date from settlement (325) and YIELD's own slope over the regular schedule from settlement to
maturity (321).
- Desktop (ironcalc-calc `FINANCIAL/PRICE_YIELD.xlsx` ODDFPRICE_ODDFYIELD, Mac 16.x cache; each K row inverts
  `J = ODDFPRICE(.., F, ..)` back to F): K2 is 0.04999999999851281 (1.49e-12 below F = 0.05, |ODDFPRICE(K2) - J|
  1.46e-9), K8 0.0499999999321231 (6.79e-11 below), and K23 0.05000000000085884, 8.59e-13 *above* the root. Of the
  31 numeric K cells 25 sit 1.3e-12 to 7.2e-11 below the root and 6 sit 1.7e-13 to 3.8e-12 above it (K23, K26, K27,
  K35, K37, K38).
- Web (excel-graph): the same both-sides pattern. probes/date-system-1900-odd-threshold S!A5 (and its 43 twins in
  the -odd-threshold and -odd-years probes, 1900 and 1904, one bond in different years) is 0.06499377053016654,
  1.61e-14 above the root 0.06499377053015046 (ODDFPRICE residual -2.1e-12); date-system-1900-odd-accrint-bound
  S!A3 is 2.49e-14 below (+1.25e-11) and S!A6 7.79e-14 above (-6.7e-12); date-system-1900-oddf-issue-after-quasi
  S!A17 5.06e-11 above (-4.3e-9); fin-argument-modes H200 1.38e-11 below (+9.9e-9); odd-bond-argument-check-order
  Calc!A258 1.53e-13 above (-1.2e-10). Residuals run from 1e-12 to 1e-8 in price.
- Rejected (T-2007 attempt 1, a scratch grid on the engine's own odd-first schedule, which ODDFPRICE matches
  bit-exact on every eval cell; 99 cells: the 31 desktop K cells read with Excel's cached J, and 68 numeric web
  cells): every candidate below is 0/99 bit-exact and 0/99 within 4 ulp. The engine's central-difference Newton from
  the coupon rate (stop at |dy| < 1e-8, then bisection), which lands on the root, is 0/99 as well.
  - Newton with an analytic slope, or a central (h = 1e-5), forward or backward (h = 1e-4, 1e-5, 1e-6, 1e-8)
    difference; from the coupon rate, 0, 0.1 or YIELD's fitted start (`yield_start` over `yield_start_years`);
    stopping on a step below 1e-6, 1e-8, 1e-10 or 1e-12 (the iterate before the step, or after it) or on
    |price - pr| below the same values; prices discounted with Excel's `^` or with libm, and with the accrued
    interest subtracted from the price or added to the target instead (1,920 variants).
  - The secant from (s, s + 0.01) for the same starts and stop rules.
  - YIELD's fitted model as is (`excel_yield_newton` from `yield_start`, step cut 1e-10).
- Back-chain (analytic Newton run backwards from the desktop cells below the root): one step back is 4e-7 to 4e-6
  below the root, and none of the iterates one to three steps back meets the coupon rate, 0, 0.1 or YIELD's fitted
  start. So whatever explains the above-root cells (a non-analytic slope, or a price inside ODDFYIELD that differs
  from ODDFPRICE's by about 1e-9 near the root) also has to explain these.
Ours: `fn_oddfyield` calls `odd_first_yield` (functions/financial/financial_bonds/yield_solver.rs), which runs
`excel_yield_newton` with `OddFirstSchedule::dprice`, the slope above; before T-2007 it was a central-difference Newton
from the coupon rate with a bisection fallback, landing on the root.
Status: UNVERIFIED (a fit; the long first periods at frequency 2 are not explained); the values are oracle-backed
Tasks: T-2007 (the fit), T-3031 (the long first periods), T-1374 (ODDFPRICE's own last digits), T-1592 (YIELD)

## YIELD's #NUM! cells and its 100-step cap
Rule: with more than one coupon left, YIELD is analytic Newton on PRICE from the fitted start ("YIELD stops short of
PRICE's root") with no fallback. An iterate where the step is not finite (PRICE or its slope leaves its domain) makes
YIELD `#NUM!`, even where PRICE has a root; after 100 steps the 100th iterate is the value, root or not.
- A coupon rate far above the root is `#NUM!`: `YIELD(36665,44491,42,106,100,4,0)` and the gen-m9 cells
  `YIELD(36665,44491,42,106,42,4,0)`, `YIELD(39779,45574,1E+10,86,101,1,3)`, `YIELD(38930,46538,1E+300,116,0.01,1,1)` and
  15 more (roots 5 to 32), while `YIELD(36665,44491,R,106,100,4,0)` computes for R = 0.5 to 30
  (roots 0.47 to 14.3). `YIELD(39681,45662,42,3.14159,102,4)` (a price far below par) is `#NUM!` too.
- A price far above par computes up to 1E4 and is `#NUM!` from 1E5: `YIELD(39448,43831,0.05,P,100,2)` is -0.0222 at
  P = 200, -0.165 at 1000, -0.220 at 2000, -0.340 at 10000, `#NUM!` at 1E5 and 1E6 (roots -0.494 and -0.633). The
  gen-m9 cell `YIELD(39572,45740,1E-10,97,(FISHERINV(Data!$E1)+Data!A2),4.9)` (redemption 1, frequency 4.9 cut to 4)
  is `#NUM!` too.
- With a redemption far above the price (1E+50 to 1E+300), YIELD returns the 100th iterate, below the root:
  `YIELD(37341,45513,1E-9,105,1E+300,1)` is 78.13551092802992 (the root is above 1E+13), the same bits at
  redemptions 1E+100 and 1E+200, 78.13550467139088 at 1E+50; at a redemption of 1E+20 Newton converges (C11). The value
  moves with the rate (78.909 at 0.01, 116.80 at 0.5) and the price (85.41 at price 1, 71.37 at 1E+10), and
  `YIELD(43831,45412,10,999999999999998,1E+300,2)` is 88233.09573268576. The same bits at redemptions 1E+100 to 1E+300
  come from the analytic slope.
Verified (T-2391, 2026-10-26): with the fitted start of T-1592 this rule matches every one of the 53 cells of
probes/yield-newton-domain in type, and every value cell within 7 ulp (the 16 C cells bit-exact); the 13 cells left are
last-digit misses of "YIELD stops short of PRICE's root". It matches the 21 gen-m9 cells T-2391 owned and the two of
T-2747 (probes/t2475-stat-subexpressions P!A214, gen-m9/44a8841352dc48e9 A318). The earlier attempt (Newton from the
coupon rate with no fallback) was `#NUM!` at prices 1500 to 10000: the fitted start, not the stop rule, decides them.
Evidence: probes/yield-newton-domain (53 cells: A price sweep, B coupon-rate sweep, C large redemptions, D low
redemptions and negative roots; excel-graph, 2026-10-24, T-2391); the gen-m9 cells listed in T-2391.
Ours: matches since T-2391 (`excel_yield_newton` in functions/financial/financial_bonds/yield_solver.rs).
Status: VERIFIED
Tasks: T-2391, T-1592

## T-bill settlement below serial 1
Rule: TBILLEQ, TBILLPRICE and TBILLYIELD floor both dates to whole days before they count days, so a
settlement from 0 to below 1 is serial 0 (1900-01-00), a valid date. The one-year limit compares the dates Excel
shows, so 1900-01-00 to 1900-12-31 (366) is within it and to 1901-01-01 (367) is not. A 29 February settlement reaches its
anniversary on 1 March: `TBILLPRICE(DATE(2024,2,29),DATE(2025,3,1),0.1)` = 89.83333333333333, and the fictitious
1900-02-29 does the same (`TBILLPRICE(60,426,0.1)` = 89.83333333333333, `TBILLPRICE(61,427,0.1)` `#NUM!`); 1 March 2024
to 1 March 2025 is within the limit too. `TBILLPRICE(0,1,0.1)`, `TBILLPRICE(0.5,1.9,0.1)` and `TBILLPRICE(39000.7,39001.2,0.1)` are all
99.97222222222221 (one day), `TBILLPRICE(0.5,0.9,0.1)` and `TBILLPRICE(0,0,0.1)` are `#NUM!` (equal days);
`TBILLPRICE(0,366,0.1)` = 89.83333333333333 and `TBILLPRICE(0,367,0.1)` is `#NUM!` (over a year), like
`TBILLPRICE(1,367,0.1)` = 89.83333333333333 one day on. `TBILLEQ(0,366,0.1)` = 0.11013984535998977 and
`TBILLYIELD(0,366,98)` = 0.020073603211776512; `TBILLYIELD(1E-15,100,1E+300)` = -3.6.
Evidence: probes/tbill-settlement-below-one (34 cells, excel-graph, 2026-10-25, T-2487); gen-m9/0a957119475760c5 A309,
gen-m9/e8d55ec393460bb7 A287.
The gen-m9 T-bill cells whose error looked out of order follow the T-bill order; an inner call differs
(probes/tbill-error-order-inner-calls, each case's Data sheet copied, excel-graph, 2026-10-25, T-2487): Excel's
`T.TEST(E11,A6:C12,2,D8)` there is `#VALUE!` (ours `#NUM!`), `PPMT(2*B11,8,360,-100,926)` 0 (ours 2E17, so ours reads
an off-calendar settlement first), `QUARTILE(C7,E9)` of a text cell `#NUM!` (ours `#VALUE!`) and `BIN2DEC` of its text
cell `#NUM!` (ours `#VALUE!`); `TBILLYIELD(36586,1,MAP(Z1:Z3,<#REF!>,..))` is `#VALUE!` like ours. Those four cells
moved to T-2496.
Ours: matches since T-2487 (`tbill_arguments` in financial/discount.rs, `is_less_than_one_year` in financial/dates.rs).

## T-bills settled in 9999
Rule: TBILLEQ, TBILLPRICE and TBILLYIELD are `#NUM!` for a settlement in the year 9999, however short the bill, in both
date systems; a settlement on 9998-12-31 still computes, to any maturity up to 9999-12-31. As if Excel needs the
settlement's one-year anniversary as a date before it applies the one-year limit.
- `TBILLPRICE(DATE(9999,1,1),DATE(9999,1,10),0.05)`, `TBILLYIELD(...,98)`, `TBILLEQ(...,0.05)` and
  `TBILLPRICE(2958464,2958465,0.05)` are `#NUM!` (A9, A17, A18, A3); `TBILLPRICE(DATE(9998,12,31),DATE(9999,1,10),0.05)`
  = 99.86111111111111 (A8), and with a settlement of 9998-12-31.5 the same (A16); `TBILLPRICE(DATE(9998,12,31),
  DATE(9999,12,31),0.05)` = 94.93055555555556 (A12).
- The 1904 workbook's last year is the same 9999: `TBILLPRICE(2957003-100,2957003,0.05)` and its TBILLYIELD and
  TBILLEQ twins are `#NUM!` there (fin-1904-calendar-readers A2, A21:A25), while 9996 dates compute (A1, A9).
Evidence: probes/fin-1900-tbill-last-day A1:A19, fin-1904-calendar-readers (excel-graph, 2026-10-27, T-2337).
Ours: `is_less_than_one_year` (financial/dates.rs) since T-2337.
Status: VERIFIED (excel-graph)
Tasks: T-2337

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
- FVSCHEDULE: a boolean principal is `#VALUE!` from a TRUE or FALSE cell (`C1`, `C2`), `INDEX(C1:C2,1)` or `INDIRECT("C1")`; a text-number cell (`"1000"`) is converted (1100). The reference path matches the typed path, so `cast_number_no_bools_ref` (the lifted kernel's read) is right (T-1992).
Evidence: probes/fin-fvschedule-bool-ref (excel-graph, 2026-10-19, T-1992).
Status: VERIFIED (excel-graph)
Implemented (T-1764; the fractional-period rules are now the fractional grid of "VDB charge order", T-2185): VDB with salvage above cost never switches to straight line (-600); a fractional end period
prorates the unclipped declining-balance charge and the total is capped at `cost - salvage` (A9 is 0; salvage <= cost only, salvage above cost keeps the clipped charge); AMORDEGRC
rejects only an asset life (1/rate) in [0, 3] or strictly between 4 and 5 (exactly 4 and 5 are valid, coefficients
1.5 and 2). The salvage-above-cost rule (A17) and the cap (A9) are each inferred from one probe cell. The AMORDEGRC bands at life 3 and strictly between 4 and 5 are documentation-only. Probes needed: T-1978. The ulp differences (VDB A6, A7, A10, A13, MDURATION, RRI) are fixed by T-1975: see "VDB charge order" and "DURATION and RRI operation order".
Implemented (T-1765): a typed boolean principal is `#VALUE!` (A18); a text-number principal still converts. A boolean principal read through a reference is `#VALUE!` too, verified by probes/fin-fvschedule-bool-ref (T-1992).
Tasks: T-1764 (incl. MDURATION, RRI ulp), T-1765

## VDB charge order
Rule: VDB sums the charges of the periods inside its window; it never subtracts two running totals. Its two modes
build the per-period charges differently, so the same inputs give different last bits with `no_switch` TRUE and FALSE:
- `no_switch` TRUE: period i charges `min(b·rate, b - salvage)` (at least 0), `rate = factor/life`, with the book
  value raised directly, `b = cost·(1-rate)^(i-1)` (square-and-multiply, like the operator `^`). The charges are
  summed in doubles; a partial first or last period is that charge times its share of the window.
  `VDB(2400,300,10,7,10,2,TRUE)` is 203.3164800000004 and `VDB(2400,300,10,0,10,3,TRUE)` is 2099.9999999999995.
  The floor at 0 holds from the first period: with salvage above cost, `VDB(2400,3000,10,0,1,2,TRUE)` is 0, where the
  switching mode gives -600. With `factor >= life` the rate is 1: `VDB(1000,100,2,0,1,3,TRUE)` is 900 and the next
  period 0 (probes/fin-rri-vdb-edges A17-A24).
- `no_switch` FALSE: the periods are laid on the fraction of the start. The first `f = start - floor(start)` of a
  period charges `f·min(cost·rate, cost - salvage)`, floored at 0, and whole periods follow from there (at `f`, `1+f`,
  ...). Each period's book value `b` steps down by its charge `min(b·rate, b - salvage)` until the straight-line
  charge `(b - salvage)/(life - t)`, `t` the period's start, is larger than the unclipped `b·rate`; from then on every
  period charges that same amount. The window's declining-balance charges (and a prorated last period) are summed in
  doubles, and the whole straight-line periods are added at the end as one product `n·sl`:
  `VDB(1234.56,78.9,12,4,9,1.75)` is 379.8854487417499, one ulp below the period-by-period sum, and
  `VDB(2400,300,10,7,10,2,FALSE)` is 203.31648000000007. A whole start has `f = 0`, the plain schedule.
  Consequences of the fractional grid:
  - A fractional start past the switch sees the same straight-line charge wherever it lies, set by its fraction:
    `VDB(2400,100,10,6.5,7.5)` and `VDB(2400,100,10,8.5,9.5)` are both 133.20886857142858, `(566.23104 - 100)/3.5`,
    and `VDB(2400,100,10,6.3,7.3)` is 132.80996324324326, `(591.396864 - 100)/3.7`, where the whole-period schedule
    charges 132.2864 (probes/fin-vdb-fractional-start A11:A15). `VDB(2400,300,10,8.3,9.9,1.5)` is 244.0757706382978,
    straight line from the 5.3 grid point over 4.7 periods (fin-vdb-schedule A44).
  - The switch test uses the unclipped charge, so a clipped last piece under one period stays declining:
    `VDB(2400,300,10,9.25,10)` is 4.512314880000062, `0.75·(b - salvage)`, while `VDB(1000,5,10,9.25,10,4)` switches and
    charges all of `b - salvage`, 4.069926399999998 (fin-vdb-fractional-start A44, fin-vdb-fractional-start-edges A1).
    This replaced T-1975's comparison with the clipped charge for whole starts too; a whole-start period rarely tells
    the two apart, mostly only with a fractional life (edges A2, A9, A12), and every whole-start cell still passes.
  - The first partial charge is clipped: `VDB(1000,900,5,0.5,1)` is 25, and `VDB(1000,950,4,1.5,3)` is 0. With salvage
    above cost it is 0 and the negative charges follow: `VDB(2400,3000,10,0.5,1.5)` is -600 (edges A3:A7). The old
    "fractional end period capped at `cost - salvage`" reading of `VDB(2400,300,10,9.5,10)` = 0 (T-1764) is this rule
    too: the 9.5 grid point's book value is the salvage.
  The model matches 153 of the 161 switching-mode cells to the bit and the other 8 to one ulp, all with a start
  fraction of .2 or .3 (T-3022). LibreOffice's model (a restart from the start with its half-life shift) and the
  running-total difference this replaced both miss visibly: fin-vdb-schedule A44 is 250.50773812499992 and
  242.0635349999999 with them.
Evidence: probes/fin-vdb-schedule (78 cells), fin-vdb-modes, fin-rri-vdb-edges (excel-graph, 2026-10-20, T-1975);
fin-vdb-fractional-start (45 cells), fin-vdb-fractional-start-edges (14 cells) (excel-graph, 2026-10-28, T-2185).
Status: VERIFIED (excel-graph)
Implemented (T-1975, T-2185): `vdb_no_switch_between` and `vdb_switching_between` in
functions/financial/financial_depreciation.rs.
Tasks: T-1975, T-2185, T-3022

## VDB, DDB and ACCRINT argument types
Rule: VDB's `no_switch` reads as a boolean. TRUE, FALSE and a number are taken, and so is the text "TRUE" or "FALSE"
in any case, typed or from a cell. Any other text is `#VALUE!` before the range checks, numeric text too ("1", "0",
"x"), and it is `#VALUE!` even with a cost of -1 (`VDB(-1,0,39,1,9,10,"x")`, A63). The value checks then apply:
`VDB(1,-0.0001,23,1,9,10,"true")` is `#NUM!`. An empty no_switch is FALSE (`VDB(1000,100,10,0,1,2,)` is 200).
VDB takes a factor of 0. With no declining balance, the switching mode is straight line and no_switch gives 0:
`VDB(1000,100,10,0,1,0)` is 90, with TRUE it is 0, and `VDB(43478,488,1E+9,3,7,0)` is 0.00017196. A negative
factor is `#NUM!`. An empty or blank factor is 0, not the default 2: `VDB(1000,100,10,0,1,,FALSE)` and a blank cell
give 90, and with TRUE they give 0 (A59:A62). VDB also takes an empty window at period 0: `VDB(1000,100,10,0,0)` and `VDB(0.01,5,25,0,0,3)` are 0.
VDB's factor and DDB's factor read a boolean as 1 or 0 and a numeric text as its number. `DDB(1000,100,10,1,TRUE)` is
100, also from a TRUE cell. With FALSE (factor 0) it is `#NUM!`, with "3" it is 300, and with "TRUE" it is `#VALUE!`.
ACCRINT's `calc_method` takes a boolean but no text: "FALSE", "TRUE" and a "FALSE" cell are all `#VALUE!`, before a
negative rate's `#NUM!`.
The error-order cells of the T-2257 sweep come from the arguments' own errors. `VDB(-1,0,39,<ACCRINT(..,"FALSE")>,9,10,0)`
is `#VALUE!` because that ACCRINT is `#VALUE!`, while `VDB(-1,0,39,SQRT(-1),9,10,0)` is `#NUM!` and
`VDB(73409,713,1/0,SQRT(-1),10)` is `#DIV/0!`, the first error in argument order.
Evidence: probes/t2492-vdb-ddb-arguments A1:A64 (excel-graph, 2026-10-25, T-2492).
Implemented (T-2492): `fn_vdb`, `fn_ddb` (functions/financial/financial_depreciation.rs) and `accrint_of`
(functions/financial/accrint.rs). VDB from a fractional start in the switching mode still misses by value (A1, A9:A12),
which T-2185 owns. A218 of gen-m9/d0854366830dcdee is `#NUM!` because `PERCENTILE(<TRUE cell>,7)` is `#NUM!` (A55), and ours
`#VALUE!` (T-2605).
Status: VERIFIED

## DB and DDB operation order
Rule: DB and DDB compute each period's charge in a fixed operation order, and the last bits depend on it.
- DB: `rate = ROUND(1 - (salvage/cost)^(1/life), 3)`; the first period is `cost * rate * (month/12)` (month over 12
  first, not `cost*rate*month/12`). Later periods keep the remaining *balance* (`balance = cost - first`, then
  `balance -= balance*rate` per period), not the accumulated depreciation; a middle period charges `balance*rate` and
  the last one (period = TRUNC(life)+1) `balance * (rate*(12-month)/12)` (rate times the months first).
  `DB(100,10,4,1,2)` is 7.299999999999999 (not 7.3), `DB(23,10,4,5,2)` is 1.8687313963962309 and
  `DB(10000,1000,6,2,7)` is 2596.3941666666665. `month` is truncated before use and cost 0 returns 0.
- DDB: `rate = factor/life` (1 once it reaches 1), the book value `old = cost*(1-rate)^(period-1)` raised with the
  operator `^` (square-and-multiply for a whole period, numbers.md "Integer exponents"), not libm `pow`
  (`DDB(9695,282,20,8,1)` is 338.5190042814453, `pow` gives ...452; gen-m9 6376e27be7a05871 Calc!A141); a fractional
  period takes `^`'s non-integer path too (`DDB(1E+300,872,39,1.005,1)` is 2.563769566671006E+298, gen-m9
  79d87fc527927626 Calc!A98), and the charge
  `MIN(old*rate, old - salvage)`, floored at 0, rather than the difference of two book values
  `old - cost*(1-rate)^period`: `DDB(10000,1000,6,2,2)` is 2222.222222222222 (not ...2217),
  `DDB(100,10,4,3,1.3334)` is 14.814814787037497 and `DDB(5832,692,25,1)` is 466.56 (not 466.5599999999995).
Evidence: ironcalc-calc FINANCIAL/DB_DDB.xlsx Sheet1 (Excel-cached; every numeric DB and DDB cell, incl. G4, G5, G7,
G10, G20, G31, G35-G37, G39, G42, H10), probes/fin-argument-modes H88, H95 and the gen-m9 financial sweep
(excel-graph, T-2257 triage, 2026-10-23).
Status: VERIFIED (excel-graph)
Implemented (T-2033): `db_depreciation` and `ddb_depreciation` in functions/financial/financial_depreciation.rs.
Tasks: T-2033

## DURATION and RRI operation order
Rule: DURATION discounts each cash flow with Excel's `^` (not libm `pow`) and weights it as `(t·cf)/v^t`, then
`num/den/frequency`; MDURATION is exactly the worksheet `DURATION(...)/(1+yld/frequency)`. A zero-coupon
`DURATION(DATE(2020,3,15),DATE(2025,9,30),0,0.06,2,0)` is 5.541666666666668, one ulp above `t/2`, because `(t·cf)/v^t`
divided by `cf/v^t` is not exactly `t`. RRI is `EXP(LN(fv/pv)*(1/nper))-1` (T-2216 sweep below; the worksheet `^` form fits only 112 of 136
numeric points): `RRI(10,1,200)` is 0.6986464646342472, where libm `pow` gives ...474.
A negative `fv/pv` is always `#NUM!` for RRI, even where the worksheet form is a number: `RRI(1,-100,200)`,
`RRI(0.5,-100,200)` and `RRI(1/3,-1,8)` are `#NUM!`, while `(200/-100)^(1/1)-1` is -3 and `(8/-1)^3-1` is -513
(probes/fin-rri-vdb-edges A1-A16).
Evidence: probes/fin-mduration-rri-fit (60 cells: 12 DURATION/MDURATION/quotient triples, 12 RRI/`^` pairs),
fin-mduration, fin-rri (excel-graph, 2026-10-20, T-1975).
Earlier open point (2026-10-21 gap triage, settled by the T-2216 sweep below): the web probes above have an integer nper or an exact `fv/pv`.
The one fractional-nper, inexact-ratio cell in the repo is Mac desktop: ironcalc-calc
FINANCIAL/RRI.xlsx Sheet1!D12, `RRI(0.5,3,4)` = 0.7777777777777775 ("Microsoft Macintosh Excel"
16.0300). The worksheet form `(4/3)^2-1` gives 0.7777777777777777, the engine's value. So either Mac
desktop differs from the web here, as for SIN(4) and ATAN2 (decision 0074), or RRI is not the
worksheet form when nper is fractional. T-2216 probed it on excel-graph (below).
T-2216 sweep (probes-numeric/m9-fin-rri-sweep, excel-graph, 143 rows: the web agrees with the Mac value, `RRI(0.5,3,4)` is
0.7777777777777775, so RRI.xlsx D12 is not Mac-only). Of the 136 numeric RRI results, the worksheet forms match
`EXP(LN(fv/pv)*(1/nper))-1` 134 times (the two misses are `RRI(1,3,0)` = -1 and `RRI(1,0,0)` = 0, where that form is `#NUM!` / `#DIV/0!`),
`(fv/pv)^(1/nper)-1` and `POWER(fv/pv,1/nper)-1` 112 times each, and `EXP(LN(fv/pv)/nper)-1` 87 times. So RRI is
`EXP(LN(fv/pv)*(1/nper))-1`, with the reciprocal taken first; the `/nper` candidate is refuted (it misses 49 of 136).
RRI is no worksheet form at the edges (rows 131-139): fv=0 is -1 and pv=fv=0 is 0; pv=0 with fv<>0 is `#NUM!`; nper <= 0 is
`#NUM!` even where the worksheet forms are numbers (nper -1 and -0.5 give -0.25 and -0.4375); `RRI(1E-5,1,2)` is `#NUM!`
(overflow) while `RRI(0.001,1,2)` is 1.0715086071861939e+301.
nper exactly 1 skips EXP/LN: RRI is `fv/pv-1` there. `RRI(1,826,40633)` is 48.19249394673123, `40633/826-1`, where
`EXP(LN(40633/826)*(1/1))-1` is ...124; 32 of 32 nper-1 rows agree, while nper 0.9999999999999999 and 1.0000000000000002 (one
ulp either side) follow the EXP/LN form (probes-numeric/m9-fin-rri-nper-one, excel-graph, 2026-10-28, T-2217; the case that
showed it is gen-m9/1cd3096e3e87c965 Calc!A291).
Status: VERIFIED (excel-graph)
Implemented (T-1975, RRI's EXP/LN form and nper-1 case T-2217): `duration_of` in functions/financial/financial_bonds.rs, `rri_of` in functions/financial/growth.rs.
Resolved by the T-2216 sweep above: RRI.xlsx Sheet1!D12 `RRI(0.5,3,4)` is 0.7777777777777775 (Mac cached, and the web), ours
0.7777777777777777; E12 `POWER(4/3,1/0.5)-1` is 0.7777777777777777, so RRI is not POWER/`^`. The engine's `excel_exp`/`excel_ln`
form with `*(1/nper)` fits D12, RRI.xlsx D3-D6 and every excel-graph RRI cell above. Implemented: T-2217 (supersedes T-2222).
Tasks: T-1975, T-2216, T-2217

## NPV with a rate of -1 or 1E+300 is #DIV/0!
Rule: NPV whose rate is -1 is `#DIV/0!` (`NPV((-1),(-0.5),D06!A$9,D06!B8:B11)`, `NPV((-1),D17!B7:B11)`, `NPV(D22!$B4,..)` with
B4 = -1, `NPV(((-1)-D12!$C4),..)` with C4 empty), and so is NPV whose rate is 1E+300 (`NPV(1E+300,(D34!E3+0.5),{-100,-200,-300})`; that an overflowing discount factor
causes it is a hypothesis, and the threshold between rates that give a value and rates that give `#DIV/0!` is unprobed). The `#DIV/0!` carries through the enclosing calls (`MODE.SNGL(D06!B11:B12,(NPV(..)+..))`).
Evidence: probes/t2475-stat-subexpressions P!A116, A148, A177, A236, A297, A429 (excel-graph, 2026-10-24, T-2475); the gen-m9 cells listed in T-2618.
Ours: at -1 a non-finite number that leaks out of NPV (TYPE reads it as a number; the cell shows `#NUM!`, and `NPV(..)+..`
gives `#NUM!`), and 8E-301 at 1E+300 (T-2618).
Status: VERIFIED (excel-graph)
Tasks: T-2475, T-2618

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

## IRR over an error element
Rule: IRR (probes/fin-irr-error-array, T-2390, excel-graph) gives `#VALUE!`, whatever the kind, when its values hold an
error: in an array constant (`IRR({-100,#N/A,60})`, also `#DIV/0!`, `#NUM!`, `#VALUE!`, `#REF!`, `#NAME?`, first, middle or last,
and two errors), in a range (`IRR(A1:A3)` with `#N/A` or `#DIV/0!` in A2), in a single cell holding an error (`IRR(A2)`) and
in an array computed from one (`IRR(A1:A3+1)`), with or without a guess. An error that is itself the argument passes through
(`IRR(NA())` is `#N/A`, `IRR(1/0)` `#DIV/0!`), as does an error guess (`IRR(E1:E3,A2)` is `#N/A`). Text and TRUE in an array
or range are skipped (-0.40000000000000013 for `{-100,"a",60}`, `{-100,TRUE,60}`, and cells C1:C3, D1:D3).
The rest of the family is not like IRR: MIRR, NPV, XIRR and XNPV return the element's own error (`#N/A`) for an array
constant or a range, and for an error in XIRR's/XNPV's dates.
Ours: matches every probed cell (after T-2390).
Status: VERIFIED
Tasks: T-2390

## Financial edge-pool smoke check (T-2248)
Rule: observed values at the edges of the M9 financial generator pools (tools/src/rustsheet_tools/gen/pools.py), one
formula per pool entry:
- The annuity `type` is any nonzero number as the start of the period: `PV(0.05,12,-100,0,t)` is 930.6414218271249 for
  t = 1, 2, -1, 0.5, 1.5, TRUE and "1", and 886.325163644881 for 0 and FALSE; `"x"` is `#VALUE!`.
- A coupon frequency truncates: `COUPNUM(39448,43831,2.5)` is 24 and `...,4.9)` 48; `"2"` is coerced (24); 3, 0, 12, -1
  and -0.0001 are `#NUM!`.
- EFFECT floors npery: `EFFECT(0.05,1.5)` = `EFFECT(0.05,1)` = 0.050000000000000044, and npery 0.5, 0.999, 0 and -1 are
  `#NUM!`. `EFFECT(0.05,1E+10)` is 0.05127110072502061 (ours 0.05127110027547532, T-2392).
- PMT: a rate of -1 and nper 0 are `#NUM!`; a negative nper computes (`PMT(0.05,-12,1000)` = 62.82541002081538); nper
  1E+6, 1E+9 and 1E+300 give -50 (`-pv*rate`). IPMT per 0, 0.5, -1 and past nper (13 of 12) is `#NUM!`; per 1.5 computes.
- IRR: no sign change, a single flow and all zeros are `#NUM!`; text and TRUE inside an array literal are skipped
  (`IRR({-100,"a",60})` = `IRR({-100,TRUE,60})` = -0.40000000000000013), and an error element makes it `#VALUE!`
  (`IRR({-100,#N/A,60})`; see "IRR over an error element").
- XNPV: dates whose first is not the earliest, dates from serial 0 to 61, and fewer dates than values are `#NUM!`; equal
  dates give 0; fractional dates truncate.
- YIELD: a price of 0 or -1 is `#NUM!`, and so are 1E+6 and 1E+300 (ours matches since T-2391); a price of
  200 computes (-0.02224401627496982). The price sweep in probes/yield-newton-domain locates the cut between 1E4 and 1E5
  ("YIELD's #NUM! cells and its 100-step cap").
Evidence: a scratch suite of 169 cells (one formula per entry of the pools rate, nper, period, cash_amount, price,
annuity_type, security_date, basis, frequency, npery, cash_flows and cash_dates), filled through excel-graph on
2026-10-22 and deleted afterwards, as the task asked; T-2390, T-2391 and T-2392 commit probes for the three mismatches.
Ours: 127 of 169 cells match; the rest are the known PMT/IPMT last-digit misses ("PMT, IPMT and PPMT are the exact
payment"), YIELD's ("YIELD stops short of PRICE's root"), two IRR last-digit misses and the three cells above.
Status: UNVERIFIED (probe pending: T-2391, T-2392 commit the probe cases)
Tasks: T-2248, T-2390, T-2391, T-2392

## XIRR and XNPV under the gen-m9 statistical cells: size and type checks (T-2255)
Rule (inferred through the enclosing call: each cell's value is that of a statistical function whose other arguments
cannot give it): XIRR whose values and dates have different counts, or whose values are one number or one cell, is
`#NUM!`, also when either range holds text or booleans: `F.INV(XIRR(Data!D11:D12,Data!C6:C9,-1),1,1)` (2 values, 4 dates,
gen-m9/0a614cfe974b59c3 Calc!A390), `CHISQ.INV((XIRR(0.5,{0,1,59,60,61})/..),8)` (b8ee55934c5b824c A122),
`F.TEST(XIRR(Data!D5,Data!D8:E9),1.005)` (4ab2ba98d26c8a5e A155), `GAMMAINV(XIRR(-2.5,Data!B8:C8),1.005,0.5)`
(a790a4d4a6a82756 A379), `LARGE(XIRR(Data!C11:D12,Data!C10:C11),1)` (FALSE and "a*" cells, bbdfc06cae0e0d71 A44) are all
`#NUM!`; an error guess is that error (`XIRR({0,0,-100,50,60},Data!A8:D11,(-0.0001/Data!B12))` with B12 empty gives
`#DIV/0!`, 0a9226016e8f5f33 A373; `XIRR(Data!B3:D3,(Data!A2/Data!$C10))` `#DIV/0!`, 40425341c5034eed A216). XNPV is
`#VALUE!` for a boolean or text one-cell dates or rate (`XNPV(0.05,Data!E4,Data!C2:E12)` with E4 = FALSE,
320d51765af9b6b7 A346; `XNPV(Data!B$7,Data!E$2,..)` with text, 984cba67bbe6972b A323; `XNPV(Data!D2,Data!A10:A10,..)`
with both TRUE, bc40d37cb4d8301d A392) and `#NUM!` for a number against a range of a different count
(`XNPV(1,2.5,Data!A4:C12)`, ee2497a5e683c5d9 A107; `XNPV(0.5,-0.5,Data!B4:E8)`, bc40d37cb4d8301d A79).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 51 cells listed in T-2428.
Order (XIRR, T-2428, from 198 gen-m9 cells now passing): an error dates argument, then (beside a number or omitted guess;
a text, boolean or error guess keeps the element reading's `#VALUE!` or that error) one value with one date is `#N/A`
unless either is a text, boolean or empty cell (`#VALUE!`; an array element such as BYROW's `{FALSE}` is not a cell, so
`#N/A`); a single date that is a text, boolean or empty cell is `#VALUE!`; a single text or boolean value cell against
several dates is `#VALUE!`; then different counts are `#NUM!` whatever a larger range holds (gen-m9/0a957119475760c5 A47);
then the element reading. Open (T-2428): an error guess beside numeric values is the guess's error in Excel
(1b60bf99d71424e7 A29, 67a4c03afd63f988 A388) but text values beside an error guess are `#VALUE!` (b72d952c5f52cb4d A50).
Order (XNPV, T-2428): the rate first, per element, where a boolean is `#VALUE!` (a cell, `XNPV(<TRUE>,..)`,
1dce4774f5c0e736 A22, or an array element, `XNPV(SORT(<FALSE>),..)`, 5c6a23c4f6597f1c A258); then one values cell that is
non-numeric text or a boolean is `#VALUE!`, before an error dates argument (`XNPV(-1,<FALSE>,<#N/A>)`, 3aa168259335be54
A334), while numeric text there is not refused (`DB(XNPV(0.1,<"1.5">,Data!C5:C6)*..)` is the count's `#NUM!`,
81bcdc0bec0c8e6b A37); then an error dates argument is that error, before the values' cells (`XNPV(0.05,Data!B1:E2,<#N/A>)`,
5c6a23c4f6597f1c A111; four cells with a `#VALUE!` dates expression beside a values range holding text); then one dates cell
that is non-numeric text or a boolean is `#VALUE!` (d0854366830dcdee A274); then values and dates of different counts are
`#NUM!` whatever they hold, an error value included (`XNPV(0.1,{-100,#N/A,60},<5 dates>)`, bc04c1f478c9896c A313;
`XNPV(10,{100,200,300},<6 cells with text>)`, 1b60bf99d71424e7 A24); then the element reading. Open (T-2611): equal counts
with text in both (1b60bf99d71424e7 A171 `#VALUE!`) and a boolean date inside a range (5ded7594e597e31e A12 `#NUM!`).
Evidence: 38 of the 48 gen-m9 cells whose formula is a bare XNPV or FVSCHEDULE call (excel-graph, direct), and the
nested cells listed in T-2428.
Ours: XIRR follows its order (`xirr_size_error`) and XNPV its (`xnpv_size_error`, the rate read without booleans) since
T-2428; the error-guess cells are T-2611's.
Status: VERIFIED (excel-graph; direct for the bare XIRR and XNPV cells, inferred through the enclosing call otherwise)
Tasks: T-2428, T-2611

## FVSCHEDULE with one empty schedule cell is the principal
Rule: a schedule that is one empty cell is an empty schedule, so FVSCHEDULE returns the principal: `FVSCHEDULE(6527,D1)`
is 6527 and `FVSCHEDULE(2758,C7)` 2758 with the cell empty, also through a reference-returning IF
(`FVSCHEDULE(1E+300,IF(Data!C7,Data!B$3,Data!B9))` is 1E+300), while text in the schedule stays `#VALUE!`.
Evidence: gen-m9/30e20ba3302fea8b A250, e8d55ec393460bb7 A3, 6376e27be7a05871 A140 (excel-graph, 2026-10-23, direct).
Ours: matches since T-2428 (`fn_fvschedule`). `FVSCHEDULE(5916,{-1E+300,1E+300})` is 0 in Excel (89f0aee953a55a45 A349),
ours `#DIV/0!` (T-2611).
Status: VERIFIED (excel-graph)
Tasks: T-2428, T-2611

## Financial functions in the gen-m9 financial sweep (T-2257)
Rule: the generated financial cells show these Excel rules that ours does not follow yet (each cell is a gen-m9 formula whose
outer call is the function; "inferred" means the other arguments nest calls whose Excel value is taken to be ours):
- Argument order of the Analysis ToolPak securities functions (PRICE, YIELD, DURATION, MDURATION, the ODD* four, the
  discount securities, AMORLINC, AMORDEGRC; inferred): an earlier out-of-range date, a settlement after the maturity, or a
  negative rate is `#NUM!` before a later argument's error, text or boolean is looked at. `PRICE(36917,-0.5,HOUR(..)<#VALUE!>,..)`
  (81bcdc0bec0c8e6b A7), `ODDLYIELD(-0.0001,39378,-2.5," x ",92,106,0)` (67a4c03afd63f988 A341), `DURATION(2958466,46410,..,
  <#VALUE!>,4,0)` (b6a02893a8d6b151 A213) and `PRICEDISC(39668,-1,<text>%,106)` (abb7d565af89b849 A294) are `#NUM!`. For PRICE,
  YIELD, DURATION and MDURATION the order is now probed ("PRICE, YIELD, DURATION and MDURATION check order", T-2482), and for
  the ODD* four ("ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order", T-2483). AMORLINC and
  AMORDEGRC give a boolean rate or salvage `#VALUE!` before a negative cost or a salvage above the cost
  (`AMORLINC(-439103,37926,39037,412,7,FALSE,..)` b73c6cc96e093f03 A360, `AMORDEGRC(27029,37050,39005,789,1E15-0.1,TRUE)`
  1b60bf99d71424e7 A298 are `#VALUE!`); now probed ("AMORLINC and AMORDEGRC check order", T-2485), as are the discount
  securities ("Discount securities check order", T-2484).
- COUP* functions: a settlement or maturity past 9999-12-31 is `#NUM!` as it is read, before a later error or text
  (`COUPDAYS(123456789.123,<FALSE>,4)` 89f0aee953a55a45 A363, `COUPNUM(37484,2958466,2,<#VALUE!>)` 1dce4774f5c0e736 A19), and
  `COUPDAYS(39123,2958466,1,0)` is `#NUM!` (a91bad4e84f49739 A153). `COUPDAYBS(37913,46720,4,4)` = 50 (7d2979fba2c95d64 A304) and
  `COUPDAYS(5E-05,2958465,2.5,1)` = 184 (b72d952c5f52cb4d A88). Now probed: "Analysis ToolPak securities functions check
  frequency and dates first", "Coupon dates keep the maturity's day" and "COUPDAYS basis 1 counts from the chained
  next coupon" (T-2486, T-2640).
- ODDLPRICE with the settlement before `last_interest` is `#NUM!`: `ODDLPRICE(59,38163,33447,0,10,110,1,4)` (0a957119475760c5 A351),
  `ODDLPRICE(36111,38608,36584,..)` (89f0aee953a55a45 A8); probed, with ODDLYIELD and a settlement on `last_interest`, in
  "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD check order" (T-2483).
- A settlement serial below 1 is valid for the T-bills: `TBILLPRICE(1e-15,1,100)` = 72.22222222222221 (0a957119475760c5 A309),
  `TBILLYIELD(0.3,1,98)` = 7.346938775510203 (e8d55ec393460bb7 A287); and AMORLINC's purchase date may be below 1:
  `AMORLINC(46557,0,39881,440,0,0.1)` = 46117 (1b60bf99d71424e7 A368); now probed and followed ("AMORLINC and AMORDEGRC
  check order", T-2485).
- CUMIPMT and CUMPRINC are `#NUM!` for a start period below 1, fractional included: `CUMPRINC(0.05,249,329832,0.3,19,0)`
  (0a957119475760c5 A40).
- SYD with a negative salvage is `#NUM!` (`SYD(36746,-100,22,3)`, 31474c20fc1dafd6 A207) and with a life of 1E+300 `#DIV/0!`
  (5ded7594e597e31e A38); DB with a negative salvage or a life of 1E+300 is `#NUM!` (`DB(0,-439103,27,10)` b72d952c5f52cb4d A303,
  `DB(6912,0,1E+300,2,6)` e168be7f6d3f84c2 A70); `DB(0.01,488,32,1E+200,10)` = -0.0033416666666666673 (762cd0e4ec9a7aa9 A151),
  a first Excel value past the "DB and VDB period cap".
- VDB takes a text `no_switch`: `VDB(17334,389,36,0.2,9,3,"FALSE")` = 9114.101951893414 (762cd0e4ec9a7aa9 A379); DDB takes a
  TRUE factor from a cell without `#VALUE!` (`DDB(3.14159,866,13,7,<TRUE>)` = 0, 81bcdc0bec0c8e6b A8); `VDB(43478,488,1E+9,3,7,0)`
  = 0.00017196 (a91bad4e84f49739 A323). Now probed and followed ("VDB, DDB and ACCRINT argument types", T-2492).
- NPER at a rate of 1E-300 is `#DIV/0!` (`NPER(1E-300,-236,2766,945)`, 3c06f9298f65a3ba A148; five more 1E-300 cells in
  T-2698), and at a rate of -1 it is `#NUM!` (`NPER(-1,-31,1)`, 3c06f9298f65a3ba A50; both implemented by T-2620, "Annuity and DB argument domains at their
  limits"). NPV at a rate
  of -1 is `#DIV/0!` (`NPV(-1,GESTEP(-2.5,-100))`, 3aa168259335be54 A18), EFFECT at a huge nominal rate `#NUM!` (`EFFECT(1E+300,11)`,
  31474c20fc1dafd6 A37).
- `IRR(CELL("row",Data!D$2),0.5)` is `#VALUE!` (5422bdbf173fc233 A63; the CELL mark, "IRR's error order", T-2728), while plain scalar flows and
  one-cell ranges are `#NUM!` (`IRR(123456789.123)` e168be7f6d3f84c2 A297, `IRR(MROUND(2^53,2.5),0.5)` d0854366830dcdee A69,
  `IRR(Data!A8:A8)` 1cd3096e3e87c965 A130, `IRR(Data!D2,2)` 1dce4774f5c0e736 A40); a guess of 1E+300 is `#VALUE!`
  (`IRR({0,0,-100,50,60},1E+300)`, e168be7f6d3f84c2 A395), `IRR(Data!A1:D12,0.05)` over mixed flows (1E+15 and
  123456789.123 among them) is `#DIV/0!` with the cause unprobed (bc04c1f478c9896c A56), and Excel gives `#NUM!`
  on 11 flows where ours finds a root near -1 or far out (T-2494). XIRR with one value and one date is `#N/A`
  (`XIRR(Data!A9:A9,Data!E9:E9)`, 30e20ba3302fea8b A110).
- PPMT at a huge rate is often exactly 0 (`PPMT(1E+10,12,278,130633)`, 5422bdbf173fc233 A285) and `IPMT(1E+10,10,320,2)` is
  -20000000000 (89f0aee953a55a45 A124).
- YIELD is `#NUM!` at a coupon rate far above the yield (`YIELD(38829,45714,100,0.25,2,4,4)`, 31474c20fc1dafd6 A321), and PRICE,
  DURATION and MDURATION at a huge yield or coupon (`PRICE(39050,44044,0.5,123456789.123,110,4,2)`, 1dce4774f5c0e736 A8;
  `MDURATION(38368,44642,0.01,1E+300,4,3)`, c0681a8b150ce301 A248). PRICE's, DURATION's and MDURATION's boundary is now
  probed ("PRICE, DURATION and MDURATION at a huge yield or coupon", T-2488); YIELD's is unprobed (T-2391).
- FVSCHEDULE with an empty schedule cell returns the principal (`FVSCHEDULE(6527,<empty>)` = 6527, 30e20ba3302fea8b A250).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2257); the cells and the rest of each family are listed in the owning tasks.
Ours: see each task; the triage (T-2257 Outcome) lists every family.
Status: VERIFIED (excel-graph; the argument-order rules inferred through nested calls)
Tasks: T-2482, T-2483, T-2484, T-2485, T-2486, T-2487, T-2488, T-2489, T-2490, T-2491, T-2492, T-2493, T-2494, T-2495, T-2496,
T-2428, T-2241, T-2391

## The discount securities read basis first (T-2455)
Rule: DISC and RECEIVED read the basis before their other arguments, so an invalid basis or a basis error beats an earlier
argument's error, a settlement error too (`DISC(1/0,2.5,1,100,"abc")` is `#VALUE!`, A91): `RECEIVED(37025,2.5,1E+300,1/0,"abc")` and `DISC(37025,2.5,1/0,100,"abc")` are `#VALUE!`, and
`RECEIVED(37025,2.5,1E+300,NA(),1/0)` is `#DIV/0!` (probes/t2455-transpose-single-value A21, A64, A65, excel-graph,
2026-10-23); gen-m9 ba4ddb817a8e700e A345 has the same shape.
INTRATE, PRICEDISC and YIELDDISC are the same (`..(37025,2.5,1/0,100,"abc")` is `#VALUE!`, A80:A82).
Ours: since T-2455 (`discount_args`, shared by the five); the full order, PRICEMAT and YIELDMAT included, is in
"Discount securities check order" (T-2484).
Status: VERIFIED (excel-graph)
Tasks: T-2455, T-2484

## Discount securities check order
Rule: DISC, INTRATE, PRICEDISC, RECEIVED, YIELDDISC, PRICEMAT and YIELDMAT read their arguments in this order and check
only the basis and the dates as they read them:
1. basis: an error, text or boolean is returned, and a basis outside 0-4 (5, -1) is `#NUM!`, before anything else
   (`DISC(NA(),43831,95,100,5)` and `YIELDMAT(NA(),..,-1)` are `#NUM!`, `DISC(NA(),43831,95,100,1/0)` `#DIV/0!`);
2. settlement, maturity, then PRICEMAT's and YIELDMAT's issue: an error, text or boolean is returned, and a serial that
   does not floor to 0-2958465 (9999-12-31) is `#NUM!` as it is read (`DISC(-1,NA(),..)`, `DISC(39448,2958466,"x",..)`,
   `PRICEMAT(39448,43831,-1,NA(),..)` are `#NUM!`; `DISC(NA(),-1,..)` is `#N/A`). Serial 0.5 passes the read
   (`DISC(0.5,NA(),..)` is `#N/A`);
3. the amounts (price, investment, discount, redemption, rate, yld): an error, text or boolean is returned;
4. only then the date order and the amount checks, `#NUM!` (`DISC(39448,39000,NA(),..)` is `#N/A`,
   `DISC(39448,43831,0,"x",0)` `#VALUE!`).
ACCRINTM reads in its own order, issue, settlement, rate, par, basis, with the same checks as it reads (both dates
on the calendar, the basis in 0-4), so its basis comes last (`ACCRINTM(-1,39448,0.1,1000,NA())` is `#NUM!`,
`ACCRINTM(NA(),39448,0.1,1000,5)` `#N/A`). A boolean is `#VALUE!` in every position of all eight, text "1" is 1.
`RECEIVED` divides as `investment / (1 - discount*DIM/B)` left to right: `RECEIVED(39448,43831,"1",0.05,0)` is 2.5, where
`discount*(DIM/B)` gives 2.5000000000000004 (A758; the "Discount securities: day count and year length" form left this
open).
Evidence: probes/discount-argument-check-order (1707 cells: each function with one argument out of its domain against
`NA()`, text and `TRUE` in every other position, `NA()` and `1/0` in every pair of positions, and `TRUE`, text and "1"
alone in each position; excel-graph, 2026-10-24, T-2484). Every cell fits the rule above.
Unprobed: a 1904 workbook (the read checks the date after the 1904 shift, as for the T-bills).
Ours: matches since T-2484 (`discount_args`, `maturity_args`, `accrintm_of` and `discount_date_of` in
functions/financial/discount.rs; `day_count_basis_of` in financial/dates.rs, shared with `bond_args`).
Status: VERIFIED (excel-graph)
Tasks: T-2484

## Booleans in ACCRINT, ACCRINTM, AMORLINC and AMORDEGRC (T-2455)
Rule: a boolean is `#VALUE!`, typed or from a cell, in ACCRINTM's par (`ACCRINTM(34825,37622,1E-9,TRUE,4)`, A41, A54; text "2"
is 2, A55), ACCRINT's issue, also before first_interest's error (`ACCRINT(TRUE,39500,..)`, `ACCRINT(TRUE,SQRT(-1),..)`, A52,
A53), ACCRINT's rate (`ACCRINT(39000,39500,0.1,TRUE,40000,2,0)`, A56, before the settlement's `#NUM!`) and AMORDEGRC's rate
(`AMORDEGRC(90812,36624,38707,453,3,TRUE,4)`, A51, A57): probes/t2455-transpose-single-value, excel-graph, 2026-10-23. gen-m9
165db58d1540f014 A141, 0a614cfe974b59c3 A265 and 7c20351562a2d71d A212 show it through F.TEST. A purchase or first_period
date past the calendar is `#NUM!` before AMORDEGRC's boolean rate (`AMORDEGRC(70537,36888,7E13%,46,12,<TRUE>,0)`, gen-m9
e8d55ec393460bb7 A374), while a date of 0 passes the read (`AMORDEGRC(-1,<blank>,38872,265,0,<#VALUE!>,0)` is `#VALUE!`,
1dce4774f5c0e736 A199) and one below 0 is `#NUM!` as it is read ("AMORLINC and AMORDEGRC check order").
Ours: since T-2455 (`cast_number_no_bools_ref` in `accrintm_of`, `accrint_of` and the AMORLINC/AMORDEGRC kernel, which since
T-2485 refuses a boolean in every argument, "AMORLINC and AMORDEGRC check order"). ACCRINT's other dates and numbers
still take booleans; a FALSE par cell is `#VALUE!` in Excel ("ACCRINT check order: calc_method first, dates as read",
T-2792), the others are unprobed.
Status: VERIFIED (excel-graph)
Tasks: T-2455

## ACCRINT check order: calc_method first, dates as read (T-2676)
Rule: ACCRINT reads its last argument, calc_method, before all the others: calc_method's own error is the result, and
text there (`"1"`, `"TRUE"`, `""`) is `#VALUE!` ("VDB, DDB and ACCRINT argument types"), whatever the first seven
arguments hold, an issue off the calendar included. Then it reads issue, first_interest, settlement, rate, par,
frequency and basis in order, and an issue or first_interest that does not floor to a calendar serial (below 0, or past 2958465) is `#NUM!`
as soon as it is read, before a later argument's error, text or boolean, as the COUP* readers do ("Analysis ToolPak
securities functions check frequency and dates first"). A par read from a FALSE cell is `#VALUE!`, before frequency
100's `#NUM!`.
- calc_method first: `ACCRINT(33996,35923,39160,<#DIV/0!>,2872,2,0,"1")` is `#VALUE!` (gen-m9/290180acf017709c A32),
  `ACCRINT(34182,2958466,37446,<#NUM!>,4792,2,4,<#REF!>)` `#REF!` (932b33b1ef8b7608 A323),
  `ACCRINT(43830,42,-0.5,<#REF!>,<#NUM!>,2,4,<#N/A>)` `#N/A` (290180acf017709c A132), and with a text issue cell
  `ACCRINT(<"a*">,35416,61,1,3393,2,3,<#NUM!>)` `#NUM!` (A22); also 290180acf017709c A297, A342, A372, A387,
  f81bd3b6dc373e5c A55, A70, A105, A145, A225, A285. Over the 240 Excel-backed gen-m9 cells with ACCRINT as the outer call,
  all 40 where calc_method is an error or text give calc_method's result, among them cells that already passed with an
  issue below 0 (290180acf017709c A82, A392) or a first_interest below 0 (f81bd3b6dc373e5c A310).
- dates as read: `ACCRINT(34205,2958466,43831,<"a" cell>,1719,1,3)` is `#NUM!` (290180acf017709c A117),
  `ACCRINT(1.005,-3.05,37242,-238.09,<#DIV/0!>,2)` `#NUM!` (932b33b1ef8b7608 A33), `ACCRINT(33191,-0.2525,37264,
  <T.TEST(..)>,3606,4,3)` `#NUM!` (290180acf017709c A47), `ACCRINT(123456789.123,35882,37568,0.5,6884,<"ü" cell>)`
  `#NUM!` (f81bd3b6dc373e5c A290), and an inner `ACCRINT(-1,35650,39448,<TRUE cell>,7682,4,3)` is `#NUM!` before the
  boolean rate's `#VALUE!` (290180acf017709c A162, through STEYX). Of the 10 gen-m9 cells with a date off the calendar
  before a later bad argument, 9 are `#NUM!` (also A37, A157, A232 of 290180acf017709c, A43 of 932b33b1ef8b7608 and A255
  of f81bd3b6dc373e5c); the tenth, f81bd3b6dc373e5c A115, is `#SPILL!` because its par is an array, which lifts first.
- boolean par: `ACCRINT(32965,35462,36817,10,<FALSE cell>,100,1.005,FALSE)` is `#VALUE!` (932b33b1ef8b7608 A233).
Unprobed: whether an issue after the settlement waits for the later arguments (ours checks the order after every
read); a settlement past 2958465 (every cell past 2958465 is an issue or first_interest) or below 0 read before a later error; a first_interest of 0 in a 1900 workbook (ours: `#NUM!`); a typed boolean par, and
booleans in first_interest, settlement, frequency and basis.
Evidence: gen-m9/290180acf017709c, 932b33b1ef8b7608, f81bd3b6dc373e5c (excel-graph, T-2676 sweep, 2026-10-26); each
cell's arguments evaluated with the engine on the case's Data sheet.
Ours: since T-2792, `accrint_of` (functions/financial/accrint.rs) reads calc_method first, then issue and first_interest, each range-checked from serial 0 as it is read (serial 0 itself still fails the later calendar check), then the other arguments in order; settlement is checked against its calendar limit only after all are read, and a boolean par, typed or from a cell, is `#VALUE!` (the engine cannot tell a typed one from a cell).
Status: VERIFIED (excel-graph)
Tasks: T-2792

## XNPV reads one empty values cell as a single 0 cash flow
Rule: `XNPV(1,A2,B11)` and `XNPV(1,A2,B11:B11)` with A2 empty and B11 = 100 are the number 0 (not `#NUM!`); an empty cell
in a multi-cell values range stays `#NUM!` (whole-column probe xirr-xnpv-whole-column-nogaps A1). The one-cell range
`A2:A2` reads the same (`XNPV(1,A2:A2,B11)` and `XNPV(1,A2:A2,B11:B11)` are 0), but an empty dates cell is no date:
`XNPV(1,B11,A2)`, `XNPV(1,B11,A2:A2)` and `XNPV(1,A2,A2)` are `#NUM!` (probes/xnpv-empty-cell-forms C1-C5). A date of 0
after a value is `#NUM!` (`XNPV(1,A2,B12)`, B12 = 0). Only direct references were probed: an expression that gives one empty cell (INDEX, OFFSET, a LET name) reads the same here, unprobed. This narrows "XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty cells" to ranges. `XNPV(1,A2:A3,B11:B12)` with both values empty is `#VALUE!` in Excel; ours is
`#NUM!` (known failure, unexplained: an empty range seems to be refused by a different check than the whole-column one).
Evidence: probes/xnpv-empty-values-cell C1-C8 and xnpv-empty-cell-forms C1-C5 (excel-graph, 2026-10-24); it surfaced through gen-m9/eccd03df42e40006 A24
(`PERCENTRANK(<no numbers>, XNPV(..))` is `#N/A`, so the XNPV is no error).
Status: VERIFIED (excel-graph)
Tasks: T-2066, T-2599 (C8: T-2603)

## Financial cells of the T-2436 sub-expression probe
Rule (excel-graph, 2026-10-24, probes/t2436): `YIELD(36563,45559,999999999999995,107,107,2,1)` (a huge rate) is `#NUM!`;
`MDURATION(-1.5,46394,<#N/A>,100,2)`, `ODDFPRICE(2958466,46506,33984,39398,-1,<#N/A>,104,4,2)`,
`YIELDDISC(9.33,1E+15,0,<#VALUE!>,5)` and `ACCRINT(39448,..,100,<#REF!>,836,4,2,2)` are `#NUM!`, consistent with a range
check before the other argument's error (the order is inferred, UNVERIFIED); `XIRR({-100,TRUE,60},<two dates>,1E-9)` is `#NUM!`; `NPV(-1,..)` is `#DIV/0!` and so is
`NPV(1E+300,{0,0,-100,50,60})`; `IRR(Data!C10:D12,<blank>)` is `#NUM!`; SYD with a negative salvage is `#NUM!`.
Evidence: probes/t2436; cells in T-2482, T-2483, T-2484, T-2428, T-2493, T-2494, T-2491.
Ours: values or the other argument's error; those tasks.
Status: VERIFIED (excel-graph), one or two cells each
Tasks: T-2482, T-2483, T-2484, T-2428, T-2493, T-2494, T-2491

## Financial cells of the T-2496 sub-expression probe
Rule (excel-graph, 2026-10-25, probes/t2496-fin-subexpressions, each gen-m9 case's Data sheet copied): in the gen-m9
financial cells whose error the outer call cannot make, an inner call differs and the outer call passes its error on.
`PV(-1+2^-52,91,-421)` is `#DIV/0!` (ours `#NUM!`; hypothesis: (1+rate)^91 underflows to 0, T-2657 probes it); `IRR(<4x4 flows>,41.9999)` is
`#DIV/0!` (ours converges on 4.1E+12, T-2657); `NPV(-1,1E-15,2.5)` and `NPV(1E+300,7,1E+15,2.5)` are `#DIV/0!` (T-2493, as
in the T-2436 probe); `XMATCH(FALSE,{1,2,3},0,0.5^1E-15)` is `#N/A` (lookup.md, T-2656). The statistical inner calls
follow statistics.md: Z.TEST's one-number `#DIV/0!` and lone-empty-cell `#N/A` (T-2470), PROB's unequal-count `#N/A` and
one-value `#DIV/0!` (T-2468), FREQUENCY's extra row for a bins argument with no number, so the enclosing ODDFYIELD and
CUMIPMT lift to a 2x1 array and show `#SPILL!` (T-2427), `T.TEST` of a blank one-cell sample (T-2469) and WEIBULL.DIST
overflow (T-2435). The T-bill cells: `PPMT(2E+15,8,360,-100,926)` is 0 (T-2490), `QUARTILE` of a TRUE cell `#NUM!`
(T-2344), `BIN2DEC("x")` `#NUM!` (T-2544).
Evidence: probes/t2496-fin-subexpressions P!A1-A99 (column B names the gen-m9 cell and sub-expression);
probes/tbill-error-order-inner-calls.
Ours: the other error or a value; the tasks named.
Status: VERIFIED (excel-graph), one or two cells each
Tasks: T-2496, T-2656, T-2657, T-2493, T-2470, T-2468, T-2427, T-2469, T-2435, T-2490, T-2344, T-2544

## NPV's residue at IRR's root is not the double sum
Rule: NPV divides by (1+r) once, at the end: NPV(r, v1..vn) = (v1/d0 + v2/d1 + ... + vn/d(n-1)) / (1+r), where d0 = 1
and d(i) = d(i-1)*(1+r) is a running product. All of it is in doubles: 1+r rounded, each product, each quotient and
each partial sum (left to right) rounded to a double. Where NPV's terms cancel, this differs from both the exact sum
and the plain double sum of v_i/(1+r)^i by more than half a unit of the 15th significant digit of the result. At M2,
M8 and Three Parameters!E2 below (and at H367 and A148) Excel is nearer the exact value than the plain double sum, at
M3 further from it.
Evidence for the rule (T-2671, probes/npv-term-order, excel-graph, 382 cells at 20 rates, the rates exact doubles in
R!A1:A20: IRR.xlsx's L2, L3, L8 and Three Parameters!D2, the m9-fin-nper-npv rates 0.05 to 0.2 and 11 rates searched
where T-1528's candidates differ most; generator docs/research/t2671_npv_term_order.py):
- one-term NPV(r,v) is v/(1+r) (40/40 cells);
- zero-padded NPV(r,0,..,0,1) over k periods is (1/d(k-1))/(1+r) with d the running product (220/220, periods 2-12).
  Not 1/(1+r)^k by any power source: 1/d(k) 141/220, libm pow(1+r,-k) 116, repeated division 108, Excel's `^` 78;
- two- and three-term NPVs with values of opposite sign and close magnitude, and the IRR flows at L2 and D2, sum the
  quotients left to right before the last division (122/122, P!A75 and A78 being the same formula; summed right to
  left 116/122). NPV(r,-1,1+r) is exactly 0 in Excel (e.g. P!A55, A189): the rule gives that zero itself
  (-1/1 + (1+r)/(1+r)), so there is no snap of near-zero sums.
Lab: `uv run rustsheet-fitlab annuity-npv` (now also on probes/npv-term-order), candidate "double: deferred (sum
v/(1+r)^(i-1))/(1+r)": 470/470 bit-exact over m9-fin-nper-npv and npv-term-order (`x87 op` the same). The others:
Horner from the last value 254/470, the running factor d*=(1+r) with v/d per term 228, sum of v/P with libm pow 190
(at 64 bits 186), with Excel's `^` (the engine today) 159, with the exact or x87-instruction power 107, correctly
rounded 82. T-1528's best on m9-fin-nper-npv alone was 27/88. Lab test: `test_npv_deferred_division_reproduces_excel`.
The rule also gives every residue value below to the bit (M2, M3, M8, Three Parameters!E2, H367 and A148).
Evidence for the residues (ironcalc-calc/FINANCIAL/IRR.xlsx, saved by Excel Online; the rate cell is an IRR that ours matches to the
bit, so NPV gets the same inputs):
- General!M2 `NPV(L2,A2:F2)`, flows {-20,5,5,5,5,5}, L2 = 0.07930826116039724: Excel 6.4212018552224666E-12, exact at
  that rate 6.421923509333131E-12, ours before T-2672 (left-to-right sum of v/(1+r).powi(i); with libm `pow` M2 is 6.419753617592505E-12) 6.418865439172805E-12.
- General!M3, flows {-1000,230,200,125,200,10}, L3 = -0.1015006760535847: Excel -2.8358465972398184E-11, exact
  -2.815865414018216E-11, ours -2.8180124900245573E-11.
- General!M8, flows {-70,12,15}, L8 = -0.44350694133432755: Excel -1.579429159894141E-10, exact
  -1.5794321715940695E-10, ours -1.5792522845003987E-10.
- Three Parameters!E2 `NPV(D2,A2:C2)`, flows {1,-2,1}, D2 = 1.091399510766422E-07: Excel 1.187938506697367E-14, exact
  1.1911525020941654E-14, ours 1.2101430968414206E-14.
- Also General!M5, M9, M10, M11, M13, FullRow!Q2 and FullColumn!B21, B22 (same flows and rates as E2, M2 and M3 over
  whole rows).
- The same holds away from a root: probes/fin-argument-modes H367 `NPV(0.1,-10000,2750,4250,3250,2750)` is
  302.233453999042 in Excel (exact at the double rate 302.23345399904366, ours 302.23345399904133); probes-numeric/m9-fin-nper-npv A148
  `NPV(0.1,-1E6,250000,250000,250000,250000,250000)` is -47548.46149808013 (exact -47548.46149807995, ours
  -47548.4614980802).
Rejected before T-2671 (gap triage 2026-10-25, doubles, on the 9 rate-exact residues and H367): sum of v/(1+r)^i left
to right or reversed, Horner from the last value with / or * 1/(1+r) (matches M8 only), a running factor by * or /,
v*(1+r)^-i, v/exp(i*log1p r), fsum, and the future-value Horner divided by (1+r)^n. T-1528's Blocked section lists its
own rejected forms on m9-fin-nper-npv.
Ours: the rule, since T-2672 (`excel_npv`; the "ours" values above are from before it). Held out on the eval
(2026-10-27): the 12 rate-exact IRR.xlsx cells above 12/12 bit-exact; fin-argument-modes H186, H367, H368, H372 4/4;
m9-fin-nper-npv 88/88; npv-term-order 382/382; gen-m9 NPV cells 252/255 (36 fixed). Of the three left, gen-m9
abb7d565af89b849 Calc!A387 `NPV(IRR(..,-0.5),..)` is IRR's: ours is 9999.199996060153 and Excel's NPV needs a rate
near 9999.19999605711, 1673 ulp lower (T-2131); 3c06f9298f65a3ba A317 is FORECAST.ETS's. The one miss of the rule is
gen-m9 9d9ceba257f8001b Calc!A85 `NPV(1E-10,<empty>,{-100,50,60,0,0},{0,0,0})`: Excel 9.999999981999995, the rule
9.999999982000002 (4 ulp, within 15 digits). There (-100 + 50*(1/b) + 60*(1/b)^2)/b and (-100 + 50*b^-1 +
60*b^-2)/b give Excel's bit, the running product, its repeated division, the plain powers and Horner do not; unprobed
which of them, if any, generalises (T-2672's phase-1b follow-up).
The 12 IRR.xlsx cells whose rate is off in ours (General!M4, M6, M12, M16-M19, Three Parameters!E3-E5, FullRow!R2,
FullColumn!B23) are the rule at Excel's cached rate to the bit (10/10 distinct cells), so what is left there is IRR's
(T-2131).
Status: VERIFIED (values: the IRR.xlsx Excel Online cache, excel-graph sidecars fin-argument-modes, m9-fin-nper-npv and
npv-term-order; the rule is bit-exact on all 470 lab cells).
Implemented in: functions/financial/financial_util.rs (`excel_npv`)
Tasks: T-2671, T-2672, T-1528, T-2131

## Annuity and DB argument domains at their limits
Rule: PV's growth factor `P = (1+rate)^nper` is Excel's `^`, errors included, and PV then divides by P:
- a rate of -1 with nper 0 is 0^0, which is `#NUM!` (`PV(-1,0,-713)`);
- a P of 0 is the division's `#DIV/0!`. That covers a rate of -1 with nper > 0, and a rate so near -1 that P underflows
  (`PV(-1+2^-52,165,-978,224,0)`, where 1+rate is 2^-52 and its 165th power is 0).

NPER divides by ln(1+rate):
- a rate so small that 1+rate is 1 is `#DIV/0!`, start-of-period type 1 too (`NPER(1E-300,-113,3037,368,1)`);
- a rate of -1 is `#NUM!` (LN(0); `NPER(-1,-31,1)`).

DB's `month` reads a boolean as 1 or 0, from a cell too, as DDB's factor does ("VDB, DDB and ACCRINT argument types").
`DB(15779,385,37,<FALSE>,<FALSE>)`, with period and month both FALSE cells, is `#NUM!`, not `#VALUE!`. Only the FALSE
pair is probed; TRUE as month 1 is inferred from DDB.

Evidence: probes/t2475-stat-subexpressions P!A39, A92, A136, A165 (excel-graph, 2026-10-24, T-2475), and the gen-m9 cells
they isolate: 11eb3ab4f4f8104c A109, 1b31092a0d79e5f1 A260, 31f09e6c48ca245c A232, 38e4fc309fe199ea A89, and
3c06f9298f65a3ba A50, A148 (T-2698's list).
Ours: `pv_of` and `nper_of` (functions/financial/annuity.rs), `fn_db` (functions/financial/financial_depreciation.rs), T-2620.
Inferred from the same formulas, unprobed: PV's `#DIV/0!` covers every P that underflows to 0 (a rate in (-2,-1)
with a large even nper too, `#NUM!` before), and NPER's LN of a non-positive ratio is `#NUM!` before the ln(1+rate)
`#DIV/0!` (`NPER(1E-300,0,100,200)` is `#NUM!`).
Status: VERIFIED for the probed cells; DB's boolean month beyond the FALSE pair and the inferred lines are UNVERIFIED
(T-2749 probes them).
Tasks: T-2620, T-2698, T-2749
