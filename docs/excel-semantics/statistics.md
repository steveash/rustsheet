# Statistical distributions

Entries found while triaging the distribution families (T-0903). Values are desktop Excel's
cached results in the upstream Excel-authored workbooks under
`crates/rustsheet-xlsx/tests/calc_tests/STATISTICAL/` (eval suite `ironcalc-calc`).

## FISHERINV is (EXP(2y)-1)/(EXP(2y)+1), with its cancellation near 0
Rule: FISHERINV(y) returns `(exp(2y) - 1) / (exp(2y) + 1)` evaluated in doubles, not a
correctly rounded tanh. Near 0 this loses digits: `FISHERINV(1E-7)` = 1.0000000005039292E-07 and
`FISHERINV(-1E-7)` = -1.0000000001086735E-07 (tanh gives 9.999999999999967E-08). For large y,
where exp(2y) overflows, the result is 1 (`FISHERINV(1000)`, `FISHERINV(1E10)`).
Evidence: `ironcalc-calc/STATISTICAL/FISHER_FISHERINV.xlsx` `FISHER FISHERINV`!C40, C41 (small y),
C38, C39 (large y), C10, C51; `ironcalc-calc/STATISTICAL/FISHER.xlsx` Sheet1!G2, G3. The formula
reproduces every numeric FISHERINV cell in both workbooks bit for bit (32 cells, checked with
glibc `exp` during T-0903, counting overflow as 1); `tanh` misses 6 of them.
The quotient is Excel's `/`, rounded twice (next entry): `FISHERINV(TAN(PI()))` is -2^-53
(gen-m9/ee2497a5e683c5d9 A30), a plain quotient one ulp further from 0 (T-2437).
Status: VERIFIED (desktop cached values; excel-graph for the twice-rounded quotient)
Implemented in: `functions/statistical/fisher.rs` `fisher_inv`
Tasks: T-0976, T-2437

## FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)
Rule: each is a short formula whose `/` and `+` behave like Excel's operators (numbers.md "Division
rounds twice") around Excel's own LN and EXP kernels:
- FISHER(x) = 0.5 * LN((1 + x) / (1 - x)), the quotient rounded twice. Near 0 this is Excel's
  cancellation, not atanh: `FISHER(1E-15/3.14159)` = 2.2204460492503126E-16 (gen-m9/4ab2ba98d26c8a5e
  A395, probes/t2436/4ab2ba98d26c8a5e A256), where a plain quotient gives 3.3306690738754686E-16 and
  atanh the argument itself, 3.18E-16; `FISHER(1.49172367472348E-11)` = 1.4917178603246007E-11 (5 digits
  of 15 off atanh) and `FISHER(1.02759470998055E-16)` = 0 (probes/t2437-fit-separators A1, A4).
- FISHERINV(y) = (EXP(2y) - 1) / (EXP(2y) + 1), the quotient rounded twice (entry above).
- EXPON.DIST's cdf at t = lambda * x is -expm1(-t) with Excel's Kahan expm1 (numbers.md "Excel's
  expm1 is Kahan's formula") for t < 1, and 1 - EXP(-t) above. The cut lies between
  t = 0.999999999999999 (Kahan's digits, A117) and 1.35079141386477 (1 - EXP's), and the two forms
  give the same double at every t from 1 to 1.35 a search tried (probes/t2437-fit-separators A67:A117:
  Kahan at all 26 points 0.198 <= t < 1, 1 - EXP at all 25 points 1.3508 <= t <= 8.07). libm's expm1 is one ulp off at `EXPONDIST(TAN(100*PI()),1E+8,
  TRUE)` = 1.959369974159244E-07 (gen-m9/06b4e8f5de409d5e A140) and `EXPON.DIST(PI(),0.01,TRUE)` =
  0.030927573695189364 (623663cacea53bbc A318). The density, lambda * EXP(-t), is unchanged.
- HARMEAN = 1 / AVERAGE(1/x): the reciprocals summed in argument order, the sum divided by n, then
  inverted, every `/` and `+` twice rounded; not n / sum(1/x). `HARMEAN(Data!D6:D10,Data!C6:C9)` =
  0.569594502522996 (gen-m9/06b4e8f5de409d5e A278), n / sum one ulp above.

Fit lab (`rustsheet-fitlab fisher|fisherinv|expon-cdf|harmean`, every JSON case of eval/ the four
functions appear in; bit-exact cells, the misses of the winners are the sweep's parser, not the kernel):

| fit (cells) | chosen form | other candidates | engine before |
|---|---|---|---|
| `fisher` (155) | **152**: 0.5*LN(q), q rounded twice | sums 1±x also twice 152 (tie); libm ln, q twice 150; LN, plain q 142; libm, plain q 140; 0.5*(LN(1+x)-LN(1-x)) 94; atanh 68 | 143 |
| `fisherinv` (143) | **133**: EXP, quotient twice | sums e±1 also twice 133 (tie); libm exp, quotient twice 133; plain quotient 121 (either exp); tanh 85 | 131 |
| `expon-cdf` (132) | **129**: Kahan below t = 1, else 1-EXP | cut at 2: 121, 0.5: 115, 4: 114, 8: 105; libm expm1 95; Kahan everywhere 91; 1-EXP 74 | 98 |
| `harmean` (122) | **119**: 1/AVERAGE(1/x), + and / twice | plain +: 118; plain /: 116; both plain 115; n/sum(1/x) 103-104 | 106 |

The sums `1 ± x` and `e ± 1` stay plain doubles: rounding them twice as well scores the same (FISHER 152,
FISHERINV 133), so the sweeps do not separate the two and the simpler form is kept. The libm/EXP and LN choices
tie on these sweeps too; the engine uses Excel's kernels (`excel_ln`,
`excel_exp`, `excel_expm1`). Two owned gen-m9 cells stay off for their inputs, not these kernels: on
Excel's own input bits both are exact. `FISHER(T.DIST.RT((1.005^1e-15),4))` (a612e89321a6df9c A191)
needs Excel's `T.DIST.RT(1,4)` = 0.18695048315002957 (probes-numeric/m9-t-dist-rt A49; ours, correctly
rounded, is 0.18695048315002943; T-2509), and `EXPON.DIST(T.DIST(4199,4,FALSE),0.3,TRUE)`
(caa1eed48b1883fe A382) is 0.3 * T.DIST(4199,4,FALSE), whose Excel value 9.192880751152396E-18 is 3 ulp
below ours (excel-graph, 2026-10-28, T-2437; the student t density, T-2585).
Status: VERIFIED (excel-graph); where in [1, 1.3508) the EXPON.DIST cut lies is not observable on
the doubles searched (the engine uses 1)
Implemented in: `functions/statistical/fisher.rs` `fisher`, `fisher_inv`; `exponential.rs` `expon_cdf`;
`count_and_average.rs` `harmonic_mean`; fits `tools/src/rustsheet_tools/fitlab/kernels/fisher_expon_harmean.py`
Tasks: T-2437

## T.DIST.RT and T.DIST.2T are not 1 - T.DIST
Rule: the Student-t upper tail is computed on its own, not as `1 - T.DIST(x, df, TRUE)`.
`T.DIST(1,1,TRUE)` = 0.75 exactly, yet `T.DIST.RT(1,1)` = 0.25000000000000006 and
`T.DIST.2T(1,1)` = 0.5000000000000001, so the two tails do not add up to 1. The far tail keeps
full relative precision: `T.DIST.RT(1E6,10)` = 1.230468749943611E-56 (equal to `T.DIST(-1E6,10,TRUE)`),
where `1 - T.DIST` would be 0. Small df have closed-form-exact CDFs: `T.DIST(1,2,TRUE)` =
0.7886751345948129 = 0.5 + 1/(2*sqrt(3)) in doubles.
Evidence: `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`!D15, F15, D16, F18, D19;
`T.DIST.2T T.INV.2T`!C15.
Model (T-0969, probes-numeric `m9-t-dist-*`): everything follows from one tail L = P(T > |t|):
`T.DIST.RT(x,n)` = `T.DIST(-x,n,TRUE)` wherever the sweep has both, T.DIST of a positive t is
1 - L (bit-exact on 118 of the 146 nonzero T.DIST cells, against 113 for the correctly rounded
CDF: `T.DIST(1,1,TRUE)` = 1 - 0.25000000000000006 = 0.75), and T.DIST.2T is 2 L. With
y = n/(n+t^2) and z = t^2/(n+t^2), L = I_y(n/2,1/2)/2. In the deep tail (y <= z) Excel's L
carries the error of exp((n/2) ln y): Cephes' power series with that power and a correctly
rounded Gamma ratio is within 4 ulp on all but 2 deep-tail cells (`T.DIST.RT(1E6,10)` is 1 ulp
off it, 33 ulp off the exact value). Away from it the exact L is 0 to 4 ulp from Excel on most
cells but not all: `T.DIST.RT(1.959999998,60)` (`T_DIST.xlsx` F7) is 13 ulp off, and Excel's own
tail noise there is not modelled yet. The density (cumulative FALSE) is noisy too: correctly
rounded on 38 of 150 cells; `T.DIST(0,10,FALSE)` = 0.38910838396603115 is 3 ulp above the
correctly rounded 945/(768 sqrt 10), and the error grows with |t| (165 ulp at
`T.DIST(1000,100,FALSE)`), the signature of a power computed as exp(a ln y).
At df 2 the density is `EXP(-1.5*LN(2+t^2))` with Excel's EXP and LN, bit-exact on all 1,024 cells of a run of
consecutive doubles. At df 3, 4, 5 and 10, a regression of Excel's relative error on the intermediates' rounding errors
over runs of consecutive t shows t*t rounded to a double but neither n + t^2 nor 1 + t^2/n. The noise left is about 1
to 5 x 2^-53 relative rms and grows with the exponent's size, as from one stored exponent. Still, no form
D * exp(E) or exp(C + E) with E = RN(-((n+1)/2) ln(1 + t^2/n)) or the ln(n + t^2) form is bit-exact on more than
about 60% of a df's cells, for any D or C (probes-numeric `t0969-t-pdf-runs`, excel-graph, 2026-10-20; T-0969
attempt 7).
Tiny t: Excel forms n + t^2 in doubles, so where it rounds to n the tail is exactly 1/2:
`T.DIST.RT(1E-10,3)` = `TDIST(1E-10,3,1)` = 0.5 (probes-numeric t1048-tdist A27, B27), not the
exact 0.49999999996324...
Status: VERIFIED (observation); the tail model is UNVERIFIED (fits, T-0969); Excel's algorithm is
UNVERIFIED
Tasks: T-0969

## Inverse distribution functions do not round-trip exactly
Rule: feeding a CDF value back into the matching inverse function usually gives back the original x
only to within a few ulp, and Excel's result is the one to match, not x:
`GAMMA.INV(GAMMA.DIST(10,9,2,TRUE),9,2)` = 9.999999999999998;
`BETA.INV(BETA.DIST(0,2,2.5,TRUE,-1,0.234),2,2.5,-1,0.234)` = -2.220446049250313E-16 (not 0);
`T.INV(T.DIST(1.959999998,60,TRUE),60)` = 1.959999997999998; `T.INV.2T(T.DIST.2T(1,1),1)` =
0.9999999999999996. The last digits look like an iterative solver stopping at a tolerance on
Excel's own CDF, so an inverse can only match once that CDF matches bit for bit.
Evidence: `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV`!F2, F19, F21;
`BETADIST_BETAINV.xlsx` Sheet1!H16; `T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`!E7, `T.DIST.2T T.INV.2T`!D15.
Status: VERIFIED (observation); the solver is UNVERIFIED (T-0967, T-0972, T-0974, T-0925)

## GAMMA is exact at small positive integers but not always correctly rounded
Rule: GAMMA(n) for a positive integer n equals the correctly rounded (n-1)! only up to n = 25 (exact up to n = 23) (T-1042 sweep: 131 of the
171 integers differ in the last digits, first at n = 26): `GAMMA(5)` = 24, `GAMMA(6)` = 120,
`GAMMA(1)` = `GAMMA(2)` = 1. Half-integers are not uniformly correctly rounded: `GAMMA(0.5)` =
1.772453850905516 (correctly rounded sqrt(pi)), but `GAMMA(-0.5)` = -3.5449077018110318, one ulp
from the correctly rounded -2*sqrt(pi) = -3.544907701811032 (and not GAMMA(0.5)/-0.5 either).
Evidence: `ironcalc-calc/STATISTICAL/GAMMA.xlsx` GAMMA!B4, B8, B9, B10; `BETA_GAMMA.xlsx` Sheet1!B6, B7.
Status: VERIFIED (observation); the algorithm is UNVERIFIED (T-0973)
Tasks: T-0973

## The normal CDF is not always correctly rounded
Rule: NORM.S.DIST(z, TRUE) can differ from the correctly rounded Phi(z) in the last place:
`NORM.S.DIST(1,TRUE)` = 0.841344746068543, while Phi(1) = 0.84134474606854294858... rounds to
0.8413447460685429 (Excel is 1 ulp high). So matching Excel needs its own erfc/CDF formulation,
not a correctly rounded one (this refines the T-0924 note in `functions/statistical/special.rs`).
Evidence: `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV`!C8, C29;
`NORM.DIST NORM.INV`!E8.
Status: VERIFIED (observation); the formulation is UNVERIFIED (T-0968, oracle sweep T-1040)
Tasks: T-0968

## Standard normal density uses sqrt(2*pi) computed in doubles
Rule (hypothesis): NORM.S.DIST(z, FALSE) equals `exp(-z*z/2) / sqrt(2*pi)` with `sqrt(2*pi)`
rounded from the double product (2.5066282746310002), not the correctly rounded constant
2.5066282746310007: `NORM.S.DIST(0,FALSE)` = 0.3989422804014327, `NORM.S.DIST(1.5,FALSE)` =
0.12951759566589174. This form reproduces all 9 numeric NORM.S.DIST density cells in the
workbook, but not every NORM.DIST (sigma != 1) or LOGNORM.DIST density cell, so the general
formulation is open.
Evidence: `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV`!C3, C5, C7, C16, C17,
C24, C25, C26, C32 (checked during T-0903).
Update: refuted as a general rule by the T-1041 sweep, see "Normal density oracle sweep (T-1041)" below.
Status: UNVERIFIED (refuted as a general rule; fits the 9 cached cells only; 272/369 on the T-1041 sweep)
Tasks: T-0975

## Normal density oracle sweep (T-1041)
Rule: on the 369-point grid z = -38..8 step 0.125, `PHI(z)` equals `NORM.S.DIST(z,FALSE)` bit for
bit (one kernel). The hypothesis above (`exp(-z*z/2) / sqrt(2*pi)`, divisor 2.5066282746310002) matches 272 of the
369 values (97 differ in the last digit), so it is refuted as a general rule; the correctly rounded
divisor 2.5066282746310007 (the engine's `SQRT_2PI` for NORM.S.DIST/NORM.DIST/LOGNORM.DIST; PHI
currently computes the ...0002 form) matches only 64. The best lead is
`exp(-z*z/2) * 0.3989422804014327`, which matches 350 of 369 (12 misses are 1 ulp, 3 are 2 ulp at z = -34.625,
-25.875, -23.25, 4 are the flushed zeros below). T-0975 must fit against the sweep, not the 9 cached cells. For the NORM.DIST grids,
`exp(-((x-m)/s)^2/2) / sqrt(2*pi) / s` matches 106/122 (s=0.5), 88/122 (1.2), 73/122 (3) and
75/122 (10): the NORM.DIST kernel is not that form; these grids and LOGNORM.DIST are otherwise left
to T-0975's fit (all cells are numbers, no errors). Results below the normal double range flush to 0:
z = -38 .. -37.625 give exactly 0 (the quotient would be subnormal, e.g. 1.25e-312), while
z = -37.5 gives the normal 1.7282337322841054e-306. All 369 cells are numbers (no errors).
Evidence: probes-numeric cases `m9-normpdf-norm-s-dist`, `m9-normpdf-phi`; companion grids
`m9-normpdf-norm-dist-s0p5`, `-s1p2`, `-s3`, `-s10` (NORM.DIST density, m in {0, 2.5}) and
`m9-normpdf-lognorm-dist` (three (m,s) pairs).
Status: VERIFIED (observation)
Tasks: T-0975, T-1041

## HYPGEOM.DIST is not exact for small integer arguments
Rule: HYPGEOM.DIST misses the exact rational result by a few ulp even for tiny populations, so it is
not a direct ratio of exact binomial coefficients: `HYPGEOM.DIST(1,4,8,18,FALSE)` (arguments
truncated from 1.8, 4.2, 8.2, 18.34) = 0.31372549019607815, while 16/51 = 0.3137254901960784 (which
is what C(8,1)*C(10,3)/C(18,4) gives in doubles); `HYPGEOM.DIST(1,4,8,22,TRUE)` = 0.5349282296650714,
while 559/1045 = 0.5349282296650718. Neither a log-gamma nor a log-factorial `exp(ln C + ln C - ln C)`
form reproduces both cells (checked during T-0903), so the formulation is open.
- T-0977 checked the 104 integer pmf cells of the T-1048/T-1049 sweeps and gen-m9 (populations to 1E5) against the exact
  rational: Excel is within about 4 ulp of it but rarely on it (`HYPGEOMDIST(0,3,1,6)` = 0.5000000000000001 where the
  exact value is 1/2; `HYPGEOM.DIST(0,3,5,12,FALSE)` = 0.15909090909090923, 5 ulp above 7/44). None of these
  reproduces more than 25 of the 84 cells with a population up to 3,000 bit for bit: exp of the nine log-factorials in any
  of the 9! summation orders (correctly rounded, lgamma or summed logs), ratios of COMBINs, a product of ratios
  walked from x = 0, or Loader's saddle-point form (R's `dhyper`).
- The cdf is the sum of Excel's own pmf terms from the lowest: `HYPGEOM.DIST(1,4,8,20,TRUE)` = 0.46542827657378727 is
  0.10216718266253864 + 0.3632610939112486 in doubles, Excel's two pmfs (t1049-hypgeomdist A1, A31, A2), where the
  exact cdf rounds to 0.46542827657378744.
- A huge population is `#NUM!` in Excel from somewhere between 2E9 (a number, t1049-hypgeomdist A51) and 1E10
  (`HYPGEOM.DIST(1,4,8,1E10,FALSE)`, A50): also `HYPGEOMDIST(2,3,8,1000000000000017.1)` (gen-m9/ee2497a5e683c5d9 A264),
  `HYPGEOM.DIST(3,4,1,753749999999999.8,FALSE)` (a013a45a8fbcab4f A352) and `HYPGEOMDIST(1,25,3,ROUND(2^53+1,8))`
  (35e8168aae2b00f8 A128); T-2429 owns the limit. The walk below stops at 2^52 only to choose a kernel.
Evidence: `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` Sheet1!F2, F3; probes-numeric/t1048-hypgeomdist,
t1049-hypgeomdist (excel-graph).
Ours: the exact value rounded once, from a double-double walk (`discrete.rs` `hypergeometric_walk`, up to 2^12 steps and
a population of 2^52; statrs' sum beyond): within 15 digits of Excel everywhere except where Excel's own pmf is
visibly off the exact value (`HYPGEOMDIST(2,2,3,11)` = 0.05454545454545458 against 3/55, gen-m9/f2bd5f380ec002bf A52;
`HYPGEOMDIST(2,4,3,19)`, gen-m9/ee2497a5e683c5d9 A310; t1049-hypgeomdist A10, A14, A16; gen-m9/60b0b6e69ad35a45 A140),
T-2481.
T-2742 grid (probes-numeric/t2742-hypgeom-small-n, excel-graph 2026-10-28): every valid (x, n, K, N) with N 6-12,
1442 pmf and 1442 cdf cells, and HYPGEOMDIST for N = 12.
- Only 286 of the 1442 pmf cells (19.8%) are the exact rational correctly rounded. The others are off by -14 to +10 ulps
  (offset = Excel minus the correctly rounded exact value), with both signs at every N and at the first, interior and
  last x alike: for N = 12 the most common offsets are 0 (82 cells), +1 (69), -1 (40), +2 (34) and -2 (31). The lowest is
  `HYPGEOM.DIST(3,6,5,11,FALSE)` = 0.4329004329004321 (A893), -14 ulps from the exact 100/231; A884 `(3,6,3,11)`, A898
  `(3,6,6,11)` and A1281 `(4,7,5,12)` are -13. The highest is +10 at A1321 `(3,8,4,12)`, then +9 at A690, A1339, A1366
  and A1375. `HYPGEOM.DIST(0,3,1,6,FALSE)` is 0.5000000000000001 (A24), not 0.5.
- The pmf is not symmetric in n and K: `pmf(x,n,K,N)` equals `pmf(x,K,n,N)` bit for bit in only 434 of the 1442 cells,
  so Excel's formulation treats the sample size and the success count differently.
- Bit-exact counts on the grid (docs/research/t2742_hypgeom_grid.py; of T-0977's candidates the 9! log-factorial
  orders, summed logs, Loader's `dhyper` and the walked product were not re-run): the exact rational 286, the double
  ratio `COMBIN(K,x)*COMBIN(N-K,n-x)/COMBIN(N,n)` 286, grouped `COMBIN(K,x)*(COMBIN(N-K,n-x)/COMBIN(N,n))` 280, grouped
  `(COMBIN(K,x)/COMBIN(N,n))*COMBIN(N-K,n-x)` 285, and Python `exp` of `math.lgamma` terms summed left to right as
  lg(K+1)-lg(x+1)-lg(K-x+1)+lg(N-K+1)-lg(n-x+1)-lg(N-K-n+x+1)-lg(N+1)+lg(n+1)+lg(N-n+1) 37 (the count depends on
  the grouping: per-COMBIN groups give 32). None comes close; offsets of over
  10 ulps suggest a log-space or iterated formulation rather than one rounding of a ratio (unverified).
- The cdf is not the running double sum of Excel's own pmf cells from the lowest x: that rule holds in 816 of the 1442
  cdf cells, and the correctly rounded exact cdf in 776. The cdf at the last x (x = min(n, K)) is exactly 1 in all 476
  such cells (inferred, unverified: a sum of off-by-ulps pmfs would not always reach 1).
- HYPGEOMDIST equals HYPGEOM.DIST(..,FALSE) bit for bit in all 407 N = 12 cells (column C), so they share one kernel.
Status: VERIFIED (observation); formulation UNVERIFIED (T-0977, T-2481)
Tasks: T-0977, T-2481, T-2742

## BETA.DIST is not exact where the exact value is a double
Rule: `BETA.DIST(2,8,10,TRUE,1,3)` (I_0.5(8,10) = 44923/65536, exactly representable as
0.6854705810546875) returns 0.6854705810546873; the density at the same point,
`BETA.DIST(2,8,10,FALSE,1,3)`, is exact (1.4837646484375). Match Excel's incomplete-beta last digits,
not the exact value.
Evidence: `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` BETA_DIST!G3, G4.
Status: VERIFIED (observation); algorithm UNVERIFIED (T-0970)
Tasks: T-0970

## VAR/STDEV switch between a one-pass and a two-pass formula
Rule: VAR.P/VARP/VARPA first compute the one-pass form V = `(n*Σx² - (Σx)²)/(n*n)`, and
VAR.S/VAR/VARA compute V = `(Σx² - (Σx)²/n)/(n-1)` (plain left-to-right double sums). Excel keeps
V unless either test below holds; then it recomputes with the two-pass form `Σ(x - m)²/(n-d)`,
with m = Σx/n and d = 0 or 1. STDEV.* is the square root of the matching VAR.*. The two tests are:
- cancellation: `V*(n-d) < 0.01*Σx²`, that is, the one-pass sum of squared deviations is below 1%
  of Σx²;
- small variance: `V < 0.001*|m|`.

Both tests use the function's own one-pass V, so the same data can take different paths:
`VAR.P({0.02,0.027})` is the two-pass 1.2249999999999998E-05, while `VAR.S({0.02,0.027})` is
the one-pass 2.45E-05. The second test is not scale-invariant. {1,2,4}×10^k uses two-pass for
k ≤ -3 and one-pass for k ≥ -2, even though nothing cancels.
Consequences:
- Small integer sets use the one-pass form and are not the correctly rounded variance.
  `VAR.S({1,2,4})` is 2.333333333333334, while 7/3 correctly rounded is 2.3333333333333335.
  `VAR.P({3,1,4,1,5,9})` is 7.472222222222222, the one-pass value; two-pass gives …221 (this is
  probes/aggregate-grid B31). One-pass results up to 39 ulps from the exact variance are kept.
- Offset data use the two-pass form. `VAR.P` of {1000000003,1000000001,…} is
  7.472222222222224, exactly the two-pass value; one-pass gives -113.78. `VAR.S({11,12,14})`
  (ratio 0.0101) is the one-pass 2.333333333333343, and `VAR.S({101,102,104})` is two-pass.
- A negative mean uses |m|. A zero mean (`{-1,1}`) never meets the second test.
- Every product and quotient flushes a subnormal result to 0, like `*` and `/` (numbers.md,
  "Subnormal results become 0"). `STDEV.S({0,2.2E-154})` is 0: each squared deviation
  (1.21E-308) is subnormal, although their sum 2.42E-308 is not. `STDEV.S({0,3E-154})`
  (squares 2.25E-308, normal) is 2.121320343559643E-154 (probes/variance-underflow, 36 cells).
- `VAR.P` of 1E+200 twice is `#NUM!` (numbers.md, "Overflow"): the one-pass Σx² overflows, V is
  NaN, neither test holds, and the NaN is returned.

Evidence (oracle: excel-graph, 2026-10-07, T-1037): probes/variance-digits (234 cells),
variance-switch (82), variance-fit (380, random offset and small-magnitude sets),
variance-scale (492, six shapes over scales 3E-5 to 0.3), variance-rule (312; column A is near
both limits, and B:M hold negative and zero means for all twelve VAR/STDEV functions) and
variance-rel (120), variance-underflow (36). Every cell matches the rule bit for bit. The limits are pinned: the
small-variance ratio V/|m| is two-pass at 0.0009999999999999998 and one-pass at
0.0010000000000000007. The cancellation ratio V*(n-d)/Σx² is two-pass at 0.0099992 and
one-pass at 0.0100036. Rejected forms: `V < 0.01*Σx²/n` fails 64 cells (VAR.S with n = 2..3),
and `V < 0.01*m²` fails 22. Comparing the two results instead of the inputs also fails, because
{13,11,14,…} and {103,101,104,…} have the same one-pass and two-pass values but take different
paths. Literals in these probes parse to the correctly rounded doubles (checked for all 242
decimal literals of variance-fit).
Status: VERIFIED. The float form of each comparison (for example `V < 0.001*|m|` against
`V/|m| < 0.001`) is UNVERIFIED at the exact boundary.
Implemented in: `excel_variance` in crates/rustsheet-engine/src/functions/statistical/variance.rs
(T-1037). DVAR/DVARP/DSTDEV/DSTDEVP are covered by the database-variants paragraph below (T-1054).

SUBTOTAL 7, 8, 10, 11 and their 107, 108, 110 and 111 forms use the same switch and the same overflow (probes/subtotal-variance-one-vs-two-pass,
excel-graph 2026-10-11, T-0298; 64 cells, data copied from probes/dvar-one-vs-two-pass). Over the offset set (1000000003, 1000000001, ...)
SUBTOTAL(10) is the two-pass 8.966666666666669 (the small set {3,1,4,1,5,9} is the one-pass 8.966666666666665); the small-magnitude sets
{0.001,0.002,0.004} (two-pass) and {1,2,4}, {11,12,14}, {101,102,104} give the same bits as DVAR/DVARP/DSTDEV/DSTDEVP on the same data (every cell
of the DVAR probe equals the matching SUBTOTAL cell), and `SUBTOTAL(c, {1E+200 x3})` is `#NUM!` for all eight codes. The 107, 108, 110 and 111 forms equal 7, 8, 10 and 11
here (no row is hidden in this probe). The engine already matches (64/64): SUBTOTAL calls `variance_result` (functions/subtotal.rs, T-1185). Status of this paragraph: VERIFIED.

Database variants (T-1054, VERIFIED): DVAR, DVARP, DSTDEV and DSTDEVP follow the same switch with the same
divisors (DVAR/DSTDEV like VAR.S/STDEV.S, DVARP/DSTDEVP like VAR.P/STDEV.P), and an overflowing Σx² is
`#NUM!` as for VAR.P (`DVAR` of three 1E+200 values, cells K10:K13). Evidence: probes/dvar-one-vs-two-pass
(32 cells over eight data sets, excel-graph 2026-10-07). Implemented by `excel_variance` over the matching
numeric values (`db_numeric_values` in functions/database.rs).

## LARGE/SMALL: a fractional k rounds up for LARGE and down for SMALL
Rule: `k` is coerced like any number argument (TRUE is 1, FALSE is 0, `"2"` is 2, `"x"` is `#VALUE!`,
a referenced TRUE is 1). k below 1 is `#NUM!` before any rounding (`LARGE(r,0.5)`, `0`, `-0.5`, FALSE).
Otherwise LARGE takes the `ceil(k)`-th largest and SMALL the `floor(k)`-th smallest, and either is
`#NUM!` when `ceil(k)` exceeds the count. With 3,1,4,1,5,9: `LARGE(r,2.9)` is 4 (3rd largest),
`LARGE(r,2.0000000001)` is 4, `LARGE(r,6.1)` is `#NUM!`; `SMALL(r,2.9)` is 1 (2nd smallest),
`SMALL(r,3.5)` is 3, `SMALL(r,5.9)` is 5, `SMALL(r,6.5)` is `#NUM!` (not the 6th, 9).
AGGREGATE 14/15 (array form) route through the same code; oracle evidence covers 14 only (aggregate-args Misc!A11, A13).
Status: VERIFIED (probe `eval/suites/probes/cases/large-small-k`, T-1034; oracle excel-graph, 2026-10-07).
Implemented in: `large_small_k` and `fn_large`/`fn_small` (crates/rustsheet-engine/src/functions/statistical/count_and_average.rs).

## MEDIAN of an even count is a + (b - a)/2
Rule: with an even count, MEDIAN takes the two middle values a <= b and returns `a + (b - a)/2`, the lower one plus
half the gap. It does not return `(a + b)/2`. The two differ by an ulp or two when `a + b` rounds:
- `MEDIAN` of -0.5 and 0.3 is -0.09999999999999998, not -0.1 (gen-m9/73f053591a54bc43 Calc!A307).
- -0.0001 and 7 give 3.4999499999999997, not 3.49995 (gen-m9/775d053de498b8c7 Calc!A162, 91e167cd8d7eb652 A117).
- 0.3 and 1.005 give 0.6524999999999999 (gen-m9/bbab6562e88b70e8 A170).
- 12.302999999999999 and 43.90399999999999 give 28.103499999999997, not ...993 (calipers pv23_051/golden.xlsx WACC!E23 and
  21 more cells of the pv23_046..055 goldens, Mac desktop cached values).
Evidence: every gen-m* MEDIAN cell over plain Data ranges with an even count (50 cells, excel-graph). `a + (b - a)/2`
matches all 50. `(a + b)/2`, `a/2 + b/2`, `b - (b - a)/2` and `(a + b)*0.5` match 44 each (gap triage 2026-10-28, T-3017).
Two 9E+307 values give 9E+307 (probes/aggregate-overflow A48, numbers.md), which this form also gives. A pair whose
`b - a` overflows is unprobed.
Status: VERIFIED (cells above). Ours: not implemented. `fn_median` (count_and_average.rs) computes `(a + b)/2`; T-3017.

## RANK/RANK.EQ/RANK.AVG: a number absent from the reference is #N/A, even when the reference holds no number
Rule: when `number` is not among the numeric values of `ref`, the result is `#N/A`, whatever `ref` holds: a numeric
range without that number, a range of only text and booleans, an empty range, or a one-cell reference (direct, or via
`IF(TRUE,ref)` / `INDEX(ref,n)`). Excel never gives `#NUM!` for an empty value set here (unlike LARGE/SMALL).
With A1:A2 = {5, 7}, A3:A4 = {"x", TRUE}: `RANK(3,A1:A2)`, `RANK(3,A3:A4)`, `RANK(3,A3:A4,1)`, `RANK(3,A6:A7)` (empty),
`RANK.EQ`/`RANK.AVG` over the same, `RANK(3,A1)`, `RANK.EQ(3,A6)` and `RANK.AVG(3,A3)` are all `#N/A`; `RANK(5,A1:A2)` is 2.
An error-valued `order` wins over the absent-number result: `RANK(3,A6:A7,1/0)`, `RANK(3,A3:A4,1/0)` and `RANK(5,A1:A2,1/0)` are all `#DIV/0!`.
Status: VERIFIED (probes `rank-absent-number`, T-1055, and `if-index-ref-args-aggregates` A46-A51, T-1003; oracle excel-graph, 2026-10-07).
Implemented in: `fn_rank_eq` and `fn_rank_avg` (crates/rustsheet-engine/src/functions/statistical/rank_eq_avg.rs).

## Normal CDF and ERFC sweep (T-1040)
Rule: on a dense grid, `ERFC(x)` and `ERFC.PRECISE(x)` are bit-identical (265 points, x in -6..27
step 0.125) and match the correctly rounded `erfc` to within 1e-15 relative wherever the result is
a normal double; they return 0 from x = 26.625 on, where the true value is subnormal (< 2.2E-308)
(last non-zero: `ERFC(26.5)` = 2.2109076642637346E-307). The grid cannot tell a final-result
subnormal flush from an underflow in an intermediate `exp`; it is consistent with the flush above.
`NORM.S.DIST(z,TRUE)` is within 7.8E-15 relative of the correctly rounded Phi(z) for z > -8, but its
lower-tail error grows: up to about 1.15E-13 (max 1.144E-13 at z = -27.25), clustering near
1.137E-13 and 5.68E-14 (2^-43, 2^-44) at many integer z (-37, -36, -34, -32 versus -30, -27, -24).
It returns 0 from z = -37.625 down (last non-zero: `NORM.S.DIST(-37.5,TRUE)` = 4.605353009581954E-308).
`NORM.S.DIST(0,TRUE)` = 0.5, `NORM.S.DIST(8,TRUE)` = 0.9999999999999993,
`NORM.S.DIST(-8,TRUE)` = 6.220960574271738E-16.
`LOGNORM.DIST(x,m,s,TRUE)` shows the same deep-lower-tail error: `LOGNORM.DIST(2,1,0.01,TRUE)`
(z = -30.7) = 4.473044928315258E-207 against a true 4.473044928316143E-207 (1.98E-13), and
`LOGNORM.DIST(0.1,0,0.1,TRUE)` (z = -23.0) = 1.2841756306435775E-117 against 1.2841756306435298E-117
(3.7E-14); its other cells are within 2E-15. `NORM.DIST(x,m,s,TRUE)` with s != 1 is within 2.1E-15
on all 20 probes (none is deeper than z = -5), and `GAUSS(z)` (z in -38..8 step 0.5) is within 3.3E-16.
No error edges appeared in these grids.
Evidence: probes-numeric cases `t1040-norm-s-dist-cdf-a`, `t1040-norm-s-dist-cdf-b`, `t1040-erfc`,
`t1040-erfc-precise`, `t1040-gauss`, `t1040-norm-dist-cdf`, `t1040-lognorm-dist-cdf` (oracle
excel-graph, 2026-10-08; errors measured against 60-digit mpmath).
Status: VERIFIED (values); the formulation that reproduces the tail error is open in T-0968.

## ERFC core factor: candidates that do not reproduce Excel (T-1187)
Rule: none known. For x >= 1 on probes-numeric/t1040-erfc (209 points, 205 of them non-zero), Excel's `ERFC(x)` divided by
`exp(-fl(x*x))` (glibc `exp`) gives an implied core factor erfc(x)*exp(x^2) that is between -3.6 and +3.1 ulp from
the true core (median +0.1). The error drifts slowly with x: mostly positive from 6.5 to 9.9 and mostly negative
from 10.75 to 12.5. That points to accumulated rounding in a summation or recurrence, not to a minimax rational.
Exact-match counts of the candidates, each scored on all 205 non-zero points. None gets close to 95%:
- Bottom-up continued fraction, depths 5 to 1000, in five forms: `x + (n/2)/t` in three spellings, the even
  `2x^2+1+4n - (2n-1)(2n)/t` form, and the `x^2+1/2` Laplace form. Each was combined with seven orders of
  multiplying by `exp(-x*x)` and `1/sqrt(pi)` (best 57/205) and seven orders of dividing by `exp(x*x)`
  (best 58/205, 28%).
- Modified-Lentz top-down evaluation of the `x + (n/2)/t` and even fractions, tolerances 1e-14 to 1e-16, with
  the seven multiply orders: best 38/205 (19%).
- Numerical Recipes' incomplete gamma, `erfc = Q(1/2, x^2)`: `gcf` with `exp(-x + a*log(x) - gln)`, and `gser`
  below x^2 = 1.5. Tried with gln exact, NR2 `gammln` or `lgamma`, and EPS from 3e-7 to 1e-16: best 4/205. Its
  error is already up to about 22 ulp at x = 1..2 and grows with x, because rounding the exp argument costs about
  ulp(x^2) relative. Excel stays within 1E-15 up to x = 26, so its exponent is `-fl(x*x)` with nothing added.
- The union of every configuration above matches 174/205 points (85%). So no single formulation in this family
  reaches 95%, even before a difference between MSVC's and glibc's `exp` is ruled in or out.
- The x87 hypothesis (T-1524, fit lab `erfc-x87`): the best fraction (`bu_even`, depths 30, 60
  and 100, both best combinations) with every operation and exp at 64 bits (idealized exp, or the
  fldl2e/f2xm1/fscale sequence) and one final rounding: at most 44/205. That is the score of a
  128-bit evaluation (depth 1500) and of the engine. A 64-bit evaluation does not help; the best fit
  so far is in doubles.
Earlier negatives (T-1186): the engine's CF in 7 orders, Cody CALERF with and without its x*16 split, and the
asymptotic series in five orders, all at most 22%.
Not tried:
- fdlibm `erfc`. Its split `z` (x with the low 32 bits cleared) avoids the `fl(x*x)` error that
  probes/subnormal-function-boundaries A5 (`ERFC(26.4)`) shows, so it does not fit.
- A Chebyshev fit such as NR3 `erfccheb`. It adds a term to the exp argument, the same objection.
- MSVC's own `exp`, which is not available here.
Evidence: probes-numeric/t1040-erfc (excel-graph, 2026-10-08), scored against 50-digit mpmath and by the
T-1187 candidate script (2026-10-11), now the fit lab's `erfc-tail` fit (T-1523,
tools/src/rustsheet_tools/fitlab/kernels/erfc.py; `uv run rustsheet-fitlab erfc-tail` in tools/ reprints the
counts above: best 57, 58, 38 and 4, union 174/205 over the four T-1187 families; the command's last union
row, 177/205, also counts libm and the engine). The engine today: 44/205 bit-exact, 203/205 within 4 ulp.
Status: SUPERSEDED. T-1778 (docs/research/t1778-erf-kernel-research-plan.md, research updates 4, 7 and 10) finds
ERFC(x) = fl(E*R) for x > 0.5, with E = Excel's EXP(-fl(x*x)) and R = N/D a stored-Horner rational ((9,10) up to 4,
(8,9) above), bit-exact on every probed input, which rules out every family above. T-1916 implements it (next
section) and closes T-1186.

## GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)
Rule: `GAMMA.DIST(x,alpha,beta,FALSE)` with x > 0 is #NUM! when x / beta is below 2.2250738585072014E-308 (subnormal),
whatever the shape, the scale or the size of the density, while the cdf there is 0.
Evidence: probes/t2738-gamma-dist-tiny-ratio (excel-graph 2026-10-25): #NUM! at `GAMMA.DIST(1E-10,1,1E+307,FALSE)`,
`(1E-10,1,1E+300,FALSE)` (density 1E-300, a normal number), shapes 2 and 0.5, GAMMADIST, `(1E-300,1,1E+10,FALSE)`
(density 1E-10), `(1E-300,1,1E+8,FALSE)`, `(3E-308,1,2,FALSE)` and `(4.4E-308,1,2,FALSE)` (ratio 2.2E-308); a number at
`(4.5E-308,1,2,FALSE)` (ratio 2.25E-308, 0.499999999999989), `(3E-308,1,1,FALSE)` and `(1,1,1E+307,FALSE)`. The cdf is 0
(companion TRUE) at `(1E-10,1,1E+300,TRUE)`, `(1E-300,1,1E+10,TRUE)` and `(3E-308,1,2,TRUE)`; a normal x / beta with a
subnormal density is 0 (`(10,1,1E+307*10,FALSE)`, `(1E+300,1,1E+307*10,FALSE)`). Near the bound Excel's density is
about 1E-14 relative off the exact value: see "GAMMA.DIST's density at a tiny x / beta" (T-2753).
Implemented in: functions/statistical/gamma.rs `gamma_dist_of`; x = 0 keeps its own rule. Unprobed: CHISQ.DIST at x
below 4.45E-308 (its scale is 2), an x / beta that overflows, and a computed subnormal x with a normal x / beta (a
typed one reads as 0, numbers.md "Formula literals").
Status: VERIFIED

## GAMMA.DIST's density at a tiny x / beta is a log-space form (T-2753)
Rule: where y = x / beta is at most about 1E-15 (and x > 0, shape a below 20), `GAMMA.DIST(x,a,beta,FALSE)` is
EXP(a*LN(y)-y)/GAMMA(a)/x, not the exact density: the log carries |LN(y)| ulp of error, so `GAMMA.DIST(1E-300,1,1,FALSE)`
is 1.0000000000000235 and `GAMMA.DIST(1,1,1E+307,FALSE)` 9.999999999999768E-308. Between 1E-15 and 1E-10 Excel takes a
form not yet found, up to 37 ulp from both this one and the direct y^a e^-y / GAMMA(a) / x (T-2852).
Evidence: probes/t2753-gamma-density-ratio-sweep (x / beta 1E-300 to 0.5 at shapes 1, 2, 0.5 and 50, column A beta = 1,
column B x = 1) and probes/t2753-gamma-density-switch (1E-19 to 1E-11), excel-graph 2026-10-27;
probes/t2738-gamma-dist-tiny-ratio A2, A4, A15, A19. With Excel's own LN, EXP and GAMMA the form is exact or 1 ulp off at
most cells below 1E-15 (exact at t2738 A2, A15, A19) and 2 to 3 ulp off at a few: switch A11 `GAMMA.DIST(2E-16,1,1,FALSE)`,
A12, A18, A24, A25, A38, B10, B11, B18, B24, B25, B38 and sweep A19 `GAMMA.DIST(1E-20,2,1,FALSE)` (T-2852); at 1E-15 it is 4
to 7 off, the direct form 4 to 12. LNGAMMA in place of GAMMA is 40 to 60 ulp off at shape 0.5.
Implemented in: functions/statistical/gamma.rs `tiny_ratio_density` (GAMMA.DIST and GAMMADIST only; CHISQ.DIST keeps
`incomplete_gamma::gamma_density`, unprobed at tiny x). Unprobed: shapes from 2 to 20 (the cutoff at 20 is the
prefactor's own split, not evidence; at shape 50 the density underflows to 0 in this regime).
Status: VERIFIED for the form (0 to 3 ulp at the probed cells; the last bits are T-2852)

## One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)
Rule: these functions are bit-identical compositions of one another, so their last digits come from one
kernel: the regularized incomplete gamma P(a, z) and its upper half Q(a, z). At shape 0.5 the probes cannot
tell whether GAMMA.DIST calls an erf/erfc pair or the reverse; it takes its own path there (below).
- `NORM.S.DIST(x,TRUE)` = `ERFC.PRECISE(-x*SQRT(0.5))/2` (1,545 of 1,545 points, the whole norm-s-dist
  fit sweep). The scaling is a multiply by the double sqrt(0.5): `-x/SQRT(2)` matches at only 692 of the
  1,352 points where the two inputs differ (both cases). `GAUSS(x)` = `NORM.S.DIST(x,TRUE)-0.5` (1,545 of 1,545).
  `NORM.DIST(x,m,s,TRUE)` = `NORM.S.DIST((x-m)/s,TRUE)` and
  `LOGNORM.DIST(x,m,s,TRUE)` = `NORM.DIST(LN(x),m,s,TRUE)` (60 of 60 each).
- `ERF.PRECISE(x)` = `GAMMA.DIST(x^2,0.5,1,TRUE)` = `CHISQ.DIST(2*x^2,1,TRUE)`, and
  `ERFC.PRECISE(x)` = `CHISQ.DIST.RT(2*x^2,1)` (600 of 600, x from 2^-40 to 6). ERFC is computed directly:
  `1-GAMMA.DIST(x^2,0.5,1,TRUE)` matches it at only 17 of the 201 points with x >= 0.5.
- `CHISQ.DIST(x,k,TRUE)` = `GAMMA.DIST(x/2,k/2,1,TRUE)` = `GAMMA.DIST(x,k/2,2,TRUE)`,
  `GAMMA.DIST(x,a,b,TRUE)` = `GAMMA.DIST(x/b,a,1,TRUE)`, and `POISSON.DIST(k,m,TRUE)` =
  `CHISQ.DIST.RT(2*m,2*(k+1))`, an upper Q (60 of 60 each). `T.DIST(x,n,TRUE)` for x < 0 =
  `T.DIST.2T(-x,n)/2` (60 of 60).
- Not identities (the lower count is the closest form tried): `EXPON.DIST` against `1-EXP(-l*x)` 58/60 and
  `GAMMA.DIST(x,1,1/l,TRUE)` 57/60; `BINOM.DIST(k,n,p,TRUE)` against `BETA.DIST(1-p,n-k,k+1,TRUE)` 53/60;
  `F.DIST` against its BETA.DIST forms 44/60; `CHISQ.DIST.RT` against `1-CHISQ.DIST` 18/60;
  `POISSON.DIST` against `1-GAMMA.DIST(m,k+1,1,TRUE)` 38/60.
- ERF is odd (460 of 460 pairs). `ERFC.PRECISE(-d)` = fl(1+`ERF.PRECISE(d)`) at 340 of 347 points with
  0 <= d < 0.5 and `ERF.PRECISE(d)` = `ERFC.PRECISE(-d)`-1 at 213 of 304 with d >= 0.5, so neither half is a
  plain reflection of the other everywhere.
What the probes show about the kernel itself (none of it yet reproduces Excel bit for bit):
- Tiny x: `ERF.PRECISE(2^-k)*2^k` = 1.1283791670955128 for every k from 27 to 79, one ulp above
  fl(2/sqrt(pi)) = 1.1283791670955126. Excel's GAMMA, GAMMALN and 2/SQRT(PI()) all give the ...126 values.
  Below about x = 1E-8 the result depends only on x's mantissa (exactly scale invariant: 100 of 100
  mantissas at five scales). There it is `fl(fl(x*A) + fl(x*B))`, a constant split in two:
  A = 1.1170334248975802 and B = 0.011345742197932508. That form reproduces 2,429 of 2,448 points (400 random
  mantissas, 2,048 consecutive ones; 99.2%). A was read off the period of Excel's rounding pattern over runs of
  consecutive mantissas spaced 2^j apart (j = 0 to 41), then confirmed by an exact solve. A + B is
  2/sqrt(pi) + 1.08E-16 (about half an ulp), which is the +0.5 ulp bias Excel shows for x < 0.5. Neither constant is a known
  coefficient, and B is not fl(2/sqrt(pi) - A) (that is 0.0113457421979324). Above 1E-8 both parts depend
  on z = x^2. Strided runs at fixed scales (`kmyst-erf-zstride`, `kmyst-erf-runs`) pin each part separately.
  - The coarse factor is fl(A + a1*z) with a1 = 0.1406815 (+-1.3E-7). One a1 gives full coverage on all 26
    runs with z up to 6E-10, allowing the same 1-in-200 rate of odd points as the tiny regime.
  - The fine part then is one exact double per run. It follows B - 0.516808*z, so the two slopes sum to
    -2/(3*sqrt(pi)) as erf requires, but the fine part scatters by about +-6E-17 around that line. That
    scatter is not the coarse rounding error, and neither a full-scale nor a half-scale double Taylor
    series explains it.
  Sweeping z finely (`kmyst-erf-zsweep`, `kmyst-erf-zsweep2`, `kmyst-erf-bridge`) explains that scatter. The
  fine part is piecewise constant, `lo = B0 + K*(W - 1)` with `W = fl(1 - BETA*z)`, a value just below 1 on the
  2^-53 grid. Bootstrapping the integer step count n = (1 - W)/2^-53 from n ~ 10 up to n ~ 2E7 gives:
  - K = 1.13669733 (+-2.5E-9);
  - BETA = 0.454657262 (+-5E-9), so K*BETA = 0.5168077 = 2/(3*sqrt(pi)) + a1;
  - a1 refined to 0.1406813111.
  With `erf = fl(fl(x*hi) + x*lo)`, evaluating the final sum in 80-bit x87 and rounding the coarse product
  through 80 bits before storing it, the model matches 66,756 of 67,048 probe points (99.6%) with z <= 6E-9.
  Every remaining miss is within a few thousandths of an ulp of a rounding tie, which points to
  extended-precision constants.
  Model source: docs/research/t1778/erf_small_model.c.
  The ladder probe `kmyst-erf-ladder` (z from 1E-8 to 1E-3) shows W is a Gaussian, not a line.
  - The grid count (1 - W)/2^-53 falls short of BETA*z/2^-53 by amounts that grow like z^2 (-92,736 units at
    z = 1E-5, -8.4E7 at z = 3E-4).
  - It matches (1 - exp(-BETA*z))/2^-53 up to a residual linear in z, which is error in the fitted BETA.
  So the kernel is erf(x)/x = K*exp(-BETA*x^2) + a correction polynomial in x^2, a scaled Gaussian plus a
  polynomial.
  Not yet fitted: the correction's z^2 and higher coefficients, and the exact double values of K, BETA and a1.
  With coefficients derived from Taylor matching, a_k = c_k - K*(-BETA)^k/k!, about 70% of mid-range points
  match, so the correction is not the Taylor remainder. It may be a minimax fit, or W may not be libm exp.
  Later probes (`kmyst-erf-scatter`, `kmyst-erf-zsweep3`) show that a single z cannot separate the two parts:
  shifting the coarse factor by whole ulps and the fine factor the other way reproduces the same outputs.
  The split shows only through where each part's rounding steps fall as z changes. A global exact-match fit
  with free a1, a2, a3, K and BETA in 64-bit doubles (docs/research/t1778/erf_global_fit.c) reaches 93.9% for
  z <= 1E-6 and 87.5% for z <= 1E-3. It pulls K to 1.1366973387, which conflicts with the 1.1366973293 that the
  small-z bootstrap pins, so either the fine factor is not exactly K*(W - 1) or the coarse factor is not a
  plain double polynomial. The coarse steps at z = 1E-8, 1E-7 and 1E-6 do not fit one quadratic either.
  The excel-graph oracle is 64-bit Excel for the web, so x87 is run only as an emulated variant. It improves
  the small-z score from 99.35% to 99.56%.
  Re-analysis with the alias-free observables (the z where a part's rounding steps, read through the
  per-run lo interval; scripts and data extraction in docs/research/t1778/README.md)
  supersedes three statements above:
  - The coarse product is rounded through 80 bits (x87 double rounding). The three hard misses of the tiny
    split all have x*A within 0.0002 ulp of a tie, inside the 2^-12 window where an 80-bit round makes a
    tie. With that, and B carried in extended precision (0.0113457421979325066854), the tiny form matches
    3,043 of 3,048 points. The 5 left are final-sum near-ties (0.501 to 0.503 ulp).
  - The coarse factor is hi = fl(H(z)) with H(0) exactly the double A (offset -0.005 to +0.0006 ulp). The
    step positions bootstrapped from z = 2E-15 to 4.3E-3 (one unique hi in each of 80 runs) give
    H = A + h1*z + h2*z^2 + h3*z^3 + h4*z^4 with h1 = 0.14068131264(22..36), h2 = 0.04481917(05..09),
    h3 = 0.0018050(2..5) and h4 = 0.00009..0.00027. The earlier one-quadratic conflict was the alias choice:
    the z = 1E-6 step sits at hi = A + 633572509 ulp, two lattice periods from the nearest candidate.
  - hi + lo (smooth part) equals the exact erf(x)/x plus 1.07E-16 to within the two rounding sawtooths at
    every z probed, 2E-15 to 1E-3. So the smooth part is computed to better than double precision.
  - W is not exp(-BETA*z). The phase of the W sawtooth, read at eight sweeps from z = 2E-15 to 1E-4, rules
    out exp, 1/(1+bz), the (1,1) Pade, a line, 2exp(-bz/2)-1, sech(sqrt(2bz)) and 1/sqrt(1+2bz). Only
    W = fl(sqrt(1 - 2*b*z)) survives, through third order, with b = 0.4546567435376 or
    0.4546589639848 (two branches). The ladder probe's "z^2 shortfall" above has the sign of sqrt, not exp.
    Neither branch, nor sqrt of a rounded argument, is exact past z = 1.2E-3, so W's exact form is still
    open there.
  - Correction (probes `kmyst-erf-wsweep` and `kmyst-erf-wsweep2`, 21 more fine sweeps, with exact interval
    phases): the sqrt survival above was an artefact of too few sweeps, and every one-parameter form now
    fails by z = 3E-8. What fits is lo = B + K*(fl(W) - 1) + R(z), where W is a polynomial in z
    (W = 1 - c1*z - c2*z^2 - c3*z^3 - ...). Its linear term is fixed by the rest:
    c1 = (h1 + 2/(3*sqrt(pi)))/K = 0.4546572672773 (+-3E-14). With K = 1.13669732977 and R = 0, the
    phases at all twelve sweeps from 2E-15 to 3E-7 are predicted with no free parameter. From 1E-6 on, the
    fit needs c2 = -0.0598953 against -0.0598389 from erf's own z^2 term. That difference is
    R ~ 6.4E-5*z^2, a smooth part of lo that bypasses W. c3 (about 0.022 to 0.033) is not yet unique.
  - Two-argument `ERF(a,b)` is the exact difference of the two rounded single values: b one to five ulps
    above a gives 1 to 5 ulps of erf at x = 0.3 to 1E-5 (`kmyst-erf-twoarg`, 48 points), so it exposes no
    hidden precision. In the same probe the worksheet subtraction `ERF.PRECISE(b)-ERF.PRECISE(a)` returns
    exactly 0 for 25 of the 47 nonzero differences (relative size at most 1.39E-15) and the exact
    difference for the other 22 (relative size 1.11E-15 and up). The two ranges overlap, so the cut is not a
    plain relative threshold on the result. It is Excel's near-cancellation snap on `-`; its rule is not
    derived here.
- Generic shape a: at z = m*2^-60, `GAMMA.DIST(z,a,1,TRUE)` for a = 2, 3, 10 is off by up to 640E-16
  relative, and that error follows the rounding of fl(a*fl(ln z)) to within about 2E-16 (the docs/research
  script prints both). For a = 10 a constant -25E-16 remains, the error of Excel's own
  `GAMMALN(11)` = 15.104412573075518 (true 15.1044125730755153). So the prefix z^a/Gamma(a+1) is
  exp(a*ln z) times a factor built from Excel's GAMMALN, not pow. The density
  `GAMMA.DIST(z,a,1,FALSE)` is computed from the CDF's prefix: at z = m*2^-60 it equals
  fl(`GAMMA.DIST(z,a,1,TRUE)` * fl(a/z)) at 100 of 100 points for a = 2, 3 and 1.5.
- Shape 0.5's CDF does not take that path: it stays within 2 ulp at tiny z, while its density there is off
  by up to about 15 ulp, as exp(0.5*ln z) would be. Shape 1 is not on it either: it returns z exactly at
  z = m*2^-60 (100 of 100), where exp(fl(ln z)) would be off by up to 16 ulp.
- ERFC tail: removing the rounding of a naive `exp(-fl(x*x))` cuts Excel's error from a standard deviation
  of 115 ulp to 1.85 ulp for x >= 4 (1,702 points). The rest (at most 7.4 ulp) has a smooth part of up to
  about +-2.5 ulp: positive around x = 6.5-7.5, 9-9.75, 13-15.5 and 19.75-22.75, negative around
  10.25-11.75, 16-19.5 and 23-23.75 (means over bins of 0.25). It is the drift T-1187 reported.
- Rational kernel (round-1 parallel investigation, 2026-10-18). This supersedes the additive hi + K*(1 - fl(V)) + R
  model above. That model linearises a division, so its K, V0, c1, c2, R and d0 are artefacts.
  - The kernel is ERF.PRECISE(x) = fl53(fl80(co)/W), with co = fl53(fl64(x*N)), N = fl(P(z)) and W = fl(Q(z)).
    N and W are stored doubles, and the divide is x87 (80-bit, then rounded).
  - P(0) = A = 0x1.1df5e70c7956bp+0 and Q(0) = 0x1.fada15778ece6p-1 = 0.98994509777494666.
    A/Q(0) = 2/sqrt(pi) + 1.067E-16, which is the old +0.5 ulp bias.
  - Tiny x: fl(fl(x*A)/Q(0)) matches 3,048 of 3,048 points with no fitted constant; the old form matched 3,043.
    Forming x*A without the store matches 2,322.
  - Small z: letting N and W step one ulp at a time, every run of the wknee long runs (253,952 points) and of the
    zs3, wsweep, wladder and wlong2 sweeps (z up to 7.4E-3) fits with zero misses.
  - Fitted coefficients:
    - P = A + 0.1406813126423 z + 0.04481917070 z^2 + 0.0018054 z^3 + ...
    - Q = Q(0) + 0.45465726643 z + 0.0922778683 z^2 + 0.0104635 z^3 + ...
    - Q's terms follow from P and the erf series.
  - The old "grid doubling at lo = 0" is W crossing 1.0. The old V phases could not be chained by any smooth
    polynomial because the true step size N/W^2 varies with z.
  - The form (degree 4 over 5 in z, x*N/W) matches Cephes `ndtr.c` erf but not its coefficients. The normalised
    ratios are close to the [4/5] Padé of erf(x)/x. Scripts: `docs/research/t1778/tinydiv.c`.
  - The exact coefficients and their evaluation order are OPEN (T-1778).
- Bit-exact small-x erf (round 3, 2026-10-18). For 0 <= x < 0.5 (and x = 0.5 itself), ERF.PRECISE(x) is computed
  as follows:
  - z = fl53(fl64(x*x)).
  - N = P(z) and W = Q(z) by x87 Horner from the top coefficient, storing every operation to a double:
    t = fl53(fl64(s*z)), then s = fl53(fl64(t + c_k)).
  - co = fl53(fl64(x*N)), then erf = fl53(fl64(co/W)).
  - The coefficients are doubles, listed in hex in `docs/research/t1778/round3/sol100.txt`. P(0) = A =
    0x1.1df5e70c7956bp+0 and Q(0) = 0x1.fada15778ece6p-1, as above.
  - Matches: 775,272 of 775,272 oracle points across every small-x probe, and 35,718 of 35,718 points never used
    in the fit.
  - p0..p2 and q0..q2 are pinned to single doubles. The top coefficients lie on a thin feasible family, and other
    members score the same on all data.
  - How it was found: exact interval inversion of each rounding step into linear constraints, solved by LP
    (`docs/research/t1778/round3/`).
  - The source of the coefficients is not identified. No minimax, Padé or Chebyshev construction reproduces q2.
- ERFC kernel for x >= 0.5 (round 3; structure only, not bit-exact):
  - **Split.** The kernel starts at x = 0.5 + 2^-53; x = 0.5 still takes the small-x branch.
  - **Form.** ERFC(x) = fl53(fl64(E*R)). E is Excel's own EXP(-fl53(fl64(x*x))) (numbers.md, "EXP is an x87
    2^(a*log2 e) evaluation"), and R is a stored double that depends only on x. All 121,798 observed results
    can be written this way; with a plain double multiply, 17 cannot.
  - **Other results.** ERF(d) = fl(1 - ERFC(d)) and ERFC(-d) = fl(2 - ERFC(d)). A result below DBL_MIN is flushed
    to 0; there is no x threshold, and the last nonzero result is near x = 26.5432.
  - **Related functions:**
    - CHISQ.DIST.RT(2z, 1) = ERFC.PRECISE(SQRT(z)) at 32,768 of 32,768 points.
    - NORM.S.DIST(-y, TRUE) = ERFC.PRECISE(fl(y*sqrt(0.5)))/2 in the tail, not y/sqrt(2) (3,072 of 3,072
      against 590).
  - **R's error structure.**
    - R's error is a deterministic staircase of about ±3 units of 2^-53: flat on [0.5, 1.843), then sharp
      steps (9.0503 and 19.3667 are among the sharpest).
    - An all-double or stored-x87 Horner rational shows the same pattern, with a jump wherever an intermediate
      crosses a power of two. So the staircase is a rounding fingerprint of the exact coefficients, not an
      approximation-error curve.
    - The sharpest change sits at x ≈ 1.843, and a likely branch at x ≈ 8.03-8.05. Above that branch, two
      fixed spectral lines (0.28893 and 0.32594 cycles per x-ulp) point to an intermediate linear in x.
  - Superseded for x > 4 by the bit-exact tail entry below. The rest of this entry stands for [0.5, 4].
  - **Ruled out as R:**
    - Cephes ndtr erfc, Hart 5725, Cody CALERF and fdlibm (16-40% exact);
    - Numerical Recipes gcf, the Laplace continued fraction and asymptotic-series term counts;
    - tables in x or z, and branches at any of 1, 1.5, ..., 20.
  - **Best smooth model.** fl87(E*fl(erfcx(x))) matches 32.5% of points. The ceiling for any smooth R is about
    39%, so parity needs the exact coefficients and operation order.
- ERFC tail for 4 < x <= 26.55 (rounds 6-7, 2026-10-18): matches at 1,540,475 of 1,540,477 points.
  - **Form.** ERFC(x) = fl53(fl64(E*R)), where E is Excel's own EXP(-fl53(fl64(x*x))) and R = fl53(fl64(N/D)).
  - **N and D.** N (degree 8) and D (degree 9) are Horner polynomials in x itself. Every multiply and add is
    stored to double: s = fl(fl(s*x) + c).
    - The leading coefficients satisfy N0/D0 ≈ 2/sqrt(pi), the erfcx asymptote.
    - The constant term of D is exactly 128.
    - The coefficients are in `docs/research/t1778/round7/erfc_tail_coef.txt`.
  - **Branch.** x = 4 itself takes the lower kernel, and the switch is exactly at x > 4 (Cody's break point).
    Nothing changes at 1, 2, 8 or 16; there is no binade range reduction.
  - **Results below DBL_MIN** are flushed to 0, as elsewhere (596 cutoff points).
  - **Score.** With Excel's own EXP column as E, `erfc_tail_check.c` matches 1,540,475 of 1,540,477 points (all
    kmyst-erfc-* cases with an EXP column; 370,404 of 370,405 on cases not used in the fit). The 2 misses are 2 ulp
    in the result at x = 0x1.6462e6ea85447p+3 and 0x1.f9b8b10efd91fp+3. Both are input artefacts: Excel read those
    cells' xlsx text as the next double up, and the model at that double matches exactly (numbers.md, "Stored numbers
    in an xlsx are not always parsed to the nearest double"). So the tail model agrees with every probed point.
  - **Uniqueness.** The top coefficients are pinned to single doubles. A few late ones are equivalent on all data
    within a few ulps; the ranges are in the coefficient file.
  - **How it was found.**
    - Stored products and sums were located from the spectral lines and from the rounding grid in R's error.
    - Constants were pinned by bisecting binade crossings to the ulp, then by phase or vernier scans and global
      regressions.
    - Once N was exact, D's stages followed from per-stage correlation searches.
    - The low bits were pinned by peeling down from high x.
  - **Still open:** EXP's last F2XM1 cells. The [0.5, 4] kernel is bit-exact (below).
- Wrappers of the ERFC kernel on 0.5 < x <= 4 (round 9 side-channel search; `kmyst-erfc-r9side1..4`, about 147k
  cells). None exposes N, D or R beyond B = ERFC.PRECISE(x). All 794 exact ties in 1-B and all 419 in 2-B resolve
  ties-to-even from B itself, so nothing reads the unrounded fl64(E*R).
  - ERF.PRECISE(x) = fl53(fl64(1 - B)): x87 double rounding, at 8,096 of 8,096 points. Plain fl(1 - B) misses
    x = 3.1000000000001076. ERFC(-x) = fl53(fl64(2 - B)).
  - NORM.S.DIST(-y, TRUE) = ERFC.PRECISE(d)/2 with d = fl53(fl64(y*fl(sqrt(0.5)))), rounded twice: 600 of 600 on
    discriminating y, against 9 for a single rounding. NORMSDIST and NORMDIST are the same.
    NORM.S.DIST(+y) = fl53(fl64(1 - 2*NORM.S.DIST(-y))).
  - ERF(a, b) is the exact difference ERF.PRECISE(b) - ERF.PRECISE(a), a multiple of 2^-53. The worksheet `-`
    near-cancellation snap does not apply inside ERF(a, b): 1 of 129 points differs from the cell formula.
  - Legacy ERF and ERFC are identical to the .PRECISE forms.
  - CHIDIST, CHISQ.TEST at df 1 and Z.TEST are wrappers too.
  - Z.TEST with a single-value array returns #DIV/0!. A typed TRUE as the array is such a single value:
    `Z.TEST(TRUE,0.5,1)` is #DIV/0! (probes-numeric/t1049-ztest A51; a direct boolean is accepted as one value; whether it is counted as 1 or skipped is not distinguishable here, both give #DIV/0!, T-2049), while typed text is
    #VALUE! (`Z.TEST("a",4)`, A50) and `Z.TEST(7,4)` #DIV/0! (A41).
  - Odd-df upper chi-square (Q(1.5), Q(2.5)) and the shape-0.5 density use separate generic paths, not this kernel.
- ERFC kernel for 0.5 < x <= 4 (rounds 7-9): bit-exact.
  - **Form.** The same family as the tail. ERFC = fl53(fl64(E*R)) with R = fl53(fl64(N/D)). N (degree 9) and D
    (degree 10) are Horner polynomials in x, evaluated from the top coefficient. Every operation is rounded
    twice, s = fl53(fl64(fl53(fl64(s*x)) + c)), and D's constant term is exactly 1. E is Excel's own EXP(-(x*x)).
    - The double rounding of the products matters: single-rounded products give 99.86%.
  - **Coefficients.** In `docs/research/t1778/round9/erfc_mid_coef.txt`, highest degree first. Each of the 21 is
    unique: a 1-ulp change in any one adds misses.
  - **Score.** `round9/mid_check.c` (gcc -O1 -mfpmath=387) matches 5,694,938 of 5,694,943 points with 0.5 < x <= 4
    (all kmyst-erfc-* cases with an EXP column, through `r11*`).
    - The 5 misses are at x = 2.4500000000004998, 3.0900000000000998, 2.2677103947185002, 2.6493240961407998 and
      1.3246620480703999. Each matches exactly at the neighbouring double, so these are the xlsx misparse quirk
      (numbers.md), not kernel errors.
    - Held out: none of the 54 `r11*` cases (the epsilon ladder and the non-dyadic runs, 876,544 points)
      were used in the fit, and all of them match.
  - **Generator.** A relative-error minimax (9,10) rational of erfcx on [0.5, 4], D0 = 1, like the tail's (8,9) on
    [4, 26.54].
    - Excel's doubles are not the plain 90-digit Remez solution (`round9/gen910.txt`). They are 1.5E-9 to 1.6E-8
      higher in relative terms (up to 1.3E8 ulps), smoothly in degree.
    - N0 and D0 agree exactly, and the shift moves along a near-null direction of the rational. So the function
      agrees to about 1E-20, but no ulp-level reconstruction of Excel's generator exists. The doubles had to be
      measured.
  - **How the coefficients were found.**
    - A spectral test picked the degree: line-spectrum overlap with exact stored-Horner simulations is 0.68 for
      (9,10), and chance for every other degree and for Cody's (8,8).
    - Global step, a phase lattice. Each stored product's sawtooth phase, locked in per (run, stage), is linear
      in the integer mantissa errors. A closest-vector lattice solve (fpylll) over about 850 equations with a
      1E9-ulp box took the generator from 31% to 85.5% exact.
    - Top coefficients. Products near x = 4 hide their phase errors, so these came from top-down stage peeling
      on non-dyadic-start runs (`r11b*`), cross-checked against a chain solve up the Horner recursion. The two
      methods agreed to a few ulps on D6..D9.
    - Last, 2-D exact searches on the top pairs.
  - **Lesson.** Peeling on runs that all start at multiples of 1/128 (`r8d*`) gives lattice aliases: the low bits
    are right but the high part is wrong. A candidate has to hold in several disjoint x regions, or on irregular
    run starts.
  - The (7,7) least-squares reading of rounds 7-8 is superseded: its tiny top coefficient, invisible top stages
    and lambda = 1.020495 were all artefacts of the wrong degree.

- GAMMA.DIST(z, 0.5, 1, TRUE) = ERF.PRECISE(SQRT(z)) = CHISQ.DIST(2z, 1, TRUE) at 15,360 of 15,360 consecutive
  doubles z, about half of which are not squares of doubles (`kmyst-erf-decouple`).
  - So the kernel works from x = sqrt(z), and that internal sqrt is rounded twice (64-bit, then 53), like the
    worksheet SQRT.
  - Shapes 0.5 and 1 are special-cased on exact equality. a = 0.5 +- 1 ulp already takes the generic
    exp(fl(a*fl(ln z))) path, whose error tracks that rounding with slope 1.00 +- 0.02 (`kmyst-erf-ashape`).
  - The "0.5 +- 1 ulp" columns of `kmyst-gamma-exp` are formula literals cut to 15 digits (0.5 and
    0.499999999999999), not true neighbours of 0.5.
- For 0.03 <= x <= 0.32, ERFC.PRECISE(x) = fl(1 - ERF.PRECISE(x)) (ties to even) and
  ERFC.PRECISE(-x) = fl(2 - ERFC.PRECISE(x)) at 12,288 of 12,288 points (`kmyst-erf-erfcties`). So ERFC exposes no
  extra precision at small x.
Candidates that do not reproduce Excel (exact matches):
- Small-x erf (1,875 to 2,677 points, x < 0.5): Taylor series, CALERF and Cephes rationals with
  c = ...128 in four operation orders, at most 772 (41%); Taylor coefficients rounded to 15, 16 or 17
  digits, 623; incomplete-gamma series (Numerical Recipes and Cephes forms, seven prefixes, six tolerances), 703;
  the same with sqrt(z), exp(-z) and Gamma constants within 3 ulp, 782 of 2,533.
- Generic-shape prefix: exp(fl(a*fl(ln z))) times exp(-GAMMALN(a+1)), 1/GAMMA(a+1) or exp(-GAMMALN(a))/a
  in 14 orders: at most 285 of 600 (a in {2, 3, 1.5, 0.25, 2.5, 10}); the residual is within 2 ulp. Cephes
  `exp` and `log` (Moshier's cmath) in place of libm: 269.
- ERFC tail shape: the exact-arithmetic error curves of CALERF, Cephes `erfc` and Boost's 53-bit erfc
  are flat (within 0.35 ulp) and do not follow Excel's drift; CALERF with coefficients cut to 10 to 18
  digits shifts the curve but adds no drift. Numerical Recipes `gcf` (tolerance 1E-15 or below) and the
  Cephes `igamc` fraction stop with a truncation error below 0.03 ulp for x >= 5, so truncation is not
  the drift. The asymptotic series stopped by a relative or absolute tolerance from 3E-17 to 3E-14
  correlates at most 0.39 with it.
Evidence: probes-numeric cases on branch `agent/research/T-1778-kernel-map` (not on main; curated copies with
provenance are probes-numeric `t1916-*`, below) `kmyst-norm-erfc-a`, `kmyst-norm-erfc-b`, `kmyst-erf-gamma`,
`kmyst-erf-tiny`, `kmyst-erf-dense`, `kmyst-erfc-dense`, `kmyst-gamma-tiny` and `kmyst-kernel-map`
(excel-graph, 2026-10-17); the tiny-x erf split uses `kmyst-erf-mantissa`, `kmyst-erf-scale`,
`kmyst-erf-consecutive`, `kmyst-erf-stride06` to `kmyst-erf-stride41`, `kmyst-erf-runs`, `kmyst-erf-zstride`, `kmyst-erf-zsweep`, `kmyst-erf-zsweep2`, `kmyst-erf-bridge`, `kmyst-erf-ladder`, `kmyst-erf-scatter`, `kmyst-erf-zsweep3`, `kmyst-erf-zladder`, `kmyst-erf-wsweep`, `kmyst-erf-wsweep2`, `kmyst-erf-twoarg`, `kmyst-erf-wladder1/2`, `kmyst-erf-wknee1..4`, `kmyst-erf-decouple`, `kmyst-erf-ashape`, `kmyst-erf-erfcties`, `kmyst-erf-rladder1..7` and the `kmyst-erfc-*` tail probes, and the
internal-exp check uses `kmyst-gamma-exp`. `uv run python ../docs/research/t1778_incomplete_gamma_kernel_map.py` in tools/
reprints the identity counts, the tiny-x constant, the GAMMA.DIST error tracking and the ERFC tail figures.
Engine (T-1916): `crates/rustsheet-engine/src/functions/statistical/erf.rs` implements the three kernels and the
wrappers exactly as above: the small-x rational for 0 <= x <= 0.5, the (9,10) rational for 0.5 < x <= 4, the (8,9)
rational for 4 < x <= 26.55, ERF = fl(1 - ERFC) and ERFC = fl(1 - ERF) on either side of the split,
ERFC(-x) = fl(2 - ERFC(x)), ERF(-x) = -ERF(x), and 0 beyond 26.55 or below DBL_MIN. E comes from `x87_exp.rs`
(numbers.md, "EXP is an x87 2^(a*log2 e) evaluation"), and every operation is the x87 `fl53(fl64(.))` of
`arithmetic.rs`' `extended_*` helpers. `normal_cdf` uses the NORM.S.DIST identity (d = fl53(fl64(-z*fl(sqrt(0.5)))),
z = (x-m)/s), so NORM.DIST, NORM.S.DIST and LOGNORM.DIST inherit the kernel.
Engine (T-1929): the wrappers compute through the same compositions, so each identity above but T.DIST's holds bit for bit
(unit tests in `crates/rustsheet-engine/src/test/statistical/test_erf_identities.rs`):
- CHISQ.DIST.RT, CHIDIST and CHISQ.TEST at df 1 are `erfc(sqrt64(x/2))` and CHISQ.DIST(x,1,TRUE) is
  `erf(sqrt64(x/2))`, sqrt64 the twice-rounded root of `extended_sqrt` (`erf.rs` `gamma_half_upper`,
  `gamma_half_lower`). GAMMA.DIST's cdf at shape exactly 0.5 is `erf(sqrt64(x/b))`.
- GAMMA.DIST's cdf at every shape scales by the quotient x/b, rounded twice like the worksheet `/`, and not by
  statrs' x*fl(1/b): `GAMMA.DIST(200000,2,1000000,TRUE)` = 0.017523096306421772 then matches
  (probes-numeric/m9-gamma-dist-a2 A21). No cell that matched before changed; three that fail either way moved
  (m9-gamma-dist-a1 A21 from 1 to 4 ulp off, m9-gamma-dist-a9 A21 closer, t1048-gammadist A11/B11 from 19 to 27 ulp
  off). Against an exact P(a, x/b) the twice-rounded x/b gives Excel's value at shapes 1 and 2 (A21), so the
  remaining error is the generic kernel's, not the scaling's. An x/b that underflows to 0 or overflows keeps
  the port's branches (`GAMMA.DIST(1E300,1,1E-10,TRUE)` stays #NUM!, probes/incomplete-gamma-cap E6). Unprobed
  (T-1942): a subnormal x/b is kept, not flushed like a worksheet `/` result, so `GAMMA.DIST(2E-300,0.5,1E8,TRUE)`
  is about 1.6E-154 (0 before); and the tiny-x lower tail at shape 0.5 moved with the identity
  (`GAMMA.DIST(3E-308,0.5,1,TRUE)` and `CHISQ.DIST(3E-308,1,TRUE)` about 1.4E-154 to 2E-154, 0 before; T-1891).
- Z.TEST is `NORM.S.DIST(-z,TRUE)`, not `1-NORM.S.DIST(z,TRUE)`: 120 of 120 cells of
  probes-numeric/t1916-erf-wrappers column I (`Z.TEST(HSTACK(a,a),0,SQRT(2))`) against 19, and column J likewise.
  The statistic is z = fl53(fl64(fl53(fl64(mean - x)) * sqrt64(n))) / sigma, the quotient rounded twice too:
  240 of 240 cells of columns I and J, and all 30 numeric cells of t1048-ztest and t1049-ztest. The textbook
  (mean - x)/(sigma/sqrt(n)) matches 240 of 240 in columns I and J (there sigma/sqrt(n) is exactly 1) but only 24 of
  the 30, and the plain-double (mean - x)*sqrt(n)/sigma 30 of 30 but only 141 of 240. The mean and the sample
  standard deviation are not separated by these probes (plain and twice-rounded forms both match). One value is
  #DIV/0! with sigma given too and sigma < 0 is #NUM! (the T-1048/T-1049 rules below). Unprobed: a plain-double
  sigma/sqrt(n) that underflows stays #DIV/0! (the fork's guard) and a non-finite p is #NUM!, including an
  omitted sigma that overflows to infinity (T-1942). No held-out cell separates the three z formulas at the p level.
- `T.DIST(x,n,TRUE)` for x < 0 = `T.DIST.2T(-x,n)/2` is NOT routed. The engine's T.DIST.2T is statrs' 2*(1 - cdf(x)),
  which cancels to 0 for a large x, so the routed T.DIST(-1E6,10,TRUE) was 0 (Excel 1.230468749943611E-56) and the
  T.INV of it #NUM! (ironcalc-calc T_DIST.xlsx "T.DIST T.INV T.DIST.RT" D19, E19). Taking T.DIST.2T from the lower
  tail, 2*cdf(-x), keeps the identity but loses 8 Excel-matching T.DIST.2T cells (T_DIST.xlsx C11, C25, C26, C34,
  C35, C38, C39 and probes-numeric/m9-t-dist-2t A64, all `T.DIST.2T(1,10)` = 0.34089313230205986). So Excel's
  T.DIST.2T is neither form, and the identity waits for its T kernel (T-0969). POISSON.DIST's cdf and CHISQ.DIST.RT at df 2(k+1) are the same Q(k+1, m) of
  the capped series, so that identity holds; neither is Excel's kernel yet. GAUSS was already
  `NORM.S.DIST(x,TRUE)-0.5`.
- Bulk check against every single-argument ERF/ERFC cell of the research probes (`ERF.PRECISE(A)`, `ERF(A)`:
  928,038 cells; `ERFC.PRECISE(A)`, `ERFC(A)`: 7,259,712): all match except 27 ERFC cells. Each of the 27 sits on a
  row whose `EXP(-(x*x))` cell also differs from model V6: 7 are the misparsed inputs (numbers.md), and 20 are V6's
  F2XM1 near-ties, where E itself is one ulp off. So the kernel is exact given Excel's E.
- The 20 residual cells (ours minus Excel, in ulps, against the T-1778 run of 2026-10-17/18). Excel itself is not
  stable on them: re-filling probes-numeric `t1916-erfc-exp-ties` from excel-graph on 2026-10-18 22:09 UTC changed 9 of
  the 20 to exactly the engine's value (last column), while all 37,181 other curated cells came back identical. That
  fits the numbers.md hypothesis that F2XM1 near-ties depend on the server's CPU. The case keeps the re-filled values;
  11 cells fail there:

  | x | source cell | Excel | ours | ulps | 2026-10-18 re-fill |
  |---|---|---|---|---|---|
  | 0.6072288384003274 | `kmyst-erfc-r8a02!B11142` | 0.39047776383350075 | 0.3904777638335007 | -1 | unchanged |
  | 0.7384214715396948 | `kmyst-erfc-r8a27!B2131` | 0.2963532455163474 | 0.29635324551634745 | +1 | now ours |
  | 1.0120360000006228 | `kmyst-erfc-r8c02!B2806` | 0.15236286518682227 | 0.1523628651868223 | +1 | unchanged |
  | 1.133855197359528 | `kmyst-erfc-r8c11!B12999` | 0.10882135357193193 | 0.10882135357193196 | +2 | now ours |
  | 1.2477434152698585 | `kmyst-erfc-r8c19!B14809` | 0.07763510913068844 | 0.07763510913068843 | -1 | now ours |
  | 1.2552298757609002 | `kmyst-erfc-r8c20!B4056` | 0.0758709582369251 | 0.07587095823692512 | +1 | unchanged |
  | 1.3895972295864865 | `kmyst-erfc-r8c28!B12529` | 0.049392569300292365 | 0.04939256930029237 | +1 | now ours |
  | 1.5019749479478826 | `kmyst-erfc-r8c35!B2555` | 0.033660667142911285 | 0.03366066714291129 | +1 | unchanged |
  | 1.6234407324888618 | `kmyst-erfc-r8c41!B12972` | 0.021682062911535203 | 0.021682062911535207 | +1 | now ours |
  | 1.9777427541726267 | `kmyst-erfc-r8c58!B1853` | 0.005158741609633451 | 0.005158741609633452 | +1 | unchanged |
  | 1.999999999994299 | `kmyst-erfc-r10a03!B7094` | 0.004677734981165087 | 0.004677734981165086 | -1 | unchanged |
  | 2.0980403743165046 | `kmyst-erfc-r8b05!B6701` | 0.0030064550726207197 | 0.0030064550726207192 | -1 | now ours |
  | 3.4470393500808028 | `kmyst-erfc-r8b46!B8686` | 1.088912745444291e-06 | 1.0889127454442908e-06 | -1 | now ours |
  | 3.4677215861840427 | `kmyst-erfc-r8b47!B6701` | 9.385719563052113e-07 | 9.385719563052112e-07 | -1 | now ours |
  | 3.5625000000032463 | `kmyst-erfc-r8d09!B7311` | 4.700878390220142e-07 | 4.7008783902201417e-07 | -1 | unchanged |
  | 3.664062500001378 | `kmyst-erfc-r8d22!B3104` | 2.1979381408397832e-07 | 2.197938140839783e-07 | -1 | unchanged |
  | 6.033383929192591 | `kmyst-erfc-r6edge2!B2734` | 1.432267499362933e-17 | 1.4322674993629332e-17 | +1 | unchanged |
  | 8.023485545192495 | `kmyst-erfc-r4zoomf1!B4817` | 7.681947674754094e-30 | 7.681947674754095e-30 | +1 | now ours |
  | 10.807240000000013 | `kmyst-erfc-r4cat3!B5888` | 9.813472760469224e-53 | 9.813472760469222e-53 | -1 | unchanged |
  | 13.046480000000013 | `kmyst-erfc-r4tile3!B17525` | 5.1656301475547037e-76 | 5.165630147554705e-76 | +2 | unchanged |
- Curated evidence on main (excel-graph values copied unchanged, provenance in each case description):
  probes-numeric `t1916-erf-small` (4,878 cells), `t1916-erfc-mid` (9,024), `t1916-erfc-tail` (10,142),
  `t1916-erf-wrappers` (13,137) and `t1916-erfc-exp-ties` (20). Every ERF, ERFC, ERF(a,b), NORM.S.DIST and
  NORM.DIST cell passes, and since T-1929 every CHISQ.DIST.RT and Z.TEST cell too; the failures are the 20 residuals.
- `ironcalc-calc/ENGINEERING/ERFs.xlsx` E8 and G8 (`ERFC(6)`, `ERFC.PRECISE(6)`) stay known failures: that workbook was
  cached by Microsoft Macintosh Excel, which gives 0x1.8cf81557d20bap-56, while the excel-graph oracle (Windows,
  probes-numeric `t1040-erfc` A97) gives 0x1.8cf81557d20b8p-56, the engine's value. A platform difference
  (Mac Excel does not run the x87 code), not a kernel miss.
Status: VERIFIED. ERF, ERF.PRECISE, ERFC and ERFC.PRECISE are bit-exact on every probed input except the 20 EXP
near-tie cells. Implemented in `erf.rs` and `x87_exp.rs` (T-1916).
Tasks: T-1778, T-1916; T-1929 routed the wrappers, and T-1931 put the worksheet EXP on the same model V6 (both
done); T-0968, T-1187, T-0971 and T-0972 inherit the map.

## RANK of a non-reference array
Rule: RANK, RANK.EQ and RANK.AVG need a reference as their second argument. An array that is not a reference
(`IF(A1:C1>0,A1:C1)`, 1x3; `IF(A1:B2>0,A1:B2)`, 2x2) gives `#VALUE!` in every element of the array's own shape that is not an error (a spill), so
`RANK(5,IF(A1:C1>0,A1:C1))` fills F1:H1 with `#VALUE!`. References produced by INDEX, CHOOSE and OFFSET rank normally
(`RANK(5,INDEX(A1:C2,1,0))`, `RANK(5,CHOOSE(1,A1:C1))`, `RANK(5,OFFSET(A1,0,0,1,3))` = 2 with A1:C1 = 7,1,5). A literal second argument
(`{7,1,5}`, `5`, `"5"`, `TRUE`) is not usable in a workbook at all: excel-graph refuses to open a file containing
`RANK(5,{7,1,5})`, `RANK.AVG(5,{7,1,5})`, `RANK.EQ(5,{7,1,5})`, `RANK(5,5)`, `RANK(5,"5")` or `RANK(5,TRUE)` (HTTP 501
`unsupportedWorkbook`; `RANK(5,SORT(A1:C1))` as well), so their values are not observable through the web oracle (M14 desktop question, decision 0039).
Evidence: probes/rank-nonref-06..08 (references), rank-nonref-12..14 (arrays), probes/multiarea-order-stats H14:J14 (excel-graph, 2026-10-09, T-1203).
Ours: matches since T-1203: the RANK family returns a `#VALUE!` array of the argument's shape, and static analysis
makes such a formula a dynamic-array anchor so it spills (since T-1999 the `ref` is `ArgKind::ReferenceByCaller`: static
analysis broadcasts the array's shape with the lifted `number` and `order`, and the driver broadcasts its errors).
For a literal array (`{7,1,5}`) we also spill `#VALUE!` 1x3; that is
UNVERIFIED (the oracle refuses the workbook). Scalar literals (`RANK(5,5)`, `"5"`, `TRUE`) still rank as a
one-element list, also UNVERIFIED (decision 0039).
Error and text arguments next to such a ref (probes/rank-nonref-errors, excel-graph, 2026-10-09, T-1208; RANK and RANK.AVG
agree): an error `number` is that error in every element of the ref's shape (`RANK(#N/A,IF(A1:C1>0,A1:C1))` fills F1:H1 with
`#N/A`, a `#DIV/0!` number fills `#DIV/0!`), a text `number` is `#VALUE!` in every element, and an `order` of an error, text or a
number `1` is `#VALUE!` in every element (the array ref decides the shape and its `#VALUE!` wins over an `order` error). An error
`number` therefore takes precedence over the ref's own `#VALUE!` and the result still spills. Not probed: an error `number` together
with an error `order` (which wins).
RANK.EQ shares the RANK implementation and was not probed separately.
Ours: matches since T-1215 (the error or text `number` spills over the ref's shape); the `order` cases matched already.
Error elements inside such a ref (probes/rank-nonref-error-elements, excel-graph, 2026-10-10, T-1303; RANK, RANK.EQ and RANK.AVG
agree): each element that is an error keeps its own error and every other element is `#VALUE!`, so
`RANK(5,IF(A1:C1>6,NA(),1/0))` with A1:C1 = 7,1,5 fills `#N/A`, `#DIV/0!`, `#DIV/0!`, and `RANK(5,IF(A1:C1>1,1/0,A1:C1))` fills
`#DIV/0!`, `#VALUE!`, `#DIV/0!`. A text element is `#VALUE!` like a number. An error `number` still fills every element with
its error (`RANK(#N/A,IF(A1:C1>6,1/0,A1:C1))` is `#N/A` x3). An error element also wins over an `order` of `1` or `"x"`
(`RANK(5,IF(A1:C1>6,A1:C1,1/0),"x")` is `#VALUE!`, `#DIV/0!`, `#DIV/0!`).
Refs whose shape a formula does not show (probes/rank-unknown-shape-ref, excel-graph, 2026-10-10, T-1303): `A1:<undefined name>`
(`A1:CWWWW1`) is `#NAME?`, so `RANK(5,IF(A1:C1>0,A1:CWWWW1))` spills `#NAME?` x3, and with A1:C1 empty (every test FALSE) it
spills `#VALUE!` x3. A defined name for A1:C1, `A1:B1:C1`, `A1:C1 A1:C1` and `(A1,B1:C1)` are references and rank to 2.
`RANK(5,IF(A1:C1>0,A1:XFD1))` is `#SPILL!`.
Ours: matches since T-1303. Static analysis makes a RANK call whose `ref` has an unknown shape a dynamic-array formula, so a
reference result is 1x1 and an array result spills (before T-1303 such an array reached a scalar cell, a fuzz crash).
An `@`-prefixed ref (probes/rank-at-if-ref, excel-graph, 2026-10-15, T-1596) is the top-left value of the array, a value and not a
reference, and ranks like a one-element non-reference array, in one cell with no spill: with A1:B2 = 7,1/5,3,
`RANK(5,@IF(A1:B2>0,A1:B2))`, `RANK(7,@IF(...))`, `RANK(5,@IF(A1:B2>9,A1:B2))` (FALSE), `RANK(5,@(A1:B2*1))`, `RANK(5,@{3,4})` and
`RANK("x",@IF(...))` are `#VALUE!`; `RANK(5,@IF(A1:B2>0,A1:ANK))` (ANK an undefined name) is `#NAME?` and `RANK(5,@IF(A1:B2>0,1/0))`
`#DIV/0!` (the element's error); RANK.EQ and RANK.AVG agree. `@` of a picked reference stays one: `RANK(5,@IF(TRUE,A1:A30))` in row 28
ranks against blank A28 (`#N/A`), and `RANK(5,@IF(TRUE,A1:B2))` off the range is `#VALUE!`. Ours: matches since T-1596 (before, the
whole 2x2 reached a cell static analysis took for a scalar, a fuzz crash).
An `@`-prefixed scalar literal (`RANK(5,@5)`, `@"5"`, `@TRUE`) is `#VALUE!` in our engine by the same rule, unlike the bare literal; UNVERIFIED (not probed).
Status: VERIFIED (excel-graph); literal arguments UNVERIFIED
Tasks: T-1152, T-1203, T-1208, T-1215, T-1303, T-1596

## VARA family: single-cell reference arguments
Rule: a single-cell reference argument of VARA, VARPA, STDEVA and STDEVPA, also one picked by IF, INDEX or XLOOKUP, coerces like a cell inside a range: text counts 0 and TRUE counts 1 (A1:A3 = 1, 2, 4; B1 = "x": `VARA(A1:A3,B1)` → 2.9166..., B2 = TRUE → 2). An array literal counts only its numbers (`VARA({1,2,4,"x"})` and `VARA({1,2,4,TRUE})` → 2.3333...). A direct `"2"` and TRUE coerce (1.5833..., 2) and a direct non-numeric text is `#VALUE!` (S!D37-D39 are known failures, T-1291).
Evidence: probes/vara-single-ref-picked (41 cells; oracle: excel-graph, 2026-10-15, T-1617); probes/stat-argument-modes Sheet1!H69, H72, H75, H78 (direct forms).
Ours: matches except the direct forms. Forms `(A1)`, `Sheet1!A1`, LET-bound and TAKE-picked cells were not probed: UNVERIFIED.
Status: VERIFIED (plain reference, IF/INDEX/XLOOKUP-picked, array literals); other forms UNVERIFIED
Tasks: T-1567, T-1617, T-1291

## Statistical function argument modes
Rule: every scalar argument of the M9 statistical functions lifts elementwise over an array literal and over a multi-cell
range (all 302 lift cells are 2: 210 `ROWS(F(..{v;v}..))` and 92 `ROWS(F(..G1:G2..))`): distribution parameters including
`cumulative`, LARGE/SMALL `k`, PERCENTILE `k`, QUARTILE `quart`, PERCENTRANK and Z.TEST `x`, RANK `number`, T.TEST
`tails`/`type`, TRIMMEAN `percent`, FORECAST `x`, PROB `lower_limit`, FORECAST.ETS `target_date` and FORECAST.ETS.STAT
`statistic_type`. The data arguments of AVEDEV, DEVSQ, GEOMEAN, HARMEAN, KURT, SKEW, SKEW.P, MEDIAN, STDEV*, VAR* skip
text, booleans and blanks inside a range but coerce typed ones (`AVEDEV(C1:C6,5)` over 1, "2", TRUE, "x", 4, blank is
AVEDEV(1,4,5); `AVEDEV(1,"2",4,TRUE)` = 1, AVEDEV(1,2,4,1)). The A-variants (AVERAGEA, MAXA, MINA, STDEVA, STDEVPA,
VARA, VARPA) count range text as 0 and TRUE as 1 (`AVERAGEA(C1:C6,5)` = 11/6) and a typed "2" as 2. MODE, MODE.SNGL
and MODE.MULT are `#N/A` for `(1,"2",4,TRUE)` because they skip typed text (numeric text included) and typed booleans:
`MODE(1,2,2,"x")` and `MODE(1,2,2,TRUE)` are 2, `MODE(2,"x",2)` is 2, `MODE("2","2",5)`, `MODE(TRUE,TRUE,5)` and `MODE(FALSE,FALSE,5)` are `#N/A`, and `MODE.SNGL(1,TRUE,2,2)` is 2 (a TRUE read as 1 would make 1 the first mode); MODE.SNGL and MODE.MULT agree (probes/mode-typed-arguments A1-A20, excel-graph, 2026-10-16, T-1291). Also a direct error propagates (`MODE(1,"1",#DIV/0!,2,3)` is `#DIV/0!`, A22), an empty argument is `#VALUE!` (`MODE(1,1,)`, A27), a blank single-cell reference is `#VALUE!` (A23) and a reference to a text cell is skipped (`MODE(K1,K2,"1",2)` is 2, A25). A direct LAMBDA argument is `#VALUE!` in VARA, VARPA, STDEVA, STDEVPA, VAR and AVERAGEA (`VARA(LAMBDA(x,x),1,2)`; probes/vara-direct-lambda, excel-graph, 2026-10-16, T-1291). Array arguments (CORREL, COVAR, F.TEST, T.TEST, LARGE, PERCENTILE, QUARTILE, TRIMMEAN,
Z.TEST, LINEST, ...) take array literals whole with the range's value; LARGE, SMALL, PERCENTILE*, QUARTILE*,
PERCENTRANK*, TRIMMEAN, Z.TEST, F.TEST and T.TEST skip text and booleans in a range, GROWTH, TREND, LINEST and LOGEST are
`#VALUE!` for them, and the pairing functions are `#N/A` for unequal sizes with text (size-only: `CORREL(A1:A5,B1:B6)` `#N/A`; with equal sizes the text pair is skipped instead, `CORREL(C1:C6,B1:B6)` 0.9014729960756244; probes/regression-pairs-a L2, L9, T-1620; unequal sizes with text are `#N/A` too, `CORREL(C1:C6,B1:B8)`, as in PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, CHISQ.TEST and CHITEST: the size check decides before the text is looked at; T-1292). The FORECAST.ETS family is `#N/A` for a
plain 8-point series on the timeline 1..8 (T-1293). These decide the generator modes in eval/catalog/functions.yaml
(`value`, `aggregate`, `aggregate_a`, `array`; RANK `ref`, COUNTBLANK and the *IFS ranges are `range`).
Evidence: probes/stat-argument-modes (573 cells, excel-graph, 2026-10-09, T-1080).
Ours: LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN lift (T-1772, section below), and so do the
44 functions whose every argument is scalar and whose kernel is bounded (T-1290, "Later arguments and element values
of the scalar statistical functions" below); the other scalar arguments lift since their own tasks (before, `#N/IMPL!`
for an array, `#VALUE!` for a range): Z.TEST and PROB since T-1813, FORECAST and T.TEST since T-1982, RANK since T-1999;
BETA.INV, F.INV, T.INV, CONFIDENCE.T and their variants lift
since T-1848 ("Quantile searches: per-call cap and lifting" below), the incomplete-gamma functions (CHISQ.*,
CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON, POISSON.DIST) since T-1872 ("Incomplete gamma:
per-call cap and lifting" below), and HYPGEOM.DIST and HYPGEOMDIST since T-1873 ("HYPGEOM.DIST: per-call cap and
lifting" below); the pairing functions are `#N/A` for unequal sizes since T-1292, which also made F.TEST take twice the smaller tail of the F distribution (`F.TEST(C1:C6,B1:B8)`: Excel 0.9195979047175995, ours 0.9195979047175997, not 2 minus that; T-1927); FORECAST.ETS is
`#N/IMPL!`; many distribution values are a few ulps off (the existing accuracy entries above).
Status: VERIFIED (excel-graph)
Tasks: T-1080, T-1290, T-1291, T-1292, T-1293, T-1848, T-1872, T-1873

## Later arguments and element values of the scalar statistical functions (T-1290)
Rule: the later arguments of the statistical functions whose every argument is scalar lift over a multi-cell range too,
on the formula's own rows (not an intersection), and so do the optional ones over an array: the last argument of
BETA.DIST, BETADIST, BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST, NEGBINOMDIST, BINOM.INV, CRITBINOM,
NORM.DIST, NORMDIST, NORM.S.DIST, NORM.INV, NORMINV, CONFIDENCE, CONFIDENCE.NORM, STANDARDIZE, EXPON.DIST, EXPONDIST,
WEIBULL, WEIBULL.DIST, LOGNORM.DIST, LOGNORMDIST, LOGNORM.INV, LOGINV, PERMUT, PERMUTATIONA, T.DIST, T.DIST.2T,
T.DIST.RT, TDIST, F.DIST, F.DIST.RT and FDIST over a two-cell range is 2 rows (H1-H73), each element the scalar call
(I, J, rounded to 12 places: `INDEX(NORM.DIST(1.5,1,2,G25:G26),2)` with G26 FALSE is the pdf, 0.193334058401). BETA.DIST's and
BETADIST's `[A]` and `[B]` and BINOM.DIST.RANGE's `[number_s2]` lift over an array and a range (H3-H9, H15, K3-K16).
Element values follow the scalar rules: `{1;"x"}` gives `#VALUE!` in its element and `{1;"1"}` reads the text
(L4, L5), an error element is its error (`PERMUT({5;#N/A},2)`, L6), each element is checked alone
(`GAMMA({1.5;-1})` element 2 `#NUM!`, `PERMUT({5;5},{2;6})` element 2 `#NUM!`, L18, L20), TDIST checks x and
`tails` per element (`TDIST({-1;1.5},5,1)` and `TDIST({1.5;1.5},5,{3;2})` element 1 `#NUM!`, L9, L10), and
broadcasting is the operators' (`NORM.DIST({1;2;3},{0,1},1,TRUE)` is 3x2, L16, L17). BETA.DIST's `cumulative` takes
the text "TRUE" (L2) and a one-cell range or reference as its value (`BETA.DIST(0.5,2,3,G1:G1)` with G1 TRUE is
0.6875, L11, L12). A typed TRUE as an array element reads as 1 in BETA.DIST's x (L1: 1), F.DIST's x (L13:
0.5 rounded), STANDARDIZE's x (L14: 0) and NORM.S.DIST's z (L15), as a direct TRUE does in F.DIST.RT, FDIST
and STANDARDIZE (probes-numeric/t1049-fdistrt A19, A26, m9-legacy-fdist A90; "SKEW, SKEW.P, KURT, ..." below).
Evidence: probes/stat-later-argument-lift (141 formula cells, value cells rounded to 12 places, excel-graph, 2026-10-17, T-1290); probes/stat-argument-modes.
Ours: matches (T-1290); BETA.DIST, F.DIST, F.DIST.RT and FDIST read a TRUE as 1 since T-1849 (L1, L13;
PHI's read is probed separately). Every argument of these functions is `Lift`
in the lifting table and each kernel reads its arguments in its scalar order; T.DIST.2T and TDIST read every argument
before they check any ("Overflowing arguments, PERMUT's limit and argument error order" below). One scalar change
here: BETA.DIST's `cumulative` given as a one-cell range was `#VALUE!` and is now its value (L11). BINOM.INV and
CRITBINOM are `KernelWork::Weighted(4096)`: their search halves a `u64` interval at most 64 times, one incomplete-beta
cdf each, up to about 65 us an element in release (`BINOM.INV(1E12,0.3,alpha)`). PERMUT is `Constant`: its product
stops once it is infinite, after at most 171 factors, where `PERMUT(1E15,1E15)` looped 10^15 times. The quantile searches of BETA.INV, F.INV,
T.INV and CONFIDENCE.T lift since T-1848 (next section), and the incomplete-gamma functions (CHISQ.*, GAMMA.DIST,
GAMMA.INV, POISSON) since T-1872 ("Incomplete gamma: per-call cap and lifting" below), and HYPGEOM.DIST, its cdf
sum capped, and HYPGEOMDIST since T-1873 ("HYPGEOM.DIST: per-call cap and lifting" below).
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted BINOM.INV or CRITBINOM is charged 4096 units per element
against the shared cell-visit cap of 2^27 (`limits::MAX_LIFTED_CELL_VISITS`), so one of more than 32,768 elements is
`#ERROR!` before it runs where Excel computes it (`ROWS(BINOM.INV(5,0.4,SEQUENCE(40000)/40001))`, unit test
`weighted_kernels_are_charged_their_weight`). At the worst case measured, 32,768 elements take about 2.1 s, the budget
of the Bessel entries (engineering.md "BESSEL order cap").
Status: VERIFIED (excel-graph)
Tasks: T-1290, T-1849, T-1755, T-1848

## Quantile searches: per-call cap and lifting (T-1848)
Rule: BETA.INV, BETAINV, F.INV, F.INV.RT, FINV, T.INV, T.INV.2T, TINV and CONFIDENCE.T lift every argument over an array
and over a two-cell range on the formula's own rows, each element the scalar call (the last argument over a range is 2
rows, H1-H25 with values I, J; BETA.INV's and BETAINV's `[A]` and `[B]` over a range and an array, H3-H11, K1-K8). Element
values follow the scalar rules: `{0.4;"0.4"}` reads the text and `{0.4;"x"}` is `#VALUE!` in its element (L4, L5),
`CONFIDENCE.T(0.4,2,{15;1})` element 2 is `#DIV/0!` (L6, and `CONFIDENCE.T(0.4,2,1.5)` too: size truncates to 1, L17),
an error element is its error (L8), broadcasting is the operators' (`T.INV({0.1;0.2;0.3},{5,10})` is 3x2, L9, L10), a
one-cell range is its value (L11), `T.INV.2T(1,5)` is 0 (L18), and every argument is read before any is checked
(`BETA.INV(0.4,-1,NA())`, `CONFIDENCE.T(0.4,-2,NA())`, `T.INV(2,NA())` and `F.INV(2,NA(),5)` are `#N/A`, L12-L16). A TRUE
element reads as 1: `F.INV.RT({0.4;TRUE},5,5)` element 2 is 0 (L19), and `F.INV`, `BETA.INV` and `CONFIDENCE.T` of a
TRUE element are `#NUM!` (p = 1 or alpha = 1; L2, L3, L20).
Excel also inverts where statrs' search fails: `F.INV(1E-300,5,5)` = 5.081599884913164E-121, and `F.INV(p,5,5)` follows
the power law 5.0816E-09 * (p/1E-20)^0.4 from p = 1E-20 to 1E-300 (E1-E5, E12); `F.INV(1E-50,6000,4)` =
0.0004924127886173762 (E6); `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` = 0.0691959487211844 (E9), far
from the true quantile (about 0.003), as Excel's BETA.INV is erratic where a shape is extreme ("Inverse CDF functions at
a tiny or near-1 probability" below). A cell holding `=1-2^-53` is a number that `=C1=1` calls equal to 1 (E13, E14).
Evidence: probes/stat-quantile-lift (124 formula cells, excel-graph, 2026-10-18, T-1848); probes/stat-argument-modes
H90-H99, H173-H177, H231-H245, H518-H529.
Ours: matches the lift cells (T-1848). A TRUE element reads as 1 in BETA.INV, F.INV and F.INV.RT
since T-1849 (L2, L3, L19). Their quantile comes from statrs 0.18's
`inv_beta_reg`, ported line for line to `statistical/inverse_beta.rs` with a per-call cap of its own: at most 512 Newton
steps and 4,096 step adjustments (64 and 512 for Student's t). Since T-2119 BETA.INV and BETAINV refine that estimate to
the crossing of BETA.DIST's cdf, the T-0970 incomplete beta, as F.INV does. Where statrs finishes, the port returns its double bit for bit
(unit test `the_port_is_statrs_where_statrs_finishes`; probes-numeric is unchanged cell for cell), so the values
probes-numeric checks are main's. statrs needs at most 377 steps where it finishes (13 million samples: F at p down to
1E-162, Beta shapes 1E-5 to 1E5, Student t). Below the old thresholds the T.INV quantile is still a bisection on the cdf of
at most 264 evaluations; BETA.INV, BETAINV, F.INV, F.INV.RT and FINV refine statrs' estimate to the crossing of the
T-0970 incomplete beta (`special::refine_increasing`, at most 129 cdfs, 65 without a start; T-0974, "F.INV is a crossing of
F.DIST's tails" below). The lifting table charges BETA.INV, BETAINV, F.INV, F.INV.RT and FINV 2^18 units an element and
T.INV, T.INV.2T, TINV and CONFIDENCE.T 2^16: the bound of each kernel's caps at a full-cost incomplete-beta cdf (140
continued-fraction steps, 3.1 to 3.4 us in release), about 2.5 ms (the statrs search plus the refinement of BETA.INV and F.INV) and 0.82 ms (T.INV's 264-cdf bisection); the cases measured cost far less (167 us for
`BETA.INV(1E-300,0.5,1E6)`, 17.5 us for `T.INV.2T(1E-9,99999)`); lift/kinds/statistical.rs has the measurements.
Since T-0974 the F quantiles below p = 1E-20, where statrs has no start, bisect the cdf: `F.INV(1E-300,5,5)` to
`F.INV(1E-100,5,5)` (E1-E5) and the lifted A7 of the unit test are Excel's power-law values to 13 digits, no longer
`#NUM!`. `F.INV(1E-50,6000,4)` is 0.016348827552952715 where Excel gives 0.0004924127886173762 (E6), and
`F.INV(1E-50,2000,2)` 0.008195481623830967 where Excel gives 0.004452511009947361 (E7): our deep tail of the incomplete
beta at these df is not Excel's (T-1871). E8, E11 and E12 are within the 15th digit and E10 is 25 units of it off (T-1871).
Where statrs' search never finished (its step loop never ends once a Newton step is infinite, found once for BETA.INV at
p = 1-2^-53), the capped search has no start and the refinement bisects the cdf: since T-2119
`BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` is 0.003013141..., a known mismatch: Excel gives
0.0691959487211844 (E9; it was `#NUM!`). Unit test `quantile_searches_that_hung_or_panicked_are_bounded`; T-1871 gives
Excel's value there. (statrs also panicked for every F.INV below p = 1.6E-162 and hung for about 7% of F.INV
calls below p = 3.6E-27; since T-0974 those bisect the cdf without a start and give a number, see E6 above.)
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted BETA.INV, BETAINV, F.INV, F.INV.RT or FINV of more than 512
elements, and a lifted T.INV, T.INV.2T, TINV or CONFIDENCE.T of more than 2,048, is `#ERROR!` before it runs where Excel
computes it (2^27 units, `limits::MAX_LIFTED_CELL_VISITS`, over 2^18 and 2^16 an element;
`ROWS(BETA.INV(SEQUENCE(513)/514,2,3))`, unit test `quantile_kernels_are_charged_their_weight`). At the bound of the caps,
512 BETA.INV elements take about 1.7 s and 2,048 T.INV elements about 1.7 s, the 2 s budget of the Bessel entries
(engineering.md "BESSEL order cap"). Showing that a slow cdf and a long search never coincide would allow larger lifts.
Status: VERIFIED (excel-graph)
Tasks: T-1848, T-1849, T-1871

## Incomplete gamma: per-call cap and lifting (T-1872)
Rule: CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV,
POISSON and POISSON.DIST lift every argument over an array and over a two-cell range on the formula's own rows, each
element the scalar call (stat-argument-modes H132-H156, H293-H314, H476-H485; the last argument over a range is 2 rows,
stat-quantile-lift H27-H50 with values I, J). Element values follow the scalar rules: `{1.5;"x"}` is `#VALUE!` in its
element and `{1.5;"1.5"}` reads the text (L1, L2), a TRUE element is 1 in CHISQ.DIST's x (L3: 0.037434226753), each
element is checked alone (`CHISQ.DIST(1.5,{5;0.5},TRUE)`, `POISSON({5;-1},1,TRUE)`, `CHIDIST({1.5;-1},5)` and
`GAMMAINV({0.4;1},2,3)` element 2 `#NUM!`, L4, L5, L15, L16; `CHIINV({0.4;1},5)` element 2 is 0, L17), an error
element is its error (L6), every argument is read before any is checked (`GAMMA.INV(2,NA(),1)`, `CHISQ.INV(2,NA())`,
`POISSON(-1,NA(),TRUE)` are `#N/A`, L7-L9), broadcasting is the operators' (`GAMMA.INV({0.1;0.2;0.3},{2,3},1)` is 3x2,
L10, L11), `cumulative` takes the text "TRUE" and "FALSE" per element and `"x"` is `#VALUE!` (L12, L13), POISSON's x
truncates per element (L14), and a one-cell range is its value (L18).
Excel computes a huge shape: `GAMMA.DIST(1E16,1E16,1,TRUE)` = 0.5000000013298076, `POISSON(1E16,1E16,TRUE)` =
0.4999999986701924, `GAMMA.DIST(1E300,1E300,1,TRUE)` = 0.5, `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)` = 1,
`GAMMA.INV(0.5,1E300,1E-300)` = 1.0002658235505755 (E1-E5), `GAMMA.DIST(1E10,1E10,1,TRUE)` = 0.5000013298076014,
`GAMMA.DIST(1E12,1E12,1,TRUE)` = 0.5000001329807602, `POISSON(1E10,1E10,TRUE)` = 0.5000026596152026 and
`POISSON(1E12,1E12,TRUE)` = 0.5000002659615203 (E8-E11), `CHISQ.INV(0.26774,1312890000)` = 1312858246.6164234
(E14), `GAMMA.INV(0.5,1E9,1)` = 999999999.6666666 (E19), and `CHISQ.DIST(1E10,1E10,TRUE)` = `GAMMA.DIST(5E9,5E9,1,TRUE)`
= 0.5000018806319452 (E16, E17). `GAMMA.DIST(1E300,1,1E-10,TRUE)` and its pdf are `#NUM!` (x / beta overflows; E6,
E18), while `GAMMA.DIST(1E-300,2,1E100,TRUE)` is 0 (x / beta underflows; E7).
Evidence: probes/incomplete-gamma-cap (38 formula cells, excel-graph, 2026-10-18, T-1872); probes/stat-quantile-lift
H27-H50; probes/stat-argument-modes H132-H156, H293-H314, H476-H485.
Ours: matches the lift and element cells (T-1872), each kernel reading its arguments in its scalar order. The
incomplete gamma is the accurate pair of T-0971 ("Incomplete gamma: accurate kernel" below; it replaced the port of
statrs 0.18's `gamma_lr` and `gamma_ur` that T-1872 capped) and its quantile statrs' `Gamma::inverse_cdf` (halve and
double a bracket, 8 bisections, 4 Newton steps), ported line for line to `statistical/incomplete_gamma.rs`, with a
per-call cap of its own: at most 2^19 terms in one cdf, and in one whole inversion over all its cdfs (unit test
`a_call_past_its_cap_is_nan`). The series needs about 6.8 * sqrt(a) terms at x = a: 481,117 at
the largest chi-squared shape (df 1E10, a = 5E9), so every chi-squared cdf is kept. CHISQ.TEST uses the same capped
cdf (a sheet's cell count keeps its shape under 2^33, where the series can need about 630,000 terms: past the cap, see
the DEVIATION below). The lifting table charges every entry 2^18 units an element: the bound of the cap at the slowest
term, the continued fraction's NaN path (4.1 to 4.7 ns in release; a converging fraction at the cap, which rescales on
about 65% of its terms at a shape of 1E15, costs 2.4 ns, and a series term 2.5 ns), plus the inversion's 2,120 cdfs,
about 2.6 ms (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT` has the measurements; 1.2 ms for `CHISQ.DIST(1E10,1E10,TRUE)`).
Where x / beta underflows to 0 (statrs panicked) the cdf is 0 at every shape, as Excel's `GAMMA.DIST(1E-300,a,1E100,TRUE)` is for a = 2 (E7) and for a = 1, 0.5, 0.01, 1E-17 and 1E-300 (probes/t2982-gamma-inv-huge-shape A117-A122), although the true P(1E-17, 1E-400) is 1 - 9E-15; its density is `#NUM!` (A124, A125), and so is the density where x / beta overflows (`GAMMA.DIST(1E300,1,1E-10,FALSE)`, E18), as the cdf there is (E6; T-2982). At a statistic of 5E-324, whose half underflows, `CHISQ.DIST(x,2,TRUE)` is 0 and `CHISQ.DIST.RT(x,2)` and
`CHIDIST(x,2)` 1 where main panicked (Excel unprobed). CHISQ.TEST cannot reach that statistic: its sum of
`diff*diff/exp` is never 2^-1074.
From a shape of 1E8 the pair is Temme's uniform asymptotic expansion (`incomplete_gamma.rs` `temme_pair`, T-1891):
with phi = (x - a)/a - ln(x/a) and z = ±sqrt(2 phi), the smaller tail is erfc(sqrt(a phi))/2 ± e^(-a phi)/sqrt(2 pi a)
· sum C_k(z)/a^k (C_0..C_3), a fixed amount of work charged as one term, where the series needed 675,286 terms at
a = x = 1E10 and never ended at 2^53. It is bit-exact on Excel's E1-E4, E8-E12, E16, E17 and E20 (the kept chi-squared
cdfs at df 1E10, 0.4999982128217492 on statrs and main, are now Excel's 0.5000018806319452) and 1 ulp off at E13; a
shape of 1E8 needs no term past C_3 (they fall by 1/a each). The cap of 2^19 terms now binds a cdf nowhere (below 1E8
the series takes at most about 68,000 terms) and still binds an inversion's cdfs together below that shape.
DEVIATION (decision 0014, CLAUDE.md directive 5): an inversion whose cdfs together need more than 2^19 terms, or that
statrs' search cannot finish, is `#NUM!` where Excel gives a number: `GAMMA.INV(0.5,1E300,1E-300)` (E5, Excel
1.0002658235505755). Above a shape of 1E9 the quantile is statrs' search alone (`REFINE_SHAPE_MAX`), so
`CHISQ.INV(5.6221E-26,4715100000)` is 4714346327.59633 where Excel's is 4714082845.627977 (E15); E14 and E19 are now
1 ulp off Excel. A CHISQ.TEST's cdf is never past the cap. Unit tests `incomplete_gammas_that_hung_or_panicked_are_bounded`
and `a_call_past_its_cap_is_nan`; T-2982 gives Excel's values for the inversions.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted CHISQ.*, CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV,
GAMMAINV, POISSON or POISSON.DIST is `#ERROR!` before it runs where Excel computes it once its elements' charges pass
2^27 units (`limits::MAX_LIFTED_CELL_VISITS`). Since T-2795 an element is charged by value
(`ValueWork::IncompleteGamma`), at most 2^18: in proportion to the bound of its work at the measured costs (4.7 ns a
term, 49 ns a cdf) against the bound 2^18 stands for. A cdf of shape at most a takes at most `cdf_terms_bound(a)` terms:
the power series a + 58 (it runs for x < a + 1, where every ratio x / (a + n) is below 1 and at most 1/2 from n = 2x - a),
the small-shape Q series 20 more, and the continued fraction a budget of 256 + 32 * cbrt(a) terms, enforced by the kernel
(NaN past it). A sweep of shapes 1E-3 to 1.6E15 needed at most about 10 * cbrt(a) terms (110 below a = 10), so the budget
is at least twice the need and changes no value (unit test `the_continued_fraction_stays_within_half_its_budget`). An
inversion is charged 2,250 cdfs of that bound, within its one budget of 2^19 terms. The shape comes from the shape
argument (alpha, df / 2, or POISSON's x + 1); an element with an error argument, or an inversion whose probability is
outside (0, 1), a logical or a blank, is charged 1, as the kernel returns before any cdf; a text shape or probability is
charged 2^18. So `CHIDIST(x,5)` costs 150 units an element and a CHIDIST over a 731-element PIVOTBY runs (gen-m9/
c2974c061384fdd9 A56, Excel `#SPILL!` in its layout), while 513 inversions at a huge shape are still refused
(`ROWS(CHIINV(0.4,1E10+SEQUENCE(513)))`, unit test `incomplete_gamma_kernels_are_charged_their_weight`), and so are
about 535 cdfs at a huge shape. At the bound of the cap, 512 elements take about 1.3 s, within the 2 s budget of the
Bessel entries (engineering.md "BESSEL order cap").
Status: VERIFIED (excel-graph)
Tasks: T-1872, T-1891, T-0971, T-0972, T-2795

## Incomplete gamma: accurate kernel (T-0971)
Rule: Excel's regularized incomplete gamma (GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, CHISQ.TEST,
CHITEST, POISSON.DIST and POISSON cdfs) is close to correctly rounded against a 50-digit reference, in both tails:
the upper tail is not 1 - P.
CHISQ.TEST and CHITEST take the direct tail at every df (T-3058; `CHITEST` at chi2 = 18, df 2 is `EXP(-9)`,
probes-numeric/m9-legacy-chitest A23). A shape of 1 is the exponential distribution in doubles: `GAMMA.DIST(1,1,1,TRUE)` =
0.6321205588285577 = `-EXPM1(-1)` (BETA_GAMMA.xlsx L2, GAMMA.xlsx E4, E6). At a huge shape Excel keeps its digits
where statrs lost them: `CHISQ.DIST(1E10,1E10,TRUE)` = 0.5000018806319452 (probes/incomplete-gamma-cap E16; statrs
0.4999982128217492), and `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)` is 1 (E4), where statrs' continued fraction never
ended.
Evidence: ironcalc-calc STATISTICAL/GAMMA.xlsx, CHISQ.xlsx, CHISQ_TEST.xlsx, BETA_GAMMA.xlsx, POISSON.xlsx (Excel-authored
cached values); probes-numeric `m9-gamma-dist-a*`, `m9-chisq-dist`, `m9-chisq-dist-rt`, `t1048-*`, `t1049-*` (T-1045);
probes/incomplete-gamma-cap (T-1872).
Ours: `statistical/incomplete_gamma.rs` `incomplete_gamma_pair`, replacing statrs' `gamma_lr`/`gamma_ur` (whose
prefactor exp(a ln x - x - ln Gamma(a)) carries an ulp of that exponent into the result, and whose Q is 1 - P below
x = a). The prefactor x^a e^-x / Gamma(a) is `powf * exp / tgamma` below a = 20 (each factor normal) and DiDonato and
Morris' `rcomp` above it (sqrt(a / 2 pi) exp(-a rlog(x/a) - Stirling correction)); P is the power series below
x = a + 1, with the geometric sum of the terms it leaves (near x = a they add up to thousands of times the last one:
7E-15 of P(1E6, 1E6)), and Q the modified-Lentz continued fraction above it; below a shape of 1/2 and x of 1 Q is formed
directly (TOMS 654's small-shape Q, with ln Gamma(1 + a) from its zeta series); a = 1 is (-expm1(-x), exp(-x)).
Unit tests (`incomplete_gamma.rs`): within 8 units of 2^-52 of mpmath at a in {0.001, 0.01, 0.3, 0.5, 9, 500}, P + Q = 1
within 2 of them, the a = 1 identity, the b = 1E6 scale. The 2^19-term cap (section above) is unchanged; the series now
runs to 2^-54 instead of 1E-15, about 8% more terms near x = a, so an inversion's budget runs out at a somewhat smaller
shape than statrs' (`GAMMA.INV(1E-5,7.9E8,1E-6)` is now `#NUM!`; 3E8 still finishes; T-2367 measures the bound).
Where the prefactor underflows to 0 the tail it multiplies is 0 at once, as statrs' `ax < MIN_LN` shortcut had it:
past x = 2^54 the continued fraction's `b += 2` leaves b unchanged and its delta stays an ulp below 1, so without the
shortcut `CHISQ.DIST(1E30,10,TRUE)`, `GAMMADIST(1E300,123456789.123,1E5,TRUE)` and
`GAMMA.DIST(0.5,123456789.123,1E-300,TRUE)` spent the cap and were `#NUM!` where Excel is 1 (gen-m9).
Below a shape of 1/2 and x of 1, P is 1 - Q wherever that Q is at most 1/2: the series then gives P an ulp low as it
nears 1, and `GAMMADIST(0.5,1E-300,1,TRUE)` is Excel's 1 (POISSON.DIST truncates it as its k, gen-m9/11eb3ab4f4f8104c
A252).
The density (GAMMA.DIST, GAMMADIST, CHISQ.DIST with cumulative FALSE) is the same prefactor at x / scale, divided by
x (`gamma_density`). A shape of 1 is e^(-x/scale) / scale. x = 0 and an x / scale that is not normal keep statrs'
closed forms. That gives `GAMMA.DIST(10,9,2,FALSE)` = 0.032639019674079374, Excel's and correctly rounded (GAMMA.xlsx
E3, E34, E42, E58, BETA_GAMMA.xlsx L6; statrs' exp(ln pdf) was 78 ulp off).
Not yet Excel's (T-2638): deep upper tails and tiny-x values such as `CHIDIST(100,8)` and `CHISQ.DIST(1E-10,8,TRUE)`,
and `CHIINV(0.975,5)`. Ours is correctly rounded there against 50 digits, and Excel is 7-37 ulp off.
Not Excel's either, and not correctly rounded on either side (gap triage 2026-10-29, 60-digit mpmath reference;
T-3147): CHISQ.DIST at 50 and 100 degrees of freedom, the `rcomp` branch of the prefactor (shape 20 and up).
`CHISQ.DIST(10,100,FALSE)` is 9.838365191393825E-32 in Excel, 63 ulp below the true 9.8383651913938939E-32, and ours
is 65 ulp above it; `CHISQ.DIST(10,100,TRUE)` is 57 ulp below and ours 57 above; `CHISQ.DIST(5,50,TRUE)` 52 below,
ours 10 above; `CHISQ.DIST(50,100,FALSE)` 5 below, ours 11 above (probes-numeric/m9-chisq-dist A61, A62, A51, A64; also
A52, A54, A58, A63). At shape 5, `CHISQ.DIST(40,10,FALSE)` (A50) is 7 ulp above the truth in Excel and 1 below in
ours. So Excel's large-shape exponent rounds differently from TOMS 654's `rcomp` with `(x/a - 1) - ln(x/a)` outside
[a/2, 2a], and the shape-20 split of `gamma_prefactor` is not Excel's (UNVERIFIED).
Status: VERIFIED (excel-graph)
Tasks: T-0971, T-0972, T-2638, T-3147

## HYPGEOM.DIST: per-call cap and lifting (T-1873)
Rule: HYPGEOM.DIST and HYPGEOMDIST lift every argument over an array and over a two-cell range on the formula's own rows,
each element the scalar call (stat-argument-modes H331-H336, H338-H342: every argument over `{v;v}`, the first over a
range; the last argument over a range is 2 rows, stat-quantile-lift H51-H54 with values I, J:
`INDEX(HYPGEOM.DIST(2,3,5,10,G51:G52),2)` with G52 FALSE is the pmf, 0.416666666667, and
`INDEX(HYPGEOMDIST(2,3,5,G53:G54),2)` with G54 12 is 0.318181818182).
Evidence: probes/stat-argument-modes H330-H342 (excel-graph, 2026-10-09); probes/stat-quantile-lift H51-H54
(excel-graph, 2026-10-18).
Ours: matches the lift cells (T-1873); the scalar H330 and H337 keep main's last-digit residues ("HYPGEOM.DIST is not
exact for small integer arguments" above, T-0977). Both are `Lift` entries over one kernel of the four counts
(HYPGEOMDIST is its pmf, no longer a call with a fifth argument node), which reads its arguments in the scalar order,
a boolean count `#VALUE!` as on main (Excel reads a TRUE as 1: probes-numeric/t1049-hypgeomdist A53, a known failure
unchanged here), and checks each element alone. Since T-0977 the pmf and cdf are a double-double walk of at most 2^12
steps (78 us in release) up to a population of 2^52, and past that walk the fallbacks below. The fallback pmf is
statrs' closed form. The fallback cdf is statrs 0.18's `Hypergeometric::cdf`, which sums the pmf term by term from 0 to
x, one `exp` of three `ln_binomial`s
a term: x + 1 terms whatever the distribution, so `HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` took 4.8 s
in release and an x near 2^64 would never finish. It is now ported to `statistical/hypegeom.rs` with a per-call cap of
its own: the port skips the terms that are exactly 0, those below max(0, n + K - N) (`exp(-inf)`) and, at a population
up to 2^52, those far enough from the mode that the computed exponent is below -750 with a margin for its rounding (a
bisection on each side of the mode, at most 131 exponents; the true log pmf rises to the mode and falls after it; the
skip relies on the platform `exp` returning exactly 0 below about -745.13, as glibc's does), and sums the rest in
statrs' order. Adding +0.0 leaves the sum unchanged, so where at most 2^18 terms are left the value is statrs' double,
bit for bit (unit tests `the_port_is_statrs_within_the_cap`, `skipping_the_zero_terms_keeps_statrs_double`,
`the_slowest_call_on_main_keeps_its_double`: 96,259 terms summed there, 0.5000797091291159 as on main; probes,
probes-numeric, ironcalc-calc and gen-m9 are unchanged cell for cell). The non-zero terms lie within about 39 standard
deviations of the mode, so the cap keeps every population up to about 1.84E8 whatever the sample and x. The lifting
table charges HYPGEOM.DIST 2^22 units an element, the bound of the cap: 2^18 + 131 terms at 250 ns, about 66 ms. The
slowest terms measured in release are where most of the window is subnormal (`exp`'s subnormal path and additions with
a subnormal operand), 206 to 234 ns: 32 elements of
`HYPGEOM.DIST(99994114707,200000000000,200000000000,400000000000,TRUE)`, 262,144 terms each (96,639 exactly 0, 153,739
subnormal), took 1.73 to 1.96 s; normal terms cost 167 to 193 ns (32 elements at the widest distribution of a
population of 1.8E8, 258,239 terms each, 1.38 to 1.55 s). hypegeom.rs `MAX_TERMS` has the measurements.
DEVIATION (decision 0014, CLAUDE.md directive 5): where more than 2^18 terms are left the cdf is `#NUM!` where main
gave a number, and Excel's value there is unprobed (T-1896; no suite case is near the cap: the largest sums 501
terms). That is a cdf whose standard deviation is above about 3,400 (a population above about 1.84E8 with a wide
sample), such as `HYPGEOM.DIST(537370912,1073741823,1073741823,2147483646,TRUE)`. Since T-2429 a count of 2^31 - 1 or
more is `#NUM!` as in Excel ("HYPGEOMDIST: a sample below the possible range is 0; a negative or huge count is
#NUM!"), so populations past that, such as `HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)` (main
1.0000065293398746 after 191 s in release) and the subnormal window measured above, no longer reach the cap. Unit tests
`a_call_past_its_cap_is_nan`, `the_cap_counts_the_terms_summed` and `a_cdf_sum_is_bounded`; T-1896 probes Excel's
values there.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted HYPGEOM.DIST whose elements together are charged more than
2^27 units (`limits::MAX_LIFTED_CELL_VISITS`) is `#ERROR!` before it runs where Excel computes it. Since T-2433 an
element is charged by its counts, not the cap's bound: a cdf is either the walk or the sum, and the sum's window is at
most m + 1 terms, m = min(sample_s, number_sample, population_s) truncated (above min(number_sample, population_s) the
cdf is 1 before any sum), so an element is charged the larger of the walk's 2^14 units and (m + 132) * 16 (its terms
and the window's 131 exponents at an upper bound of about 250 ns each), at most 2^22, the bound of the cap.
So at most 32 elements whose m
is about 2^18 or more (`ROWS(HYPGEOM.DIST(SEQUENCE(33)*0+1000000,1000000,1000000,1,TRUE))`: the charge reads the
counts, not that this element would be `#NUM!`), and at most 8,192 elements with m up to 892
(`ROWS(HYPGEOM.DIST(2,3,5,10,SEQUENCE(8193)>0))`; before T-2433 33 such elements were `#ERROR!`, where Excel spills:
gen-m9/7bd219ec0c0c5e22 A159 over 40 elements). A text count bounds nothing (it may read as any count), so m is the
least of the other counts, and the bound when all three are texts. An element is charged 1 when a count is an error,
negative, or 2^31 - 1 or more: the kernel refuses it before either path. Unit tests
`hypgeom_dist_is_charged_its_weight` and `hypgeom_dist_elements_are_charged_by_their_counts`. Measured in release
(benches/lift.rs): 32 elements at the bound take 1.35 to 1.53 s; 8,192 4,000-step walks (the cdf's walk is about
120 us, more than the pmf's 78 us) 0.99 to 1.0 s; 1,634 cdf sums of 5,001 terms, charged 82,112 units each, 1.25 to
1.4 s: within the 2 s budget of the Bessel entries (engineering.md "BESSEL order cap").
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted HYPGEOMDIST of more than 16,384 elements is `#ERROR!` before
it runs where Excel computes it (2^13 units an element, the bound of its pmf walk, twice BINOM.DIST's; T-0977:
`ROWS(HYPGEOMDIST(1,4,8,SEQUENCE(16385)+19))`, unit test `hypgeom_dist_is_charged_its_weight`): about 1.3 s at the
bound.
Status: VERIFIED (excel-graph) for the lifting (the Rule's cells); the term cap is a DEVIATION whose Excel values are unprobed (T-1896)
Tasks: T-1873, T-1896, T-0977, T-2433

## Discrete distributions: within an ulp of the exact sum (T-0927)
Rule: BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST are close to the exact value of their finite sums, not to a
regularized incomplete beta evaluated in doubles: over the 148 numeric cells of those three sheets of ironcalc-calc
STATISTICAL/BINOM.xlsx, the exact sum rounded once (ours, below; checked against Python `fractions` from the double `p`)
is Excel's double at 78, within 4 ulp of it at 57 more, and within 4E-14 relative at the other 13 (below). Exact arithmetic also reproduces the cells whose exact value is a short decimal: `BINOM.DIST(7,8,0.7,TRUE)` is 0.94235199 and
`BINOM.DIST(2,10,0.5,TRUE)` 0.0546875, where statrs' incomplete beta gave 0.9423519899999973 and 0.05468750000000168.
Excel is not always the exact value: `BINOM.DIST(3,10,0.5,TRUE)` is 0.17187500000000006, one ulp above the exact
0.171875, and `NEGBINOM.DIST(5,10,0.5,TRUE)` 0.15087890625000006. `NEGBINOM.DIST(0,100,0.01,TRUE)` (the pmf p^r) is
1.0000000000000348E-200, which is `exp(100*ln(0.01))` in doubles bit for bit, 226 ulp from the exact 0.01^100
(1.000000000000002E-200): Excel's prefactor is `exp(a*ln x)`, as for BETA.DIST ("Incomplete beta: Excel's closed forms
and the best candidate", T-1744). The 13 cells further than 4 ulp from both the exact value and ours are tails where that
prefactor's rounding shows: BINOM.DIST E24, E32, E67; BINOM.DIST.RANGE E10, E21, E23; NEGBINOM.DIST E3, E26, E30, E37,
E40, E74, E75 (for example E24, `BINOM.DIST(2,10,0.999999,TRUE)`: Excel 4.499992001038836E-47, exact
4.499992001038802E-47). Neither Loader's saddle-point pmf summed (R's `dbinom_raw`: bit-exact at 49 of the 122 cells
tried, more than 4 ulp off at 15) nor the prefactor `exp(a*ln x)` times the exact remaining factor reproduces them.
POISSON.DIST is unfitted. On ironcalc-calc STATISTICAL/POISSON.xlsx, `fl(fl(S)*exp(-mean))`, with S the partial
exponential series `sum mean^i/i!` over i <= k (cdf) or its last term (pmf) summed exactly, reproduces all 5 distinct
small pmfs and 8 of 11 small cdfs bit for bit, against 6 of 11 for the exact value rounded. The excel-graph probes
contradict it, though: probes-numeric/t1048-poisson `POISSON.DIST(10,4,FALSE)` is 0.005292476676420117 (that model
...0195), `POISSON.DIST(1,0.1,FALSE)` 0.09048374180359597 (...596), `POISSON(10,4,TRUE)` 0.9971602338794863 (...862)
and `POISSON(1,0.1,TRUE)` 0.9953211598395555 (...556), all of which statrs gets right. Applied to even-df
CHISQ.DIST.RT through T-1778's identity, the same model gives `CHISQ.DIST.RT(1E-06,30)` = 1.0000000000000002 where
Excel gives 1.0 (probes-numeric/m9-legacy-chidist A15, A16), and it misses m9-chisq-dist-rt A32 and t1049-chisqdistrt
A8. So POISSON.DIST stays on statrs here (T-2055).
T-2145 dense sweep (excel-graph; probes-numeric/t2145-binomial-sweep, t2145-negbinom-sweep, t2145-binom-range-sweep:
3260 BINOM.DIST, 1100 NEGBINOM.DIST and 790 BINOM.DIST.RANGE cells, p in {0.5, 0.25, 0.125, 0.1, 0.2, 0.3, 0.7, 0.9,
0.99, 0.999}, no error cells). Against the exact value rounded once (Python `fractions`, from the exact decimal p),
BINOM.DIST is bit-exact at 1017 of 3260 (31%). Of the other 2243, 1017 are within 4 ulp (steps between doubles), 1011 more
within 1E-14 relative and 215 more within 1E-12 relative (against the exact value); none is beyond (worst 1.4E-13). NEGBINOM.DIST is exact at 394 of 1100 and
BINOM.DIST.RANGE at 393 of 790. Patterns in the BINOM.DIST cells (all counts from the sidecars):
- Exact share by p: 48% (0.1), 50% (0.125), 40% (0.2), 45% (0.25), 37% (0.3), 43% (0.5), then 16% (0.7), 14% (0.9),
  8% (0.99), 10% (0.999): it drops sharply above 0.5.
- pmf against cdf: the pmf is exact at 234 of 1630 (14%), the cdf at 783 of 1630 (48%).
- Every cdf cell with k = n is exactly 1 (170 of 170), which may be a short-circuit (1 is also the exact value, so this does not prove one). The k = 0 pmf, `(1-p)^n`,
  is exact at only 44 of 170, so even that single power is not the exact value rounded once.
- The exact share falls with n: 30 of 40 at n = 1, 72 of 220 at n = 10, 158 of 720 at n >= 30.
- For p in {0.5, 0.25, 0.125}, where the exact value is often a short decimal, Excel is still off in 50 to 57% of
  cells, from 1 ulp (31% of the misses) to about 210 ulp (median 2): the exact short decimal is not Excel's answer. Even the exact power of two
  `BINOM.DIST(0,50,0.5,FALSE)` = 0.5^50 is 8.881784197001244E-16, 8 doubles below the exact 8.881784197001252E-16
  (t2145-binomial-sweep A2781; A2782 the cdf, A2803 the pmf at k = 50 likewise).
NEGBINOM.DIST goes the other way: its exact share is 15 to 17% at p = 0.1 and 0.2, 27% at 0.125, 31% at 0.3, 40% at
0.25, 56% at 0.5, then 36%, 42%, 45%, 48% at 0.7 to 0.999, so the BINOM.DIST trend must not be carried over to it.
Hypothesis only (not tested): the growth of the error with n and with p near 1 is what a prefactor `exp(a*ln x)`
would give. The per-cell values are the fitting data for T-2146.
Evidence: ironcalc-calc STATISTICAL/BINOM.xlsx and POISSON.xlsx (Excel-cached); probes-numeric/t2145-binomial-sweep,
t2145-negbinom-sweep, t2145-binom-range-sweep (per-cell values in their .oracle.json sidecars); probes-numeric/t1048-poisson,
m9-legacy-chidist, m9-chisq-dist-rt, t1049-chisqdistrt (excel-graph); the exact values from Python `fractions`.
Ours (T-0927): `statistical/discrete.rs` walks the pmf terms, each from the last by the exact ratio of consecutive
terms, in double-double with a power-of-two exponent beside it (the terms start at q^n, as small as 0.5^3000), and
rounds once: the binomial from the nearer end (a pmf), from 0 (a cdf, stopping once the terms past the mode no longer
move the sum; past the budget 1 minus the sum from n where that is at least 2^-20), and from the cheaper end (a
BINOM.DIST.RANGE). NEGBINOM.DIST is that binomial with n = f + r and the roles of p and q swapped: P(F <= f) is the
probability of at most f failures in f + r trials, and its pmf is r/(f + r) times the binomial pmf at f. A walk takes
at most 2^10 steps (`MAX_STEPS`); past it the functions fall back to statrs as before. The 13 binomial cells above are
the residue. probes-numeric/m9-legacy-binomdist (21 -> 33 of 54 bit-exact) and t1048-negbinomdist (18 -> 28 of 40)
lose no passing cell, but three of their tails move further from Excel, the same `exp(a*ln x)` residue:
`BINOMDIST(2,5,0.999,FALSE)` (A35, A36: Excel 9.980010000000044E-09, main ...049, ours ...027) and
`NEGBINOMDIST(10,2,0.9)` (A4, B4: Excel 8.909999999999952E-10, main ...973, ours ...981).
DEVIATION (decision 0014, CLAUDE.md directive 5): BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST and
NEGBINOMDIST are charged 4096 units an element against the shared cell-visit cap of 2^27
(`limits::MAX_LIFTED_CELL_VISITS`), the bound of two walks of 2^10 steps at about 32 ns, so a lifted call of more than
32,768 elements is `#ERROR!` before it runs where Excel computes it (as BINOM.INV, "Later arguments and element values of
the scalar statistical functions" above).
Status: VERIFIED (excel-cached) for the values; the weight is a DEVIATION
Tasks: T-0927, T-1744, T-2055, T-2145, T-2146

## Overflowing arguments, PERMUT's limit and argument error order (T-1290)
Rule: an F.DIST, F.DIST.RT or FDIST call whose `deg_freedom1 * x` overflows a double is `#DIV/0!` for the cdf and the
right tail and `#NUM!` for the pdf (`F.DIST(1E308,2,3,TRUE)` `#DIV/0!`, `F.DIST(1E308,2,3,FALSE)` `#NUM!`,
`F.DIST.RT(1E308,2,3)` and `FDIST(1E308,2,3)` `#DIV/0!`; with deg_freedom1 = 1 the product does not overflow and the
cdf is 1, the pdf 0; A1-A6, B12, E11). A BETA.DIST whose `B - A` overflows is 0 at x = A and 1 at x = B in the cdf
and `#DIV/0!` everywhere else, the pdf at x = B included (`BETA.DIST(1E308,2,3,TRUE,-1E308,1E308)` 1,
`BETA.DIST(0,2,3,TRUE,-1E308,1E308)` `#DIV/0!`, `BETADIST(1E308,2,3,-1E308,1E308)` 1; A7-A10, E9; with a finite width
`BETA.DIST(1E308,2,3,TRUE,0,1E308)` is 1, E10). PERMUT's number must be below 2^31 - 1 after truncation:
`PERMUT(2147483646,1)` is 2147483646 and `PERMUT(2147483646.5,1)` too, `PERMUT(2147483647,1)`, `PERMUT(1E10,1)` and
`PERMUT(1E30,0)` are `#NUM!` (B8-B11, D1-D9, E1-E8; `PERMUT(2147483646,0)` is 1), and a product past the double range
is `#NUM!` (`PERMUT(171,171)`, `PERMUT(200,180)`; `PERMUT(170,170)` is 7.257415615307994E+306; A13-A18). Errors
between arguments: T.DIST.2T and TDIST read every argument before they check x or tails (`T.DIST.2T(-1,NA())`,
`TDIST(-1,NA(),1)` and `TDIST(1,NA(),3)` are `#N/A`, and per element too; F1-F6), as do F.DIST and BETA.DIST
(`F.DIST(-1,NA(),3,TRUE)`, `BETA.DIST(2,-1,NA(),TRUE)` `#N/A`; F7, F8). The 1E308 values sit in cells: a literal
1E308 is not written into a formula, and a text "1e308" cell is `#VALUE!` in these functions.
Also `TDIST(-1,5,NA())` is `#N/A` (an error in tails wins over x < 0), the BETA.DIST pdf at x = A with B - A
overflowing is `#DIV/0!`, and with x = 8E307 and deg_freedom1 = 2 (df1*x = 1.6E308, finite) F.DIST is 1 (cdf) and 0
(pdf), F.DIST.RT and FDIST 0 (stat-overflow-arguments-b A1-A9).
Evidence: probes/stat-overflow-arguments and -b (excel-graph, 2026-10-18, T-1290).
Ours: matches (T-1290). Before, the overflowing F.DIST and BETA.DIST calls panicked in statrs' incomplete beta
(an unwrapped NaN), the F pdf of a huge x was `#NUM!` (statrs' direct form overflows to inf/inf), PERMUT gave numbers
of 2^31 - 1 and more, and TDIST checked x before reading deg_freedom (`TDIST(-1,NA(),1)` was `#NUM!`). The F pdf falls
back to logarithms only when the overflow comes from x alone: `df1^df1`, `df2^df2` and `(df1+df2)^(df1+df2)` finite,
and only a result that underflows to 0 is taken from it, so `F.DIST(1E308,1,3,FALSE)` and `F.DIST(8E307,2,3,FALSE)`
are 0. A positive density there (moderate x with df1 near 30-140, `F.DIST(20,100,3,FALSE)`) and degrees of freedom
large enough to overflow those powers (`F.DIST(3,50,100,FALSE)`, `F.DIST(2,5,1E10,FALSE)`) were `#NUM!` until T-1864,
because the logarithmic form is not accurate enough there; they are now Loader's form ("F density at large degrees of
freedom (T-1864)").
Status: VERIFIED (excel-graph)
Tasks: T-1290, T-1864

## RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments
Rule: RANK `number`/`order`, FORECAST `x`, PROB `lower_limit`/`upper_limit`, Z.TEST `x`/`sigma` and T.TEST
`tails`/`type` lift over an array and a range, and each element is the scalar call over the data read once. An
element checks its own scalar arguments before the data's error: with D2 = `#N/A`, `Z.TEST(D1:D3,"a")`,
`PROB(D1:D3,F1:F3,"a")`, `FORECAST("a",D1:D3,A1:A3)` and `RANK("x",D1:D3)` are `#VALUE!` and `T.TEST(D1:D3,A1:A3,3,2)`
is `#NUM!`, while a lifted element with good arguments is the data's `#N/A` (`PROB(D1:D3,F1:F3,{1;2})`,
`Z.TEST(D1:D3,{1;2})`, `FORECAST({1.5;2},D1:D3,A1:A3)`, `T.TEST(D1:D3,A1:A3,{1;2},2)`; `RANK({1;3},D1:D3)` is `#N/A`,
not the rank of 1). The order-statistic functions are the other way round: `SMALL(NA(),0)`, `PERCENTILE(NA(),2)`,
`TRIMMEAN(NA(),2)` and `QUARTILE.EXC(NA(),9)` are `#N/A`. Z.TEST's `x` coerces numeric text and booleans
(`Z.TEST(A1:A8,"5")`, `TRUE`, also as array elements) and a reference to other text is `#VALUE!`; a `sigma` of 0 or
below is `#NUM!` per element, but after the data's error (`Z.TEST(D1:D3,4,0)` is `#N/A`). The scalar arguments also come
before the data's own validation: `PROB(A1:A3,F1:F4,"a")` (unequal sizes), `PROB(A1:A4,{0.5;0.5;0.5;0.5},"a")` (a sum of 2)
and `Z.TEST({"a","b"},"x")` (no numbers) are `#VALUE!`, and `PROB(A1:A3,F1:F4,{1;"a"})` is `#N/A`, `#VALUE!`. PROB's limits coerce TRUE (`PROB(A1:A4,F1:F4,{1;TRUE})` element 2 is the probability of
1), `lower > upper` is 0, and a ragged pair of limits is `#N/A` past the shorter one. T.TEST truncates `tails` and `type`
(`1.7`, `2.9` are 1 and 2) and `tails` 3 or `type` 4 is `#NUM!` per element. A lifted RANK `number` against a
non-reference `ref` broadcasts with the ref's shape, every element `#VALUE!` (`RANK({4;"x"},IF(A1:C1>0,A1:C1))` is
2x3). A 1x1 array argument gives a 1x1 array (`TYPE(Z.TEST(A1:A8,{4}))`, `TYPE(RANK({4},A1:A8))`,
`TYPE(LARGE(A1:A8,{1}))` are 64).
Evidence: probes/stat-test-lift P1-P89, stat-test-lift-b P90-P95 (excel-graph, 2026-10-19, T-1813); probes/rank-direct-boolean (T-1999).
Ours: Z.TEST/ZTEST and PROB match since T-1813 (`statistical/z_test.rs`, `prob.rs`: the data argument is
`WholeByCaller`, read once). FORECAST/FORECAST.LINEAR, T.TEST/TTEST and the order-statistic precedence match
since T-1982 (`forecast.rs`, `t_dist.rs`; PERCENTILE's and TRIMMEAN's readers keep an error value). T.TEST's
P64-P67 are still NumericClose (P64: 0.32346094498174827 against 0.32346094498173894), which is T-0969's
work. RANK, RANK.EQ and RANK.AVG match since T-1999 (`statistical/rank_eq_avg.rs`: `number` and `order` lift, a
reference `ref` is read once, a non-reference `ref` broadcasts its errors through `lift_call_beside`).
RANK's `number` and `order` outside an array (probes/rank-direct-boolean H1-H18, excel-graph, 2026-10-19, T-1999; RANK,
RANK.EQ and RANK.AVG agree where probed): `number` coerces a boolean and numeric text, typed or from a cell (`RANK(TRUE,A1:A8)` and
`RANK(K1,A1:A8)` with K1 = TRUE are 8, the rank of 1; `RANK("4",A1:A8)` and a text cell "4" are 5; FALSE is 0, absent, `#N/A`).
`order` coerces a boolean (`RANK(4,A1:A8,TRUE)` is 4, ascending; FALSE and a FALSE cell are 5) but not text, numeric
text included: `RANK(4,A1:A8,"1")` is `#VALUE!`. `order` is read before the data's error: with D2 = `#N/A`,
`RANK(4,D1:D3,"x")` and `RANK(1,D1:D3,"x")` are `#VALUE!` and `RANK(4,D1:D3,1/0)` is `#DIV/0!`. So an element reads
`number`, then `order`, then the data's error, then ranks (`#N/A` when absent). Before T-1999 a direct boolean was `#VALUE!`,
`"1"` was an ascending order and the data's error came before `order`'s. Not probed: an `order` from a cell holding text, and `number`'s error against `order`'s (T-2019).
Not yet probed (T-2000): T.TEST's `type` against the samples' error, FORECAST's `x` against the data's
own validation, and an error data argument of PERCENTRANK, QUARTILE, PROB and FREQUENCY.
Status: VERIFIED (excel-graph)
Tasks: T-1813, T-1982, T-1999, T-2000

## LARGE, SMALL, PERCENTILE, QUARTILE, PERCENTRANK and TRIMMEAN: each lifted element is the scalar call
Rule: when `k`, `quart`, `x`, `significance` or `percent` is an array or a multi-cell range, every element is the
scalar call over the whole data argument, with the scalar rules per element. With A1:A8 = 1,2,4,3,7,5,8,6:
`LARGE(A1:A8,{1;2;9})` is 8, 7, `#NUM!` (W1-W2); `LARGE(..,{0.5;2.5})` is `#NUM!`, 6 (k below 1, then the ceiling;
W3-W4); `SMALL(..,{2.5;"3";TRUE})` is 2, 3, 1 (typed text and booleans coerce; W5-W7); over a range G1:G4 = 2, "x",
blank, TRUE the elements are 7, `#VALUE!`, `#NUM!` (a blank cell is 0), 8 (a referenced TRUE is 1; W8-W11); an error
element is that error (`{1;#DIV/0!}`, W16). The shape is the lifted argument's: a row (`COLUMNS` 3, W14), 2-D (W15),
and PERCENTRANK's `x` and `significance` broadcast a column against a row (`PERCENTRANK.EXC(A1:A8,{4.5;2.5},{1,2})` is
2x2, element (2,2) 0.27; W32-W33). `significance` lifts over an array and a range (W29-W31) and is `#NUM!` below 1 per
element (W34). A data error or data with no numbers keeps the lifted shape and gives the scalar error in the elements
probed: `LARGE(B1:B3,{1;2})` with B2 = `#N/A` has 2 rows (W12) and element 2 is `#N/A` (W13); `SMALL(C1:C3,{1;2})`
over "x", TRUE, blank has 2 rows (W18) and element 1 is `#NUM!` (W17); `PERCENTRANK(C1:C3,{1;2})` element 2 is
`#N/A` (W39). The data argument keeps its own reading: an arithmetic array (W19), an array literal (W38), a union
(`LARGE((A1:A4,A5:A8),{1;2})` element 2 is 7, W45). A one-cell range in the lifted argument is its value (`ROWS` 1,
W20, W43), and a range a function returns lifts (`IF(TRUE,G5:G6)`, W42). A 1x1 array argument (`LARGE(A1:A8,{1})`) is
unprobed: ours gives a 1x1 array (TYPE 64, where the scalar call is TYPE 1), by analogy with INDEX's 1x1 array
index (lookup.md "INDEX lifts over array and range indices", probes/index-1x1-array-index, T-1507); T-1813 probes it.
Evidence: probes/stat-whole-lift W1-W48 (excel-graph, 2026-10-17, T-1772); probes/stat-argument-modes H345-H463,
H549-H550.
Ours: matches (T-1772): the data argument is read once by the function's own reader and sorted once per call
(`functions/statistical/sorted_data.rs`), and the scalar arguments lift through `lift_call`.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted call is charged against the shared cell-visit cap of 2^27
(`limits::MAX_LIFTED_CELL_VISITS`) before its first element: numbers + elements for LARGE, SMALL, PERCENTILE*, QUARTILE*
and PERCENTRANK* (`LiftCost::Prepared`, the data is sorted once), numbers × elements for TRIMMEAN (`LiftCost::Cells`,
each element averages the kept values); and its result is bounded by the shared 1,048,576-element array bound (arrays.md,
"Array results above 1,048,576 cells are #ERROR!"). Past either the call is `#ERROR!` where Excel computes it:
`ROWS(TRIMMEAN(A1:A1000000,SEQUENCE(200)/1000))` over a filled column (unit test `trimmean_is_charged_per_element`) and
`ROWS(PERCENTRANK(A1:A1000000,SEQUENCE(1100),SEQUENCE(1,1000)))`, a 1,100,000-element result (unit test
`a_sorted_column_against_many_elements_runs_and_the_bound_holds`, which also shows `ROWS(LARGE(A1:A1000000,SEQUENCE(200)))`
is 200).
Status: VERIFIED (excel-graph)
Tasks: T-1772, T-1813

## Array arguments given a scalar, a single cell or an error
Rule: an array argument of the paired and single-array statistical functions takes a plain number or a single cell as an
array of one value, and an error value as that error; the usual rules then apply. For the paired functions, unequal sizes
are `#N/A` and one point is `#DIV/0!` (COVAR of one pair is 0): `TTEST(A7:C7,<number>,1,1)`, `STEYX(D4:D10,B6)`,
`COVAR(0,C6:D6)`, `COVAR(D4:D8,<number>)`, `RSQ(A4,B11:C12)`, `INTERCEPT(B9:E9,1E-15)`, `CHISQ.TEST(B4,C5:D11)`,
`FORECAST.LINEAR(x,C2:D7,<number>)` and `FORECAST.LINEAR(x,0.2,D11:D12)` are `#N/A`; `SLOPE(123456789.123,<number>)`,
`INTERCEPT(1.005,2.5)`, `FORECAST.LINEAR(x,0.2,1)` and `FORECAST.LINEAR(x,7,137.5)` are `#DIV/0!`; `COVAR(0,D6)` is a number (inferred 0;
`(B6-0.5)>COVAR(0,D6)` is TRUE). `PEARSON(D5,SEQUENCE(0))` is `#CALC!`. `T.TEST(B11,A8:B9,0.1,0.3)` with B11 = FALSE (one value, no number) is `#NUM!` (tails/type before
the paired unequal-size `#N/A`; whether an error array argument beats a bad tails/type is unprobed). GROWTH of one value is a number (`GROWTH(GAMMA(100))` =
9.332621544394404E+155), `TREND(#NUM!)` is `#NUM!`, and `GROWTH(B2:B3,<number>,..)` with B2:B3 = "?b", 0 is `#REF!` (the size
check before the text check). LARGE of an error is that error (`LARGE(<#VALUE!>,k)`, `LARGE(<#REF!>,0.1)`), and so is
PERCENTRANK (`PERCENTRANK(<#N/A>,100)` is `#N/A`). A single cell holding text is no number, like text inside a range: `QUARTILE(A1,1)`, `QUARTILE(A1,7)`, `QUARTILE.INC/.EXC`,
`PERCENTILE.INC(A1,1)` and `PERCENTILE.EXC(A1:A3,0/1)` with A1 = " x " are all `#NUM!` (empty data), as is `QUARTILE(A3,1)` of an empty
cell and `BETAINV(2,QUARTILE(A1,7),1)` (probes/quartile-single-text-cell B1-B11, excel-graph 2026-10-24, T-2066). Numeric-looking text is no number either: with A1 = the text "5", `QUARTILE(A1,1)`, `PERCENTILE.INC(A1,0)` and `QUARTILE.EXC(A1,1)` are `#NUM!` and `PERCENTRANK(A1,5)` is `#N/A`; k's error comes before the empty data in every one, `PERCENTILE(<no number>,1/0)`, `PERCENTILE.EXC(..,1/0)` and `QUARTILE.EXC(..,1/0)` being `#DIV/0!` (probes/quartile-numeric-text-cell C1-C9). PERCENTRANK of that text cell or an empty cell is `#N/A` for a number x, but an error x (`1/0` `#DIV/0!`) or text x (`#VALUE!`) comes first, EXC too, with or without a significance (B12-B28), and `PERCENTILE.EXC(A8:B8,<#VALUE!>)`
with no number in A8:B8 is `#VALUE!` (k's error before the empty-data `#NUM!`). `PERCENTRANK.EXC(B11,1.005)` with B11 =
1.005 is not 0.5 (inferred `#N/A` through `VLOOKUP(..)`; later probed: a single value equal to x is 1, probes/percentrank-single-value B6).
A single cell holding nothing is not one value: in the paired functions and F.TEST it is `#VALUE!` (see "A one-cell
reference to an empty cell is #VALUE! in the paired functions and F.TEST", T-2255), which supersedes the inference here.
Evidence: gen-m9 (excel-graph, 2026-10-12, T-1289): 1782efcecd5d4b1b Calc!A10, A25, A38; 3462856259a37891 A22;
50f10c4062fb2e98 A3, A17, A31; 696af2c8cee2f57e A9; 89693ab0cafc29df A24, A30; a2e51f48717af2a1 A34; adb3fd63c7368166 A26,
A33; b58b6d7cd39d195f A2, A31; bc0901ce6193d576 A12; d244411dd90f7fbd A10, A17, A21; eefcc3989f31de76 A22;
95efb37a9874f82e A2; b5352ebdb9c5f1a1 A24; 288b99797f58f149 A3; d5bd4272655c5211 A31; 65faaeb48419087d A17. The arguments
are read from the cases' Data sheets; each cell's other operands were checked not to decide the result.
Ours: GROWTH and TREND match the single-value, error and size-before-text rules (T-1402); the paired functions match (T-1401), a single cell holding text, a boolean or nothing being one value that is no number (a typed text or boolean literal too, for the regression functions and CHISQ.TEST: T-1723); `LARGE(#N/A,1)` is that error (gen-m9/1782efcecd5d4b1b A25); PERCENTILE.EXC reports k's error before the empty-data `#NUM!` (T-1403; PERCENTILE.INC's order is unprobed);
`PERCENTRANK.EXC` of a single value equal to x is 1, not 0.5 (probes/percentrank-single-value B6). `PERCENTRANK` and `PERCENTRANK.INC` of a single value equal to x are 1 (probes/legacy-value-argument-intersection-c T2, U2); with one value, x below or above it is `#N/A` for all three, an error x (`#DIV/0!`, text `#VALUE!`) beats that `#N/A`, and any numeric significance (0, negative, fractional) gives 1 rather than `#NUM!` (a text significance is `#VALUE!`) (probes/percentrank-single-value B2-B20); an empty array is `#N/A`.
Status: VERIFIED (web) for the listed cells
Tasks: T-1401, T-1402, T-1403

## SKEW.P of two numbers is #DIV/0!
Rule: `SKEW.P` of two numbers is `#DIV/0!`, like SKEW: `SKEW.P(E11:E11,A11:A12,A5:A7)` over -0.0001 and 1 (blank, FALSE,
TRUE and "a" skipped) is `#DIV/0!`. Three numbers are enough: `SKEW.P(1,2,3)` is 0, `SKEW.P(1,2,4)` 0.3818017741606059; a constant series of
three or more (`SKEW.P(5,5,5)`) and one number are `#DIV/0!`. Same bound as SKEW (`SKEW(1,2)` `#DIV/0!`, `SKEW(1,2,3)` 0).
Evidence: gen-m9/3b8e9078261b5bd2 Calc!A13 (`LAMBDA(v,acc,v)(SKEW.P(..),..)`, excel-graph, 2026-10-12, T-1289);
probes/skew-p-bound A1-A14 (excel-graph, 2026-10-19, T-1404).
Ours: matches (T-1404).
Status: VERIFIED (excel-graph)
Tasks: T-1404

## FORECAST.ETS.CONFINT in the generated gen-m9 suite: argument errors only
Rule: the three gen-m9 FORECAST.ETS.CONFINT cells are argument errors on excel-graph, never a forecast: a text timeline
(`FORECAST.ETS.CONFINT(..,A8:C8,A11:C11,7)` with A11 = " x ") is `#VALUE!`; an error in values
(`FORECAST.ETS.CONFINT(0.25,<#VALUE!>,<number>,NA())`) is `#VALUE!`, so values' error comes before confidence_level's `#N/A`;
values and timeline of different sizes (8 and 18 cells) are `#N/A`, and that `#N/A` beats MINVERSE's `#VALUE!` on the
right of `+`. They score the family's argument validation, not its algorithm (which is `#N/A` on excel-graph even for a
plain 8-point series; see "Statistical function argument modes", T-1293).
Evidence: gen-m9/1782efcecd5d4b1b Calc!A27, gen-m9/3462856259a37891 Calc!A28, gen-m9/55fc8c4ffbd7c7cc Calc!A8 (excel-graph,
2026-10-12, T-1289).
Ours: `#N/IMPL!` for every call.
Status: VERIFIED (web) for these three shapes
Tasks: T-1293, T-2424 (the gen-m9 sweep's 1,196 such cells: "FORECAST.ETS family on excel-graph: argument errors, text and
lifting decide before the #N/A (T-2255)" below)

## FORECAST.ETS family: excel-graph never computes it
Rule: on excel-graph (Excel for the web) every valid call of FORECAST.ETS, FORECAST.ETS.CONFINT,
FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT is `#N/A`, whatever the series: 12, 24 and 36 points (timeline 1..n,
a period-4 seasonal signal), seasonality omitted/0/1/4, `statistic_type` 1 and 8 (`data_completion` and
`aggregation` were only set together with an invalid value, so valid settings are unprobed, T-1553). The out-of-range arguments are `#N/A` too (seasonality 5, data_completion 2, aggregation 3,
statistic_type 0 and 9, confidence_level 0 and 1, values/timeline of 23 and 24 cells), so the web value does not tell
a valid call from an invalid one. The only distinguishable result is a text `target_date` (`"x"`), which is `#VALUE!`
before the `#N/A`. So the web oracle gives no evidence for the algorithm or for most argument validation; it is a
golden only for the type-coercion step (decision 0039). Catalog `oracle: none` is not used: the canary does not
refuse these workbooks, Excel for the web just does not calculate the family.
Consequence: a validation-only engine (`#N/A` like excel-graph) would score on excel-graph values but would be wrong on
desktop Excel, so the family stays `#N/IMPL!` until desktop values exist (M14, Phase 4, T-1553).
Evidence: probes/forecast-ets-longer-series.case.json K1:K42 (excel-graph, 2026-10-14, T-1293).
Ours: `#N/IMPL!` for every call that passes the argument step (next section, T-2424). Before that, `target_date` (FORECAST.ETS, CONFINT) or `statistic_type` (STAT) is lifted and read as a number: text there is `#VALUE!`, and an array or two-cell range gives that many `#N/IMPL!` elements (T-2028). UNVERIFIED (T-2028 follow-up probe): the optional settings (seasonality, data_completion, aggregation, confidence_level) also lift in ours, as the catalog's `value` mode says, with no Excel cell showing the shape of an array or range there (since T-2424 settled for SEASONALITY's data_completion
and aggregation by gen-m9 `#SPILL!` cells 67f5e0fe790e6bec A47, A307 and 22656c225ff4f22a A60; see the T-2255 section); and text `target_date` in CONFINT and text `statistic_type` in STAT (K38 covers FORECAST.ETS only), and a boolean, number-text or error `target_date`/`statistic_type` reads by the usual scalar coercion (an error is returned as itself;
settled by gen-m9 22656c225ff4f22a A1, fa25422d6f18c157 A181 and 43a13a091907b402 A261, see the T-2255 section).
Status: VERIFIED (web) that the family is `#N/A` there; desktop behavior UNKNOWN
Tasks: T-1293, T-1553, T-2028 (the text `target_date` and the lifting cells)

## Database function argument modes
Rule: in the database functions (DAVERAGE, DCOUNT, DCOUNTA, DGET, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM,
DVAR, DVARP), `database` must be a reference. A database given as an array literal
(`DSUM({"Name","Qty","Price";"a",1,2.5;...},"Price",E1:E2)`) makes Excel for the web refuse the whole workbook
(createSession 501 `FileCorruptTryRepair`, excel-graph 2026-10-11; a refused workbook cannot be committed. To
reproduce, recalc a one-cell spec with `=DSUM({"Name","Qty","Price";"a",1,2.5;"b",3,4},"Price",E1:E2)`).
DGET returns the matching cell's value, text included (T-1390 sample: "?b").
`field` is a header name or a 1-based column index: `DSUM(A1:C6,"Price",E1:E2)` = `DSUM(A1:C6,3,E1:E2)` = 14.
Over an array literal `field` lifts (`ROWS(DSUM(A1:C6,{2;3},E1:E2))` is 2), but a two-cell range is
`#VALUE!`. An unknown name (`"Nope"`) and the numeric text `"2"` are `#VALUE!`, so text is never read as an
index. A boolean field `TRUE` is column 1 and `FALSE` is `#VALUE!` (T-1948, probes/db-field-legacy-true-numeric-first-column, excel-graph 2026-10-30:
`DSUM(J1:L6,TRUE,N1:N2)` over a numeric first column is 19, the sum of column 1, `DAVERAGE` 4.75, `DMAX` 7, `DCOUNT`/`DCOUNTA` 4, `DGET` over one match 7, over several `#NUM!`; `DSUM(…,FALSE,…)` is `#VALUE!`).
Earlier probes over a text first column gave 0 / `#DIV/0!` only because text is skipped by the numeric functions (DGET's `#NUM!` there came from several matching records).
A legacy (non-dynamic-array) formula gets no implicit intersection on the `field` of a D-function: `=DSUM(A1:C6,G1:G2,E1:E2)` on a row of G1:G2 is `#VALUE!` (FORMULATEXT shows no `@`), while `G1:G2&""` is intersected (`@G1:G2&""`) and gives 19.
An empty field (`DCOUNT(A1:C6,,E1:E2)`) gives DCOUNT and DCOUNTA the count of
matching records (4), and `#VALUE!` in every other function.
A criteria given as an array literal (`{"Qty";">2"}`) lifts elementwise to `#VALUE!` per element, so it
spills (`#SPILL!` where blocked).
DGET with more than one matching record is `#NUM!`. A text cell and a boolean cell in the field column are
skipped by the numeric functions and counted by DCOUNTA.
These set the generator modes in eval/catalog/functions.yaml: `database` and `criteria` are `range`, and
`field` is `value`.
Evidence: probes/db-argument-modes (121 cells, excel-graph, 2026-10-11, T-1086).
The data is A1:C6, with headers Name, Qty and Price. Price holds 2.5, 4, "x", TRUE and 10. The criteria
are E1:E2 (Qty ">2") and F1:F2 (Name "e").
Ours: `field` (LiftArrays) and `criteria` (LiftArraysRangeWhole) lift through SPEC-0011's driver, `database`
is read whole, and DCOUNT/DCOUNTA with an empty field count the matching records (T-1389). A generated
sample also showed gaps in header matching and criteria (T-1390).
Lifted calls are bounded (directive 5): each lifted element is charged the database's rows times the
criteria headers and criteria cells it scans (`Model::db_element_cost`), and a call over
`MAX_LIFTED_CELL_VISITS` (2^27) is `#ERROR!` where Excel returns a value, e.g. `DSUM(A:Z, SEQUENCE(1,26), crit)`
over whole columns. A call that does not lift is not charged.
Status: VERIFIED (excel-graph); the lifted-call bound is DEVIATION (decision 0014)
Tasks: T-1086, T-1389, T-1390

## Database criteria
Rule: in every D-function a `database` or a `criteria` range of one row (headers only, `DSUM(A1:E6,2,H1)`,
`DSUM(A1:E6,2,H1:I1)`, `DSUM(A1:E1,2,H1:H2)`) is `#VALUE!`. A record matches when any criteria row (OR)
has every non-blank cell holding (AND); a criteria row that is blank under its headers therefore matches
every record (`H1:H3` with `H3` blank sums all records), as does a blank header over a blank condition.
- **Header matching.** A criteria header names the database column whose header has the same text,
  ignoring case and not trimming (`" Qty"` names nothing). Numbers and booleans read as their coerced text
  (numbers.md "Number → text conversion"): the number 0.5 and the text `"0.5"` name the number header
  0.5, `"TRUE"` names the boolean header TRUE, and the header 1E-15 is named by the number 1E-15 and by
  `"0.000000000000001"` but not by `"1E-15"`. A blank or error criteria header names no column, even
  when the database has a blank header. The same text rule resolves a text `field`, and there `""`
  names a blank database header (`DSUM(A1:E6,"",H1:H2)` sums the blank-headed column).
- **Computed criteria.** A condition under a header that names no column (blank, an error, or text that
  is no database header) is a computed criterion: it holds when its value is TRUE, a non-zero number
  (-1, 0.3 and 2 all hold) or the text `"TRUE"`, and fails for FALSE, 0, an error and any other text,
  `">2"` and `"1"` included. So an
  unknown header over `">2"` matches nothing, and over TRUE matches everything. A formula condition is
  evaluated again for each record with its relative references moved down to that record: a blank
  header over `=B2>2` (cached FALSE, the first record's Qty is 1) sums the records whose Qty is above 2.
  The references are rebased onto the first record wherever the criteria block sits: below or above the
  database, or on another sheet, `=B6>2` (first record on row 6) sums the records from the third on (12),
  `=B9>2` tests each record's B cell three rows down (sum 3), and the absolute `=$B$6>2` stays on the
  first record and matches nothing (0). VERIFIED (probes/db-criteria-formula-placement H1-H12, excel-graph;
  ours already agrees). The text `"true"`, `"True"` and `"TRUE"` all hold under a blank header (H13-H15).
- **DGET** of the one matching record's blank field cell is `#VALUE!`, as when nothing matches.
- **Field before shape** (probes/db-field-error-order, excel-graph 2026-10-27, T-2516). The `field` is resolved
  before the database or criteria shape is checked: an error `field` (`NA()`, `1/0`, a `VLOOKUP` that is
  `#N/A`) gives that error even when the criteria or the database is one row (`DSUM(A1:C4,NA(),G1)` and
  `DSUM(A1:C1,NA(),E1:E2)` are `#N/A`, `1/0` is `#DIV/0!`). A field that names no column (`"Nope"`) is
  `#VALUE!` with a good, a one-row-criteria and a one-row database; a field out of range (5) or 0 is `#VALUE!`
  with one-row criteria. `GAMMA.DIST(TRUE,42,1E-15,"FALSE")` is the number 0, so
  `DGET(db,<that>,<one-row criteria>)` is a field of 0: `#VALUE!`. Ours already agrees. The `#VALUE!` of gen-m9
  88cabddc4935351c A14 and c9d7659896d95368 A271 is the field expression's own error passing through (a
  `#N/A` VLOOKUP field gives `#N/A`, L23), so they confirm the order; 7bd219ec0c0c5e22 A324 is the gamma case.
- **Matched conditions** use the shared criteria grammar (criteria.md); `Qty` over TRUE matches only
  the boolean TRUE record, over `=NA()` nothing.
Evidence: probes/db-header-criteria-shapes (72 questions; database A1:E6 with headers Name, Qty, 0.5,
TRUE and a blank) and probes/db-computed-criteria-shapes (53 questions; headers Name, Qty, 1E-15,
123456789.123 and a blank), excel-graph, 2026-10-23, T-1390.
Ours: `db_criteria_columns`, `db_row_matches_criteria` and `db_computed_condition` in
functions/database.rs; a formula condition is evaluated in the context of the criteria cell moved
down by the record's offset. A formula condition that itself runs a D-function with formula
conditions would re-evaluate them without passing through a cell, its work growing as the records to
the power of the nesting, so only one level runs: a nested formula condition is `#ERROR!` and fails
(directive 5; DEVIATION, decision 0014). What Excel does with a nested formula condition is unprobed.
Status: VERIFIED (excel-graph); the one-level re-evaluation is DEVIATION (decision 0014), Excel's nested
behavior UNVERIFIED
Tasks: T-1390, T-2516

## GAMMA and GAMMALN sweep (T-1042)
Rule: Excel's GAMMA is exact only for small integers, GAMMALN is not correctly rounded at integers,
and the two functions have different domains.
- **GAMMA at integers**: `GAMMA(n)` equals the correctly rounded `(n-1)!` for n = 1..25 and differs
  from it in the last digits for 131 of the 171 integers 1..171, first at n = 26 (1.5511210043330984e+25
  against the correctly rounded 1.5511210043330986e+25). A table of factorials is wrong for n >= 26.
- **GAMMALN at integers** is not the correctly rounded `ln((n-1)!)`: `GAMMALN(3)` = 0.6931471805599454
  (ln 2 correctly rounded is 0.6931471805599453); 91 of the 171 integers differ from the correctly
  rounded value (the smallest are n = 3, 8, 9, 10, ...; 60-digit reference), `GAMMALN(1)` = `GAMMALN(2)` = 0.
  The engine's fdlibm kernel (`libm::lgamma`) misses 63 of these 171 cells; glibc 2.39 `lgamma` misses 61 (CPython `math.lgamma` 99).
- **Overflow edge**: `GAMMA(171.5)` = 9.483367566824801e+307 and `GAMMA(171.6)` = 1.5858969096672508e+308
  are finite; `GAMMA(171.7)`, `GAMMA(171.99)`, `GAMMA(172)` and `GAMMA(200)` are `#NUM!`.
  `GAMMALN` has no upper limit: `GAMMALN(200)` = 857.9336698258575, `GAMMALN(1E+300)` = 6.897755278982137e+302.
- **Poles**: `GAMMA(0)`, `GAMMA(-1)`, `GAMMA(-2)`, `GAMMA(-10)` are `#NUM!`; between poles GAMMA is
  defined (`GAMMA(-0.5)` = -3.5449077018110318, `GAMMA(-2.25)` = -1.742814865728253,
  `GAMMA(-9.9)` = 3.542684553080843e-06).
- **Near zero**: `GAMMA(1E-300)` = 9.999999999999999e+299 (not 1e300), `GAMMA(-1E-300)` =
  -9.999999999999999e+299, `GAMMA(1E-10)` = 9999999999.422787, `GAMMA(-1E-10)` = -10000000000.577215.
  `GAMMALN(1E-300)` = 690.7755278982137.
- **GAMMALN of any x <= 0 is `#NUM!`** (negative half-integers and non-integers, 0, negative integers,
  -1E-300), although GAMMA is defined at the negative non-integers.
- **Coercion**: `GAMMA("a")` and `GAMMALN("a")` are `#VALUE!`; `GAMMA(TRUE)` = 1, `GAMMA("3")` = 2,
  `GAMMALN(TRUE)` = 0, `GAMMALN("3")` = 0.6931471805599454.
- **GAMMA(n) for n <= 88 is the downward product in doubles** (T-1524, fit lab `gamma-int`):
  `(n-1)*(n-2)*...*2`, each product rounded to a double (an Excel `*` gives the same here), matches
  every n = 1..88 and 92 of the 171 integer cells. From n = 89 on Excel uses something else: it is
  up to 32 ulps from the downward product and 29 from the correctly rounded factorial (`GAMMA(167)`,
  t1042-gamma-int-91-171 Calc!A77; all but 3 of the 83 cells are within 11 of the product), and matches the
  product only at 4 of the 83 cells (n = 117, 120, 122, 124). `GAMMA(101)` = 9.332621544394407e+157:
  the product is 9 ulps away, the correctly rounded value 7. The upward product `2*3*...` gets 38, correctly
  rounded 40, libm `tgamma` 38, a 64-bit product (either order) 40, the engine 1.
- **GAMMALN at integers and GAMMA at half-integers are not explained** (T-1524): the natural log of the
  downward product (libm or 64-bit log) gets 82 of 171 GAMMALN cells (correctly rounded 80, libm
  `lgamma` 72, the engine's fdlibm 108). For `GAMMA(m+1/2)` (62 cells) the recurrence from sqrt(pi)
  gets at most 34 in doubles (upward; 30 starting from Excel's own `GAMMA(0.5)`, which is the
  correctly rounded sqrt(pi) 1.772453850905516), correctly rounded and any 64-bit recurrence 45, the engine 0.
  Excel's GAMMA does not keep 64-bit intermediates either: that model only ties the correctly
  rounded value (numbers.md "x87 precision is per operator, not per kernel").
Ours: GAMMA is Excel's own routine since T-2129 ("GAMMA is Excel's own x87 routine" below); GAMMALN is Excel's own routine since T-2128 ("GAMMALN is Excel's own x87 routine" below).
Evidence: probes-numeric cases `t1042-gamma-int-1-90`, `t1042-gamma-int-91-171`, `t1042-gamma-half-int`,
`t1042-gamma-unit-interval`, `t1042-gamma-negatives-edges` (first half of each case's cells is GAMMA,
second half GAMMALN, in formula order; GAMMA overflow edge: `t1042-gamma-negatives-edges` Calc!A18:A24;
GAMMALN of x <= 0: Calc!A28:A44 there and A63:A73 of `t1042-gamma-half-int`). Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-0973 (kernel, including the 171 < x < 171.62 overflow edge), T-1529 (GAMMALN and non-integer GAMMA, time-boxed)

## GAMMALN is Excel's own x87 routine (T-1941, T-2128)
Rule: GAMMALN and GAMMALN.PRECISE are one routine of x87 operations, each stored to a double (fl53(fl64(.))), with LN the
FYL2X model (numbers.md "LN is FLDLN2 then FYL2X"). By the exact comparisons Excel makes:
- **x < 0.7** (the double 0x1.6666666666666p-1): fl(GAMMALN(fl(x + 1)) - LN(x)), one step up (12,288 of 12,288 points, given
  the kernel above 0.7; tiny x is the same formula, since fl(1 + x) = 1 and GAMMALN(1) = 0).
- **[0.7, 1.5]**: K3 = fl(fl(x - 1) * fl(N/D)), a (9,9) raw-x rational.
- **(1.5, 2]**: fl(fl(x - 1) * fl(fl(x - 2) * fl(N/D))), its own (9,9) rational (D0 not a power of two).
- **(2, 3]**: K2 = fl(fl(x - 2) * fl(N/D)), raw-x (9,8).
- **(3, 8)**: t = x - k in [2, 3), p = t (t + 1) ... (t + k - 1) ascending with every product stored, and fl(K2(t) + LN(p)).
- **x >= 8**: Stirling's series with exactly ten Bernoulli terms, every op stored: a = x - 0.5, b = a LN(x), d = b - x,
  e = d + 0.9189385332046727, z = 1/x, z2 = z z, r = B_10, r = r z2 + B_k (k = 9..1), GAMMALN = e + r z.
- The ends are exact: GAMMALN(1.5) is K3's -0.12078223763524523 (the (1.5, 2] form gives ...452), GAMMALN(3) is K2's
  0.6931471805599454 (the reduction would give LN(2) = ...453), and GAMMALN(4) is the reduction's LN(6).
The four rationals use Excel's own coefficient doubles, which are not yet known to the bit (T-2122); the engine uses the
research's best sets (`functions/statistical/gamma_kernels.rs`). K2 near 2 takes reviewer 3's "cand9" set for x - 2 < 0.01 and
`k2_ref.py`'s (N's first three sums and D's first two in registers) above. That cut is measured: on the (2, 3] points cand9
scores 0.99 for x - 2 below 1e-5, against 0.945 for k2_ref, and k2_ref wins from x - 2 = 0.01 on; the total over (2, 3]
peaks at a cut of 0.01 (0.95295, against 0.94959 for k2_ref alone) and is flat from 5e-3 to 2e-2. It is a choice between
two models, not Excel's switch. Exact rates on the T-1941 GAMMALN.PRECISE cells (18,360,467 distinct x, excel-graph):

| Region | Exact | of |
|---|---|---|
| x < 0.7 | 0.869 | 203,520 |
| K3 [0.7, 1.5] | 0.322 | 8,082,879 |
| (1.5, 2] | 0.428 | 7,548,389 |
| K2 (2, 2.02) | 0.983 | 192,512 |
| K2 [2.02, 3] | 0.950 | 1,862,260 |
| (3, 8) | 0.991 | 178,258 |
| x >= 8 | 1.000 (292,645 of 292,649) | 292,649 |

The four x >= 8 misses are LN's FYL2X near-ties, where Excel's own LN column took the other neighbour in the same run, so
the series is bit-exact given Excel's LN. The misses elsewhere are a few ulps (at most 7, in K3, on the committed 10,500-point sample), never visible
at 15 digits; a correctly rounded lgamma (the fdlibm `lgamma` the engine used before) matches K2's data at 0.29-0.35 (HANDOFF2 §3).
Below 1 the K3 and (1.5, 2] gaps reach the beta density (T-2513). On probe t2513-r3-m1-residual-gl (Excel's GAMMALN at
569 shape arguments) the engine model differs from Excel on 14 of 47 arguments below 1 (0.71: -2 ulps, 0.77: +2, 0.99:
+2); the verifier, with its hold-out's 54 more, counts 24 of 66 below 1 and 19 of 541 from 1 on (verify-r3-m1-residual
`vchecks.out`). With the engine model in place of Excel's values, the M2 density's GL branch on probe a drops from 1,648
to 1,410 bit-exact of 1,648, so density research uses measured GAMMALN until T-2122 lands.
Evidence: `docs/research/t1941/` and the `kmyst-gam-*` cases on branch `agent/research/T-1941-crack-gammaln-gamma-ln` (HANDOFF2.md
§2, §3, §12; `gammaln_big_check.c`, `k2_ref.py`, `review3/final/cand9.py`, `w5/p4/ss_fc.txt`, `w6/data/xl_chain_d3ls.txt`);
ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx (desktop cache) now passes all 91 cells, and probes/stat-argument-modes
two more.
Implemented in `functions/statistical/gammaln.rs` (`excel_gammaln`), `gamma_kernels.rs` and `x87_register.rs` (K2's register
head). `test/fixtures/gammaln_kernel.fixture` (1,500 random points per region) pins the engine to the research checkers on
every point; `gammaln_matches_the_t1941_runs` reruns the table above from `RUSTSHEET_T1941_ROWS`.
Status: VERIFIED (excel-graph and desktop) for the structure and the bit-exact pieces; the four rationals are best known
models (T-2122)
Tasks: T-1941, T-2128, T-2122, T-2513

## GAMMA is Excel's own x87 routine (T-1941, T-2129)
Rule: GAMMA is its own routine (not EXP(GAMMALN)), x87 ops each stored to a double (fl53(fl64(.))), LN the FYL2X model,
EXP model V6 (numbers.md), and the reflection's sine an FSIN, RN53(RN64(sin)) of the true sine. By the exact
comparisons Excel makes, in order:
- **Integers n <= 88**: the downward product (n-1)(n-2)...2 (T-1524); 0 and the negative integers are `#NUM!`.
- **Exact half-integers** (x = floor(x) + 0.5): the ascending product 0.5 * 1.5 * ... * (x - 1) times √π
  (0x1.c5bf891b4ef6bp+0); a negative one is ±fl(fl(π/Gh(-x))/(-x)). At -171.5 that is a subnormal, which Excel gives
  as +0 (`kmyst-gam-h102`); from -172.5 down Gh(q) overflows and the result is 0.
- **x >= 10** (integers from 89 on included): h = x/2, r = x^floor(h) by right-to-left binary powering, v = r EXP((h -
  floor(h)) LN(x)) = x^(x/2), the Stirling series s in 1/x (the same ten-term sum as GAMMALN's), and EXP(-x) √(1/x) v v
  2.50662827463100050242 EXP(s), multiplied in that order. Because v is used twice, GAMMA near 150 is up to about ±40 ulp
  from the true value.
- **1 <= x < 10**: the descending stored product (x-1)(x-2)... down to y in [1, 2), times G(y).
- **0 < x < 1**: fl(fl(1/x) G(fl(x + 1))).
- **-10 <= x < 0**: z = 1/x, y = x + 1, then z = z/y and y = y + 1 while y < 1, and z G(y).
- **x < -10**: q = -x, z = q - round-half-down(q), s = |FSIN(fl(π z))|, ±fl(fl(π/GAMMA(q))/fl(q s)); 0 once GAMMA(q)
  overflows: GAMMA(-172.0000000001), GAMMA(-171.55) and GAMMA(-200.3) are 0 while GAMMA(-171.0000000001) is
  8.058901809475927e-300 (probes/gamma-x87-edges). The cut is exactly -x > 10 (`kmyst-gam-h104`).
- The last finite argument is 171.6243769563027 (GAMMA = 1.7976931348622233e308); the next double up is `#NUM!`
  (probes/gamma-x87-edges, the three doubles either side; the model gives the same edge).
G on [1, 2) is a (7,7) raw-y rational fl(N/D) with D0 = -1/16 and Excel's own coefficients, not yet known to the bit
(T-2122); the engine uses W4's best set (`gamma_kernels.rs` `K1_N`, `K1_D`). Exact rates on the T-1941 GAMMA cells
(4,680,197 distinct x, excel-graph):

| Region | Exact | of |
|---|---|---|
| integers, half-integers | 1.000 | 330 |
| x >= 10 | 0.998 | 138,552 |
| [1, 10) (through G) | 0.784 | 4,229,812 |
| (0, 1) (through G) | 0.838 | 280,575 |
| [-10, 0) (through G) | 0.876 | 24,620 |
| x < -10 | 0.999 (6,304 of 6,308) | 6,308 |

Of the four reflection misses, three inherit 1-3 ulp from the engine's GAMMA(q) (Excel's own GAMMA(-x) column in
`kmyst-gam-h103` differs there by as much), and the fourth, x = -0x1.94efe288f63adp+5, is the FSIN tie the research found:
RN64(sin) lands on a 53-bit midpoint. The [1.8, 2) part of G is the weakest (about 0.5, HANDOFF2 §3.3). The misses through G
are a few ulps, invisible at 15 digits.
Evidence: `docs/research/t1941/` and the `kmyst-gam-*` cases on branch `agent/research/T-1941-crack-gammaln-gamma-ln`
(README "GAMMA for x >= 10", HANDOFF2 §2, §3.3; `gamma_big_check.c`, `gamma_neg_check.py`, `w4/k1r/plain1.txt`);
probes/gamma-x87-edges (excel-graph, 2026-10-20: the overflow edge and the reflection past it, 15 of 15);
ironcalc-calc/STATISTICAL/GAMMA.xlsx passes every GAMMA cell (13 known failures gone; what is left there is GAMMA.DIST and
GAMMA.INV), BETA_GAMMA.xlsx 8 more and probes/stat-argument-modes one more.
Implemented in `functions/statistical/gamma_x87.rs` (`excel_gamma`), `gamma_kernels.rs` (K1) and `trig.rs` (`excel_fsin`).
`test/fixtures/gamma_kernel.fixture` (600 random points per region plus every integer and half-integer) pins the engine to
the composed research checkers (`docs/research/t1941/engine/garef.py`) on every point; `gamma_matches_the_t1941_runs` reruns
the table from `RUSTSHEET_T1941_ROWS`.
Status: VERIFIED (excel-graph and desktop) for the structure and the products, half-integers, x >= 10 and reflection; G is a
best known model (T-2122)
Tasks: T-1941, T-2129, T-2122

## Student t sweep: df truncation, df limits, T.INV.2T above 1 (T-1043)
Oracle sweep of the Student t family over df in {1,2,3,4,5,10,30,100,1000,1E6}, x up to 1E6 and p
down to 1E-10 / up to 0.9999999999. The values are the targets for T-0969 and T-0967.
- **df is truncated, not rounded**: for df in [1,2) every function gives the df = 1 value
  (`T.DIST(2,1.999,TRUE)` = `T.DIST(2,1,TRUE)` = 0.8524163823495667), and likewise 2.9 -> 2.
- **df < 1 is `#NUM!`** for `T.DIST(...,TRUE)`, `T.DIST.RT`, `T.DIST.2T`, `T.INV`, `T.INV.2T` (0.5, 0.999, 0),
  but `T.DIST(x,df,FALSE)` (the pdf) is `#DIV/0!` for df in (-1,1) (so df is truncated toward zero first:
  `T.DIST(2,-0.5,FALSE)` is `#DIV/0!`) and `#NUM!` for df <= -1 (-1, -5). `T.DIST(2,0,TRUE)`,
  `T.DIST.RT(2,0)` and `T.DIST(2,0.5,TRUE)` are `#NUM!`.
- **Upper df limit**: df = 1E10 and 9999999999 work everywhere; df >= 1.5E10 makes `T.DIST(...,TRUE)`,
  `T.DIST.RT`, `T.DIST.2T`, `T.INV` and `T.INV.2T` `#NUM!` (each probed at 1.5E10 and 2E10; `T.DIST.2T(2,1E10)` and
  `T.INV.2T(0.1,1E10)` still give values) while the pdf stays finite
  (`T.DIST(2,1E11,FALSE)` = 0.05399096651413289). The cut is exactly 1E10 on the truncated df: `T.INV(0.9,10000000001)` and `T.INV(0.9,1.0000001E10)` are `#NUM!`,
  `T.INV(0.9,10000000000.5)` (truncates to 1E10) is 1.2815518424115688; `CONFIDENCE.T(0.05,1,n)` is a number for
  n = 10000000001 (df 1E10) and `#NUM!` from n = 10000000002 (probes-numeric case `t1518-t-df-cut`, excel-graph,
  2026-10-14). The pdf `T.DIST(2,1.4E10,FALSE)` stays finite (0.05399096651993693).
  Excel's `T.INV(0.9,1E10)` = 1.2815518424115688 is not the exact quantile (1.28155156554...; the
  normal quantile plus the 1/df correction): Excel is off by about 3E-7 there, so the large-df values need a fit
  (T-0967). Our implementation above df = 1E5 uses the normal quantile with the A&S 26.7.5 series to stay bounded in time.
- **T.INV.2T does not reject p in (1,2)**: `T.INV.2T(1.5,5)` = -0.7266868438004216,
  `T.INV.2T(1.0001,5)` = -0.0001317152766640225, `T.INV.2T(1.999,5)` = -6.868826625881273 (negative
  quantile: `T.INV.2T(p,df)` = `T.INV(1 - p/2, df)`, e.g. `T.INV.2T(1.5,10)` = `T.INV(0.25,10)` =
  -0.6998120613124317 and `T.INV.2T(1.9,5)` = `T.INV(0.05,5)` = -2.0150483733330233, equal to the last
  digit in those cells); `T.INV.2T(1,5)` = 0, while p = 0, p >= 2 and
  p < 0 are `#NUM!`. `T.INV(1.5,5)`, `T.INV(0,5)` and `T.INV(1,5)` are `#NUM!` as usual.
- `T.DIST.2T` and `T.DIST.RT`: negative x is `#NUM!` for `T.DIST.2T` but valid for `T.DIST.RT`
  (`T.DIST.RT(-2,1)` = 0.8524163823495667). `CONFIDENCE.T(0.05,1,1)` is `#DIV/0!` (n = 1), sd = 0 and
  alpha = 0, 1, -0.1 or 1.5 are `#NUM!`, n is truncated (`CONFIDENCE.T(0.05,1,10.9)` = 0.7153569059706648;
  n = 2.5 acts as n = 2).
- Values are not exactly symmetric: `T.INV(0.1,1)` = -3.077683537175254 but `T.INV(0.9,1)` =
  3.0776835371752544; `T.DIST.2T(1,1)` = 0.5000000000000001 and `T.DIST.RT(1,1)` = 0.25000000000000006.
- The cumulative T.DIST is the exact CDF at every df probed, including 1E6: `T.DIST(1,1E6,TRUE)` =
  0.8413446250832111 and `T.DIST(-2,1E6,TRUE)` = 0.022750266925659606 are each 1 ulp from the exact value
  (mpmath, 60 digits), and the exact CDF rounded is bit-exact on 127 of the 160 cumulative T.DIST cells in m9-t-dist-cdf,
  m9-t-nonint-df and m9-t-edges (118 of m9-t-dist-cdf's 150), 155 within 4 ulp. An earlier note here (1e-10 relative off, 152,191 ulp) measured the fit lab's mapping, which rounds
  y = n/(n+t^2) before I_y (T-2088), not Excel. Hill's integer-df series (AS 3) is 74/150 exact (T-0969).
- The upper tail is computed directly, neither as 1 - T.DIST nor as 0.5 minus the rounded centre
  I_z(1/2,n/2)/2, and `T.DIST.2T` is exactly 2 * `T.DIST.RT`. It carries algorithm noise around the exact
  value: `T.DIST.RT(1.96,60)` = 0.02732246486826467 is 13 ulp below it and `T.DIST.RT(1E6,10)` =
  1.230468749943611e-56 33 ulp above it. In the deep tail (t^2 >= n) the noise is that of
  exp(a ln y), y = n/(n+t^2): Cephes' power series with x^a = exp(a ln y) is within 4 ulp on 131 of 135
  probe cells, the correctly rounded I at the double y on 107 (worst 269 ulp at `T.DIST.RT(1E6,30)`, 275 from the exact
  value) (T-0969 attempt 2).
- **Large df stays exact; the noise is Excel's own** (T-0969 attempt 4). Up to df 1E10 the tails at
  |t| up to about 5 are within a few ulp of the exact value (`T.DIST.RT(0.1,1E6)` = 0.46017217274602157,
  `T.DIST(2,1E10,TRUE)` = 0.977249868038323), and so is the density to df 1E11. Any model that rounds
  y = n/(n+t^2) first misses them by about n eps (1E-11 relative at 1E6). Further out Excel drifts from the exact
  value by roughly |E| eps, where E is the exponent of the density or tail (about t^2/2, or ((n+1)/2) ln(1+t^2/n)).
  That is the signature of exp of a rounded double exponent:
  - `T.DIST(30,1E6,FALSE)` = 1.8033900017556897E-196 is 200 ulp below exact.
  - `T.DIST(1000,100,FALSE)` is 165 ulp off and `T.DIST(100,30,FALSE)` 38 ulp off.
  - `T.DIST.RT(30,1E6)` = 6.0100471168320144E-198 is 398 ulp above exact.

  In doubles, ratio / sqrt(n) * exp(-((n+1)/2) log1p(t^2/n)) carries the same kind of |E| eps error, with its own
  sign. It is closer to Excel than the exact density at df 1000 and 1E6 for |t| up to about 10, and at
  `T.DIST(30,1000,FALSE)` (2 ulp against 94). It is further away at `T.DIST(30,1E6,FALSE)` (348 against 200) and
  below df 1000. No tried double formula (pow,
  exp of ln, exp of log1p, ln Gamma differences) reproduces Excel's density noise at df <= 100.
- **The density at df 2 is `EXP(-1.5*LN(2+t^2))`** (T-0969 attempt 6): bit-exact on all 13 df-2 density cells
  of m9-t-dist-pdf, m9-t-nonint-df and m9-t-edges, including `T.DIST(1E6,2,FALSE)` = Excel's value 15 ulp from the
  exact one; `POWER(2+t^2,-1.5)` matches 9, the 1 + t^2/2 form 11. From df 3 on no single exp of one double exponent
  of the forms exp(C - h ln(n+t^2)) or exp(C - h ln(1+t^2/n)), h = (n+1)/2, reproduces a df's cells for any double C,
  though Excel's value is always within about one ulp of the exact exponent's exp. Excel's density and tails are
  up to 482 ulp from the exact value (`T.DIST(1E6,30,FALSE)`, `T.DIST.RT(30,1E6)` 398), so an exact kernel cannot match them.
- **The centre tail at small df can follow the rounded y** (T-0969 attempt 6): for t^2 < n, I_y(n/2,1/2)/2 at
  y = fl(n/(n+t^2)), as T.TEST computes it, is bit-exact at `T.DIST.2T(0.1,2)`, (2,100) and (3,100) where the
  exact tail is 8 to 23 ulp off, but 29 and 97 ulp off at `T.DIST.RT(0.5,100)` and (0.5,1000), where the exact
  tail is within 1. No (df, t) cut on the swept cells separates the two (superseded: the denser grid of T-0969 attempt 7
  finds the cut, see "T.DIST.RT follows the rounded y beyond t = 1").
Evidence: probes-numeric cases `m9-t-dist-cdf`, `m9-t-dist-pdf`, `m9-t-dist-rt`, `m9-t-dist-2t`, `m9-t-inv`,
`m9-t-inv-2t`, `m9-confidence-t`, `m9-t-nonint-df`, `m9-t-edges`. Oracle: excel-graph, 2026-10-13. Exact reference values for
the T.DIST accuracy bullets: mpmath `betainc` at 60 digits (T-0969 attempt 2, 2026-10-20).
Status: VERIFIED (excel-graph)
Tasks: T-0969 (T.DIST family), T-0967 (T.INV, T.INV.2T, CONFIDENCE.T)

## T.TEST rounds y = df/(df+t^2) to a double before the tail (T-0969)
Rule: T.TEST (and TTEST) is `I_y(df/2, 1/2)` x tails/2 at y = df/(df + t^2) rounded to a double, not the Student-t
tail at its own t, which is what T.DIST.RT and T.DIST.2T compute. The rounded y moves the p-value by up to tens of
ulp where y is near 1 (small t). At `T.TEST(G1:G10,H1:H10,2,2)` (df 17, t = 0.19827206571599848), Excel's
0.8451838964108318 is exactly I_y at the rounded y = 0.9976928755998522. The exact two tails are 42 ulp away, both at
the double t and at the exact t, so the gap is not in the statistic (probes-numeric t1048-ttest B31).
- t^2 is the square of the double statistic for types 2 and 3. For type 1 (paired) it is md^2 / (vd / n): bit-exact
  on 7 of the 11 paired cells, and every one within 4 ulp. md^2 n / vd and (md / (sd / sqrt n))^2 are each worse on
  at least one cell.
- With exact I at the rounded y (exact statistic), 32 of the 43 T.TEST cells in t1048/t1049-ttest are within 2 ulp
  of Excel, against 18 for the exact tail at t. The rest are 3 to 6 ulp off: Excel's incomplete beta has its own noise, as in the T.DIST sweep.
- Welch's df is fractional. There `I_y` needs an accurate Gamma ratio: libm's tgamma ratio is 9 ulp off at
  `T.TEST(D1:D10,F1:F10,2,3)` (B18), and Cephes' plain fraction is 111 ulp off at `T.TEST(D1:D10,F1:F11,2,3)` (B27).
Evidence: probes-numeric `t1048-ttest`, `t1049-ttest`; ironcalc-calc `STATISTICAL/T_TEST.xlsx`. Exact values: mpmath
`betainc` at 50 digits (T-0969 attempt 5, 2026-10-20).
Ours: `student_t.rs` `student_t_tail_at` since T-0969 attempt 5. T.TEST/TTEST visibly-off cells went from 19 to 3:
t1048-ttest A3/B3 (6 ulp) and t1049-ttest A48 (5 ulp), all Welch.
Status: VERIFIED (excel-graph)
Tasks: T-0969

## T.DIST.RT follows the rounded y beyond t = 1 (T-0969)
Rule: away from the deep tail, T.DIST.RT (and so T.DIST, T.DIST.2T and TDIST) takes the tail at
y = n/(n + t^2) as the worksheet rounds it, fl(n / fl(n + fl(t*t))), as T.TEST does, once t > 1 and
y <= 0.99. For t up to 1, or y above 0.99 (t^2 under about n/99, so from df 100 on the switch moves out to
t = sqrt(n/99)), it is the exact tail at t.
- Evidence: probes-numeric `t0969-t-tail-grid` (excel-graph, 2026-10-20): T.DIST.RT on t = 0.0013 + 0.05k for
  df 2 to 1000, and runs of 512 consecutive doubles at (df, t) = (100, 2.0123456789), (30, 3.0123456789),
  (3, 1.0123456789).
- On the runs, Excel's relative error against the exact tail regresses on the rounding errors of the
  intermediates with coefficient -1 on fl(n + t^2) and +1 on fl(n / that), in units of d ln L / d ln y (df 100;
  1.2 and -1.26 at df 30). That is the signature of I_y at that doubly rounded y. Excel's own incomplete-beta noise
  (about 2.5 x 2^-53 relative rms) remains on top.
- The switch shows on the grid as the side where the two models differ by 6 ulp or more:
  - for df 20 to 100, the exact tail wins up to t = 1 and the rounded y from about 1.05;
  - for df 200, 500 and 1000, the exact tail wins up to t = 1.40, 2.20 and 3.10, and the rounded y from 1.45, 2.25
    and 3.20.
  These bracket both y = 0.99 and y = 100/101; the cut is not pinned to the ulp.
- Our model with that switch is bit-exact on 548 of the grid's 3,232 cells and within 4 ulp on 2,210, against 325
  and 1,229 for the exact tail everywhere.
- df 15 is an outlier: Excel is 6 to 27 ulp off both models on almost every cell, with one sign beyond t = 1 and
  the other inside it. That is the pattern of a prefactor (a Gamma ratio at a = 7.5) about 1.5E-15 relative
  off. It is not modelled. probes-numeric `t0969-t-df-ratio` (T-0969 attempt 8) confirms it as a prefactor:
  `T.DIST.RT(0.5,15)`, `T.DIST.RT(1.5,15)` and `T.DIST.RT(2,15)` are all off by the same -20 x 2^-53 relative
  (on the centre's A for t = 0.5, on L beyond t = 1), while the df-15 densities `T.DIST(0.5|2,15,FALSE)` are within
  0.3 x 2^-53 of exact. See "Student t prefactor per df" below.
- df 2 is the exception: see "T.DIST.RT at df 2 is BETA.DIST's closed form down to y = 0.1".
Ours: `student_t.rs` `unflushed_upper_tail` (since T-0969 attempt 7).
Status: VERIFIED (excel-graph) for df 2 to 1000; above df 1000, and the cut's exact position, UNVERIFIED
Tasks: T-0969

## The a = 1 closed form cancels only from x = 0.1 up (T-0974)
Rule: I_x(1,b) is the cancelling 1 - exp(b*ln(1-x)) from x = 0.1 up (BETA.DIST(0.1,1,0.0001,TRUE) above) and keeps
its digits below, -expm1(b*ln1p(-x)) as far as the cells tell, the same cut as T.DIST.RT at df 2 (below).
- At BETA.INV's x (column D of probes-numeric/t1745-beta-inv-roundtrip), `BETA.DIST(1.9999999998999996E-10,1,0.5,TRUE)`
  is 9.999999999999999E-11 (G35), and `BETA.DIST(4.000000000119999E-11,1,2.5,TRUE)` and
  `BETA.DIST(1.0000000000449999E-11,1,10,TRUE)` are within an ulp of 1E-10 (G48, G55), where 1 - exp misses by 8E-8
  relative; `BETA.DIST(0.0004001200640416299,1,2.5,TRUE)` (G49, about 0.001) is 112 ulp from 1 - exp.
- The quantiles follow: `BETA.INV(1E-10,1,0.5)` = 1.9999999999E-10 (m9-beta-inv A36) and `F.INV.RT(1-1E-10,2,7)` =
  1.0000000828046573E-10 (gen-m9/4e55500bb8ad3cd8 A77), which a search on 1 - exp misses by 1E-6 relative.
- The F family follows through the a = 1 incomplete beta of F.DIST.RT's tail at df2 = 2 (a = df2/2): with the cut in
  `beta_reg_complement`, gen-m9 05d44f6dd6cfc567 A356, 146b46e74f2dac3b A109 (F.TEST), 27601819397dd56e A56
  (`FINV(1E-10,6,2)`) and 2801c1c11c21e134 A130 (`FDIST(1E+10,4,2)`) pass, which failed with 1 - exp below x = 0.1.
- At df 2, half the uncancelled tail is Excel's `T.DIST.RT(4.245,2)` = 0.025631992844658943 (t0969-t-df2-cut
  B450) bit for bit, 1 ulp from the closed form.
Ours: `incomplete_beta.rs` `shape_one_tails` (oriented, x0 < 0.1 there) for BETA.DIST's cdf, BETA.INV and the F tails,
and `fisher.rs` `unoriented_shape_one_tail` with the cut `A1_EXPM1_MAX_X` for F.TEST and the F quantiles' search (T-2977).
Status: VERIFIED (excel-graph) at x from 1E-11 to 4E-4 below the cut and at 0.1; the cut itself between 4E-4 and 0.1
is UNVERIFIED (assumed T.DIST.RT's).
Tasks: T-0974, T-2119

## F.INV is a crossing of F.DIST's tails (T-0974)
Rule: as BETA.INV (T-1745), F.INV and F.INV.RT/FINV return a double where Excel's own cdf crosses p. The smaller tail is
solved: F.INV at p > 0.5 crosses F.DIST.RT = 1 - p (exact there), and F.INV.RT at a small p crosses F.DIST.RT = p
directly, not F.INV(1 - p) (`F.INV.RT(1E-16,9,3)` = 60647458731.09812, gen-m9/0a9226016e8f5f33 A45, keeps digits 1 - p
would lose). Refining statrs' Newton estimate to the crossing of the engine's incomplete beta takes the F cells visibly off
(more than half a unit in the 15th digit) from 422 to 229 and the T-0974 sweep cells from 51 to 12 of 96, with none newly
visibly off. Below p = 1E-15 the deep tail of our cdf carries exp(a ln y)'s rounding, and its crossing is visibly off
Excel's `F.INV(1E-20,5,5)` = 5.081599921802649E-09 (probes/stat-quantile-lift E12), which statrs' estimate matches, so
statrs' value stays there; a deep upper tail is refined all the same, since statrs solved the rounded 1 - p there
(`F.INV.RT(1E-16,9,3)` was 7% off; `FINV(1E-16,1,7)` = 183379.00311608665, gen-m9/146b46e74f2dac3b A117, is now 15
digits right). Where statrs has no estimate (below p = 1E-20) the bisection of the cdf gives Excel's
`F.INV(1E-300,5,5)` = 5.081599884913164E-121 to 13 digits (E1), which was #NUM!, but not at every df:
`F.INV(1E-50,6000,4)` is 0.0163 against Excel's 0.000492 (E6), so the deep tail of our cdf is not Excel's there (T-1871).
At df1 near 1E5 the crossing was far off only because our forward tail lost digits: the large-shape prefactor took
ln x of a rounded x near 1. With ln(1 - y) from the exact complement (T-2465), `F.INV.RT(0.05,99999,9)` is the true root
2.7067247920355326, 5 ulp from Excel's 2.7067247920355353 (gen-m9/27601819397dd56e A81, 48 units of the 15th digit before),
and `F.INV.RT(0.001,99999,1)` 4 ulp from the true 636616.25590812706 where Excel is 6 ulp above it (c18dbf21f11093f3
A190, 345 units before). There Excel is the less accurate side, so its last digit is a kernel detail (T-2500).
Status: VERIFIED as a crossing to 15 digits on the cells above except A81 and A190 (within about a unit) and E6 (T-2500);
the solver's start, stop and tie rule are UNVERIFIED.
Tasks: T-0974, T-2465, T-2500

## T.DIST.RT at df 2 is BETA.DIST's closed form down to y = 0.1 (T-0969)
Rule: at df 2 (after truncation), for y = 2/(2 + t^2) as the worksheet rounds it, fl(2 / fl(2 + fl(t*t))), from 0.1 up,
the tail P(T > |t|) is half of BETA.DIST's a = 1 closed form, (1 - exp(ln(1 - y)/2))/2 (see "Incomplete beta:
Excel's closed forms"). That holds inside t = 1 as well, where every other df takes the exact tail at t. T.DIST,
T.DIST.2T, TDIST and T.TEST follow from the same tail. For T.TEST, two df-2 cells became bit-exact with it: probes-numeric
t1049-ttest A44, `T.TEST({1,2,3},{2,4,7},2,1)`, and A48, `T.TEST({1,1,1},{2,3,4},2,3)`, whose Welch df is exactly 2.
- Evidence: probes-numeric `t0969-t-df2-cut` (excel-graph, 2026-10-21), `T.DIST.RT(t,2)` for t = 2 + 0.005k: the
  closed form is bit-exact on all 449 cells from t = 2 to 4.24 (y = 0.100112) and on 12 of the 352 from t = 4.245
  (y = 0.099900) to 6. So the cut is y = 0.1 (or z = 0.9), to within those two cells.
- It is also bit-exact on 41 of the 43 df-2 cells of `t0969-t-tail-grid` (t = 0.0013 to 2.1013; B1 and B3 are 1 ulp off), on
  `T.DIST.2T(0.1,2)`, `T.DIST.RT(0.1,2)`, `T.DIST(0.1,2,TRUE)` and the t = 1 and t = 3 cells of `m9-t-dist-*`,
  and on `T_DIST.xlsx` F16 and C16. The exact tail is up to 56 ulp off there: `T.DIST.RT(0.0013,2)` is
  0.4995403807864147 in Excel, 1 ulp from the closed form and 56 ulp from the exact tail.
- Below y = 0.1 Excel is accurate (`T.DIST.RT(1E6,2)` = 4.999999999992501E-13, which the closed form misses by
  1E11 ulp) and not any simple form: on those 352 cells plus the six `m9-t-dist-rt` cells from t = 5 to 1E6,
  -expm1(log1p(-y)/2)/2 is bit-exact on 208 of 358, y/(1 + sqrt(1 - y))/2 on 175, our Cephes power series on
  111, and -expm1 of ln(1 - y) or of ln z on about 40.
- Far tails at df 1 and 2 are within an ulp of exact (gap triage 2026-10-30, mpmath at 60-80 digits on the exact t,
  excel-graph values). The examples are gen-m9/46ac3ef5e61583ca A236 `T.DIST.2T(1E15,1)` = 6.366197723675813E-16 (0
  ulp; ours +9), gen-m9/11eb3ab4f4f8104c A174 `T.DIST.2T(1E15,2)` = 9.999999999999999E-31 (-1; ours +24),
  gen-m9/bbab6562e88b70e8 A181 `T.DIST.2T(123456780.723,2.5)` = 6.561000999157746E-17 (0; ours +19), and
  `T.DIST.RT(100,1)` (m9-t-dist-rt A10, 0; ours +5). Every failing df-1 or df-2 cell with literal arguments in the
  report (49) has Excel within 1 ulp of the exact tail. Task: T-3187.
Ours: `student_t.rs` `closed_form_tail` (since T-0969 attempt 8), the kernel's oriented closed form with the
y0 <= 0.9 cut since T-2502; below it the general tail.
Status: VERIFIED (excel-graph) from y = 0.100112 up and below y = 0.099900. The exact cut is UNVERIFIED: whether y = 0.1
itself takes the closed form, and whether Excel tests y or z = 1 - y, need a dense probe near t = sqrt(18).
T-2464 places the cut. It is the kernel's a0 = 1 test y0 <= 0.9 on the oriented arguments, and BETA.DIST at
x0 = fl(1 - 0.9) takes the closed form. See "Incomplete beta closed forms act on the oriented arguments".
Tasks: T-0969, T-2502, T-3187

## T.DIST.RT is BETA.DIST's incomplete beta, switched at the mean (T-0969)
Rule: Excel's Student-t tail is its BETA.DIST kernel. With the worksheet-rounded y = fl(n / fl(n + fl(t*t))) and
z = fl(fl(t*t) / fl(n + fl(t*t))):
- beyond t = 1 (and y <= 0.99), `T.DIST.RT(t,n)` = `BETA.DIST(y,n/2,0.5,TRUE)/2`, to the bit;
- up to t = 1, `T.DIST.RT(t,n)` = `0.5 - BETA.DIST(z,0.5,n/2,TRUE)/2`, to the bit, with z formed from t, not as 1 - y
  (`BETA.DIST(1-y,0.5,n/2,TRUE)` is up to 197 ulp away at df 1000).
t = 1 is y = n/(n+1), the mean of Beta(n/2, 1/2). A reading, not observed (algorithm UNVERIFIED): the switch of
"T.DIST.RT follows the rounded y beyond t = 1" (pinned by that entry's grid, not by this probe, which samples t = 0.5
and t >= 1.5) is an incomplete beta's usual side choice at the mean, as Cephes `incbet` swaps to the complement when
x > a/(a+b), though Excel's centre takes z from t rather than as 1 - y. Either way, what is left of
the family's noise beyond t = 1 is Excel's BETA.DIST noise, df 15 included, and an accurate Student-t kernel cannot
remove it; only Excel's incomplete-beta algorithm can (T-1744, T-1759).
- Evidence: probes-numeric `t0969-t-beta-link` (excel-graph, 2026-10-21), 184 (t, df) pairs, t in {0.5, 1.5, 2, 2.5,
  3, 4, 5, 8} by df in {1 to 10, 12, 13, 15, 16, 17, 20, 25, 30, 50, 100, 200, 500, 1000}, with the worksheet's y and
  z in cells D and G:
  - t > 1 and y <= 0.99: T.DIST.RT = E/2 on 136 of 155 cells. The other 19 are 1 to 4 ulp off, all at df 7 to 30
    and t = 1.5 to 3, 10 of them at t = 1.5 (nearest the mean).
  - t <= 1: T.DIST.RT = 0.5 - H/2 on 22 of 23 cells; the other is df 2 (1 ulp), where the closed form of "T.DIST.RT at
    df 2" applies.
  - t > 1 and y > 0.99 (df 500 and 1000, t = 1.5 to 3; 6 cells): the two at t = 1.5 take the centre form
    0.5 - H/2 to the bit, as attempt 7's y > 0.99 rule says; the other four are 8 to 155 ulp from both forms. Not
    modelled.
  - `T.DIST(-t,n,TRUE)` = `T.DIST.RT(t,n)` on all 184.
- The density is not tied to BETA.DIST's: `BETA.DIST(y,n/2,0.5,FALSE) * t n/(n+t^2)^2` and four regroupings of it are
  bit-exact on at most 20 of the 184 `T.DIST(t,n,FALSE)` cells.
- Ours: `student_t.rs` is a separate, more accurate kernel; our BETA.DIST cdf is still statrs (T-2119) and is visibly
  off on 142 of the probe's 184 `BETA.DIST(y,n/2,0.5,TRUE)` cells, where our T.DIST.RT is visibly off on 15.
- T-2464 explains the switch. It is TOMS 708 bratio's dispatch on one call K(n/2, 1/2, y, z), not a switch at the mean,
  and the y > 0.99 cells are bgrat's ln x switch at z = 0.01. See "Student t tails are one kernel call".
Status: VERIFIED (excel-graph) for df 1 to 1000 at those t; region y > 0.99 beyond t = 1 UNVERIFIED
Tasks: T-0969, T-2119, T-2509

## Student t prefactor per df (T-0969)
Rule: Excel's tails carry a relative error that is fixed for a df and the same at every t, the signature of a
prefactor (the ratio Gamma((n+1)/2)/(Gamma(n/2) sqrt(pi))) that is not correctly rounded. It is large at df 15 and
small (a few x 2^-53) elsewhere. Its algorithm is unknown.
- Evidence: probes-numeric `t0969-t-df-ratio` (excel-graph, 2026-10-21), sheet Calc: for df 1 to 200 and 250 to
  1000 by 50, `T.DIST.RT(0.5,df)`, `T.DIST.RT(1.5,df)`, `T.DIST.RT(2,df)` and the densities at 0.5 and 2. Errors are
  relative to the exact model (the exact tail at t = 0.5, read on the centre A = 1 - 2L; the exact tail at the rounded
  y beyond t = 1), in units of 2^-53:
  - df 15: -19.9, -20.3 and -20.6 on the three tails. The densities are +0.3 and -0.2.
  - At t = 0.5 and 2 the errors agree within 0.5 at df 7 (-4.9, -5.2), 13 (+4.4, +4.8), 16 (+6.8, +7.1), 26 (+4.6,
    +4.9) and 147 (+4.6, +4.8), though t = 1.5 does not always follow (df 16: +1.6). Every other df is within about
    +-6, with no common sign.
- Not the prefactor: ln Gamma differences with Excel's own GAMMALN (`gammaln::excel_gammaln`). exp(GAMMALN((n+1)/2) -
  GAMMALN(n/2) - GAMMALN(1/2)), in either order, does give -18 to -22 at df 15, +5 to +9 at df 13 and +3 to +6 at df
  16. It then also predicts +26 at df 17, -24 at df 18, -32 at df 20, +55 at df 28 and +215 at df 147, where Excel's
  error is under 4 in magnitude. Excel's GAMMA (`gamma_x87::excel_gamma`) ratios are within 2 at every df, df 15 included,
  and `GAMMA(7.5)` itself is correctly rounded (1871.2543057977884, probes-numeric t1042-gamma-half-int A19).
- One fixed prefactor double does not explain df 15 either: scaling the exact tail by any one double ratio is
  bit-exact on at most 24 of the 120 df-15 cells of the two probes, as Excel's other tail noise stays on top.
- It is BETA.DIST's Beta function, B(1/2, 7.5), and only a narrow band of b is off (T-0969 attempt 9). probes-numeric
  `t0969-beta-density-b` (excel-graph, 2026-10-21): `BETA.DIST(x,0.5,b,FALSE)` at x = 0.05 and 0.01, b = 0.25 to 40 by
  0.25, is within 8.1 x 2^-53 of the exact density at every b except 7.5 (-18.0, -20.3) and 7.75 (-37.3, -38.8);
  b = 7.25 is +7.8 and +6.0, b = 8 is +4.3 and +3.5. `BETA.DIST(x,1.5,b,FALSE)` is within 8 at every b, 7.5 and 7.75
  included. So the error belongs to the pair (1/2, b) for b just below 8, the signature of a method switch in Excel's
  Beta function near 8, not of GAMMA(7.5) (correctly rounded) or of a + b alone. probes-numeric
  `t0969-beta-prefactor` (BETA.DIST cdf and pdf for a around 7.5 and several b at x = 0.05 and 0.2) shows the same
  -14 to -22 at (7.5, 0.5), but its other cells carry 10 to 90 x 2^-53 of power-term noise, too much to isolate
  a prefactor.
  Ours (since T-2119, BETA.DIST's cdf on the T-0970 incomplete beta): four of these cells are visibly off, past
  half a unit of the 15th digit. They are t0969-beta-prefactor D85 (Excel 1.6851260494385718E-11, 18 ulp) and D86
  (Excel 8.417666830127187E-07, 33 ulp; bit-exact on statrs' cdf), and t0969-t-beta-link E56 `BETA.DIST(D56,B56/2,0.5,TRUE)`
  (Excel 9.114921170875367E-05, 4 ulp) and F6 `=E6/2` (Excel 0.07797913037736931, 6 ulp). T-2633 owns them.
- The algorithm, for a < 1 and b <= 8 (T-0969 attempt 10): B(a,b) = EXP(GAMMALN(a) + (GAMMALN(b) - GAMMALN(a+b))),
  with Excel's own GAMMALN and in that grouping. probes-numeric `t0969-beta-switch` (excel-graph, 2026-10-21) has
  `BETA.DIST(x,a,b,FALSE)` at x = 0.01 and 0.03 for a in {0.25, 0.5, 0.75, 1.5, 2.5} and b = 6.5 to 9.5 by 1/32, with
  the worksheet's GAMMALN and GAMMA of a, b and a + b beside them:
  - a < 1, b <= 8: the form predicts the density's error within 3 x 2^-53 on 140 of 147 rows (the other 7 within
    3.5). Excel's errors run from -39 to +22.
  - Rejected there: GAMMA(a)*GAMMA(b)/GAMMA(a+b) in three orders (at most 30 of 147), and the other two groupings
    of the GAMMALN sum (36 and 58).
  - b > 8, or a >= 1.5: Excel is within the power terms' noise of the exact B, and the GAMMALN form misses
    (at most 4 of each 48-row region). GAMMALN switches to Stirling's series at 8 ("GAMMA, GAMMALN and LN kernels").
  - For the Student t this is B(n/2, 1/2) for df up to 16. In the engine only df 13 to 16 take it. Below df 13
    it costs probes-numeric cells, as the df-ratio errors there are small.
  - b under 6.5 (T-0969 attempt 11, from the existing `t0969-beta-density-b`, b = 0.25 to 8 by 0.25): the same form
    holds for a = 1/2. At x = 0.05 it predicts Excel's density error within 2 x 2^-53 at 29 of the 32 b (b = 3.75,
    4.25 and 4.5 are within 4). At x = 0.01, every b carries about -2 more, the power terms' share. For a = 1.5 it
    misses by up to 21 (b = 6.75). The form's own error below b = 6.5 is under 7 x 2^-53 (df 11 and 12: -4 to -7),
    so the tails' loss below df 13 is the tail kernel's noise, not B.
  - The branch structure is that of TOMS 708's `betaln` (Didonato and Morris 1992). For min(a,b) < 1 and
    max(a,b) < 8 it is `gamln(a0) + (gamln(b0) - gamln(a0 + b0))`, and from 8 `algdiv`. That makes TOMS 708 a
    candidate for BETA.DIST (an interpretation). Its `bratio` (R's `toms708.c`, with TOMS 708's own `gamln`) is not
    Excel's tail: on `t0969-t-beta-link` column E (`BETA.DIST(y,n/2,0.5,TRUE)`, 176 cells), it is bit-exact on 27 and
    the exact I_y on 36, in every bratio branch (bpser 19 of 86 against the exact 25, bup+bgrat 6 of 29 against 7). On
    the centre columns H and I (`BETA.DIST(z,0.5,n/2,TRUE)`) it does better than the exact value: 122 and 131 of 176,
    against 102 and 115.
  - The tail takes the GAMMALN terms smaller shape first, `GAMMALN(1/2) + (GAMMALN(n/2) - GAMMALN(n/2 + 1/2))`.
    BETA.DIST's argument order, `GAMMALN(n/2) + (GAMMALN(1/2) - ...)`, scored in `log_gamma_ratio` for df 13 to 16,
    gains 8 probes-numeric cells but loses 13 (10 of them df-15 `t0969-t-tail-grid` cells).
- Ours: `student_t.rs` `log_gamma_ratio`, which scales the series tail for df 13 to 16. Visibly-off cells:
  `t0969-t-tail-grid` 260 -> 245, `t0969-t-df-ratio` 9 -> 8, `t0969-t-beta-link` 42 -> 40, with none lost
  in the family's probes.
Status: VERIFIED (excel-graph) for a < 1, 6.5 <= b <= 8; b < 6.5 and a in [1, 1.5) UNVERIFIED
Tasks: T-0969

## T.DIST.RT's noise beyond t = 1 grows with a ln y (T-0969)
Rule: beyond t = 1 (y <= 0.99), where `T.DIST.RT(t,n)` is `BETA.DIST(y,n/2,0.5,TRUE)/2` at the worksheet's y,
Excel's error against the exact `I_y(n/2,1/2)/2` at that y grows about linearly with |E| = |(n/2) ln y|, the
signature of a prefactor exp(E) with E held in a double. Part of it is ln y rounded to a double and then scaled
by n/2; the rest is not identified.
- Evidence: 700 cells sampled (seed 1) from the 2,531 `t0969-t-tail-grid` cells with t > 1, y <= 0.99, y > 1 - y and
  df other than 2 (excel-graph). Errors are relative, in units of 2^-53:
  - The standard deviation of Excel's error is 3.3 for |E| < 1, 3.1 for |E| in [1, 2), 3.9 for [3, 4), 7 to 12 for
    [4, 10), and 24 for |E| >= 20. By df it is 2.7 to 5.2 up to df 100, and 15 to 21 at df 200 to 1000. The mean is
    0.2, so the error has no bias.
  - (n/2) times the rounding error of ln y correlates 0.38 with Excel's error. It is the same whether ln y is
    Excel's LN (`x87_log10::excel_ln`) or libm's. It predicts a standard deviation of 3.0 against Excel's 7.9 and
    leaves a residual of 7.3.
  - Weaker fits: fl((n/2) * ln y) correlates 0.32, and so does E as a double sum with ln B, 0.23 to 0.32 over three
    groupings, with B either exact or in Excel's GAMMALN form ("Student t prefactor per df").
- Rejected as Excel's incomplete beta, bit-exact counts out of 184 on `t0969-t-beta-link` column E
  (`BETA.DIST(y,n/2,0.5,TRUE)`), where the exact I_y gets 39 and 145 within 4 ulp:
  - R's `toms708.c` `bratio` with Excel's GAMMALN in place of `gamln`: 29, the same as with TOMS 708's own `gamln`,
    because `betaln` is not on any path these arguments take.
  - Cephes `incbet` with the prefactor exp(a ln x + b ln(1-x) - ln B) and Excel's B: 35 or 36. With the exact B
    it gets 31. Numerical Recipes `betai` gets 25 with Excel's B and 26 with the exact B.
  - On the 700 tail-grid cells (exact: 92 bit-exact, 589 within 4 ulp, 53 visibly off), Cephes `incbet` gets 52
    bit-exact with exp-log powers and 60 with `pow`. Each leaves about 175 cells visibly off.
- Ours: `student_t.rs` returns the exact tail at that y, rounded once. That is closer to Excel than any candidate above.
Status: VERIFIED (excel-graph) as an observation; the algorithm is UNVERIFIED
Tasks: T-0969

## Student t density noise is not input rounding (T-0969)
Superseded in part (T-2513): the algorithm is Loader's `dt` in Excel's x87 pieces, with the log branch at x² > n, a
two-step denominator and its own per-df constant ("Student t density is Loader's dt"). That explains the noise left
with exact inputs below. The R `dt` rejected below is R 2.x as written (0.2n cut, one sqrt(2π·w) denominator).
Rule: Excel's `T.DIST(t,n,FALSE)` for n >= 3 carries 1 to 2 ulp of noise of its own, not explained by the
rounding of t*t, n + t^2 or 1 + t^2/n. With those inputs exact, the best model is the exact density
times a fitted constant.
- Evidence: probes-numeric `t0969-t-df-ratio`, sheet Exact (excel-graph, 2026-10-21): the density at df 3, 4, 5
  and 7 for t = 1 + k/2^20 and 0.25 + k/2^22 (k < 512), where t*t and n + t*t are exact doubles. The models
  K * exp(-h fl(ln u)), K * exp(fl(-h ln u)), K * pow(u, -h), K * exp(-h log1p(t^2/n)), each with u = n + t^2 or
  1 + t^2/n and the best of 601 doubles K near the exact constant, are bit-exact on at most 491 of 1,024 cells
  (df 3), 459 (df 4), 458 (df 5) and 457 (df 7). The correctly rounded density with the same K-scan reaches about
  the same counts: 490, 458, 424 and 457.
- Rejected on the 14,914 cells of `t0969-t-pdf-runs`: R's `dt` (Catherine Loader's saddle-point form, with
  `bd0` and `stirlerr`) is bit-exact on 363 of the 1,024 df-2 cells (where `EXP(-1.5*LN(2+t^2))` gets all 1,024),
  853 of 4,826 at df 3, 2,049 of 4,826 at df 4 and about a third elsewhere.
Status: VERIFIED (excel-graph) as an observation; the algorithm is UNVERIFIED
Tasks: T-0969

## Student t density is Loader's dt (T-2513)
Rule: for df >= 3, T.DIST(t,n,FALSE) is Loader's saddle-point `dt` (R 2.x structure) in Excel's x87 pieces, every op
stored to a double; df 1 and df 2 are closed forms:
```
xx = fl(t·t);  q = fl(xx/n);  w = fl(1 + q)
xx > n:   u = fl(fl(LN(w)·n)/2)                                    (not R's x² > 0.2n)
else:     u = fl(-bd0(n/2, fl(fl(n + xx)/2)) + fl(xx/2))          the beta density's Excel bd0 (cut 0.2, v stored)
E = fl(C - u);   T = fl( fl(EXP(E) / fl(sqrt(w))) · fl(1/sqrt(2π)) )
C ≈ -bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2)        exact Loader constant; Excel is ±1-2 units off it
df 1: fl(1/fl(π·fl(1 + fl(t·t))))         df 2: EXP(fl(-1.5·LN(fl(2 + fl(t·t)))))
```
- Scores with the exact constant, exact / 4 ulp / 15 digits (verifier): hold-out t2513-r3-tdensity-hold2 (960, df 9-53)
  573/960/960 (CR 266/776/839); the verifier's t2513-verify-r3-tdensity-hold (1,000, df 13-50) 505/918/977 (CR
  93/555/787), so 23 visibly off against CR's 213. On cells.tsv df >= 3 (18,699): 5,956/18,383/18,631 (CR
  3,788/18,075/18,472). df 1: 27 of 27 (1/(π + π·t·t) also 27, not separated; CR 18); df 2: 1,050 of 1,050 (CR 452).
- The log branch starts at x² > n: on the verifier's hold-out 505 exact against 391 for R's 0.2n and 417 for no log
  branch (hold2: 573, 480, 520); its bands q ≈ 0.5-0.9 (bd0 path) and q ≈ 1.1 (log path) carry the difference.
- The denominator: at t ≈ 4.3, 38-45 of every 48 Excel values are not EXP of any double, so a value-space factor follows
  EXP. Rounding regressions select fl((n/2)·LN(w)), t·t, q and fl(sqrt(w)) at coefficients 0.85-1.35 (x² > n) and y,
  LN(y) of bd0's log form (x² < n). Free-constant scan, 2,112 cells: this structure 1,257; denominator
  fl(EXP/fl(sqrt(fl(2π·w)))) 1,051; E extended 890; R 3.x's form 812-949; R 2.x's 0.2n cut 1,011; 18 generic exp-of-log
  forms at most 754.
- The constant: per-df offsets of +2.5 to -1.6 units from the exact constant, shared by every t of a df (fitting C per
  block gives the same counts). Per-df constants fitted on the in-sample probes raise cells.tsv df >= 3 to
  11,517/18,684/18,697. REFUTED (verifier): that they are r3-loader-const's stirlerr table e(v) plus a gauge (model
  LT1e, hold2 501 against 573 for the exact constant; distance to each df's plateau rms 0.80 against 0.37). Also
  rejected: GAMMALN differences (-29 units at df 20 against a needed -0.11), 768 bd0 arithmetics for bd0(n/2, (n+1)/2),
  and every GAMMALN/GAMMA/EXP/SQRT chain for T.DIST(0,3,FALSE) (+3 ulp from CR).
- Rejected as the density: one stored product K·POWER(1 + t·t/n, -h), K·POWER(n + t·t, -h) or K·|t|·F.DIST(t·t,1,n)
  (at most 133 of 355 per df); BETA.DIST(y,n/2,1/2) or BETA.DIST(z,1/2,n/2) times the Jacobian (at most 109 of 355);
  TOMS brcomp(n/2, 1/2, y, z)/|t| (at most 302 of 2,112).
- Open: the constant's ±1-2 units; the bd0 series (most cells with x² < n/2); sw3 against sw2 (the 1/sqrt(2π) product
  first) near the mode (no variant above 444 of 576); `T.DIST(±10,1E6,FALSE)` -46 ulp.
Evidence: probes-numeric t2513-r3-tdensity-a1, -a2, -hold, -hold2, t2513-verify-r3-tdensity-hold, t0969-t-pdf-runs,
m9-t-dist-pdf; docs/research/t2464/r3-tdensity (`tdlib.py`, `s10_eset.py`, `s11_fsel.py`, `s12_logbr.py`,
`s17_final.py`, `s21_small.py`), verify-r3-tdensity (`vhold.out`, `hold2.out`, `vconst.out`, `vband.out`, `vsmall.out`,
`vold_perdf.out`).
Ours: `student_t.rs` `student_t_density`, a double-double cos^(n+1) times a gamma ratio.
Status: PARTIAL (excel-graph): the structure VERIFIED on two hold-outs for df >= 3, df 1 and df 2 VERIFIED; the per-df
constant UNVERIFIED
Tasks: T-2585, T-2586, T-0969

## Incomplete beta sweep (T-1044)
Rule: Excel's incomplete-beta values are not the correctly rounded ones, are not exactly symmetric, and saturate to exactly 1.
Oracle sweep (excel-graph) of `BETA.DIST` (a, b in {0.0001, 0.5, 1, 2, 2.5, 5, 10, 50, 100}, x in {0.1, 0.5, 0.9},
cum TRUE/FALSE), `F.DIST`, `F.DIST.RT`, `BETA.INV`, `F.INV` and `F.INV.RT` (p from 1E-10 to 0.9999999999). All
cells are numbers: no error appeared on the in-domain grid. The values are the targets for T-0970 and T-0974.
- Values are not the correctly rounded ones and are not exactly symmetric: `BETA.DIST(0.5,0.0001,0.0001,TRUE)` =
  0.5000000000000016 (the exact value is 0.5).
- Saturation: `BETA.DIST(0.9,0.0001,100,TRUE)` = 1 exactly, while the pdf stays finite and tiny
  (1.1116748024117171E-103); `BETA.INV(0.9999999999,0.5,0.5)` is exactly 1.
- Small tails keep full relative precision: `BETA.INV(1E-10,0.5,0.5)` = 2.4674011002723397E-20.
Evidence: probes-numeric cases `m9-beta-dist-a<a>` (one per a; `p` stands for the decimal point, e.g. `a2p5`),
`m9-f-dist`, `m9-f-dist-rt`, `m9-beta-inv`, `m9-f-inv`, `m9-f-inv-rt`. Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-1044 (sweep), T-0970, T-0974

## Incomplete beta: Excel's closed forms and the best candidate (T-1744)
Superseded in part by T-2464, which keeps the evidence below:
- The closed forms act on bratio's oriented arguments, after the swap at x > 0.5. See "Incomplete beta closed forms act
  on the oriented arguments", which also explains the two a = 1 misses.
- Excel's algorithm is TOMS 708 `bratio` with Excel's own pieces. See "Incomplete beta is TOMS 708 `bratio`" and the
  entries after it. It has no switch at the mean for min(a,b) <= 1, and it uses no Cephes fraction.
- The "best combination" below is the T-0970 engine kernel, not Excel's.

Rule as first recorded: BETA.DIST(x,a,b,TRUE) takes a closed form for b = 1, and probably for a = 1:
- I_x(a,1) = exp(a*ln(x)). This matches all 27 b = 1 cells. `pow` does not: BETA.DIST(0.1,5,1,TRUE) is
  1.0000000000000016E-05, not 1E-05.
- I_x(1,b) = 1 - exp(b*ln(1-x)), or equally 1 - pow(1-x,b), with the cancellation either implies:
  22 of 24 a = 1 cells. BETA.DIST(0.1,1,0.0001,TRUE) is 1.0535996061822317E-05, 3.4E-12 relative
  above the true value. The two misses, BETA.DIST(0.9,1,2.5,TRUE) and BETA.DIST(0.9,1,5,TRUE), are
  1 ulp below; no log1p, expm1, pow or correctly rounded form matches them. (T-2464: x > 0.5 swaps these to b0 = 1 at
  x0 = fl(1 - 0.9), and the oriented form is exact.)

Every other cell is unfitted. Excel's prefactor carries exp(a*ln x)'s rounding: at
BETA.DIST(0.1,100,2,TRUE) (m9-beta-dist-a100 Calc!A19) Excel is 1 ulp from that model and 66 from
the correctly rounded value. Its continued
fraction is not Numerical Recipes' Lentz.

The best combination found is bit-exact on 158 of 283 BETA.DIST/BETADIST cells (279 with numeric arguments):
- the closed forms above;
- otherwise Cephes' two continued fractions (incbcf/incbd, without the power series);
- a switch at the mean, x > a/(a+b);
- the prefactor exp(a ln x)*exp(b ln(1-x))*Gamma(a+b)/(Gamma(a)Gamma(b))/a*w.

For comparison:

| candidate | BETA.DIST cells exact |
|---|---|
| best combination | 158 |
| correctly rounded | 130 |
| Cephes, exp(a ln x), Gamma correctly rounded (best Cephes variant) | 121 |
| Cephes incbet as published (pow, tgamma) | 89 |
| Numerical Recipes betai | 98 |
| the engine (statrs `beta_reg`) | 87 |

Through the F and t mappings:
- F.DIST(x,d1,d2,TRUE), df truncated as Excel does: the best combination 15 of 33 cells (30
  with numeric arguments), against 6 for the engine and 17 correctly rounded.
- T.DIST(t,n,TRUE), mapped through the smaller of n/(n+t^2) and t^2/(n+t^2) (but I_y/2 directly
  for a lower tail under 0.1, where 0.5 - I_z/2 cancels): Cephes 105 of 150, the best combination
  95, the engine 78, correctly rounded 113.

Evidence: probes-numeric m9-beta-dist-a*, m9-legacy-betadist, m9-f-dist, m9-legacy-fdist,
t1049-fdist, m9-t-dist-cdf, t1048-tdist; fits `beta-dist`, `f-dist`, `t-dist`
(`fitlab/kernels/beta.py`).
T-1759 tested LibreOffice's GetBetaDist, the closest public relative. It uses the same mean
reflection, A&S 26.5.8's fraction, a pdf·x·y prefactor for a, b > 1, and a Lanczos lnB. It
scores 156 of 283 with Excel's closed forms (134 with its own pow and expm1), level with the
Cephes combination and not Excel's algorithm.
The remaining gap is the direct (unreflected) branch. Counts below are de-duplicated (x, a, b)
points with 0 < x < 1:
- **Reflected branch (1 − small):** LibreOffice matches 87 of 95 points (92%), the Cephes
  combination 75 (79%) and the correctly rounded value 78.
- **Direct branch:** no candidate matches more than 15 of 107 points (Cephes 15, LibreOffice 5,
  correctly rounded 7).
- **Error model:** Excel's relative error on the direct branch depends on x as well as (a, b). It
  tracks the total rounding error of the double product fl(a·fl(ln x)). For BETA.DIST(0.1,100,b,TRUE)
  that predicts 1.18E-14, and Excel's error ranges from 1.04E-14 to 1.35E-14 for b <= 5 (b = 10, 50 and 100 give -2.51E-14, -5.0E-15 and +2.31E-14, unexplained). At
  BETA.DIST(0.01,10,1.5,TRUE) it predicts 6.11E-15 against 6.15E-15 observed. Subtracting it
  leaves an x-independent residual for 8 of 38 (a, b) pairs; subtracting a·err(ln x) does so for 4.
- **Hypothesis (UNVERIFIED):** x^a is its own EXP(a·LN(x)) factor, the form of Excel's b = 1
  closed form. If the product were first summed with b·ln y − lnB, rounding that ≈ −230 sum
  would add up to ±1.4E-14 of b-dependent noise, but the spread across b <= 5 is about ±1.5E-15.
  (T-2464: confirmed in part. bpser's prefactor is EXP(fl64(fl(a·LN x) - lnB)), with no y; see "Incomplete beta
  bpser".)
- **What did not close the residual:** Excel's own GAMMA values (from T-1042), with best 62 of
  136 points, and 360 combinations of fraction, prefactor form and operation order, with best
  16 of 107. Reproduce all of this with docs/research/t1759_beta_direct_branch.py.
Status: VERIFIED (b = 1 closed form). a = 1 is VERIFIED in its oriented form (T-2464). The general algorithm is in the
T-2464 entries.
Tasks: T-1744, T-1759, T-0970, T-0969, T-1745, T-2464

## Incomplete beta is TOMS 708 `bratio`: one kernel behind BETA.DIST, F, T, BINOM and NEGBINOM (T-2464)
Rule: every incomplete-beta caller makes one call K(a, b, x, y) into a port of Didonato and Morris's TOMS 708 `bratio`
(the 1992 Fortran, not R's `toms708.c`). The caller forms both x and its complement y. The kernel computes the tail that
bratio's dispatch picks and returns the other as `(0.5 - w) + 0.5`, x87 stored.
- Dispatch: min(a,b) <= 1 swaps to (b, a, y, x) when x > 0.5; min(a,b) > 1 swaps when λ = a - (a+b)x < 0 (the mean).
  Then bpser, w1 = bpser(b0, a0, y0) at x0 >= 0.3, bup + bgrat, bgrat (b0 > 15), L140 (b0 < 40), bfrac or basym
  (b0 >= 40). `r2-bratio/bratio_excel.py` is the line-by-line model.
- **x0 >= 0.3, not R's 0.29**: on the 434 cells of t2464-r2-bratio-x29 in [0.29, 0.3) where the two dispatches give
  different doubles, Excel equals the 0.3 dispatch on 284 and the 0.29 dispatch on 87 (verifier). This is bit-level
  evidence only: both dispatches are within 15 digits on 434 of 434, and 63 cells match neither.
- One evaluation, bratio's tail: F.DIST and F.DIST.RT at one point fit one evaluation on 1,578 of 1,578 pairs; where a
  pair shows the computed tail (1,208), bratio predicts it every time and a mean switch misses 8 (e.g. F(1,1,100)).
  NEGBINOM.DIST(f,r,p,TRUE) and BINOM.DIST(r-1,f+r,p,TRUE) are exact complements on 441 of 460 pairs (the rest are
  BINOM's k = 0 shortcut and closed-form edges), and bratio predicts the computed tail on 348 of 348.
- Caller arguments. BETA.DIST: x and y = fl(1 - x) (after the [A, B] rescale). F (d truncated): x = 1/(1 + d2/(d1·X)),
  y = 1/(1 + (d1·X)/d2), 197 of 197 b0 = 1 closed cells against 180 for d1X/(d1X + d2) (1/(1 + (d2/d1)/X) is not
  separated). BINOM.DIST(k,n,p,TRUE) is the upper tail of I_p(k+1, n-k) and NEGBINOM.DIST(f,r,p,TRUE) is I_p(r, f+1),
  with x = p, y = fl(1 - p); a running sum of Excel's own pmfs matches only 501 of 1,371. T: "Student t tails are one
  kernel call" below.
- Positive control: the port in R configuration equals R's compiled `bratio` on 36,431 of 36,433 points.
Evidence: probes-numeric t2464-r2-bratio-x29, t2464-w2f-*, and the F, BINOM, NEGBINOM and T cells of
docs/research/t2464/cells.tsv; w5-triangulation (`f_pairs2.py`, `binom_negbinom.py`, `f_argform_closed.py`,
`ix_evidence.tsv`), r2-bratio (`x29_test.py`, `ctl_r.py`), verify-r2-bratio/x29check.out.
Ours: outside the closed forms (which follow bratio's swap and F's argument forms, T-2502: "Incomplete beta closed forms
act on the oriented arguments"), `incomplete_beta.rs` swaps at the mean for every shape, uses Cephes' fractions and
1 - v, and forms F's x as d1X/(d1X + d2); BINOM and NEGBINOM walk the pmf outside the closed forms, T is
`student_t.rs`. BETA.DIST's cdf runs on the port's dispatch (T-2505): `bratio/mod.rs` `dispatch` (x0 >= 0.3) and
`beta_dist_lower_tail`, with the bpser branches ported and every other branch, and a shape of 1 outside the closed
forms, still on `incomplete_beta.rs` (T-2847).
Status: VERIFIED (excel-graph) for the dispatch, computed tail, complement and argument forms; each branch's
arithmetic is in the entries below
Tasks: T-2464, T-2504, T-2505

## Incomplete beta closed forms act on the oriented arguments (T-2464)
Rule: after bratio's swap, on (a0, b0, x0, y0), with Excel's EXP and LN: b0 = 1 gives w = EXP(a0·LN(x0)); a0 = 1 with
y0 <= 0.9 gives w1 = EXP(b0·LN(fl(1 - x0))), 1 - x0 recomputed in the kernel (not the caller's y); a0 = 1 with y0 > 0.9
has no closed form. The other tail is (0.5 - w) + 0.5. The cut lies between x0 = 0.0998999751 (general kernel) and
x0 = 0.09999999999999998 = fl(1 - 0.9) (closed): T.DIST.RT at df 2 brackets it (t = 4.24 closed, t = 4.245 not),
`BETA.DIST(0.1,1,50,TRUE)` (m9-beta-dist-a1 A43, y0 = 0.9) is closed, and so is `BETA.DIST(0.9,0.5,1,TRUE)`
(m9-beta-dist-a0p5 A17), which x > 0.5 swaps to a0 = 1 at x0 = fl(1 - 0.9). `BETA.DIST(0.9,1,2.5|5,TRUE)` do not test
this cut: x > 0.5 swaps them to b0 = 1. A cut at x0 >= 0.1 scores 1,258 against 1,286 (`closed_variants.py`).
- Bit-exact on 1,286 of 1,286 cells (BETA.DIST 133, F 367, T.DIST.RT df 2 508, BINOM 112, NEGBINOM 138), 633 of 633 on
  the md5 hold-out and 193 of 193 on a case-level hold-out; the correctly rounded value matches 602. Through r2-bratio's
  full kernel, 2,345 of 2,345 over every data set.
- Variants: complement 1 - w1 1,278; the caller's y0 for fl(1 - x0) 1,258; a cut at x0 >= 0.1 1,258; the unoriented
  forms of the T-1744 entry 1,190.
- T-1744's two a = 1 misses, BETA.DIST(0.9,1,2.5|5,TRUE), are this rule: x > 0.5 swaps them to b0 = 1 at fl(1 - 0.9).
Evidence: closed-form cells of docs/research/t2464/cells.tsv (m9-beta-dist-a1, t0969-t-df2-cut, F, BINOM and NEGBINOM
probes); w5-triangulation `closed_final.py` (`closed_final.out`), `closed_variants.py`;
verify-w5-triangulation/verify_closed.out; verify-r2-bratio/analyze.out.
Ours: all 1,286 cells bit-exact through the worksheet functions (T-2502). Still pending:
- F.DIST, F.DIST.RT and FDIST pass w5's x = 1/(1 + d2/(d1·X)), y = 1/(1 + d1·X/d2) to the closed forms and the bratio
  port; where neither applies, today's Cephes kernel keeps d1X/(d1X + d2) and d2/(d1X + d2). w5's forms there move
  gated cells off Excel: `FDIST(Data!$D5,1,9)` (gen-m9/c18dbf21f11093f3 A151, also 7c20351562a2d71d A205) by 9 ulp and
  `2*F.DIST.RT(1E+12,22,5)` (probes/t2829-ftest-tail-over-b S!A17) visibly (T-2977, `fisher.rs::f_tail`).
- F.INV, F.INV.RT and FINV search the crossing of the unoriented tail (Cephes on d1X/(d1X + d2), a shape of 1 by
  x^a or 1 - (1 - x)^b, no closed forms; `fisher.rs::f_quantile_tail`): the ported F tail instead fixes 9 gated cells
  and moves 9 others 2 to 8 ulp off Excel's quantile (T-2977; `FINV(0.025,7,10)`, gen-m9/fec39efaba9284c3 A336;
  gen-m9/9cca17438a7e6f8a A114 by 8 ulp; ironcalc-calc F_DIST.xlsx J3, L3). T-2511.
- F.TEST at a shape of 1 is not the oriented closed form: x^a at b = 1 and 1 - (1 - x)^b at a = 1 on F.TEST's own x
  (no swap; -expm1(b ln1p(-x)) below x = 0.1) is bit-exact at gen-m9/68d8ffb69ff739d8 A143 (a = 1, b = 1/2,
  x = 0.571), a790a4d4a6a82756 A192 (a = 4, b = 1) and bbdfc06cae0e0d71 A356 (a = 5.5, b = 1), which the closed
  forms put 2, 3 and 10 ulp off (the last visibly; T-2977, `fisher.rs::unoriented_shape_one_tail`).
- a0 = 1 with y0 > 0.9: see "Incomplete beta at a shape of 1 outside the closed forms" (T-3007).
- BINOM.DIST's k = 0 is BINOM's own shortcut, not this kernel (T-2645).
Status: VERIFIED (excel-graph)
Implemented in: `incomplete_beta.rs::closed_form_tails` (Excel's EXP and LN, `x87_exp.rs`, `x87_log10.rs`), through
`beta_reg_complement` and `beta_reg_upper` (BETA.DIST's cdf and BETA.INV's refinement), `fisher.rs::f_tail` (F.DIST,
F.DIST.RT, FDIST), `student_t.rs::closed_form_tail` (T.DIST* at df 2), `binom.rs` (BINOM.DIST at k = n - 1,
NEGBINOM.DIST at r = 1 or f = 0); then `bratio/mod.rs::ported_tails` (bpser, w1 bpser and L140 bup+bpser) for BETA.DIST's
cdf, F.DIST, F.DIST.RT, FDIST, BINOM.DIST's cdf and NEGBINOM.DIST's cdf (T-2505, T-2506, T-2847), and for F.TEST's
large shapes and BETA.INV's huge-shape crossing (`bratio/mod.rs::kernel_tails`, T-2977)
Tasks: T-2502, T-2505, T-2511, T-2977, T-3007

## Incomplete beta at a shape of 1 outside the closed forms (T-3007)
Rule: after bratio's swap, at a0 = 1 with y0 > 0.9 (x0 below about 0.1, b0 not 1), Excel computes one tail, w, and
forms the other as (0.5 - w) + 0.5, as bratio does on every branch; it does not form w1 = y0^b0 on its own. Its w is
-expm1(b0·ln1p(-x0)) with Excel's own expm1 (Kahan's form, numbers.md "Excel's expm1 is Kahan's formula") and Excel's
own log1p (the stored atanh series, "Excel's log1p is a 14-term atanh series in alnrel's variable"), the product
b0·ln1p a double; TOMS bpser(1, b0, x0) (bratio's dispatch there: WBpser) is further off.
- Variants of w, scored as per-cell changes over every suite against libm's expm1 and log1p (T-3170, `make eval`,
  `docs/research/t3007/cell_diff.py`): Excel's expm1 of b0 times Excel's log1p makes 71 cells bit-exact (62
  probes-numeric, 6 gen-m9, 3 probes) and moves 6 off: probes/ftest-decomposition B12,
  t2220-f-dist-sweep-cdf A191, A195 and t2220-f-dist-sweep-d2-1 A16 by 1 ulp, t2220-f-dist-sweep-rt A195 by 4 ulp,
  and t1745-beta-inv-roundtrip G145 (Excel 0.04999999999999938, now 0.04999999999999949) by 16 ulp, visibly off: a
  1-ulp step of w near 0.95 is 16 ulp of w1 = (0.5 - w) + 0.5 at 0.05. No visibly-off cell is fixed. The same with the product held extended is identical on every cell. libm's expm1
  with Excel's log1p, and Excel's expm1 with libm's log1p, each win fewer cells and lose more (F.DIST.RT 21/6 and
  15/9 closer/further, against 37/1); (0.5 - EXP(b0·log1p)) + 0.5 and every form on the caller's y0 (LN(y0), with
  either EXP or expm1) are worse on most cells (F.DIST.RT 6/48 to 1/89).
- Examples: `F.DIST.RT(10,6,2)` = 0.09368601255412709 (gen-m9/146b46e74f2dac3b A329), `FDIST(40,8,2)` =
  0.024614204935833432 (gen-m9/abc2939ebd1e2365 A318), both 1 ulp low on libm's.
- Variants, scored as per-cell changes against the engine's earlier w = -expm1(b0·ln1p(-x0)), w1 = y0^b0 over every
  suite (`make eval`, origin/main 5f35109): w1 = (0.5 - w) + 0.5 from that w fixes 77 cells and moves 2 off
  (`F.DIST(1.5,50,2,TRUE)` and `F.DIST(5,7,2,TRUE)`, probes-numeric/t2220-f-dist-sweep-cdf A815 and A518, 1 ulp);
  the ported bpser for w with its complement fixes 64 and moves 90 off, most of them cells that return w.
- `F.DIST(5,20,2,TRUE)` = 0.8203482998751552 (t2220-f-dist-sweep-cdf A718; y0^b0 3 ulp low), `F.DIST(5,4,2,TRUE)` =
  0.8264462809917356 (A318), `FDIST(1E-10,2,7)` = 0.9999999999 (gen-m9/58e3253bbd3a7a82 A273), and
  `F.DIST.RT` at probes/ftest-decomposition B5 = 0.7381591796246967, which the same cell's BETA.DIST (B12,
  0.7381591796246966, x formed by BETA.DIST's caller) also matches.
- Of the shape-of-1 cells with literal arguments in the eval, 116 were off before (52 returning w, all within 4 ulp;
  64 returning w1, 18 of them beyond 4 ulp) and 59 after (the same 52, and 7 returning w1 within 4 ulp).
- Other off cells that moved, still off: t1745-beta-inv-roundtrip G112 (1 to 2 ulp), ironcalc-calc
  BETADIST_BETAINV.xlsx H31 (-4 to -3 ulp), t2220-f-dist-sweep-cdf A616, A817, A818 and t2220-f-dist-sweep-rt A162, A163
  (closer, mostly to 1 ulp).
- Every literal cell agrees with the complement, including the cells where x0·b0 > 0.7, where TOMS would dispatch
  w1 to bgrat (`F.DIST.RT(5,2,100)`, `F.DIST(0.25,100,2,TRUE)`, w1 down to about 0.0085). UNVERIFIED below
  w1 = 2^-53, where the complement is 0 or a multiple of 2^-53 (`BETA.DIST(0.95,1000,1,TRUE)` was about 5.3E-23 on the
  earlier y0^b0 and is now 0): no oracle cell has w1 below about 1E-16 (T-3150).
- BETA.INV searches this cdf, and w1 is now a step of one ulp of w, signed ulp from Excel's quantile, before and
  after: `BETA.INV(0.999,1,100)` (probes-numeric/m9-beta-inv A69, t1745-beta-inv-roundtrip J67) -4 to -40;
  `BETA.INV(0.001,100,1)` (A149, J144) +1 to -5; `BETA.INV(0.95,1,100)` (A68, J66) -16 to -6. Excel's BETA.DIST at
  those quantiles (t1745 G144, H144, I144, G145, H145, I145) is now bit-exact. Where in the flat step Excel's search
  stops is T-2511's. `BETA.INV(1E-20,1000,1)` (Excel `#NUM!`, probes-numeric/t1543-inverse-cdf-tiny-p A88) moves from
  0.9549925860214359 to 0.9632618940201478 (see "Inverse CDF functions at a tiny or near-1 probability").
Evidence: probes-numeric t2220-f-dist-sweep-cdf, t2220-f-dist-sweep-rt, t2220-f-dist-sweep-d1-1, t2220-f-dist-sweep-d2-1,
t1745-beta-inv-roundtrip; probes/ftest-decomposition; gen-m9 58e3253bbd3a7a82, 60b0b6e69ad35a45, fd9fa22d212080d6
(excel-graph).
Ours: `incomplete_beta.rs::shape_one_tails` (w = -`excel_expm1`(b0·`excel_ln_one_minus`(x0)), w1 = (0.5 - w) + 0.5),
through `bratio::kernel_tails` and `fisher.rs::f_tail`. The 6 cells above stay off (T-3192).
Status: VERIFIED (excel-graph) for the complement down to w1 of about 1E-16; w's form PARTIAL (71 cells for it, 6
contrary cells open, T-3192); below 2^-53 UNVERIFIED
Tasks: T-3007, T-2511, T-3150, T-3170

## Excel's log1p is a 14-term atanh series in alnrel's variable (T-2464)
Rule: for ln(1 + a) with -0.375 <= a < 0, i.e. ln(1 - x) at x <= 0.375 (the beta density, BINOM.DIST(0,n,p), bgrat's
ln x near 1), Excel computes, every op an x87 op stored to a double (fl53(fl64(.))):
```
r = a / (a + 2)          (the quotient double-rounded);  t2 = r * r;  p = r;  s = 0
k = 1..14:  p = p * t2;  s = s + p * c_k      c_k = RN(1/(2k+1)), forward;      l = 2 * (r + s)
```
Above x = 0.375 it is LN(fl(1 - x)); at 0.375 both give the same double and the next double up takes LN.
- BINOM.DIST(0,n,p) = EXP(fl(n·l)) pins l per cell (n = 2^k with n·|l| in [256, 512) makes the inversion unique). The
  model is bit-exact on 15,764 of 15,791 pinned rows (verifier's recount; CR log1p 10,821), every miss exactly 1 ulp.
- Out of sample: all 345 BINOM.DIST(0,n,p) cells of cells.tsv (general n; CR 311), including w5's p = 0.3 readout
  l = -0.3566749439387324, one ulp below CR. The `hold` case fed later structure scans, so it is not a clean hold-out
  (verifier); those 345 cells are.
- The density uses the same l: ln(BETA.DIST(p,1,n+1,FALSE)) - n·l is one constant per n on all 15,596 rows.
- Rejected: correctly rounded log1p, fdlibm, Cephes, x87 FYL2XP1, Kahan's two forms, log(1+t), TOMS alnrel's (3,3)
  rational (2.2e-16 off at 0.23); a 2r·W outer form (9,989 against about 15,121 of 15,197); Horner (15,722), smallest
  first (15,723), an IEEE quotient (15,762), 11 terms (15,691; 12 and up are identical); any piece held extended (25-35
  rows fewer).
- Not this function: algdiv's alnrel of a positive argument (the series there drops the density from 2,314 to 1,086
  of 3,200), and the density's ln x at x > 0.625 (LN(x)).
Evidence: probes-numeric t2464-r2-log1p-pilot, -runs, -grid, -hold, -hix; docs/research/t2464/r2-log1p
(`excel_log1p.py`, `pin.py`, `control.py`, `final_score.py`, `algdiv_try.py`); verify-r2-log1p/vscore.out.
Implemented in: `excel_log1p` and `excel_ln_one_minus` in `crates/rustsheet-engine/src/x87_log1p.rs` (T-2645), used by
BINOM.DIST(0,n,p) in `functions/statistical/binom.rs`. Ours: the beta density still uses powf (T-2503).
Status: VERIFIED (excel-graph) for -0.375 <= a < 0 (27 of 15,791 rows 1 ulp off); a > 0 UNVERIFIED
Tasks: T-2645


## BINOM.DIST is #NUM! from 2^31 - 1 trials (T-2464, T-2645)
Rule: BINOM.DIST and BINOMDIST are `#NUM!` when the truncated trial count is 2^31 - 1 = 2147483647 or more, whatever
k, p (0 and 1 included) and the cumulative flag. BINOM.DIST.RANGE is `#NUM!` there too, though it was probed only at
number_s = 0 and p = 1E-12 (A33, A45). 2^31 - 2 trials still evaluates
(`BINOM.DIST(0,2^31-2,1E-12,FALSE)` is 0.9978548205473029), while 2^31 - 0.5 truncates to 2^31 - 1 and is `#NUM!`.
So the accepted range is n < 2^31 - 1, one short of the largest 32-bit integer. NEGBINOM.DIST has no such limit
(`NEGBINOM.DIST(2^31,1,1-1E-12,FALSE)` is 0).
T-2464's powers of two bracketed it first: n = 2^30 evaluates, 2^31 to 2^33 are `#NUM!`.
Evidence: probes-numeric/t2645-binom-trials-limit A1-A46 (excel-graph, 2026-10-25); probes-numeric t2464-r2-log1p-hold
(rows with n = 2^30 .. 2^33), docs/research/t2464/verify-r2-log1p/vscore.out.
Unprobed: BINOM.INV and CRITBINOM at these trial counts.
Implemented in: `TRIALS_LIMIT` in `crates/rustsheet-engine/src/functions/statistical/binom.rs` (T-2645).
Status: VERIFIED (excel-graph)
Tasks: T-2464, T-2645

## BINOM.DIST pmf: closed forms at k = 0 and k = n, and one kernel with BETA.DIST (T-2513)
Rule: BINOM.DIST(k,n,p,FALSE) takes Loader's (2000) own end branches, `if (x==0) return exp(n*log(1-p)); if (x==n)
return exp(n*log(p));`, not R's dbinom_raw (which takes bd0 forms there), and in between shares one dbinom_raw with
BETA.DIST(p,k+1,n-k+1,FALSE):
- k = n: EXP(fl(n·LN(p))), 172 of 172 cells.tsv cells (CR 70), p = 0.99 and 0.999 included; 320 of 320 on the
  verifier's probe.
- k = 0: EXP(fl(n·l)) with Excel's log1p ("Excel's log1p is a 14-term atanh series"): 173 of 173 pmf cells (CR 54);
  318 of 320 on the verifier's probe. The 2 misses are subnormal results that the excel-graph oracle reads as 0
  (decision 0042: Graph shows 0 where Excel keeps a subnormal), so they are not evidence against the rule:
  `BINOM.DIST(0,999,0.5220313151772082,FALSE)` is 0 where the model gives 5.2E-321, and p = 0.5183174306860642 is 0
  against 1.2E-317 (t2513-verify-r3-binom-pmf-h A229, A238).
- Interior: on 56 runs of consecutive p, BINOM's and BETA's exponent errors differ by sd 0.38-1.13 units (the
  output-rounding floor) while each alone reaches 195 (verifier: 0.47-0.89 on 10 new runs, each alone up to 50). Only a
  per-shape constant differs (LN(n+1), the final rounding). So the pmf is a window on the beta density's Loader pieces
  at integer shapes: bd0's log form (stored ratio, LN, product; M - k stored beyond ratio 2) has coefficients 0.95-1.10,
  and runs where n·p is exact sit at the floor ((100,256): 0.41 units, from sd 142).
- The interior exponent is extended and EXP is applied once: nearly no cell with |E| >= 4 is EXP of a double (verifier:
  29 of 112), and Loader's non-log exp(lc)·sqrt(n/(2πk(n-k))) is 0 of 96 on every run with |lc| >= 16. lf/2 and the
  stored chain are not identified: best hold-out (t2513-r3-binom-pmf-hold, BINOM + BETA + NEGBINOM, 10,368)
  2,006/8,234/8,917 against CR 2,021/6,951/8,031. Large n (1,000 to 10^7) about 48% within 15 digits.
- Rejected: EXP of GAMMALN sums in any store pattern (rms 156-231 units, Loader with exact pieces 35); R's
  nq = n - n·p; other ratio formations (fl(k/np) extended, fl(fl(k/n)/p)); 120 summation trees of C, h, b1, b2 (best
  47%); stored E.
Evidence: probes-numeric t2513-r3-binom-pmf-a, -n, -hold, t2513-verify-r3-binom-pmf-h; BINOM cells of
docs/research/t2464/cells.tsv; docs/research/t2464/r3-binom-pmf (`a1_survey.out`, `b2_share_a.out`, `b3_reg_a.out`,
`b19_cfit_mul.out`, `c1_hold.out`), verify-r3-binom-pmf (`vscore_v.out`, `vstruct_v.out`).
Implemented in: `binom_dist_of` in `crates/rustsheet-engine/src/functions/statistical/binom.rs` (T-2645): k = 0, pmf and
cdf, is `excel_exp(n * excel_ln_one_minus(p))`; the pmf at k = n is `excel_exp(n * excel_ln(p))`. Ours: the interior pmf
is `discrete.rs` `binomial_pmf`, a double-double walk.
Status: VERIFIED (excel-graph) for k = 0, k = n and the shared kernel; the interior pmf UNVERIFIED
Tasks: T-2645, T-2146, T-2586

## NEGBINOM.DIST pmf sits on the shifted binomial kernel (T-2513)
Rule: NEGBINOM.DIST(f,r,p,FALSE) is built on dbinom_raw(r-1, f+r-1, p, q), the binomial kernel shifted by one, not R's
dnbinom (r/(f+r))·dbinom_raw(r, f+r, p, q).
- Regressed on the shifted kernel's bd0 roundings the coefficients are about 1 and the sd drops: (30,100) 44.2 → 0.62
  units, (3,8) 4.49 → 0.52, (7,20) 8.91 → 0.87. With the unshifted kernel nothing drops, and NEGBINOM.DIST(n-k,k,p)
  shares no rounding with BINOM.DIST(k,n,p) (sd of the difference is the quadrature sum).
- The final step is not identified. fl(p·BINOM(r-1, f+r-1)) is not bit-exact (verifier): at r = 1, where the kernel is
  q^f, 168 of 384 on the hold-out and 74 of 240 on the verifier's probe; p·EXP(E0) extended, EXP(fl(E0 + LN p)) and
  EXP(E0 + LN p) are not bit-exact either (at most 172 of 384 and 69 of 240). For r > 1 the shifted product still
  beats R's two forms (hold-out 328/2,316/2,541 against 233-275/1,775-1,779/2,203-2,211 of 3,072).
Evidence: probes-numeric t2513-r3-binom-pmf-n, -hold, t2513-verify-r3-binom-pmf-h; docs/research/t2464/r3-binom-pmf
(`b21_neg.out`, `b3_reg_neg_shift.out`, `b3_reg_neg.out`), verify-r3-binom-pmf (`vneg1.out`, `vneg2.out`).
Ours: `discrete.rs`, a double-double walk.
Status: VERIFIED (excel-graph) for the shifted kernel; the final step UNVERIFIED
Tasks: T-2146, T-2586

## Beta density for min(a,b) <= 1: lnB and a double-double exponent split (T-2464, T-2513)
Rule: BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1 is model M2 (r3-m1-residual, verifier-confirmed), for every b, b = 1
included. Every op is an x87 op stored to a double, double-rounded (on the worker's hold-out that gains 3 cells over
pure double ops):
```
tx = fl(fl(a-1)·LN(x));  ty = fl(fl(b-1)·ly)     ly = Excel's log1p(-x) for x <= 0.375, else LN(fl(1 - x))
lnB: a = 1 or b = 1     -> -LN(the other shape), whatever its size
     min < 1, max <= 8  -> fl(GL(min) + fl(GL(max) - GL(a+b)))     GL = Excel's GAMMALN; 8 inclusive (TOMS has < 8)
     min < 1, max > 8   -> fl(GL(min) + algdiv(min, max))          algdiv OPEN; TOMS algdiv in doubles is the best tried
hi = fl(fl(tx + ty) - lnB);   lo = (tx + ty - lnB) - hi, exactly (the two additions' rounding errors)
density = fl(EXP(hi)·fl(1 + lo))                    EXP = Excel's EXP (model V6) of the double hi
```
The products' rounding errors are not carried. EXP(hi)·EXP(lo) and EXP(hi)·(1 + lo) give the same double on every row.
- Scores, exact / 4 ulp / 15 digits (verifier): w1's probe1 12,796/12,799/12,799 of 12,800 (M1 10,375); the old pdf
  cells with min <= 1 815/1,101/1,103 of 1,107 (M1 566); the worker's hold-out 3,106/3,991/4,062 of 4,096 (M1 2,318);
  the verifier's hold-out 649/816/846 of 864. The GL branch is 960 of 960 (and the w4 block 159 of 160) on the worker's
  hold-out and 394 of 405 on the verifier's, all 11 misses at (0.37, 1) below. The rest of the misses are the algdiv
  branch: 66 of 270 on the verifier's hold-out, where each shape fits all its rows with a free lnB shifted -8 to +1 ulp.
  An accurate algdiv does better on both hold-outs (3,466 and 724 exact) but loses probe1's (0.5, 20) (682 against
  1,599 of 1,600).
- The min = 1 rule: -LN(max) fits all 32 shapes with min(a,b) = 1 in the worker's data (hold-out a = 1 block
  957 of 960, b from 8.25 to 1.1e9), including (1, 9), where TOMS algdiv is 1 ulp low, and (1, 2), where the GL form
  misses. The verifier's 7 such shapes are 189 of 189. For a < 1 with b = 1 it rests on one shape: (0.37, 1) needs lnB
  1 ulp below the GL form (16 of 27), which is -LN(0.37). UNVERIFIED beyond that shape.
- GL must be Excel's measured GAMMALN: with the engine's GAMMALN model the GL block of probe a drops from 1,648 to 1,410
  of 1,648, and the hold-outs from 3,106 and 649 to 2,804 and 562 exact, with the same 15-digit counts ("GAMMALN is
  Excel's own x87 routine").
- History: M1 (T-2464, w1) fed an extended E = tx + (ty - lnB) to EXP, stored at b = 1, with -LN(b) only for a = 1,
  2 < b < 8. It left a run-constant exponent offset of ±(1-4)·2^-55 that no EXP variant, table-driven exp, E rounding or
  product form closes (`expvar.out`, `tbin.out`, `cprod.out`). M2's lo term is that offset. Rejected for lo: only one
  addition's error (8,867 and 6,467 of 12,800 on probe1), the products' errors added (7,783-11,172), an unrounded
  1 + lo (11,437).
- Also seen, UNVERIFIED: where M2 gives a subnormal at a = 1, b = 9.1e6 and 4e8, Excel returns 0 (3 cells).
- Rejected (T-2464): a cdf prefactor over x or x·y; x87 base-2 pow forms; GAMMA-ratio B; R lbeta, NSWC/Apache logBeta;
  TOMS gamln/gamln1 or LN(GAMMA) for lnΓ(v < 1) (GL beats them in every combination, `lgam_*.out`). For algdiv
  (`lnbcands.out`, `algsearch.out`): accurate algdiv, TOMS fully extended, correctly rounded or series alnrel, and
  3,072 x87 storage variants, none reproducing both the probe a pins and probe1's (0.5, 20).
Evidence: probes-numeric t2513-r3-m1-residual-a, -hold, -gl, t2513-verify-r3-m1-residual-hold, and the T-2464 cases
m9-beta-dist-a*, t0969-beta-switch, t0969-beta-density-b, t2464-w2f-xruns-2, t2464-r2-log1p-*; w1's probe1
(w1-density/probe, not in eval/suites); docs/research/t2464/r3-m1-residual (`final_score.py`, `m1score.out`,
`hivar.out`, `lnbshape_a.out`, `lnbfit.py`), verify-r3-m1-residual (`vscore_all.out`, `vshape_vhold.out`,
`vchecks.out`); w1-density, r2-log1p `dens_m1.py`, w6-literature `h1_lnb.py`.
Implemented in: `functions/statistical/beta_density.rs` `beta_density_small_shape` (T-2503), with the engine's GAMMALN:
w1's probe1 12,796/12,799 (exact/15 digits) of 12,800, the old cells 806/1,103 of 1,107, the worker's hold-out 2,810/4,065
of 4,096, the verifier's 562/846 of 864. The engine's GAMMALN(1.2) is 1 ulp above Excel's (T-2122), consistent with the
(0.5, 0.7) block of t2464-w2f-xruns-2 losing 472 bit-exact cells, each now 1 ulp high. F.DIST's density keeps
`incomplete_beta::beta_density` (powf·tgamma ratio). Visibly off with it: the 4 old algdiv-branch cells at (0.0001, 50)
and (50, 0.0001), and probes-numeric t2464-w2f-absweep A2642, A6668, A7054 (GAMMALN form, shapes next to (0.5, 6.25))
and t2513-r3-m1-residual-a D1152, 5-8 ulp; cause not split between GAMMALN and the model yet (T-2821).
Status: PARTIAL (excel-graph): VERIFIED for max <= 8 and a = 1 or b = 1 with max > 1 (a < 1 with b = 1 on one shape);
algdiv (min < 1 < 8 < max) open
Tasks: T-2503, T-2512, T-2122, T-2513, T-2821

## Beta density for min(a,b) > 1 is Loader's saddle point (T-2464, T-2513)
Rule: BETA.DIST(x,a,b,FALSE) with both shapes > 1 (min >= 8 too: no a0 = 8 switch, no TOMS brcomp) is R dbeta's
(a+b-1)·dbinom_raw(a-1, a+b-2, x, q) (Loader 2000), with q = fl(1 - x) a separate input, in x87 ops on Excel's LN:
```
k = a - 1;  n = fl(a+b) - 2;  nk = fl(n - k)            stored, R's form; M = fl(n·x), fl(n·q)
bd0(k, M), log form when |k - M| >= 0.2·(k + M):  t = fl(k·LN(fl(k/M))), bd0 = t + (M - k), M - k stored
           series below the cut: arithmetic not identified (see below)
E = L5 + Cs - lf/2 - bd0_p - bd0_q    held extended into EXP (V6), never stored; each bd0 probably returned as a double
L5 = LN(n + 1) (Excel's);  Cs = accurate stirlerr(n) - stirlerr(k) - stirlerr(nk);  lf = ln 2π + LN(k) + LN(nk) - LN(n)
```
- Extended E: at (45.2, 60.9) (E from -8.5 to -13.6) none of 112 Excel values is EXP of any double, and over 14,319
  cells with |E| >= 8 only 615 are, chance level (verifier). Free-constant consistency: extended 8,596 of 11,264 sweep
  cells against stored 7,724 (diverse 5,990 against 4,379); r2's stored chain, the value-space p·f and a stored lc
  are rejected.
- Stored k, n, nk: single-ulp neighbours of a shape jump by up to ±20 units, and the jump is ε = n - k - nk (regression
  coefficient 1.13 on the sweep, 1.01 diverse). Within-shape sd 1.33 units for R's form against 7.4-11.4 for nk = b - 1,
  n = fl(k + b - 1), n - k unrounded or extended.
- bd0's log form is r2's x87 form at the bit level (each modelled rounding has coefficient -0.03 to +0.02; a bd0 store
  0.81-1.12). Beyond ratio 2 (|v| > 1/3), M - k is stored (Sterbenz covers it inside). The cut is 0.2: 0.25 drops
  (5.5, 12.5) from 113 to 13 of 120 and the verifier's hold-out from 4,535 to 3,534 of 11,776.
- The series branch carries +1 × the rounding error of fl(2k·v), v = fl(d/(k + M)) stored (verifier: median ρ +0.54 on
  10 of 10 hold-out blocks, 0 for M's rounding), the opposite sign of every natural formula that stores 2kv. After it,
  3-41 units of structured noise remain even with n·x exact; no R or Loader loop in any storage pattern fits (section
  3.1 of r3-loader-bd0). The series and log-form branches also differ in value by a shape-dependent offset (+10.5
  units at (40.25, 89.75)).
- (4, 4) is not an exception: the x87 model reproduces its q-tie response (+0.14 against Excel's +0.15; r2's -1.61 was
  the series linearisation). Integer shapes take no table path.
- The constant: stirlerr is accurate (every GAMMALN or double-rounded lgamma form misses by rms 15-26 units, exact 3.2);
  R's half-integer table is right to about a unit; L5 and lf's pieces are Excel's stored LN. A residual of 2-4 units sd
  is unexplained: sd 1.7 on the half-integer grid with an n trend from +1.7 (n < 8) to -0.4 (n 22-31), -10 to -14
  units at n 70-105, and noise at min(k, nk) < 3. Bit-exact counts are very sensitive to the ln 2π constant.
- Scores, exact / 4 ulp / 15 digits. Model M3 (structure above, exact stirlerr) and M4 (M3 with bd0 stored):
  - worker hold-out t2513-r3-loader-const-hold1 (log form, 3,296): M3 952/2,990/3,176, M4 960/2,996/3,172; r2's LDe
    685/2,675/3,029; exact constant 658/2,987/3,161; CR 251/1,738/2,435;
  - verifier hold-out (1,600): M3 133/1,035/1,351, exact constant 98/1,043/1,350, CR 66/533/909;
  - r2's 32,264 cells: M3 6,312/25,807/31,461, LDe 5,765/25,254/31,385, CR 3,573/26,024/29,280.
- Excel is not close to correctly rounded here, so a more accurate kernel does not reach its visible digits (T-2507,
  decision 0114's stage test): on `docs/research/t2464/beta_exact.tsv`'s 928 a, b > 1 density cells Excel is
  123 exact, 744 within 4 ulp and 737 within 15 shown digits of the 60-digit value, with errors up to 235 ulp
  (`BETA.DIST(0.9,5,100,FALSE)`) and 24-182 ulp at a dozen cells of shapes 10-100; on t2464-w2f-absweep 327 / 1,997 /
  2,259 of 3,072 and t2464-w2f-xruns-1 299 / 4,611 / 5,013 of 6,144. The same reference with q = fl(1 - x) scores
  worse (626 of the 928 within 15 digits), so Excel's error is not q's rounding alone.
- With one free constant per block (a structure test, not a model), on the verifier's bd0 hold-out (1,920 cells):
  r3's knobs 668/1,501/1,732, r2's 568/1,337/1,678, CR 163/940/1,407, a free-constant null 202/912/1,413. Both-log
  blocks are 387/474/474 of 640: the worker's 656 of 770 was a favourable subset.
- Rejected: R dbeta's log form, brcomp/(x·y) and TOMS's a0 >= 8 Stirling brcomp; Loader on (a, b); R's bd0 order
  fl(fl(t + M) - k) and 0.1 cut; cuts 0.25, 0.3; t, LN or the ratio kept extended; R >= 2.7.2 and R 2.0 lf forms;
  GAMMALN stirlerr; M2 = fl(n - M1) and nq = n - fl(n·x) for (4, 4); 896 lf / L5 / chain compositions (all within 0.1 rms).
Evidence: probes-numeric t2513-r3-loader-const-sweep1, -diverse1, -grid1, -hold1, t2513-r3-loader-bd0-a, -b, -hold,
t2513-verify-r3-loader-const-hold, t2513-verify-r3-loader-bd0-hold, and T-2464's t2464-r2-density-large-p1,
t2464-w2f-xruns-*, -absweep; docs/research/t2464/r3-loader-const (`m1_model.py`, `a2_forms.py`, `a5_xreg.py`,
`a6_struct.py`, `a10_grid.py`), r3-loader-bd0 (`final_score.py`, `phase.py`, `tie44.py`), verify-r3-loader-const
(`vscore.out`, `vimage.out`), verify-r3-loader-bd0 (`vhold_par.out`, `vhold_ph.out`, `cut_hold.out`); r2-density-large,
w4-fractions section 5, w6-literature H2.
Ours: powf × tgamma below a + b = 171.6, statrs above.
Status: PARTIAL (excel-graph): the structure (extended E, stored k/n/nk, bd0's log form, the 0.2 cut) VERIFIED on
hold-outs; the constant's residual and the series branch UNVERIFIED
Tasks: T-2507, T-2585, T-2586

## Incomplete beta bpser: a forward series times EXP(fl(a LN x) - lnB) (T-2464)
Rule: every bratio bpser call (min(a,b) <= 1 direct, w1 = bpser(b0, a0, y0), a, b > 1 with b0 < 40 and b0·x0 <= 0.7,
and tiny b0: Excel has no fpser) is, in x87 stored ops:
```
s = fl(1/a); c = 1; n = 1, 2, ...: c = c·(((0.5 - b/n) + 0.5)·x); s = s + c/(a+n)   until s stops changing
L = fl(GL(a0) + fl(GL(b0) - GL(fl(a0 + b0))))   b0 < 8, (a0, b0) = (min, max), GL = Excel's GAMMALN
L = fl(GL(a0) + algdiv(a0, b0))                 b0 >= 8 (best tried, not solved)
z = fl64(fl(a·LN(x)) - L)  (extended);   I = fl(EXP(z)·s)  (Excel's EXP of the extended argument)
```
- On the 2,000-shape probe t2464-r2-bpser-small-shapes (never tuned on; verifier-confirmed), exact/4 ulp/15 digits:
  b0 <= 1 2,311/3,082/3,083 of 3,083 (CR 918/2,959/3,052); 1 < b0 < 8 3,369/4,602/4,602 of 4,602 (CR
  1,219/3,646/4,152); b0 >= 8 1,489/2,840/2,861 of 2,868 (CR 781/1,751/2,074). Shape hold-out exact: 1,229, 1,628 and
  741, against CR 509, 605 and 399.
- All 120 expression trees and rounding patterns over (alx, g, G, GL) were scored; this one wins in every sub-branch,
  tune and hold-out. Planted data are recovered on 6,999 of 6,999 (next tree 6,928).
- The series (w3) runs forward from fl(1/a) with no final /a. It breaks the 639-668 of 1,024 plateau of every exact-sum
  form on w2's runs (950). The factor form ((0.5 - b/n) + 0.5, 1 - b/n, (n - b)/n) is invisible (±3 cells).
- No fpser: for b0 < 1e-15·min(1, a0), bpser scores 132/184/184 of 184, TOMS fpser 6/32/95. No apser (or a narrower
  one): a0 < 1e-15 falls through bratio's dispatch, 14 of 15 and 84 of 113 within 15 digits against TOMS apser's 6, 38.
- Rejected: TOMS bpser's prefactors (b0 <= 1 gam1 product, 1 < b0 < 8 m-loop, b0 >= 8 algdiv: 879, 1,901 and 949
  bit-exact against 2,311, 3,369, 1,489); TOMS's (1 + a·Σ) series and eps = 1e-15 stop; product forms
  fl(fl(EXP(alx)·K)·s); EXP's result kept extended (one run 512 → 367); an extended alx; Boost's prefix-scaled
  ibeta_series (1,538 against 4,442 of 5,183); a stored z out of sample (364/2,774/3 against 405/3,196/1,625 bit-exact
  on r2-bratio's three held-out sets).
- Open: L at b0 >= 8 (Excel's algdiv is about 1 unit from TOMS's: implied L -1.88, +1.65, +6.05 units on three runs);
  lnΓ(a0 < 1) inside L is shape-dependent (GL below 0.3, fl(gamln1(v) - LN v) at w2's (0.5, 6.25)), so on the old
  1 < b0 < 8 cells this L loses to r2-bratio's (3,365 against 4,038 of 7,455, verifier); b < 1 shapes top out at 55-84%
  bit-exact for any constant L (one stored intermediate, unidentified). The port needs Excel's GAMMALN exact at the shape
  arguments; the engine model is exact on 274 of 771 in [1, 2) and 1,738 of 2,585 below 1 (t2464-r2-bpser-small-gl).
Evidence: probes-numeric t2464-r2-bpser-small-shapes, -runs, -tiny, -tiny2, -gl, t2464-w2f-*; docs/research/t2464/
w3-series (`model.py`, `accum2.py`, `control2.py`), r2-bpser-small (`lib.py`, `zforms.py`, `bscore.py`,
`tiny_noapser.py`), verify-r2-bpser-small/verify.out, old_sets.out.
Implemented in: `functions/statistical/bratio/bpser.rs` `bpser` (T-2505) through `ported_tails`, for BETA.DIST's cdf
(`beta_dist_lower_tail`), F.DIST, F.DIST.RT and FDIST (`fisher.rs::f_tail`), BINOM.DIST's cdf and NEGBINOM.DIST's cdf
(`binom.rs`; T-2847), and F.TEST's tail with its own L (`f_test_tail.rs`, T-2681): the w3 series and
EXP of the extended z, with GL53/glalgdiv's L for min(a, b) < 1 and the extended GAMMALN sum for min >= 1, max <= 8.
Its algdiv (b0 >= 8) is `beta_density::algdiv` in doubles, where the model runs TOMS algdiv x87 stored.
A caller's shape of exactly 1 stays on Cephes' fractions (both shapes > 1 with max > 8 did until T-3007, below); L140's bpser(a0, 1, x0) at a whole b0 takes L = -LN(a0) (T-2506 above a0 = 8, T-2847 at every a0). On
probes-numeric it moves a net 12,408 cells to bit-exact and takes visibly-off numbers from 104,206 to 100,013; 987
bit-exact cells moved off and 7 became visibly off (listed in T-2847). T-2847 moved F, BINOM and NEGBINOM onto it: gen-m9
3,666 -> 3,602 known failures, ironcalc-calc 617 -> 595, probes 2,913 -> 2,912, with 9 cells 1 to 5 ulp off recorded
(T-2847 Outcome, decision 0141). L = -LN(a0) at a0 <= 8 too (T-2847) makes BINOM.DIST's whole shapes exact
(ironcalc-calc BINOM.xlsx E5:E8, E75, 7/64-type dyadic values) and fixes BETADIST_BETAINV.xlsx G15 (BETA.DIST at the
workbook's x = (0.7-0.2)/(1.2-0.2) = 0.49999999999999994, Excel 0.4999999999999999, now bit-exact), while the BETA.INV
on it (H15, Excel 0.7) goes 2 ulp low, 0.6999999999999997: BETA.INV's own search, its input now exact (T-2978).
Re-scored on the full eval (T-3007, origin/main 12c48a76; the same tree with L = GAMMALN's sum at a0 <= 8 against
L = -LN(a0) there, `docs/research/t3007/cell_diff.py`): -LN(a0) at a0 <= 8 makes 116 cells bit-exact (101
probes-numeric, 11 ironcalc-calc, 4 gen-m9; 114 from 1 to 4 ulp, 2 from within 15 digits) and moves 28 off by 1 to 4
ulp, none visibly: 27 probes-numeric BETA.DIST, BINOM.DIST, NEGBINOM.DIST and F cells (t2464-w2f-xruns-1's (4, 4) run
A10247 to A12259 has 21 of them; t2145-binomial-sweep A358, A1710, t2145-negbinom-sweep A556, A578,
t2220-f-dist-sweep-cdf and -rt A234), and BETADIST_BETAINV.xlsx H15, BETA.INV's own (above). Per function, cells
closer to Excel / further (every cell that moved, so more than the bit-exact counts): BETA.DIST 71/28, BINOM.DIST
32/2, NEGBINOM.DIST 11/2, BINOMDIST 6/0, F.DIST.RT 1/1, FDIST 1/0, F.DIST 0/1, BETA.INV 0/1. So the form stays at every a0; no visibly-off count changes either way.
Both shapes > 1 with max > 8 (T-3007): bpser's L is the accurate ln B at half-integer shapes with a + b <= 400 (the
model's `w3_big = "acc"`, the double-double B F.TEST's tail already took; `bratio/ln_beta.rs`
`neg_ln_beta_half_integer`) and TOMS `betaln` in doubles with Excel's GAMMALN and LN elsewhere (`toms_betaln`: bcorr for
both >= 8, gamln + algdiv for a < 2, the reduction of a otherwise). Against the previous Cephes fractions on the full
eval: bit-exact +67, visibly off -79 (BETA.DIST +6/-4, BINOM.DIST +27/-62, F.DIST +11/-4, F.DIST.RT +16/-3, FDIST 0/-1,
NEGBINOM.DIST +7/-5, BETADIST -1/0, BINOMDIST +1/0); gen-m9 3,312 -> 3,311 known failures (c6cfda6bd10cae93 A289),
probes 2,746 -> 2,741 (t2829-ftest-tail-over-b A12, A13, A15, A19, A20 now within the gate), no gated regression.
Gross, on every scored cell (`docs/research/t3007/cell_diff.py`): 281 cells closer to Excel, 63 further; 15 bit-exact
cells moved off (all probes-numeric, ungated: t2145-binomial-sweep A1700, m9-legacy-betadist A17, A18, t2220-f-dist-sweep-rt
A480, A858, A860, t0969-beta-prefactor D129, D131, t2145-negbinom-sweep A932, A1060, t1745-beta-inv-roundtrip G123, H123,
I123, t2220-f-dist-sweep-cdf A828, m9-beta-dist-a100 A19) and 4 became visibly off (t2145-binomial-sweep A2114, A2748,
A2750, t2220-f-dist-sweep-cdf A828). The a + b <= 400 cap bounds `half_integer_beta`'s recurrence (directive 5) and is a
choice between the two research forms, not an Excel boundary: no oracle cell has half-integer shapes above it on this
branch. The half-integer/TOMS split is provisional: T-2924's fit of Excel's ln B, once found, replaces both forms.
TOMS `betaln` at every shape (half-integers included) scores bit-exact +150, visibly off -73, but moves gen-m9
05d44f6dd6cfc567 A222 (`BINOM.DIST(1,10,0.05,2)`, a = 2, b = 9, Excel 0.9138616441006836) 1 ulp low, where the accurate
-LN(90) is exact; per betaln branch it wins on every one (bcorr 35 better/9 worse cells, a < 2 16/13, reduction
189/42), so Excel's L is neither form alone (T-2924). Still visibly off: t2829-ftest-tail-over-b A1, A3, A4, A6, A7,
A9 (A18 is now within 4 ulp): T-2924's k plus whole fl(a·LN x) steps, as above.
Status: PARTIAL (excel-graph): within 15 digits in every bpser sub-branch of the shapes probe (2,861 of 2,868 at
b0 >= 8, all others); bit-exact 52-75%; both > 1 with max > 8 a provisional form (T-3007)
Tasks: T-2505, T-2512, T-2122, T-3007

## Incomplete beta at a shape near the double range: no overflowing fl(a LN x), and bpser is #NUM! once 1/B overflows (T-3007)
Rule: Excel never evaluates a bpser whose fl(a·LN x) overflows: Excel for the web refuses a workbook that holds
`BETA.DIST(0.1,a,0.5,TRUE)` at a = 1E+295, 1E+300, 1E+308 or `1E+150*1E+150` (the upload's `createSession` fails 501
`FileCorruptTryRepair` / `unsupportedWorkbook`, as for a malformed file; 1E+295 did not answer in 10 minutes), while
a = 1E+290 is 0. a·LN(0.1) only overflows above 7.8E+307, so the question T-3007 asked (`BETA.DIST(0.1,1E308,0.5,TRUE)`)
has no Excel value from excel-graph; the engine keeps today's kernel there (`bratio::bpser_with_ln_beta`'s non-finite
guard), and desktop Excel's answer is an M14 question.
Where both shapes are above 1 and bratio takes bpser (b0·x0 <= 0.7), the cdf is `#NUM!` once 1/B(a, b) overflows a
double, though the value it would give is 0: `BETA.DIST(0.1,a,2,TRUE)` is 0 at a = 6E+153 and 1.3E+154 (a(a + 1) =
1.69E+308) and `#NUM!` at 1.4E+154 (1.96E+308) and 1E+200; at a = 1E+290 it is 0 for b = 1.01 (1/B about 1E+293) and 1, and
`#NUM!` for b = 1.5 and 9; b = 2 at 1E+100, 3 at 1E+100 and 4 at 1E+77 are 0. The density `BETA.DIST(0.1,1E+200,2,FALSE)`
is `#NUM!` too. The other branches at the same shapes are values: `BETA.DIST(0.9,1E+200,2,TRUE)` (L140 bup+bgrat) is 0 and
`BETA.DIST(0.1,2,1E+200,TRUE)` (swapped, the same point) 1; `BETA.DIST(0.1,0.5,1E+290,TRUE)` (bgrat) is 1 and
`BETA.DIST(0.9,1E+290,0.5,TRUE)` 0 (since T-2528 ours too, by the large-shape expansion, "Incomplete beta at a huge first
shape"), and `BETA.DIST(0.1,1E+200,1E+200,TRUE)` (bfrac) 0, where ours is `#NUM!`; but
`BETA.DIST(0.1,1E+200,1E+100,TRUE)` (bfrac too) is `#NUM!`, as ours is. The form behind the cut (1/B, or a^b, in
doubles) is not pinned: B(a, 2) does not separate 1/B from a(a + 1).
Evidence: probes/t3007-beta-dist-ln-x-overflow (excel-graph, 2026-10-29), 27 cells; the refusals are upload errors from
throwaway fills, not cells.
Status: VERIFIED (excel-graph) for the values; the refusal threshold is between 1E+290 and 1E+295 (not bisected); not
implemented: 7 of the 27 cells are off (T-3104 owns them; T-2528 took A3, A4, A21 and A22).
Tasks: T-3007, T-3104

## BETA.DIST's cdf is not monotone in its last bit (T-3007)
Rule: Excel's BETA.DIST(x, a, b, TRUE) can fall by an ulp between two x one double apart: `BETA.DIST(x,2.5,2.5,TRUE)` is
0.05000000000000001 at x = 0.16528026665565798 and 0.05 at the next double, 0.165280266655658; `BETA.DIST(x,2.5,100,TRUE)`
is 0.0009999999999999998 at 0.001042717872682154 and 0.0009999999999999996 at 0.0010427178726821543, and 0.5 at
0.021364564784887012 but 0.4999999999999999 at 0.021364564784887016. Those are the only three falls among the 1,007
cumulative four-argument BETA.DIST cells of docs/research/t2464/cells.tsv (sorted by x per shape pair). It is the
kernel's rounding (bpser at (2.5, 2.5), bfrac at (2.5, 100)), not a rule to model: a monotonicity check on the kernel
has to allow a fall where Excel's own values fall. The engine matches the (2.5, 2.5) pair bit for bit (bpser) and is
monotone at the two (2.5, 100) pairs (bfrac, not ported yet), so off Excel at one cell of each.
Evidence: probes-numeric/t1745-beta-inv-roundtrip G84/H84, G97/H97, G99/H99 (excel-graph; G, H and I are
BETA.DIST at D, E and F, three consecutive doubles around BETA.INV's x).
Status: VERIFIED (excel-graph), observation; checked by `bratio::tests::ported_tails_rise_with_x_where_excel_does`.
Tasks: T-3007

## Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y (T-2464)
Rule: for a, b > 1 with b0 < 40 and b0·x0 > 0.7 (bratio's L140), I = fl(bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0))
for x0 <= 0.7 (bgrat after bup beyond), n = int(b0), b0' = b0 - n (1 for integer b0, n one less). bup's brcmp1 is
fl(P·fl(y0·x0))/b0' with P = BETA.DIST(y0, b0', a0, FALSE), Excel's own density as a double (M1, as b0' <= 1); the terms
are d = d·fl(fl(apb + l)/fl(ap1 + l))·y0, the quotient stored and the product in a register, and w = w + d stored.
bfrac (min(a,b) >= 40) takes the same brcomp = density·x·y, with λ = fl(a - fl((a+b)·x)).
- Tie response c = d ln F/d ln y on w2's runs: Excel -3.72, -1.36, -1.91, -8.51, -7.17; this structure -3.68, -1.31,
  -1.97, -8.49, -7.22. Every published fraction × prefactor (Cephes, NR, LibreOffice, A&S, TOMS bfrac, R bratio) has
  c >= -1 there; raw TOMS brcmp1 exponents (b0', a0) give -7.22 to -11.15.
- L140 bup+bpser: 7,516/11,684/11,724 of 11,753 (CR 3,941/10,372/11,572; verifier). Out of sample, on F, T, BINOM and
  NEGBINOM L140 cells: 1,011/1,542/1,585 of 1,620, against TOMS's brcmp1 639 and CR 705. With Excel's own P and bpser
  read off the sheet (probe w4p1): 3,571 of 4,095.
- w4's "(7.3, 11.6): the worksheet density is not bup's" was an input artefact: the literal 0.5999999999999996 is cut
  to 15 digits, 0.599999999999999 (numbers.md "Formula number literals are read at 15 significant digits"), where
  BEST gets 59/1,023/1,023 of 1,023 (median -1 ulp). r2-bratio rounded literals instead of truncating them, so its
  "lnB(0.6, 7.3) is 24 units high" is refuted (verify-r2-bratio/misparse.out).
- bfrac at (100, 120): the fraction error regresses on λ's rounding with slope -0.725 (bfrac's own sensitivity -0.71),
  and the pdf's c + 1 predicts the tie response (-10.5 against -10.58). The fraction's own arithmetic is not found:
  white residual, sd 3.8 units, after 64 storage masks, five eps values and three precisions. bfrac cells: 146/476/685
  of 1,316 (CR 109/466/677).
- At a whole b0 (b0' = 1) with a0 > 8, bpser(a0, 1, x0)'s lnB is -LN(a0), B(1, b) = 1/b as the density takes it
  (T-2506). With the engine's own density, bup arithmetic and bpser, the L140 bup+bpser cells of r2-bratio's be, w2f
  and ix sets (bratio's dispatch as ported) score 8,800/11,696/11,721 of 11,742, the ix cells alone 995/1,344/1,362
  of 1,383. -LN(a0) at a0 <= 8 too scores 72 more bit-exact there (docs/research/t2506/RESULTS.md); T-2847 took it
  at every a0 once BETADIST_BETAINV.xlsx G15's x was read as the workbook's 0.49999999999999994, and T-3007 re-scored it
  on the full eval ("Incomplete beta bpser" above: 116 cells bit-exact, 28 moved 1 to 4 ulp off).
- Rejected: stored, extended, Horner or reverse W; the (P·y0)·x order; an extended P; every CF × prefactor on 15,657
  direct cells (best 2,931, CR 2,920); bfrac's λ from y.
Evidence: probes-numeric t2464-w2f-xruns-1, -xruns-2, -absweep; w4's probe w4p1 (w4-fractions/probe, not in
eval/suites); docs/research/t2464/w4-fractions (`tomsx.py`, `ideal_bup.py`, `t4_lambda.py`, `t25_old.py`), r2-bratio
(`bratio_excel.py` `pdf_xy`, `_bup_term`), verify-w4-fractions, verify-r2-bratio; docs/research/t2506.
Ours: L140 bup+bpser for BETA.DIST's cdf, F.DIST, F.DIST.RT, FDIST, BINOM.DIST's cdf and NEGBINOM.DIST's cdf
(`functions/statistical/bratio/bup.rs` `l140_bup_bpser`, `bup`, T-2506, T-2847). Excel is not correctly rounded on
every L140 cell: `F.DIST.RT(1.5,5,5)` = `FDIST(1.5,5,5)` = 0.33360956282916693 (probes/stat-argument-modes H221, H226),
I_x(2.5, 2.5) at x = 0.6000000000000001 through bup(0.5, 2.5, 0.6, 0.4, 2) + bpser(2.5, 0.5, 0.4), is 6 ulp above the
correctly rounded value at that x (0.333609562829166621, nearest double 0.3336095628291666), and
`F.DIST(1.5,5,5,TRUE)` 0.6663904371708331 (H215) is its complement; the port is 1 ulp above it, 0.33360956282916665
(decision 0141; Excel's form is unknown, r2-impact section 6);
L140 with bgrat and bfrac on Cephes incbcf/incbd with exp(a ln x)·exp(b ln y)·Gamma ratio.
Status: VERIFIED (excel-graph) for L140 bup+bpser to 15 digits (11,724 of 11,753; 64% bit-exact); bfrac's fraction
UNVERIFIED
Tasks: T-2506, T-2508, T-2512

## Student t tails are one kernel call, K(n/2, 1/2, y, z), with bgrat at b = 1/2 in extended precision (T-2464)
Rule: T.DIST.RT(t,n) = L = I/2 with I = K(n/2, 1/2, y, z): y = fl(n/fl(n + fl(t·t))) and the complement
z = fl(fl(t·t)/fl(n + fl(t·t))), not fl(1 - y); at df 2 the complement is fl(1 - y). T.DIST.2T and TDIST(.,.,2) are I;
the upper side (T.DIST(t > 0, TRUE), T.DIST.RT(t < 0)) is fl(1 - L); when n + t·t rounds to n (y = 1), L = 0.5. The
T-0969 entries' "switch at t = 1" and "rounded y beyond y = 0.99" are bratio's dispatch and bgrat's ln x switch below;
bfrac and basym never fire (min = 1/2). Excel's bgrat at b = 1/2 (n > 30 in bgrat's region) is
```
ν = a - 1/4;  lnx = LN(y) if z > 0.01, else Excel's log1p(-z)   (z = 0.01 itself takes the series)
I = RN( F(a, lnx) · ERFC.PRECISE(fl(sqrt(fl(-ν·lnx)))) / erfc(sqrt(-ν·lnx)) )
```
with F(a, lnx) the exact I at ln x = lnx and an exact erfc below: TOMS bgrat is extended throughout except three stored
doubles, lnx, z = fl(-ν·lnx) and Excel's ERFC of fl(sqrt(z)). The implementable form is TOMS bgrat in x87 extended with
those three pieces (`r2-tdist/xlk.py` `bgrat_half_ld`).
- bgrat at b = 1/2: T bgrat cells 1,560/1,562/1,563 of 1,564 (hold-out 780 of 783; `bgrat_half_ld` 1,545); probe
  t2464-r2-tdist-p1 part C, out of sample, 4,906 of 4,915 (`bgrat_half_ld` 4,895); non-T bgrat cells at b = 1/2 (r2-bratio's data, never used
  here) 365 of 387, against 277 for r2-bratio's kernel. One piece removed: CR erfc 395, z unrounded 1,370, no ERFC
  ratio 291; TOMS bgrat in doubles with Excel's pieces 308. The z = 0.01 cut holds on 1,999 of 2,000 straddling cells
  (part B).
- The whole family on the 5,152 lower-side tails of cells.tsv: 3,383/5,126/5,145, against CR 798/2,947/3,866; visibly
  off 7 against 1,286. Upper side 141 of 153 (CR 133).
- Open: bup+bgrat for n <= 30 is within 15 digits on 1,343 of 1,344 but bit-exact on 356 (bup arithmetic). The T
  density, T.DIST(.,FALSE), is not a kernel call: it is Loader's `dt` ("Student t density is Loader's dt (T-2513)").
Evidence: probes-numeric t2464-r2-tdist-p1 (parts A-D), t0969-t-tail-grid, t0969-t-beta-link, t0969-t-df2-cut,
m9-t-dist-*; docs/research/t2464/r2-tdist (`hx.py`, `hx_ctl.py`, `xlk.py`, `tscore.py`, `oos_bgrat.py`, `p1.py`,
`pdf_*.py`), w5-triangulation `t_link2.py`, `t_pairs.py`.
- In the engine (T-2509) bgrat at b = 1/2 is double-double TOMS bgrat with the three stored pieces
  (`bratio/bgrat.rs`): T bgrat cells 1,546 of 1,564 bit-exact, 1,563 within 15 digits; part C 4,898 of 4,915. Every
  df from 3 takes the one call: the cells.tsv lower-side tails go from 1,498 bit-exact and 280 visibly off (the
  engine's composite before T-2509) to 3,261 and 5, part D from 205 visibly off to 10.
- df 1 is the exception: through the ported bpser at a = b = 1/2 (w3, GL53's ln B) 24 of cells.tsv's 67 df-1 tails are
  bit-exact, against 41 for the double-double series, and `T.DIST.2T(1,1)` (Excel 0.5000000000000001) is 5 ulp low.
  So Excel's bpser at (1/2, 1/2) is not w3's; df 1 keeps the series (T-2930).
Ours: K(n/2, 1/2, y, z) from df 3 (`student_t::kernel_tail`); df 1 and 2 keep `student_t.rs`'s own composite (a
double-double series, a Cephes deep tail, incbcf/incbd from z, and df 2's closed forms).
Status: VERIFIED (excel-graph) for the call form and bgrat at b = 1/2; the bup arithmetic and the T density UNVERIFIED
Tasks: T-2509, T-2513, T-0969

## F density is not the beta density, R's df or a GAMMALN formula (T-2513)
Rule (PARTIAL): F.DIST(x,d1,d2,FALSE) is its own formula. With m = d1, n = d2, a = m/2, b = n/2, h = a + b and
mx = fl(m·x):
- Not BETA.DIST's density times a Jacobian: at y = 1/(1 + d2/(d1·x)) and at d1·x/(d1·x + d2), Excel's F error is
  uncorrelated with the worksheet BETA.DIST's (|corr| <= 0.49 over 66 runs) and larger (sd 105 against 57 at (50,100),
  x ≈ 13). Not R's df through dbinom_raw (free constant per run: 600-653 consistent, the wrong-structure level;
  whole model 293 of 4,005 bit-exact, CR 292). Not a GAMMALN formula (40 compositions all below CR; best 114/832/1,133
  of 1,599 cells.tsv cells).
- The shape constant is swap-symmetric: F.DIST(1,m,n,FALSE) = F.DIST(1,n,m,FALSE) bit for bit on 585 of 585 pairs.
  At x = 1 Excel's value is the exact density to about one ulp at every shape measured: sd 0.6-0.9 units for
  min(d) <= 7, uncorrelated with Excel's GAMMALN errors, and within -3..+2 ulp of the correctly rounded value on 361
  shapes with d1, d2 in [16, 1000] (`F.DIST(1,300,300,FALSE)` = 3.4520635814275824, 1 ulp below the exact
  3.452063581427583; t2586-r5-f-bigconst-a B5770, B4690 at (160,160)). REFUTED (round 5): round 4's "off by -64
  units at (300,300) and -35 at (160,160), differing by side" was the round-4 model's own H rounding absorbed into
  its constant, not Excel's (docs/research/t2464/r5-f-bigconst).
- The x-dependence goes through ln(1 + r), r = min(mx/n, n/mx), with Excel's log1p switch (series to 0.375, then
  LN(fl(1 + r))): Excel's error responds to the rounding of s = fl(1 + r) with slope about -h on 12 of 12 hold-out runs
  with 0.375 < r <= 1, and |slope| <= 0.61h on all 24 below (the verifier confirms the switch).
- The best model scored (K exact, a ln(m/n) + (a-1) ln x - h·L(r) on the smaller ratio) only beats CR modestly:
  hold-out 161/776/1,091 of 1,440 (CR 114/733/1,060); the verifier's hold-out 63/435/621 of 960 (CR 47/337/568);
  cells.tsv 172/1,052/1,297 of 1,599, below CR's 1,308 within 15 digits.
- Not established: that ln x enters as a stored Excel LN product (that product gets coefficient 0). Open: the x-power
  term's form, large-shape noise (sd 20-200 units on a lattice of a [32, 64) stored term at (100,100)), a +3 to +5 unit
  offset at d1 = 1, the d <= 2 cells, non-integer df.
- The beta density × y(1-y)/x form the engine used before T-3034 is not Excel's.
- Round 4 (T-2586, verified on untouched hold-outs; still PARTIAL):
  - Not TOMS 708 brcomp(a,b,y,q)/x in any branch: below CR on every set (hold-out 69/501/822 of 1,536, CR
    111/646/904), and the s = fl(1+r) response appears at r > 0.375 in the Stirling region too (15 of 15 runs).
  - Structure: ln F = T + P - ln x, with P = fl(c·LN(x)) a stored product (c = a when mx <= n, entering as +P;
    c = b otherwise, entering as -P), 1/x applied accurately, and T one stored double holding the constant and
    -h·L(r): Excel's ln F moves in whole quanta of ulp(T) on consecutive x (256-512 units at (300,300)). The terms
    combine as S2 = fl(fl(K_hi + H) + P), H = -h·L(r), the remainder K' - K_hi added outside the rounding
    (sd 0.7-0.8 units at (100,100) and (100,50) with a free K, against 80-172 for the raw error). For r <= 0.375 the
    -h·L term is not identified (at least two stored pieces).
  - Constant: K = (h - 1/2)·ln(h/o) + 1/2·ln(w/2π) + stirlerr(h) - stirlerr(a) - stirlerr(b) with (w, o) = (a, b) on
    side 1 and (b, a) on side 2, (h - 1/2)·log1p(fl(w/o)) with Excel's log1p kept extended, and Excel's stirlerr
    errors (r3-loader-const's half-integer table) plus a global +5.33 units: in-sample rms 0.52 units over 56
    (shape, side) groups, about 1.6 units out of sample.
  - Best model (M-sd: that constant with the stored-product structure, K and E extended into EXP, F = fl(EXP(E)/x)):
    bit-exact / within 4 ulp / within 15 digits 314/919/1,029 of 1,344 on its hold-out (CR 104/688/882), 287/1,048/
    1,198 of 1,536 on another worker's (CR 111/646/904), 337/1,119/1,255 of 1,440 on round 3's (CR 114/733/1,060).
    Not bit-exact (about 35%) and not at visible parity; the large-shape class stays poor.
- Round 5 (T-2586, verified):
  - Argument and range rules, exact on an untouched hold-out (52 of 58 rule cells there are bit-exact and all 58
    within 15 digits; 1,606 of 1,606 rule cells over every earlier set): d1 and d2 truncate toward zero; a truncated df
    below 1 or above 1E10 is `#NUM!`; x < 0 is `#NUM!`; at x = 0 the result depends on trunc(d1) only (1 → `#NUM!`,
    2 → exactly 1, 3 or more → 0); a subnormal x (such as EXP(-720)) is `#DIV/0!`; the result is `#NUM!` exactly when
    fl(trunc(d1)·x) overflows; otherwise Y = EXP(E) is stored (a subnormal Y keeps its lost bits), F = fl(Y/x), and an
    F below 2^-1022 is 0.
  - At x = 1 Excel's value is the exact density to about one ulp (361 shapes with d1, d2 in [16, 1000]: 152 equal the
    correctly rounded value, all 361 within -3..+2 ulp). Round 4's large-shape "constant offset" was the model's own
    H rounding being absorbed.
  - What blocks parity is the size of the power term P = c·ln x: within 15 digits on 99% of cells with |P| < 4, 87%
    for |P| in [4, 32), 36% for [32, 256) and 10% at 256 or more. The stored-sum order S2 = fl(fl(K_hi + H) + P) is
    rejected (0 of 330 runs fit even with a free K_hi), and so is a "two stored H pieces" reading of round 4's double
    jumps (an artefact of a non-dyadic constant lattice). At power-of-two shapes such as (2,2) and (8,8), where every
    product is exact, Excel still scatters about ±1.5 units, which points at the exponential step itself (an exp2
    split or a value-space x87 pow per factor), not at the exponent's pieces.
  - Integrated model M-final (docs/research/t2464/r5-f-final/fdensity_excel.py: the rules, M-sd's exponent, the
    large-shape constant, F = fl(fl(EXP(E))/x)), frozen before its hold-outs were filled: 126/522/722 of 1,584 value
    cells on t2586-r5-f-final-hold (CR 63/269/444; M-sd with the rules 123/525/721); on the verifier's fresh hold-out
    no better than M-sd with the rules; 72% within 15 digits over 66,191 F cells. Not bit-exact, not at visible parity.
Evidence: probes-numeric t2513-r3-fdensity-a, -sw, -hold, -holdsw, t2513-verify-r3-fdensity-hold; F.DIST cells of
docs/research/t2464/cells.tsv; docs/research/t2464/r3-fdensity (`a8_pair.out`, `a3_score.out`, `b1_sweep.out`,
`b6_hold.out`, `c1_hold.out`), verify-r3-fdensity (`vcheck.out`, `vscore_vhold.out`, `vswitch_vhold.out`,
`vpair.out`); round 4: probes-numeric t2586-r4-f-brcomp-hold, -big, t2586-r4-f-power-a, -hold, t2586-r4-f-small-df-a,
-hold and the verifiers' t2586-verify-* cases; docs/research/t2464/r4-f-brcomp, r4-f-power, r4-f-small-df and
verify-r4-*; round 5: probes-numeric t2586-r5-f-*, t2586-verify-r5-*; docs/research/t2464/r5-f-combine, r5-f-smallr,
r5-f-bigconst, r5-f-edges, r5-f-final and verify-r5-*.
Ours (T-3034): `f_density.rs` `excel_f_density` is M-final for every x > 0, bit for bit: it equals
`fdensity_excel.value(x, d1, d2, "div")` on all 64,883 positive value rows of the r5-f-final score sets and the
final hold-out (unit test `final_hold_is_model_m_final` pins the hold-out's 1,549). K's exact part is formed in
double-double from Stirling's formula, ln(w)/2 + (h - 1/2)·ln(h/o) - ln(2π)/2 - μ(a) - μ(b) + μ(h) with μ Binet's
function, and the offset table (r3-loader-const's stirlerr errors, the +5.33 and -0.42 constants, numpy's linear
extrapolation above 28) is carried as data. `make eval` (probes-numeric, F.DIST): visibly-off cells 38,027 → 18,722,
bit-exact 5,595 → 16,268; per cell 21,404 move to within 15 digits and 2,591 out, and none of the 36 (side, r, df,
|P|) regions loses net, so no region keeps the old beta-density form (which, with its Loader fallback, is gone).
Status: PARTIAL (excel-graph): the argument and range rules (round 5), the negatives, the swap symmetry, the x = 1
values and the ratio switch are measured; the value is M-final, at parity only for |P| < 4
Tasks: T-2586, T-2219, T-2095, T-3034

## Incomplete beta: the integrated model, what is open and what is rejected (T-2464)
Rule: the entries above assembled in one TOMS 708 port (Excel's EXP, LN, GAMMALN; x0 >= 0.3), r2-bratio's
`bratio_excel.py` BEST, are the reference model for the engine port. They are not yet Excel everywhere.
- Scores (verifier, literals truncated to 15 digits): 22,808/36,993/38,059 of 40,046 cdf cells, against CR
  11,473/33,974/37,069. On 20,960 cells of probes built after it (t2464-r2-bpser-small-*): 11,671/20,242/20,509 against
  CR 5,413/16,780/18,771. Plain TOMS with glibc (R) is bit-exact on 10,514 of 40,014 and TOMS with Excel's EXP, LN and
  GAMMALN on 10,853 (CR 11,812): the gain is the Excel pieces, not TOMS.
- At visible parity: the closed forms, w_bpser, w1_bpser, L140 bup+bpser, bup+bgrat (min <= 1). Short of it: L140
  bup+bgrat 212/2,208/2,402 of 3,303 (CR 618/2,814/3,025); bgrat 989/1,761/2,156 of 2,451 (CR 899/2,072/2,230; mostly
  T, solved above); bfrac 146/476/685 of 1,316; TOMS's fpser/apser as BEST runs them, 14 of 752 fresh cells (Excel has
  neither, see bpser).
- r2-bratio's "Excel-only noise of sd 47 units" on the (30.7, 1.7, 0.81) a-sweep was the same literal artefact: with
  truncated literals the sd is 2.9 and the model sits a median -6 ulp off (verify-r2-bratio/misparse.out).
- Open: bgrat for b ≠ 1/2 (L140 bup+bgrat; 600 x29 BETA.DIST/F bgrat cells), first test the b = 1/2 identity with Q(b,z)
  from Excel's incomplete gamma; algdiv for b0 >= 8 (about 1 unit off TOMS, in bpser and the density); lnΓ(a0 < 1) in L
  and the 55-84% ceiling of b < 1 bpser shapes; bfrac's fraction (sd 3.8 units) and bup's arithmetic in bup+bgrat;
  a0 < 1e-15 at 0.04 <= y < 0.25 (Excel 1e-14 to 3.8e-13 relative off the truth, in bgrat's incomplete-gamma ratio);
  the densities after T-2513's round 3 (each in its entry): Loader's constant residual of 2-4 units and bd0's series,
  the T density's per-df constant, BINOM's interior pmf, NEGBINOM's final step, the F density, and the density's
  algdiv lnB for min < 1 < 8 < max (min(a,b) <= 1 is otherwise model M2).
- Rejected (decision 0056; each scored on the same data): Cephes incbet/incbcf/incbd, NR betacf/betai, LibreOffice
  GetBetaDist, A&S 26.5.8, statrs, Boost's ibeta_series prefix and the T-1744 best combination (w2's tie response:
  their c >= -1 where Excel's is -1.4 to -10.6; every CF × prefactor at or below CR on 15,657 cells); R's toms708.c as
  is (0.29, its bgrat, algdiv, bfrac association); TOMS 708's own fpser, apser, bpser prefactors, brcmp1/brcomp
  exponents, alnrel rational, gamln/gamln1 for lnΓ(v < 1) and 15-digit Stirling constants; a switch at the mean for
  min <= 1; a 1 - w complement; fdlibm, Cephes, Kahan and FYL2XP1 log1p; R lbeta, NSWC/Apache logBeta (cutoff 10) and
  LN(GAMMA) ratios; R dbeta's log form and TOMS's a0 >= 8 Stirling brcomp for the a, b > 1 density; TOMS bgrat in
  doubles with Excel's pieces (308 of 1,564 at b = 1/2; on the engine's T cells it fixes 89 and breaks 145).
- Port order (r2-impact): a plain TOMS port is net negative outside BETA.DIST (-103 visible on 10,602 F, T, BINOM and
  NEGBINOM cells; BEST +543), so each branch moves with its Excel piece. Of the 5,533 pre-T2464 visibly-off family
  cells the pieces remove about 1,454 kernel cells, the 68 BINOM k = 0 cells and about 300 inverse cells net; 792 of
  the 1,236 gated ones are argument and error handling (T-2093). These are estimates: r2-impact's hold-out was fitted
  through, and on the verifier's by-case split the ladder goes 722 → 272.
Evidence: probes-numeric t2464-*; docs/research/t2464/README.md, r2-bratio (`score.py`, `report_best.out`,
`report_ladder.out`), r2-impact (`impact.out`, `ladder.out`, `branches.out`, `offcells_pre.tsv`), w2-forensics
`tie_response.py`, w4-fractions `t1_pdfshare.py`, w6-literature (source survey), verify-r2-bratio/analyze.out,
verify-r2-impact/v2_plan.out.
Status: PARTIAL (excel-graph)
Tasks: T-2464, T-2504 to T-2511, T-2512, T-2513, T-2585, T-2586

## Incomplete beta in the ironcalc-calc workbooks: F tails and densities (T-0970)
Rule (VERIFIED on the cells named): Excel's F.DIST.RT is the upper tail of the incomplete beta,
I_w(d2/2, d1/2) with w = d2/(d2 + d1*x), not 1 - F.DIST. With the T-1744 best combination as I_x,
`F.DIST.RT(0.8,2.4,3)` = 0.5266779476655967 and `F.DIST.RT(0.7,4,2.4)` = 0.6597222222222223
(`F_DIST.xlsx` H2, H3) are bit-exact, where 1 - cdf misses by 6 and 13 ulp; F.TEST's two tails
the same way make `F.TEST({1,2,3,4},B1:B4)` and `F.TEST(A1:A3,{2;3;7})` (`F_TEST.xlsx` F1, F3)
bit-exact.
Two more rules from the non-gated probes-numeric suite. Both are PARTIAL: oracle cells support
them and oracle cells contradict them.
- **The complement is exact (PARTIAL).** The incomplete beta takes y = d1*x/(d1*x+d2) and its
  complement d2/(d1*x+d2), both formed directly, as TOMS 708 `bratio(a,b,x,y)` does, instead of
  1 - w.
  - Supporting: `FDIST(1E-6,1,1)` and `F.DIST.RT(1E-6,1,1)` are Excel's correctly rounded
    0.9993633804398389 (m9-legacy-fdist A11, A12). Cephes `fdtrc`'s form, w and then incbet's
    internal 1 - w, misses by 2E-14.
  - Contradicting: t1049-fdistrt A2 `F.DIST.RT(0.5,3,7)` = 0.6940363875688136 and A8
    `F.DIST.RT(2.5,3,7)` = 0.14350945627893918 fit the w-then-1-w form bit for bit and are 2 ulp
    off with the exact complement. At (0.5,3,7), 1 - w = 0.17647058823529416 while
    d1x/(d1x+d2) = 0.17647058823529413.
  - A lead for d1 = d2 = 1: Excel's FDIST(x,1,1) cells match the arctangent closed forms.
    - 1 - (2/pi) atan(sqrt x) at x = 0.5 and 1E-6.
    - (2/pi) atan(1/sqrt x) at x = 3.5, 10 and 100.
    - 1 - (2/pi) asin(sqrt(x/(1+x))) at x = 1.

    So an a = b = 1/2 closed form reflected at the mean may explain A11 and A12 without this rule
    (UNVERIFIED).
- **Both tails come from one evaluation (PARTIAL).** The upper tail is the complement of that
  evaluation where it is reflected, and it is computed directly where it is not.
  `F.DIST.RT(1,3,7)+F.DIST(1,3,7,TRUE)` is exactly 1 (t1049-fdistrt A27, x = 1 at the mean);
  two separate evaluations miss by 6E-16. The w-then-1-w form explains A27 as well: 1 - w =
  0.30000000000000004, just past the mean 0.3, reflects.
- **The closed forms are the exception.** They read the upper tail from w itself:
  `F.DIST.RT(0.7,4,2.4)` = 1 - exp(2 ln(1-w)) (H3), 2 ulp off with the exact complement.

- **Accuracy against a 50-digit reference (2026-10-21 gap triage).** mpmath `betainc` at 50 digits,
  regularized, at y = d1*x/(d1*x+d2), plus the closed-form density. Excel is not uniformly correctly
  rounded on this family, and neither side is uniformly closer to the true value:
  - At larger shapes Excel is close to correctly rounded and the engine is the inaccurate side.
    `F.DIST(1,50,100,TRUE)` = 0.5108794921920699 (Excel +3.4E-17, ours -3.7E-15),
    `F.DIST(1,50,100,FALSE)` = 1.6223539167134502 (Excel -4.4E-17, ours +4.2E-15) and
    `F.DIST.RT(1,100,3)` = 0.6038137503819936 (Excel +3.9E-16, ours +3.9E-15) (probes-numeric/m9-f-dist
    A27, A28, m9-legacy-fdist A40). `F.DIST(3,50,100,FALSE)` is 1.05292998522241E-05, where ours is
    `#NUM!` (m9-f-dist A30).
  - Elsewhere Excel is off the true value by more than half a unit in the 15th digit. No accurate kernel
    reproduces these, only Excel's formulation:
    - `F.DIST.RT(10,10,10)` = 0.0005715525434020334, Excel +1.4E-15 (m9-f-dist-rt A16).
    - `FDIST(3.5,5,50)` = 0.008642212995925193, +7.1E-16 (m9-legacy-fdist A47).
    - `F.DIST(2.5,3,7,FALSE)` = 0.09478619896436714, +9.5E-16 (t1049-fdist A16).
    - `F.DIST.RT(10,50,100)` = 3.009600356083383E-22, -4.1E-15 (m9-f-dist-rt A20).
  - At d1 = d2 = 1 and x = 1 the exact value is 0.5, yet Excel gives `F.DIST(1,1,1,TRUE)` =
    0.5000000000000001 and `F.DIST.RT(1,1,1)` = 0.4999999999999999 (t1049-fdist A7, m9-legacy-fdist A31).
  - Tasks: T-2219 (the accuracy cells, visible parity) and T-2221 (the fit, after the T-2220 sweep).
- **Accurate tails at half-integer shapes from a + b = 10 (T-2219).** Outside the closed forms and
  the ported bratio branches, half-integer shapes with 10 <= a + b < 171.6 now take x^a y^b (a ln x +
  b ln y in double-double, the argument above 1/2 as 1 minus its complement) times F.TEST's
  double-double series of positive terms, instead of Cephes' prefactor and fraction, which were up
  to about 40 ulp off a 50-digit reference (`F.DIST.RT(1,50,100)`: Excel 0.48912050780793015,
  Cephes 0.48912050780793204, now 0.48912050780793037; m9-f-dist-rt A18). Over every suite
  (`make eval`, against origin/main d6618cd): 172 probes-numeric F.DIST, F.DIST.RT, FDIST and
  BETA.DIST cells move from visibly off to within 15 digits (e.g. `F.DIST.RT(1,100,3)`
  m9-legacy-fdist A40, `F.DIST(1,50,100,TRUE)` m9-f-dist A27, `F.DIST(0.5,50,100,TRUE)` A25), 105
  move onto Excel's bit and 23 off it. No gated known-failure line changes; two gen-m9 known
  failures move closer to Excel (`F.DIST.RT(WEEKDAY(13197,1),3,100)` c2974c061384fdd9 A165,
  `FDIST(3.14159,100,7)` a5828d3925575713 A51). Thirteen cells move to visibly off, and on each of
  them Excel is 5 to 50 ulp off the 60-digit value while the new value is within 5 (Excel / new, in
  ulp): `F.DIST.RT(10,1,50)` and `F.DIST(0.1,50,1,TRUE)` (t2220-f-dist-sweep-rt A89, -cdf A801,
  +13 / +1), `F.DIST.RT(10,3,100)` and `F.DIST(0.1,100,3,TRUE)` (A299, A921, +13 / -4),
  `F.DIST.RT(5,3,50)` (A288, +6 / -1), `F.DIST(0.0356,100,1,TRUE)` and `F.DIST(0.24,50,1,TRUE)`
  (t2220-f-dist-sweep-d2-1 A754, A670, -50 / +5 and +10 / -1), `BETA.DIST(0.5,100,50,TRUE)`
  (m9-beta-dist-a100 A45, -35 / 0), t0969-t-beta-link E159 (-15 / 0) and t1745-beta-inv-roundtrip
  G150, H150, G158, G159 (-14 / 0, -16 / -1, +5 / 0, +18 / 0). Cephes erred the same way as Excel
  there: they are Excel's own formulation of bratio's branch, not an accuracy gap (T-3199).
- The kernel takes integer shapes too, so BETA.DIST's cdf at whole shapes from a + b = 10 changed
  with it (44 BETA.DIST cells and 16 `E../2` link cells fixed visibly, 6 moved visibly off, all
  among the 13 above); T-2119's own counts predate this.
  - Below a + b = 10 the series only trades bits (35 cells off Excel's bit, 75 onto it, 6 visibly
    fixed), among them the gated `FDIST(2,1,9)` (gen-m9/c18dbf21f11093f3 A151) and
    `F.DIST(1,3,8,1)` (c6cfda6bd10cae93 A348) 1 ulp off, so Cephes stays there.
  - BETA.INV's and F.INV's searches keep crossing Cephes' tails (`incomplete_beta::search_tails`):
    with the series under them 7 gated gen-m9 quantile cells move off Excel's bit
    (`FINV(0.025,7,10)` fec39efaba9284c3 A336, `F.INV.RT(0.975,9,9)` 40425341c5034eed A205) and
    BETADIST_BETAINV.xlsx H67 (3 becomes 2.9999999999999996), while 22 gen-m9 quantile cells would move from
    visibly off to within 15 digits and 3 the other way. Where Excel's search stops is T-2511's.
  - Still off: `F.DIST.RT(3,50,100)` (m9-f-dist-rt A19) is on the ported L140 bup + bpser branch,
    and the densities `F.DIST(0.5,50,100,FALSE)` and `F.DIST(3,50,100,FALSE)` (m9-f-dist A26, A30,
    a number since T-2586) are Excel's density model (T-3034), not this kernel (T-3198).

The engine follows both PARTIAL rules (the T-2220 sweep contradicts the strong form of the second). Against statrs, the probes-numeric F and BETA cases gain
72 bit-exact cells and lose 3, a net gain of 69. The three lost cells are now 1 ulp off:
`FDIST(100,1,1)` and `F.DIST.RT(100,1,1)` (m9-legacy-fdist A61, A62: 0.06345103486110715) and
t1049-fdist A23 (0.979990011905381).

Densities (VERIFIED on the cells named; the general form UNVERIFIED):
- BETA.DIST(x,a,b,FALSE) as pow(t,a-1)*pow(1-t,b-1)*Gamma(a+b)/(Gamma(a)*Gamma(b))/(B-A), musl
  `tgamma`: bit-exact on `BETA.DIST(3,2,3,FALSE,1,10)` and `BETA.DIST(2,8,10,FALSE,1,3)` =
  1.4837646484375 (`BETADIST_BETAINV.xlsx` G3, `BETA_GAMMA.xlsx` G4), 2 ulp on
  `BETA.DIST(0.234,2,2.5,FALSE,0.15,1.2)` (G43, G45); exp((a-1) ln t + (b-1) ln(1-t) - ln B)
  misses G4 by 4 ulp.
- F.DIST(x,d1,d2,FALSE) as that beta density at y = d1*x/(d1*x+d2) times y(1-y)/x: 1 ulp on
  `F.DIST(0.7,4,2.4,FALSE)` (F3: Excel 0.40509259259259256, 1 ulp below 175/432), statrs 28 ulp.

The cdf, BETA.DIST(x,a,b,TRUE), with the best combination ported to the engine
(`statistical/incomplete_beta.rs`): of the 33 cumulative BETA.DIST cells in
`BETADIST_BETAINV.xlsx` and `BETA_GAMMA.xlsx` that statrs misses, 8 become bit-exact and 23 come
within 4 ulp. Two stay far:
- `BETA.DIST(33587,3,600,TRUE,2000,5489582)` (G5) is the correctly rounded 0.6734553797493481.
  Cephes' fraction loses about 160 ulp there (t = 0.0058, reflected to I_0.994(600,3)), and so do
  Numerical Recipes' fraction and Cephes' power series.
- `BETA.DIST(0.234,0.0001,0.0001,TRUE,0.15,1.2)` (G26) is 0.4998779049300665, 34 ulp from the
  correctly rounded value. The best combination is 30 ulp off.

For a + b of 171.6 or more, the fit's logarithmic prefactor (ln Gamma near 3,000 cancelling) loses
about 4,000 ulp on `BETA.DIST(0.555,1000,1000,TRUE,0.1,1)` (G29). TOMS 708's `brcomp` form brings
it to 3 ulp of Excel's value: the exponent as -(a rlog1(-lambda/a) + b rlog1(lambda/b)) with
Stirling corrections, or `algdiv` when one shape is below 8.

Moving the BETA.DIST cdf to this kernel also moves nine BETA.INV(BETA.DIST(...)) cells in
`BETADIST_BETAINV.xlsx` (H2, H14, H42, H44, H46-H49, H67) from bit-exact to 1-2 ulp, with
statrs' inversion or with the inversion run on the new cdf. Under statrs they passed by chance,
with both functions wrong in the same way. Excel's own pair lands on x exactly at H2 (3) and H14
(0.7) but not at H23 (-0.23399999999999987) or H42 (0.23399999999999999), as T-1745 found. The
switch waits for T-2119.
Status: VERIFIED (F.DIST.RT and F.TEST are not 1 - cdf: F_DIST H2, H3, F_TEST F1, F3; the
densities on the cells named); PARTIAL (exact complement, one-evaluation tails; the T-2220 sweep contradicts the
strong form of the latter, see below); the cdf algorithm stays UNVERIFIED (T-1744).
Tasks: T-0970, T-2119, T-1744, T-1745, T-2219, T-2220, T-2221

### Dense F sweep (T-2220)
Evidence: `probes-numeric/t2220-f-dist-sweep-{cdf,pdf,rt}` (100 df pairs d1, d2 in {1,2,3,4,5,7,10,20,50,100}
x x in {0.1,0.25,0.5,1,1.5,2,3,5,10,100}: 1,000 cells each) and `t2220-f-dist-sweep-d1-1`, `-d2-1`
(d1 = 1 or d2 = 1, 30 x points over 1E-6..1E6: 900 and 810 cells). All 4,710 cells are numbers, no
errors. Compared against a 50-digit reference (mpmath regularized incomplete beta, density from the
closed form) rounded to a double:
- Reference: the 50-digit values are evaluated at the double x Excel receives
  (`docs/research/t2220_f_dist_sweep_stats.py` reproduces every figure here); the upper tail is computed
  directly as I_{d2/(d2+d1 x)}(d2/2, d1/2), not as 1 - cdf, which cancels below about 1E-33.
- Excel is not correctly rounded anywhere: no (d1, d2) pair, for any of F.DIST TRUE, F.DIST FALSE or
  F.DIST.RT, has all its cells equal to the correctly rounded double, and that includes d1 = d2 = 1
  and d1 = 1 or d2 = 1 with closed forms available.
- Cells equal to the correctly rounded value: cdf 765/1570 (49%), RT 607/1570 (39%), pdf 147/1570
  (9%). The density is the least accurate: 4/10 at the (2,2) pair and 1/40 at d1 = d2 = 1.
- Error size: within 4 ulp are cdf 1,419/1,570 (90%), RT 1,318/1,570 (84%) and pdf 1,052/1,570 (67%).
  Worst relative error: cdf 2.7E-14, RT 2.8E-14, density 7.8E-14. The largest RT error is 196 ulp at
  `F.DIST.RT(1E6,1,50)`; 125 RT cells are 10 or more ulp off, at every magnitude of p (e.g. 35 ulp at
  (1.61,1,50) where p = 0.21), not only in far tails. The cdf is at worst 201 ulp off and the density 619 ulp.
  A T-2221 acceptance bar of "bit-exact or within 4 ulp" therefore cannot cover all of this grid.
- `F.DIST(x,d1,d2,TRUE) + F.DIST.RT(x,d1,d2)` is exactly 1.0 in 1,486 of 1,570 cells and not in 84
  (all 84 such sums are exactly 0.9999999999999999), e.g. (x,d1,d2) = (100,1,3), (5,1,4),
  (10,1,50), (0.1,3,1), (100,3,3), (0.1,3,5), (100,3,7), (3,3,10). This bears on the second PARTIAL
  rule above (both tails from one evaluation): its strong form, an exactly-1 sum, fails in 5% of cells.
  It does not test the first rule (how the beta argument's complement is formed, d2/(d1 x + d2) directly
  versus 1 - w); that stays open for T-2221.
- d1 = d2 = 1: `F.DIST(1,1,1,TRUE)` = 0.5000000000000001 and `F.DIST.RT(1,1,1)` =
  0.4999999999999999 reproduce on the sweep (also F.DIST(3,1,1,TRUE) = 0.6666666666666666 and
  F.DIST.RT(3,1,1) = 0.33333333333333337); `F.DIST.RT(10,1,1)` and `F.DIST(0.1,1,1,TRUE)` are the same
  double (0.19498222904213666), consistent with the reflection F.DIST.RT(x) = F.DIST(1/x) at d1 = d2.
Status: VERIFIED (observation, with the case ids above); the strong form of the one-evaluation rule is
contradicted (84 of 1,570 sums are not 1), the exact-complement rule is untested here; the algorithm
stays UNVERIFIED (T-2221).

## BETA.INV solves Excel's own CDF to the crossing (T-1745)
Rule: BETA.INV(p,a,b) returns, in most cases, a double where Excel's own BETA.DIST(.,a,b,TRUE)
crosses p. The probe t1745-beta-inv-roundtrip has 188 rows: 187 distinct in-domain sweep points
plus one duplicate, (0.5,0.5,0.5). It leaves out the 5 points where Excel returns x = 1, which
have no neighbour above. The crossing holds at 155 of the 188:
- BETA.DIST at Excel's x equals p exactly at 79 points.
- p lies between BETA.DIST at x and at a neighbouring double at 76 points. In 36 of these x is at
  or just above the crossing, and in 40 it is the last double below it, so there is no
  one-sided rule.
- At 22 of the other 33, Excel's CDF is flat across all three doubles (tiny x, for example
  BETA.INV(1E-10,0.5,1) = 9.999999999999992E-21).
- At the last 11, x misses the crossing by 1 to 188 ulps of p, on both sides (8 above, 3 below).
  For example, BETA.DIST(BETA.INV(0.05,2.5,10)) is 32 ulps off and BETA.DIST(BETA.INV(0.05,2.5,100))
  is 188 ulps off, about 82 doubles from the correctly rounded quantile (rows 91 and 98). BETA.INV(0.5,10,10) is 0.5 while
  BETA.DIST(0.5,10,10,TRUE) is 0.5000000000000002. This suggests a tolerance stop, but it is not
  established.
- Recomputing BETA.INV from constant inputs reproduces x at all 188 rows.
Consequence: BETA.INV parity follows BETA.DIST parity (T-1744) plus a solver that usually
converges to the crossing.

Best candidate: a bisection to adjacent doubles on the correctly rounded I_x, bit-exact on 87 of
212 cells (202 with numeric arguments). It takes the bracket's "midpoint", which for adjacent doubles
rounds (ties to even) to whichever end has an even significand, so its edge over the one-sided ends
(75, 54) is not evidence of an Excel rule. For comparison:
- the same solver on the T-1744 best combination: 65;
- on Cephes incbet: 73;
- the engine (statrs `inverse_cdf`): 34.
Through the F and t mappings of T-1744 (F df truncated) the same solver is far from Excel's values,
because no forward candidate is Excel's CDF. It still beats the engine:
- F.INV: 8 of 54, against 2;
- F.INV.RT/FINV: 11 of 121, against 9;
- T.INV: 29 of 130, against 17;
- T.INV.2T/TINV: 25 of 162, against 9.
Evidence: probes-numeric m9-beta-inv, m9-legacy-betainv, t1745-beta-inv-roundtrip (Excel's x
and its neighbours as constants, BETA.DIST of each, and BETA.INV again), m9-f-inv, t1049-finv,
m9-f-inv-rt, m9-legacy-finv, t1049-finvrt, m9-t-inv, m9-t-inv-2t, t1048-tinv. Fits `beta-inv`,
`f-inv`, `f-inv-rt`, `t-inv`, `t-inv-2t` (`fitlab/kernels/beta_inv.py`).
Status: VERIFIED (crossing at 155/188); the 11 misses and the solver's start, step and stop are UNVERIFIED
Tasks: T-1745, T-0974

## Incomplete gamma sweep (T-1045)
Rule: Excel's incomplete-gamma values (the targets for T-0971 and T-0972) are plain numbers on the whole in-domain grid, and the CDF saturates to exactly 1 deep in the upper tail while the pdf stays finite.
Oracle sweep (excel-graph) of `GAMMA.DIST` (a in {0.5, 1, 2, 9, 50, 500}, b in {1, 2, 1E6}, x = 0.1, 0.5, 1, 1.5, 3 times the mean a*b,
cum TRUE/FALSE), `CHISQ.DIST` and `CHISQ.DIST.RT` (k in {1, 2, 3, 5, 10, 50, 100}), `GAMMA.INV` (a as above, b in {1, 1E6})
and `CHISQ.INV` / `CHISQ.INV.RT` (p from 1E-10 to 0.9999999999). All 467 cells are numbers: no error appeared on the in-domain grid.
- Saturation: `GAMMA.DIST(1500000000,500,1000000,TRUE)` = 1 exactly while the pdf is 1.0974301388641694E-204.
- Deep lower tail is not flushed to zero: `GAMMA.DIST(50,500,1,TRUE)` = 5.364386963522797E-307 and the pdf there is 4.829137127185113E-306.
- Small-p inverse keeps full relative precision: `CHISQ.INV(1E-10,1)` = 1.5707963267948968E-20,
  `GAMMA.INV(1E-10,0.5,1)` = 7.853981633974484E-21 (exactly half, as the scale relation says).
- Large p: `CHISQ.INV(0.9999999999,1)` = 41.82145620298276, `GAMMA.INV(0.9999999999,500,1000000)` = 655651503.8005778.
Evidence: probes-numeric cases `m9-gamma-dist-a<a>` (one per a; `p` stands for the decimal point, e.g. `a0p5`), `m9-chisq-dist`,
`m9-chisq-dist-rt`, `m9-gamma-inv`, `m9-chisq-inv`, `m9-chisq-inv-rt`. Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-1045 (sweep), T-0971, T-0972

## WEIBULL.DIST sweep (T-1046)
Rule: Excel's `WEIBULL.DIST` returns a plain number on the whole in-domain grid, including x = 1E-10 and the far tail; the values are the targets for T-0978.
Oracle sweep (excel-graph) of `WEIBULL.DIST(x,k,l,cum)` with k in {0.5, 1, 2, 3.7}, l in {1, 9}, x in {1E-10, 0.001, 0.1, 0.5, 1, 2, 5, 10, 30, 100, 1000} and
cum TRUE/FALSE (176 cells). No error appeared on the grid.
- Small x keeps full relative precision: `WEIBULL.DIST(1E-10,1,1,TRUE)` = 9.999999999499998E-11 and `WEIBULL.DIST(1E-10,0.5,1,TRUE)` = 9.999950000166663E-06
  (the cdf is not computed as `1 - EXP(-t)` with a cancelled difference).
- The pdf at x -> 0 for k < 1 is large but finite: `WEIBULL.DIST(1E-10,0.5,1,FALSE)` = 49999.5000025.
- Far tail does not snap to 1: `WEIBULL.DIST(1000,0.5,1,TRUE)` = 0.9999999999999816 (pdf 2.9199312344877555E-16), `WEIBULL.DIST(1000,0.5,9,TRUE)` = 0.999973567745545.
Evidence: probes-numeric case `m9-weibull-dist` (cells A1:A176). Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-1046 (sweep), T-0978

### The far-tail density is less accurate than exp(-t): t = (x/l)^k is EXP(k*LN(x/l)) (rule, T-2239)
Rule. WEIBULL.DIST and WEIBULL evaluate, with Excel's own LN and EXP kernels (`excel_ln`, `excel_exp`; numbers.md) and
each operation a double:
- t = EXP(k*LN(x/l)), not a correctly rounded power;
- the density as `k/EXP(k*LN(l)) * EXP((k-1)*LN(x)) * EXP(-t)`, left to right: k over l^k, not `k/l*(x/l)^(k-1)`;
- the cdf as minus Excel's expm1 of -t (Kahan's formula, numbers.md "Excel's expm1 is Kahan's formula") below t = 1, and
  as `1-EXP(-t)` from t = 1 on (next entry).

In the tail, Excel's density is off the true value by about one rounded logarithm's error times t. Match it; do not
correct it.
- `WEIBULL.DIST(100,1,1,FALSE)` = 3.7200759760206773E-44, while exp(-100) = 3.720075976020836E-44
  (4E-14 relative; m9-weibull-dist A64).
- `WEIBULL.DIST(1000,1,9,FALSE)` = 6.177532755504216E-50 (A88).
- `WEIBULL.DIST(5,3.7,1,FALSE)` = 9.363099887555361E-166 (A146).
- The l^k divisor shows away from the tail too: `WEIBULL(10,3,8,FALSE)` is 0.08310360884023987 (t1048-weibull A8, B8),
  where the true value, 0.08310360884023976, is what `k/l*(x/l)^(k-1)*exp(-t)` gives.

Fit (T-2239), over the in-domain cells with literal arguments (x, k, l > 0; 123 cdf, 122 density) of m9-weibull-dist,
t1048-weibull, t1049-weibulldist, the gen-m9 and probes cells. The cdf rows and the rule's density row use Excel's
kernels; the other density rows are libm re-runs:

| form | visibly off | bit-exact |
|---|---|---|
| cdf, the rule above | 0 | 121 (m9 A69 1 ulp; t2706 A34 is subnormal, flushed to 0 in the cell) |
| cdf `-expm1(-t)` (Excel's expm1) throughout | 0 | 112 (loses near-1 cells: m9 A57, A85, A127, gen-m9 5919660b7bb4d195 A296, ...) |
| cdf `-expm1(-t)`, libm expm1 | 0 | 99 |
| cdf `1-EXP(-t)` throughout (loses the small-x cells) | 24 | |
| cdf, statrs form `-expm1(-x^k*l^-k)` (the engine before T-2239) | 1 (m9 A159) | 80 |
| density, the rule above | 0 | 120 (t1048 A14, B14 1 ulp) |
| density `k*exp((k-1)*ln x - k*ln l)*exp(-t)` | 2 (m9 A156, A158) | 97 |
| density `k/l*exp((k-1)*ln(x/l))*exp(-t)` | 4 (m9 A158, A168; t1048 A8, B8) | 72 |
| density `k/l*(x/l)^(k-1)*exp(-t)`, powf | 3 (m9 A168; t1048 A8, B8) | 60 |
| density `k*t/x*exp(-t)` | 7 | 49 |
| density, statrs form `k/l*(x/l)^(k-1)*exp(-x^k*l^-k)` (the engine before T-2239) | 17 | |

t computed as `exp(k*(ln x - ln l))`, `x^k/l^k` or `exp(k ln x)/exp(k ln l)` loses 8 density cells. Libm's `exp`/`log` in
place of Excel's kernels give the same counts for the rule's density on this set, so it does not separate them; the
engine uses Excel's.
The same t gives ironcalc-calc WEIBULL_DIST.xlsx E2 and E3 exactly (`WEIBULL.DIST(0.8,1,9,TRUE)` = 0.08505277126996899).
Evidence: probes-numeric m9-weibull-dist, t1048-weibull, t1049-weibulldist, t2239-weibull-cdf-switch (excel-graph); gen-m9
(T-2256 cells, e.g. 5389dd4d5b5739d2 A378, 5cebac1f02bf30f2 A336); ironcalc-calc WEIBULL_DIST.xlsx E2, E3 (excel-cached,
Microsoft Macintosh Excel).
Ours: `functions/statistical/weibull.rs` (`scaled_power`, `weibull_cdf`, `weibull_density`).
Status: VERIFIED (excel-graph) to 15 digits on every cell above; the 3 cells off by 1 ulp stay with T-0978.
When x/l underflows to 0 (e.g. `WEIBULL.DIST(1E-300,2,1E+100,TRUE)`), the engine keeps t = 0, as before; Excel's
LN(0) would be #NUM!, so its answer here may be #NUM! or 0: UNVERIFIED, no probe yet.
Tasks: T-2239 (15 digits, done), T-0978 (last digits)

### WEIBULL.DIST's cdf switches from expm1 to 1-EXP(-t) near t = 1 (T-2239)
Rule. Below t = 1 the cdf is minus Excel's expm1 of -t; above, `1-EXP(-t)`. The switch lies between t = 0.99868 and
t = 1.35: on a 0.0003 grid of t over (0.99, 1.36) the two forms differ in only 29 cells, all below 0.99868 (expm1) or
above 1.35 (`1-EXP(-t)`), so the exact cut is not observable from these cells; the engine cuts at t = 1.
- `WEIBULL.DIST(0.9917,1,1,TRUE)` = 0.6290544527285032 (expm1; A113), `WEIBULL.DIST(1.3506,1,1,TRUE)` =
  0.7409152368565803 (`1-EXP(-t)`; A114).
- All 291 cells of the probe are cells where the two forms differ in the last bit: 226 below the cut match expm1, 65 above
  it match `1-EXP(-t)`, for k = 1 and k = 2 alike.
Evidence: probes-numeric t2239-weibull-cdf-switch (excel-graph, 2026-10-28), m9-weibull-dist A57, A85, A127.
Ours: `functions/statistical/weibull.rs` `weibull_cdf`.
Status: VERIFIED (excel-graph)

### WEIBULL's density divides by l^k: #DIV/0! when it underflows, #NUM! when it overflows (T-2239)
Rule. The density (cumulative FALSE) is `#DIV/0!` when l^k = EXP(k*LN(l)) underflows to 0 and `#NUM!` when it
overflows, even where the density's limit is 0. The cdf does not form l^k and stays a number there; both forms are
`#NUM!` when t = (x/l)^k overflows.
- `WEIBULL.DIST(0.1,10000000000.5,0.2,FALSE)` is `#DIV/0!` (gen-m9/941c409532603d32 A142), as is
  `WEIBULL(1E-15,<about 42843>,<about 0.32>,FALSE)` (0620505280d2ea6e A61).
- `WEIBULL.DIST(0.3,99999,99999,"FALSE")` (a63d1c7d6f96c56d A352), `WEIBULL(1,1E+8,1E+10,FALSE)` (7fd0a98183c6babf
  A390), `WEIBULL.DIST(2,100,10000000000.5,FALSE)` (9bc247733a1ca9fa A108) and `WEIBULL.DIST(1E-10,29178,1.5E+10,"FALSE")`
  (f82473275380882e A356) are `#NUM!`.
- t overflows: `WEIBULL.DIST(100,1E+10,0.01,1)` and `WEIBULL(1E+300,2.5,100,0.5)` are `#NUM!` (0bbab6206b5c9a07 A340,
  329b177b5d5165ad A101), while `WEIBULL(1E+15,42,10000000001,TRUE)` is a number (7f7ca726c3f68554 A308).
Evidence: gen-m9 cells above (excel-graph, T-2256 triage).
Ours: `functions/statistical/weibull.rs` `weibull_density`.
Status: VERIFIED (excel-graph)

## Legacy distribution names sweep (T-1047)
Rule: each legacy name returns exactly the same value or error as its modern equivalent on every paired cell (BETADIST = BETA.DIST cum, BETAINV = BETA.INV, BINOMDIST = BINOM.DIST, CHIDIST = CHISQ.DIST.RT, CHIINV = CHISQ.INV.RT, CHITEST = CHISQ.TEST, CONFIDENCE = CONFIDENCE.NORM, CRITBINOM = BINOM.INV, EXPONDIST = EXPON.DIST, FDIST = F.DIST.RT, FINV = F.INV.RT, FTEST = F.TEST); the aliases need no separate kernel.
Oracle sweep (excel-graph), 12 cases, 33-92 cells each. Aliases were paired with the modern name on the typical-value grid only; the edge, coercion and empty-reference bullets below were probed on the legacy names only. Quirks seen:
- Counts and degrees of freedom truncate: `BINOMDIST(2.9,10,0.5,TRUE)` = `BINOMDIST(2,...)` = 0.0546875, `CHIDIST(3,2.9)` = `CHIDIST(3,2)`, `FDIST(1,2.9,5)` = `FDIST(1,2,5.9)` = `FDIST(1,2,5)`, `CONFIDENCE(0.05,1,100.9)` = size 100, `CRITBINOM(10.9,0.5,0.5)` = 5. `CONFIDENCE(...,0.9)` (size truncates to 0) is #NUM!.
- df above 1E10 is #NUM! for CHIDIST, FDIST (both df) and FINV (`CHIDIST(3,10000000001)`, `m9-legacy-chidist`/`-fdist`/`-finv`); `CHIDIST(3,10000000000)` = 1 and `FDIST(1,10000000000,5)` = 0.5841198129434723 are accepted (the 1E10 boundary was not probed for FINV or FDIST's second df). CHIINV has no such cutoff: `CHIINV(0.5,10000000001)` = 10000000000.333334 and `CHIINV(0.5,10000000000)` = 9999999999.333334 (`m9-legacy-chiinv`), about df - 2/3 at p = 0.5.
- Boundary p: `CHIINV(1,..)` and `FINV(TRUE,2,5)` are 0 (p = 1 allowed), p = 0 is #NUM!; `BETAINV(0,..)`/`BETAINV(1,..)` are both #NUM! (open interval).
- BINOM.INV/CRITBINOM: alpha 0 or 1 is #NUM!, and probability 0 or 1 is #NUM!, even though BINOMDIST accepts them (`BINOMDIST(0,5,0,TRUE)` = 1).
- BETADIST with bounds: x outside [A,B] is #NUM! (`BETADIST(0.5,2,3,2,5)`), A = B is #NUM!, but x = A gives 0 (`BETADIST(2,2,3,2,5)`); x outside [0,1] without bounds is #NUM!.
- Coercion: text numbers (`"0.5"`) and booleans coerce in scalar arguments (`BETADIST(TRUE,2,3)` = 1, `CHIDIST(TRUE,2)` = 0.6065306597126334); an empty reference is 0 (`BETADIST($Z$1,2,3)` = 0, `EXPONDIST($Z$1,1,TRUE)` = 0, `CHIDIST($Z$1,2)` = 1) so a zero df/lambda/p is #NUM!. The `cumulative` flag takes any nonzero number as TRUE (`EXPONDIST(1,1,5)`, `BINOMDIST(2,10,0.5,5)`) and the text "TRUE".
- CHITEST/FTEST: different cell counts are #N/A (`CHITEST(B1:D2,B4:D4)`); a zero expected cell is #DIV/0!; a negative summed statistic is #NUM! (a negative expected cell alone is not an error, T-2431); FTEST with a zero-variance range or fewer than 2 numbers in a range is #DIV/0!; array literals are accepted.
- CHITEST of a 1x1 range is #N/A (`CHITEST(N1:N1,G1:G1)`). A text cell in the actual range drops that pair but the degrees of freedom still come from the range shape: `CHITEST(F1:F3,G1:G3)` with F = {5,0,"x"} = 0.00012340980408667956 (= CHIDIST(18,2)).
Evidence: probes-numeric cases `m9-legacy-betadist`, `-betainv`, `-binomdist`, `-chidist`, `-chiinv`, `-chitest`, `-confidence`, `-critbinom`, `-expondist`, `-fdist`, `-finv`, `-ftest`. Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-1047

## Legacy distribution names sweep, second family (T-1048)
Implemented in: `functions/statistical/hypegeom.rs`, `weibull.rs` (HYPGEOMDIST/HYPGEOM.DIST sample_s above min(number_sample, population_s) is 0 or 1, zero counts valid, TRUE sample_s; WEIBULL density 0 at x = 0, TRUE x)
Tasks: T-2035
Rule: GAMMADIST, GAMMAINV, HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TINV, WEIBULL return exactly the same value or error as their modern equivalent (GAMMA.DIST, GAMMA.INV, HYPGEOM.DIST with cumulative FALSE, LOGINV = LOGNORM.INV, LOGNORM.DIST cum TRUE, NEGBINOM.DIST cum FALSE, NORM.DIST, NORM.INV, NORM.S.DIST cum TRUE, NORM.S.INV, POISSON.DIST, T.INV.2T, WEIBULL.DIST) on every paired cell, so the aliases need no separate kernel. TDIST with tails 1 is the exception: it differs from T.DIST.RT for negative x (T.DIST.2T agrees with TDIST tails 2). TTEST = T.TEST and ZTEST = Z.TEST on every paired cell too.
Oracle sweep (excel-graph), 16 cases, 34-102 cells each; every paired cell (column B) is identical except the TDIST row below. Quirks seen:
- `TDIST(-1,3,1)` and `TDIST(-1,3,2)` are #NUM! (x must be >= 0); `T.DIST.2T(-1,3)` is #NUM! too, but `T.DIST.RT(-1,3)` = 0.8044988905221147. `TDIST(1,0.9,1)` is #NUM! (df truncates to 0); `TDIST(1,1.9,2)` = 0.5000000000000001 (= `T.DIST.2T(1,1.9)`, df truncates to 1); tails truncate (`TDIST(1,3,1.9)` and `TDIST(1,3,TRUE)` = tails 1, `TDIST(1,3,2.1)` = tails 2); tails 0 and 3 are #NUM!, and an omitted tails (`TDIST(1,3,)`) is #NUM!.
- Boundary p differs by function: `GAMMAINV(0,2,1)` = 0 but `GAMMAINV(1,2,1)` is #NUM!; `LOGINV` and `NORMINV` reject both 0 and 1 (#NUM!); `NORMINV(1e-300,0,1)` = -37.0470962993612.
- A boolean p is #NUM! (`GAMMAINV(TRUE,2,1)`, `LOGINV(TRUE,0,1)`, `NORMINV(TRUE,0,1)`) because TRUE coerces to 1, while a boolean x coerces to 1 (`NORMDIST(TRUE,0,1,TRUE)`, `GAMMADIST(TRUE,2,1,TRUE)`) and text numbers coerce everywhere; non-numeric text is #VALUE!.
- The `cumulative` flag of GAMMADIST/NORMDIST takes any nonzero number as TRUE, the text "TRUE", and an omitted (empty) argument as FALSE.
- Zero or negative scale is #NUM! (`NORMDIST(0,0,0,TRUE)`, `GAMMADIST(1,2,0,TRUE)`); `GAMMADIST(0,2,1,..)` = 0 for both cum values; `LOGNORMDIST(0,0,1)` is #NUM!; far tails flush to 0 (`NORMSDIST(-38)` = 0, `NORMDIST(-40,0,1,..)` = 0) while `NORMSDIST(-37)` = 5.7255712225239266E-300.
- NEGBINOMDIST with p = 1 is #NUM! even for zero failures (`NEGBINOMDIST(0,3,1)`); successes = 0 is #NUM!. HYPGEOMDIST with sample_s > number_sample is 0, not #NUM! as documented (`HYPGEOMDIST(5,4,8,20)` = 0, `HYPGEOMDIST(1,0,8,20)` = 0); population_s > number_pop is #NUM! (`HYPGEOMDIST(1,4,25,20)`, `HYPGEOMDIST(1,4,8,5)`), and so is number_pop = 0 (`HYPGEOMDIST(1,4,8,0)`, where population_s and number_sample also exceed it); number_sample > number_pop alone is #NUM! for HYPGEOM.DIST (`t1049-hypgeomdist` A40, `(1,25,8,20)`).
- TTEST: tails 0/3 and type 0/4 are #NUM!, tails accepts a boolean (`TTEST(..,TRUE,2)`) and text numbers, tails/type truncate, an omitted tails or type is #NUM!; unequal counts with type 1 (paired) is #N/A; type 1 on a pair whose differences have zero variance is #DIV/0!; a one-element range is #DIV/0!; an empty range is #DIV/0!.
- ZTEST: sigma 0 or negative is #NUM!; a one-element range is #DIV/0! (with or without sigma); an empty range is #N/A; text x "5" and TRUE coerce, non-numeric text is #VALUE!; an omitted sigma (empty argument) is #NUM!, while an omitted x (`ZTEST(D1:D10,)`) returns a value (1.3161580317707342E-09; that it equals x = 0 was not probed); a range as x lifts and spills (see "Statistical function argument modes").
Evidence: probes-numeric cases `t1048-gammadist`, `-gammainv`, `-hypgeomdist`, `-loginv`, `-lognormdist`, `-negbinomdist`, `-normdist`, `-norminv`, `-normsdist`, `-normsinv`, `-poisson`, `-tdist`, `-tinv`, `-ttest`, `-weibull`, `-ztest`. Oracle: excel-graph, 2026-10-14.
Status: VERIFIED (excel-graph)
Tasks: T-1048, T-2035

## Incomplete beta at a huge first shape: Excel is close to the true value, but not within an ulp (T-2528)
Rule: at a first shape of 5E7 to 5E9 with a second of 2.5 (the F tails at df1 = 5, df2 = 1E8 to 1E10, above the mean,
and the matching BETA.DIST reflections) Excel's incomplete beta is the true value at the double arguments to within
about 1E-15 relative where the tail is above 1E-4 (x = 2, 5 on the F scale), and up to 1.5E-14 off it further out:
`F.DIST.RT(50,5,1E10)` is 5.4969308523163854E-52 against the true 5.496930852316323E-52, `F.DIST.RT(2,5,1E9)`
0.07523524713822176 against 0.07523524713822164 (120-digit mpmath `betainc` at u = fl(5x/(5x + df2)); at 60 digits
mpmath itself is 4E-14 off at x = 50). `F.DIST(2,5,1E10,TRUE)` = 0.9247647537543169 is the true value rounded.
- Excel's error grows with z = -nu ln x (nu = a + (b - 1)/2, z about 125 at x = 50), the argument of the incomplete
  gamma in TOMS 708 `bgrat`, which is the branch bratio takes there (L140 bup + bgrat): consistent with z formed with a
  few ulp of error in Excel's own LN/log1p, not pinned.
Evidence: probes-numeric t2528-f-dist-huge-df2 (60 cells, excel-graph, 2026-10-30), t1530-f-df-cut A8, A12, A14.
Ours: since T-2528 DiDonato and Morris' expansion in Q(b, z) (`large_shape_beta.rs`) for a of 1E6 or more, b below
40 (bratio's bgrat family; from 40 Excel's other branches decide, `#NUM!` at `BETA.DIST(0.1,1E+200,1E+100,TRUE)`) and
b/a at most 1E-6, in accurate arithmetic (z and ln x in double-double, Legendre's fraction for Q/r from the bottom up):
within 2E-15 of the true value on the probe; Cephes' fraction there did not converge and was about 1E-7 off. 38 of
the 60 cells are within 15 digits of Excel; the other 22 are where Excel is itself more than half a unit of the 15th
digit off the true value (T-3209).
Status: VERIFIED (excel-graph) for the values; Excel's arithmetic inside bgrat is UNVERIFIED (T-3209, T-2510)
Implemented in: functions/statistical/large_shape_beta.rs `large_shape_lower`
Tasks: T-2528, T-3209

## F-distribution df limit (T-1530)
Rule: either df above 1E10 (after truncation) makes every F function `#NUM!`: F.DIST (cumulative and pdf), F.DIST.RT,
F.INV, F.INV.RT, FDIST and FINV, in both df positions (probed at 10000000001, 1.5E10 and 1E11). Unlike Student-t, the pdf is cut too.

The last digits at a df of 1E+10 are ours to fix: `F.DIST(1,10000000000.5,6,TRUE)` is
0.4231900476583052 against Excel's 0.4231900811940569 (probes/t2435-dist-extremes A35:A38, A40, A41;
T-2811).
Status: VERIFIED (excel-graph, 2026-10-14; probes-numeric case `t1530-f-df-cut`)

Implemented in: functions/statistical/fisher.rs `f_df_ok`, `f_quantile`

Tasks: T-1530, T-1538

- df = 1E10 and 10000000000.5 (truncates to 1E10) are accepted: `F.INV(0.5,1E10,5)` = 1.1490395521130337,
  `F.DIST(2,5,1E10,FALSE)` = 0.14167277676179968, `F.INV(0.5,1E10,1E10)` = 1.
- At df = 1E10 the left and right tail inverses are not mirror images: `F.INV(0.5,1E10,5)` = 1.1490395521130337 but
  `F.INV.RT(0.5,1E10,5)` = `FINV(0.5,1E10,5)` = 1.1490395839335394; `F.INV(0.5,5,1E10)` = 0.8702920369171896 but
  `F.INV.RT(0.5,5,1E10)` = 0.8702920610183031 (cells A1-A6).
- Ours: the hang is gone (the quantile is a bisection on the `statrs` cdf from df 1E5), but the cdf and quantile are off in
  about the 6th digit at df 1E10 (`F.INV(0.5,1E10,1E10)` = 1.0000025, Excel 1) and `F.DIST(...,FALSE)` at df 1E10 is
  `#NUM!` (`statrs` pdf) where Excel gives a number. 16 of 58 cells of the case fail (A1-A14, A57, A58); T-1538 closes them.

## F density at large degrees of freedom (T-1864)
Rule: F.DIST(x,d1,d2,FALSE) is an ordinary number wherever Excel's range steps leave the density positive (round 5
of "F density is not the beta density, R's df or a GAMMALN formula (T-2513)": Y = density·x is stored, F = fl(Y/x),
an F below 2^-1022 is 0), however large the degrees of freedom (up to the 1E10 cut, "F-distribution df limit (T-1530)"): `F.DIST(1,100,100,FALSE)` 1.9897309346794694,
`F.DIST(3,50,100,FALSE)` 1.05292998522241E-05, `F.DIST(28.1,100,1,FALSE)` 0.0026239848297062718,
`F.DIST(2,5,1E10,FALSE)` 0.14167277676179968, `F.DIST(2,1E10,5,FALSE)` 0.18825249234504016, and at moderate x with one
df above about 143 (probes-numeric t1864-f-pdf-moderate-x, 10 cells, e.g. `F.DIST(20,100,3,FALSE)`
0.0010807581083264...). Excel's value is its own formula (T-2586), not the exact density: it is up to about 6.5E-14
relative off the exact density (mpmath), so even the exact density is within 15 digits of Excel on only 26 of the 44
literal-argument cells of t2220-f-dist-sweep-pdf/-d2-1, m9-f-dist and t1530-f-df-cut.
- The edges are numbers too (probes-numeric t1864-f-pdf-edges): a huge x with a small df
  (`F.DIST(1E10,30,5,FALSE)` 8.379700461247534E-35, `F.DIST(1E20,300,1,FALSE)` 3.986099675993838E-31,
  `F.DIST(1E50,10,10,FALSE)` 6.300000000000124E-298), a tiny x with a large df2 (`F.DIST(1E-300,1,1000,FALSE)`
  3.9884255731385913E149, `F.DIST(1E-300,2,1000,FALSE)` 0.9999999999999979), and a huge x where the density is near the
  bottom of the double range (`F.DIST(1E100,2,3,FALSE)` 2.7556759606310174E-250, `F.DIST(1E200,1,1,FALSE)`
  3.1830988618378795E-301; `F.DIST(1E300,1,1,FALSE)`, whose density 3E-451 underflows, is 0). A non-integer df is
  truncated first (`F.DIST(2,150.7,3,FALSE)` 0.17202585847929508 is the density at df 150).
- At a tiny x the stored Y = density·x decides (t1864-f-pdf-tiny-x, 20 cells): `F.DIST(1E-300,3,1000,FALSE)` is 0
  although the exact density is 2.1E-150 (Y = 2E-450 underflows), as are `F.DIST(1E-300,3,5,FALSE)`,
  `F.DIST(1E-220,3,1000,FALSE)`, `F.DIST(1E-170,4,1000,FALSE)` and `F.DIST(1E-300,1000,1000,FALSE)`; a subnormal Y keeps
  only its own bits, so `F.DIST(1E-215,3,1000,FALSE)` is 6.42285339593616E-108 (Y = 13·2^-1074, 2.1% below the exact
  density) and `F.DIST(1E-65,10,1000,FALSE)` 1.4821969375236873E-258 (Y = 3·2^-1074, 11.6% above). Excel's value then
  differs from fl(Y/x) by about 1E-14 relative (A5, A6, A10, A17; T-2647).
- At a huge x an F below 2^-1022 is 0 (probes-numeric t2586-r5-f-edges-a B1285-B1303, B1350-B1360 and 38 cells of
  t2586-r5-f-edges-hold, where the density is about 1.8E-310; their `*2^600` companions such as C1296 and C1301 show
  the 0 is Excel's, not Graph's), and so is a df1·x near f64::MAX whose fl(df1·x) does not overflow
  (t2586-r5-f-edges-a B1437, t2586-verify-r5-f-edges-hold B91, B92: `F.DIST(~1.8E305,1000,3,FALSE)`).
Evidence: probes-numeric t2220-f-dist-sweep-pdf (A897, A994), t2220-f-dist-sweep-d2-1 (A710, A776), t1530-f-df-cut
A9-A10, m9-f-dist A30, t1864-f-pdf-moderate-x, t1864-f-pdf-edges A1-A12, t1864-f-pdf-tiny-x A1-A20 (excel-graph,
2026-10-25), t2586-r5-f-edges-a, -hold, t2586-verify-r5-f-edges-hold, t2586-r5-f-bigconst-a; gen-m9 285d87cea1aaa73c A64 (`F.DIST(0.1,123456792,1,FALSE)` 0.085003679623665).
Ours: since T-3034, Excel's own chain M-final (`f_density.rs`, "F density is not the beta density, R's df or a
GAMMALN formula (T-2513)"), which gives a number at every one of these cells and keeps the range steps; before it, an
accurate Loader saddle-point form (T-1864, T-1290), now removed. `f_dist_of` makes a subnormal x `#DIV/0!`
(t2586-r5-f-edges-a B931, B932).
Status: VERIFIED (excel-graph) for the range rules; the values are M-final's, within 15 digits of Excel where |c·ln x|
is small (T-2586)
Tasks: T-1864, T-1290, T-2586, T-2642, T-2647

## F quantile at a huge df goes through Excel's BETA.INV (T-1538)
Rule: at a huge df Excel's F.INV and F.INV.RT are Excel's BETA.INV mapped to the F scale, and that BETA.INV
misses its own CDF's crossing by up to about 1E-16 in x (absolute), in either direction, so for an x near
1E-10 it is off by up to about 1E-6 relative. The quantiles are therefore not the true ones: `F.INV(0.5,5,1E10)` =
0.8702920369171896 while the true quantile is 0.87029203827791371 (ours 0.8702920382779133).
- Reciprocal identity: F.INV(p,d1,d2) and 1/F.INV.RT(p,d2,d1) (computed in double) agree within 2 ulps at every
  df probed (5/1E10, 1E10/5, 7/3, 3/7, 2/1E8, 1E8/2, 4/4, 6/1E10, 1E10/6, 1E6/5, 5/1E6; p = 0.5, 0.025, 0.975,
  1E-6, 0.999999, 0.3; bit-equal at 38 of 66 pairs). So `F.INV(0.5,1E10,5)` = 1/`F.INV.RT(0.5,5,1E10)` exactly.
- F.INV(p,5,1E10) is d2 y / (d1 (1 - y)) with y = Excel's `BETA.INV(p,2.5,5E9)`, to within about 10 ulps
  (the mapping's rounding is not pinned). F.INV.RT(p,5,1E10) is 2E9 (1 - y') / y' with y' = `BETA.INV(p,5E9,2.5)`
  within 3 ulps at p = 0.5, 0.025, 1E-6 and 0.3; at p = 0.975 and 0.999999 that is 6E-7 relative off, and
  F.INV.RT(p,5,1E10) is instead bit-equal to F.INV(1-p,5,1E10). Which path Excel takes is not pinned.
- Excel's BETA.DIST is accurate there (about 1E-15 relative; 8E-15 at x = 1.29E-12, p = 1E-6), so the miss is
  in BETA.INV: `BETA.DIST(BETA.INV(0.025,2.5,5E9),2.5,5E9,TRUE)` is 0.025000051345569204, and `F.DIST(F.INV(0.025,5,1E10),5,1E10,TRUE)` is 0.02500005134556904.
  `F.DIST(2,5,1E10,TRUE)` = 0.9247647537543169 is the true value (t1530-f-df-cut A8).
- BETA.INV(p,2.5,5E9) and BETA.INV(p,3,5E9) miss the true quantile by about -2.9E-17 to +7.7E-17 in x. For x above
  about 1E-9 (p = 0.975, 0.999999) x is a multiple of 2^-50 to 2^-53 with 21 to 25 significant bits; plain bisection
  of [0, 1] on an accurate CDF reproduces it (shape 2.5: 52nd or 53rd halving; shape 3: 50th or 52nd; lower or
  upper end). For smaller x the result is off that grid (denominators 2^81 to 2^92), and neither bisection, linear interpolation in the
  last bracket, nor Newton from the mean reproduces it.
Evidence: probes-numeric t1538-f-inv-huge-df (156 cells) and t1538-beta-dist-at-excel-inv (24 cells), excel-graph,
2026-10-23.
- F.INV(p,d1,d2) at a df of 1E5 or more is (d2/d1) (y / (1 - y)) with y = Excel's BETA.INV(p,d1/2,d2/2), and
  F.INV.RT(p,d1,d2) is taken as 1/F.INV(p,d2,d1) (a chosen form: the evidence is agreement within 2 ulps): with y from the entry below, `F.INV(0.5,1E10,5)` is 1.1490395521130337
  bit for bit (A3; d2 y/(d1 (1 - y)) is 1 ulp below). For p above 1/2 the root BETA.INV's flip solves, s = 1 - y,
  is mapped as it is, (d2/d1) ((1 - s)/s): `F.INV(1-1E-12,100000,5)` = 97570.08944108484 (probes-numeric
  t1538-f-quantile-switchover A9), where forming 1 - y from the flipped y misses by 9E-8 relative. This explains the
  6E-7 paths above: at p > 1/2 the flip moves the solve to the other side. Either side of df 1E5 (99999, 1E5, 1E6 at
  p = 1E-12 and 1 - 1E-12) the engine is within 2E-13 relative of Excel on all 24 cells of that case.
Status: VERIFIED for the values above; Excel's BETA.INV stop where its quantile is tiny is UNVERIFIED (T-2555)
Implemented in: functions/statistical/fisher.rs `f_quantile_through_beta`
Tasks: T-1538, T-1745, T-2555

## BETA.INV at a huge shape is AS 109's Newton search (T-1538)
Rule: Excel's BETA.INV has the structure of AS 64/109 (the search statrs ports, `inverse_beta.rs`): for p above 1/2
it is 1 - BETA.INV(1 - p, b, a), and otherwise a Newton search for the crossing of an accurate CDF. Where that
quantile is near 1 (the first shape huge) the search converges: Excel's value is the double nearest the true root.
Where it is tiny (the second shape huge, p <= 1/2) the search stops early, a Newton step or so short.
- Flip, bit for bit: `BETA.INV(0.975,2.5,5E9)` = 1.2832501727899626E-09 = 1 - `BETA.INV(0.025,5E9,2.5)` (A141, A138);
  likewise A143/A140, A149/A146, A151/A148. Such an x is on the 2^-53 grid near 1, hence its 21 to 25 significant bits.
- Converged near 1: AS 109 run to convergence on an accurate CDF (no start clamp to [1E-4, 0.9999], acu 1E-40) is
  bit-exact on 16 of the 24 BETA.INV cells of t1538-f-inv-huge-df (all 12 near-1 cells and the 4 flipped ones of
  p = 0.999999 and 0.975 at shape 3), 1 ulp off at A142. With statrs' own acu (1E-14 here, a stop at a step below
  1E-7) and its start clamp it overflows (the density at x = 1E-4 is exp(5E5)) or stops 1E-3 relative off.
  `BETAINV(1E-20,1E10,0.5)` = 0.9999999956419133 is the true quantile rounded (gen-m9/0a9226016e8f5f33 A358).
  Exception: `F.INV.RT(0.5,2,123456789.1229)` (gen-m9/f34f66fb7f81e673 A209, y = BETA.INV(0.5,61728394.5,1) near 1,
  I_x = x^a) is one double from the nearest root, which lies almost halfway (0.50/0.50 ulp) between two doubles: a tie
  Excel's own rounding decides, UNVERIFIED (T-2555).
- Early stop where tiny: the cells A133, A135, A137, A139, A145, A153, A155 (x from 1.3E-12 to 5.3E-10) are 6E-16
  to 9.3E-7 relative off the true root, near AS 109's second Newton iterate from its own start (1E-12 to 4E-8
  relative), but no fixed acu, statrs' acu, or ln B from a double lgamma reproduces them (T-2555).
- Ours: the crossing needs the CDF to about 1E-8 relative to pick Excel's neighbour near 1, and the T-0970 incomplete
  beta was about 1E-7 off there (its fraction does not converge; since T-2528 the CDF takes the large-shape expansion,
  "Incomplete beta at a huge first shape"): 1 ulp off at 4 of 12 near-1 cells. Where
  b/a is at most 1E-7, I_x(a, b) is taken as its large-a limit Q(b, -nu ln x), nu = a + (b - 1)/2 (the leading term
  of TOMS 708 `bgrat`, relative error about (b/a)^2 by mpmath: 1E-19 at a = 5E9), which is bit-exact on all of them.
- Search failure at an extreme p: `F.INV(1E-200,1E10,6)` = 1.3006292404573862E-07 and
  `BETAINV(1E-100,123456789.123,0.1,0.1)` = 0.9999919015105116 are not roots of the CDF (the root of the latter is
  0.9999983733488269), so Excel's search ends somewhere else there (gen-m9/7c20351562a2d71d A37,
  8dd2312c96effb1b A169; UNVERIFIED, T-2555).
Consequence: F.INV at a huge df (T-1538 entry above) is exact wherever its beta variable is near 1.
Evidence: probes-numeric t1538-f-inv-huge-df A133-A156 (excel-graph, 2026-10-23); AS 109 replayed with an
mpmath CDF (T-1538 PR).
Status: VERIFIED for the flip and the converged near-1 values; the early stop is UNVERIFIED (T-2555)
Implemented in: functions/statistical/inverse_beta.rs `beta_quantile_crossing`, functions/statistical/special.rs
`nearest_crossing`
Tasks: T-1538, T-2555

## Student-t quantile at a tiny probability (T-1531)
Rule: T.INV, T.INV.2T, TINV and CONFIDENCE.T never error for a tiny p or alpha: the quantile follows the power-law tail, so it is finite and huge for a small df (`T.INV(1E-300,1)` = -3.183098861837907E299, about 1/(pi p)) and about -79 for df = 99999.

Status: VERIFIED (excel-graph, 2026-10-14; probes-numeric case `t1531-t-tiny-p`, 105 cells, p from 1E-300 to 1E-10, df 1, 1.5, 5, 99999, 100000)

Implemented in: functions/statistical/t_dist.rs `t_tail_quantile`, `t_upper_quantile`

Tasks: T-1531, T-1542

- `T.INV(1E-300,1)` = `T.INV(1E-300,1.5)` = -3.183098861837907E299 (the df = 1.5 value is the df = 1 value: df truncates); `T.INV(1E-300,5)` = -1.568392559099332E60.
- `T.INV.2T(1E-300,1)` = 6.366197723675814E299 (exactly twice the one-sided magnitude); `CONFIDENCE.T(1E-300,1,2)` = 4.50158158078553E299.
- Ours: `statrs`' `StudentsT::inverse_cdf` is wrong for small df well before it fails (`T.INV(1E-10,1)` gives -8.9E7, Excel -3.18E9) and panics from p about 1E-200. For p < 1E-5 we use a closed-form tail asymptotic once x = df/(df+t^2) < 1E-8; otherwise `statrs` for p >= 1E-8 (it is bit-exact with Excel there) and bisection on its cdf below 1E-8 (`statrs`' inverse hangs at p = 1E-100 for df = 1000). Results agree with Excel to about 1E-13 relative for df <= 5, but only 7 of 105 cells are bit-exact, and for df >= 99999 at p <= 1E-200 we are wrong (-37.17 where Excel gives -79.06: for df = 99999 the `statrs` cdf saturates in the bisection branch, for df = 1E5 the normal series is used): T-1542.
- `T.INV.2T` with p in (1,2) is `T.INV(1 - p/2)`, a negative number (`T.INV.2T(1.5,1)` = -1, cells A121-A128); ours implements it since T-2096 (`t_inv_of`), see the entry above. Near p = 1 the quantile has an absolute error of about 1e-8 (`TINV(1+2^-52,2)` is -1.49e-8, Excel -3.14e-16): T-2839.

## Student-t quantile at one degree of freedom is 1/TAN(π·p/2) through Excel's TAN (T-2858)
Rule: with a df that truncates to 1, the Student-t quantiles are the Cauchy closed form evaluated in doubles with
Excel's own TAN kernel (numbers.md's x87-style tangent, `excel_tan` in math_and_trigonometry/trig.rs), not a search
on the cdf. With `t2(q)` = `1/TAN(PI()*q/2)`:
- `T.INV.2T(p,1)` and `TINV(p,1)` are `t2(p)` for p up to 1, and `-t2(2-p)` for p in (1, 2);
- `T.INV(p,1)` is `-1/TAN(PI()*p)` below 0.5 and `1/TAN(PI()*(1-p))` above it;
- `CONFIDENCE.T(alpha,sd,size)` with size truncating to 2 is `t2(alpha)*sd/SQRT(2)`.
So Excel's values are exact at the round points (`T.INV.2T(0.5,1)` = `T.INV(0.75,1)` = 1, `T.INV.2T(1.5,1)` =
`T.INV(0.25,1)` = -1, `T.INV.2T(0.75,1)` = 0.4142135623730951) and follow TAN's rounding near p = 1, where they
are far from the true quantile: `TINV(1-1E-10,1)` is 1.570797438519081E-10 while the true value is
1.5707964567631675E-10 (6E-7 relative; libm's `1/tan` gives 1.5707974382682531E-10, Excel's TAN gives Excel's
bits), and `TINV(1+2^-52,1)` is -5.053466325954936E-16 (true -3.48786849800863E-16). Forms that are not Excel's,
scored on the 41 two-tailed literal cells: `TAN(PI()*(1-p)/2)` 1, `COS/SIN` 25, libm `1/tan` 27; for p above 1 the
reflection is needed (`1/TAN(PI()*1.5/2)` is -0.9999999999999998, Excel -1). A p literal keeps Excel's 15-digit
parse: `T.INV.2T(1.9999999999999998,1)` is -63712901546918.43, the value at p = 1.99999999999999.
Evidence: gap triage 2026-10-27 evaluated these forms with the engine's TAN on a scratch case against all 112 failing
df = 1 cells in eval/ (excel-graph; T_DIST.xlsx and CONFIDENCE.xlsx desktop cache), and all 112 match bit for bit.
Among them `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`!D2 `T.INV.2T(T.DIST.2T(60,1),1)` = 60 exactly
and D15/D37 (p = 0.5000000000000001) = 0.9999999999999996; probes-numeric/t1531-t-tiny-p (its df = 1 cells, listed in T-2858; A36, A39, A42 are df 2),
m9-t-inv-2t A1-A13, m9-t-inv A1-A13; gen-m9 334f3dc9ca937f10 A23 `TINV(1E-300,1)`, 5389dd4d5b5739d2 A299
`TINV(1-1E-10,1)`, a4090567758520df A307 `TINV(1+2^-52,1)`. The full cell list is in T-2858. At df = 1 this closed form takes precedence over the general
SQRT(df*(1/y-1)) form below (T-3061), which gives the same cells only to 0 or 1 ulp.
Ours: `statrs`' inverse or the tail asymptotic (`t_quantile`, `t_tail_quantile` in t_dist.rs), from 1 ulp to the 14th
digit off (`TINV(0.5,1)` 1.0000000000000016), and to 100% off near p = 1 (`TINV(1-1E-10,1)` 1.05E-8).
Status: VERIFIED (excel-graph and Excel-cached; the formula is a fit that every evidence cell matches)
Tasks: T-2858

## Chi-squared quantile at df 2 and gamma quantile at alpha 1 are closed forms (T-2860)
Rule: with a deg_freedom that truncates to 2, `CHISQ.INV(p,2)` is `-2*LN(1+(-p))` computed as `-2·log1p(-p)`, and
`CHISQ.INV.RT(q,2)` and `CHIINV(q,2)` are `-2·ln(q)` on the upper-tail probability itself; with alpha exactly 1,
`GAMMA.INV(p,1,beta)` and `GAMMAINV` are `-beta·log1p(-p)`. These are the exponential distribution's quantiles in
doubles with libm's `ln`/`log1p`, so Excel is exact where they are (`GAMMA.INV(1E-300,1,1)` = 1E-300,
`CHISQ.INV(1E-10,2)` = 2.0000000001E-10; `-2·ln(1-p)` would give 2.000000165580742E-10).
Evidence: gap triage 2026-10-27 checked the forms against every failing cell of these shapes in eval/: 34 of 34
bit-exact (excel-graph): probes-numeric/m9-legacy-chiinv A13, A14, A23, A24, A33, A34 (`CHIINV(0.01,2)` = 9.210340371976182),
m9-gamma-inv A19, A20, A26, A27 (`GAMMA.INV(0.999,1,1)` = 6.907755278982136), m9-chisq-inv A12/A13, m9-chisq-inv-rt A9/A10,
t1049-chisqinv, t1049-chisqinvrt, t1543-inverse-cdf-tiny-p A55/A58, and 12 gen-m9 cells (8fec7b207f4dd753 A265
`CHISQ.INV.RT(0.001,2)` = 13.815510557964274; ecc87f80077959e0 A154 `CHISQ.INV.RT(0.001,2.5)`, same value). Passing
cells were not re-checked. T-0972 already noted CHIINV at df 2 as `-2*LN(p)` (gen-m9/4df1b22d3ed63f58 A9).
Ours: the incomplete-gamma search (`gamma_quantile`), 1 to 40 ulp off and 4 cells visibly off.
Implemented in: T-3016 (`exponential_quantile` in `incomplete_gamma/mod.rs`, the smaller-tail form); T-2860 pinned the cells
with a unit test.
Status: VERIFIED (excel-graph)
Tasks: T-2860

## Student-t quantile is SQRT(df*(1/y-1)) of the rounded beta quantile, away from the centre (gap triage 2026-10-28)
Rule: away from the centre (one-sided tail q = min(p, 1-p) up to at least 0.45), T.INV, T.INV.2T, TINV and
CONFIDENCE.T return t = fl(sqrt(fl(df * fl(fl(1/y) - 1)))), where y is within one ulp of the double nearest df/(df+t*^2)
(the nearest on 60 of 75 cells) and t* is the exact quantile (y is the beta quantile, I_y(df/2, 1/2) = 2q). The subtraction 1/y - 1 rounds to the spacing of
doubles near 1, so where t^2/df is small Excel is off the exact quantile by up to the order of df * eps / t^2 relative
(measured: 2.2E-7 at
df 1E10 (`T.INV(0.9,1E10)` = 1.2815518424115688, exact 1.2815515656292591), 2.5E-11 at df 1E6 and 8E-15 at df 1000).
The values look arbitrary, but this form gives them exactly.
Status: VERIFIED (excel-graph; reproduced, not probed directly)
Evidence: an mpmath triage script (exact quantile, then the double y, then the form in doubles) gives 75 of the 80
cells where Excel is visibly off the exact quantile. 60 are 0 ulp with the nearest y. The other 15 need y one ulp from
the nearest, so Excel's y is within an ulp of correctly rounded but not always correctly rounded (T-3061 lists them):
the T-3061 cells within probes-numeric m9-t-inv A108-A130, m9-t-inv-2t A35-A126 (not its centre cells A36, A114, A115), m9-confidence-t A5-A45, m9-t-nonint-df A59-A66,
m9-t-edges A38, t1048-tinv A21/B21, t1518-t-df-cut A14-A23, t1531-t-tiny-p A77-A117 (q from 1E-20), probes
stat-argument-modes H518, and gen-m9 1b31092a0d79e5f1 A275, 2138b909d8f810ea A241, 41248d1cb153c66e A113,
5919660b7bb4d195 A53, 775d053de498b8c7 A176. The full cell list is in T-3061.
It does not hold near the centre. At two-tailed p = 0.99 and 0.999 (q = 0.495, 0.4995; m9-t-inv-2t A10-A115) Excel is
within 1E-15 of the exact quantile, and the form is thousands of ulps off. The switch lies between q = 0.45 and
0.495 (T-2724). At df 30 and q = 5E-17 (`T.INV.2T(1E-16,30)`, gen-m9 583ca2baea0bb30a A147) Excel's t is 2E-12
off (its y about 4E-12; T-1542).
Ours: the exact quantile (A&S 26.7.5 from df 1E5, statrs' inverse below), so up to 2.2E-7 relative away at huge df:
T-3061.

## Inverse CDF functions at a tiny or near-1 probability (T-1543)
Rule: NORM.INV, NORM.S.INV, LOGNORM.INV, CHISQ.INV, CHISQ.INV.RT, GAMMA.INV and BETA.INV return a value (or `#NUM!`) for p from 1E-300 to 1-1E-16; none panics or hangs in Excel.

Status: VERIFIED (excel-graph, 2026-10-15; probes-numeric case `t1543-inverse-cdf-tiny-p`, 102 cells, p in 1E-300, 1E-200, 1E-100, 1E-20, 1-1E-16, 1-1E-10)

Implemented in: functions/statistical/beta.rs `beta_quantile`, functions/statistical/inverse_beta.rs `beta_quantile_crossing` (BETA.INV); functions/statistical/incomplete_gamma.rs `gamma_quantile` and chisq.rs `chisq_inv_of` (CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.INV, GAMMAINV; T-1634)

Tasks: T-1543, T-1634

- CHISQ.INV.RT and CHIINV quirks (T-1634; the literal-argument cells of gen-m9, this probe and m9-legacy-chiinv):
  - At one degree of freedom (df is truncated, so 1.5 too) the right-tail quantile is the left one of 1 - p:
    `CHISQ.INV.RT(1E-10,1)` = 41.82145620298276 = `CHISQ.INV(0.9999999999,1)` (the true right-tail quantile is
    41.82145636476129), and a p of 2^-54 (about 5.55E-17) or less, where 1 - p rounds to 1, is `#NUM!`
    (`CHISQ.INV.RT(1E-20,1)`; p between 5.55E-17 and 1E-16 is unprobed). The rule is exact only to Excel's search
    on its double-valued lower cdf: `CHIINV(1E-9,1)` (probes-numeric/m9-legacy-chiinv A1, A2) is 37.324892889999234
    in Excel, about 2.2E-7 (one step of the cdf's resolution near 1) below the complement crossing
    37.324893106518715 (ours) and the true quantile 37.32489305136234 (T-2561).
  - With 3 to 10 degrees of freedom a right-tail p of 1E-100, 1E-200 or 1E-300 is `#NUM!` (21 literal gen-m9 cells:
    `CHISQ.INV.RT(1E-100,3)`, `CHIINV(1E-200,6)`, `CHISQ.INV.RT(1E-300,10)`), though the quantile exists, while
    1E-20 is a value for every df probed (`CHIINV(1E-20,5)` = 103.42897723775795). df 2 (`CHISQ.INV.RT(1E-300,2)` =
    1381.5510557964274, -2 ln p) and df 50 (`CHISQ.INV.RT(1E-300,50)` = 1592.6841019848946) are values at every p.
    The boundary p between 1E-20 and 1E-100 is unprobed; the engine takes `#NUM!` below 1E-20
    for df 3 to 10 (df 22 at `CHISQ.INV.RT(1E-100,...)` is 542.4417060011164 in Excel, gen-m9; df 11 to 21 unprobed, UNVERIFIED).
  - A quantile below the smallest normal double is 0 (`CHISQ.INV(1E-200,1)`, crossing about 1.6E-400).
- Excel's gamma quantiles (T-0972, against an mpmath 50-digit reference of the 545 literal-argument CHISQ.INV,
  CHISQ.INV.RT, CHIINV, GAMMA.INV and GAMMAINV cells in eval/, df truncated, df 1 right tail as the lower quantile of
  1 - p): 509 are within half a unit of the 15th significant digit of the true quantile, 256 within half an ulp.
  The misses are where the cdf is flat in its double: near p = 0.999 Excel is 40 to 100 ulps off the truth
  (`GAMMA.INV(0.999,5,1)` = 14.794149222537294, true 14.79414922253720821; `CHISQ.INV.RT(0.999,20)` 72 ulps low), near
  the edge of the band of doubles whose cdf rounds to p, so it solves the tail it is given (the lower one for
  CHISQ.INV and GAMMA.INV, the upper for CHISQ.INV.RT) on a double-valued cdf, and at p = 1E-200 and 1E-300 with
  df 4 to 50 it is 20 to 2,500 ulps off (`CHISQ.INV(1E-300,4)` 81 ulps low), its own far-tail cdf's error. Neither
  is fitted.
  The p = 0.999 misses run both ways: `GAMMA.INV(0.999,2,1)` is 40 ulps low, `CHISQ.INV(0.999,10)` 48 high,
  `CHISQ.INV.RT(0.999,3)` 106 low and `CHISQ.INV.RT(0.999,10)` 57 high (probes-numeric m9-gamma-inv A34,
  m9-chisq-inv A34, m9-chisq-inv-rt A20 and A34; gap triage 2026-10-29). `CHISQ.INV.RT(0.999,3)` =
  0.024297585815692378 is not Excel's own `CHISQ.INV(0.001,3)` = 0.02429758581569273 (1 ulp from the truth), so the
  right tail is not inverted as the lower quantile of 1 - p near p = 0.999, unlike df 1 at 1E-10 above. At p =
  1 - 1E-10 the same probes are within 2 ulps. The landing rule is open: T-3148 probes it, T-3146 fits it.
  Evidence from probes-numeric t3148-gamma-quantile-flat-band (T-3148, excel-graph, 361 literal cells; ulp errors
  against a 50-digit bisection of the cdf at the double p):
  - Where Excel leaves the true quantile by more than 4 ulp. `GAMMA.INV(p,a,1)` over a in 0.5 to 50: the worst
    shape is already 4.5 ulp off at p = 0.99 (a = 2; the other eight are within 4), then up to 20 ulp at p = 0.995,
    25 at 0.998, 49 at 0.999, 92 at 0.9995 and 473 at 0.9999. `CHISQ.INV.RT(p,df)` over df 1 to 100: 21 ulp at
    p = 0.99 (df 3; the other seven are within 4), 28 at 0.995, 149 at 0.998, 107 at 0.999, 838 at 0.9995 and
    3,084 at 0.9999. At p = 0.99999 both are within 3.4 ulp (GAMMA.INV within 0.5), the 1 - 1E-10 behaviour above.
    The onset is at or below p = 0.99, which this case does not cover; the error grows with p to 0.9999 and is gone
    at 0.99999.
  - Excel's quantile is constant across neighbouring p doubles, with one break at 0.999. For p = 0.99899999999999967,
    ...78 and ...89 (the three doubles below 0.999) `GAMMA.INV(p,2,1)`, `GAMMA.INV(p,5,1)` and `CHISQ.INV.RT(p,5)`
    each return one value: 9.233413476450405, 14.794149222535964 and 0.21021260262930572 (A120 to A128). At 0.999
    and the three doubles above they return another: 9.233413476451513, 14.794149222537294 and 0.210212602629218
    (A129 to A140). The ulp errors in this range only reflect the true quantile moving under a constant Excel
    output (`GAMMA.INV(p,2,1)` is low at all seven p, 40 to 595 ulp; `CHISQ.INV.RT(p,5)` is high at the six
    neighbours, 302 to 2,769 ulp, and 44 ulp low at 0.999).
    Correction (gap triage 2026-10-29, T-3175): this "break" is the probe's literals, not Excel's search. Formula
    number literals are cut to 15 significant digits (numbers.md), so `0.99899999999999967`, `...78` and `...89`
    are all 0.998999999999999 and `0.99900000000000011` to `...33` are all 0.999. The two values of
    `GAMMA.INV(p,2,1)` differ by 1.108E-12, which is 1E-15 of p over the gamma density 9.04E-4 at the quantile. Neither
    group is a set of neighbouring doubles, so nothing above says how Excel's quantile moves between adjacent p. The
    same cut applies to the 17-digit x literals of the cdf series below (A141 to A361): for small k several x = q + k
    ulp literals read as one 15-digit x, so the band edges below are measured on 15-digit x, not on the doubles.
  - Excel's cdf is monotone in x at every sampled series, with a flat run at 0.999. For the doubles
    `x = q + k ulp`, k in 0, +-1 ... +-128 (q Excel's p = 0.999 quantile), `GAMMA.DIST(x,a,1,TRUE)` (and
    `CHISQ.DIST.RT(x,df)`) equals 0.999 for k in [-32, 64] (a = 1), only k = 32 (a = 2), [-32, 2] (a = 5),
    [-8, 0] (a = 25) and [-2, 8] (a = 50); `CHISQ.DIST.RT` equals 0.999 for k in [-16, 128] (df 3), [-32, 128]
    (df 5), [-64, 8] (df 10) and [-16, -4] (df 50). These are the sampled k (powers of two), so the true band edges
    lie within one step of them. Excel's own quantile (k = 0) is inside the band except for a = 2 and df 50, where
    Excel's quantile does not evaluate back to p (`CHISQ.DIST.RT(24.673905271877409,50)` = 0.9989999999999999); it
    sits at the high edge only for a = 25. So it is not at a fixed edge of the band. T-3146 fits from these cells.
- Shape 1 (T-3016): Excel's quantile is the closed form of the exponential, taken in the smaller tail: with t the
  smaller tail's probability (p, or the exact 1 - p for p above 1/2), `-LN(1-t)` (by `ln_1p`) where that tail is the
  lower one and `-LN(t)` where it is the upper, times the scale. `CHISQ.INV(0.5,2)` = `CHISQ.INV.RT(0.5,2)` =
  1.3862943611198906 (2 ln 2), `CHISQ.INV(0.001,2)` = 0.002001000667167067, `CHIINV(1E-300,2)` =
  1381.5510557964274, `GAMMA.INV(1E-300,1,1)` = 1E-300. This is Excel's double on 106 of the 108 literal-argument
  cells in eval/ at df 2 (df 2.5 and 2.9 truncate to 2) or GAMMA.INV shape 1; the two others are 1 ulp off it
  (`CHISQ.INV.RT(0.9995,2)` = 0.0010002500833644855, `CHISQ.INV.RT(0.99999,2)` = 2.0000100000575652E-5, probes-numeric
  t3148-gamma-quantile-flat-band A97, A113). The scale multiplies the unit quantile, not divides it by the rate 1 /
  beta: `GAMMA.INV(0.95,1,1000000)` = 2995732.27355399, where the quotient is 2995732.2735539903 (probes-numeric
  m9-gamma-inv A26; also A23, A27 and A28). Evidence: those literal-argument cells (excel-graph). Before T-3016 the
  crossing of the cdf was 1 to 2 ulps off on 69 of them. One shape-1 formula cell is unexplained: gen-m9
  146b46e74f2dac3b A167 `CHISQ.INV(DOLLARDE(0.1,28),2)` is 0.8836655045580786 in Excel, but the closed form is
  ...785 at p = 0.35714285714285715 and ...787 at the next double, 0.3571428571428572. Either Excel's DOLLARDE is
  another double or the cell is an exception (T-3239).
- Ours (T-0972): the crossing of the smaller tail's capped incomplete gamma, always (T-1634 kept statrs' search where
  its result solved that tail to 1E-13 relative, up to 7E-15 off Excel and the truth: `GAMMA.INV(0.5,9,1)`). The
  lower tail's `powf * exp(-x - ln Gamma(a + 1))` form is used only at x * rate up to 1 (T-1634 used it up to the
  shape: at shape 50 its exponent's rounding is 1E-14 of the tail). With `x^a` by `powf`, `GAMMA.INV(1E-300,1,1)` is
  9.999999999999999E-301, one ulp under Excel's 1E-300: the crossing is exact, and the ulp is lost to
  `refine_increasing`'s midpoint (`special::nearest_crossing` would return 1E-300, a lead for T-2561). Above shape 1E9
  it stays statrs' search alone: Excel is `#NUM!` for `GAMMAINV(0.999999,10000000000.5,1E+10)`.
  The crossing is not always the correctly rounded quantile: on 49 gated cells it lands 1 to 3 ulps off Excel where
  Excel is the correctly rounded closed form (`CHISQ.INV(0.5,2)` is 1.3862943611198906 = 2 ln 2 in Excel, ours
  1.3862943611198908; `CHISQ.INV.RT(0.5,2)` ours 1.3862943611198904; GAMMA.xlsx F75 Excel 0.6931471805599453 = ln 2,
  ours 0.6931471805599454). T-3016 fixed the shape-1 ones ("Shape 1" above); T-3238 owns A168 and T-3239 the rest. Other cells still
  off are Excel's own last digits: `CHISQ.INV(1E-300,50)` is 2.0354283669760313E-11 in Excel while the correctly
  rounded quantile (ours) is 2.035428366976834E-11, and `GAMMAINV(0.5,0.01,100)` is 4.4655350189103716E-29 in Excel
  against the true 4.46553501891034868E-29 (ours ...0346E-29): Excel's tail kernel is not fitted (T-2561 for the
  visibly-off cells, T-0972 for the last digits).
  Since T-0971 the search and the refinement run on the accurate incomplete gamma pair (section "Incomplete gamma:
  accurate kernel"), except the lower tail in the series' range, which keeps T-1634's `powf * exp(-x - ln Gamma(a + 1))`
  prefactor over the accurate series: with the pair's `x^a e^-x / Gamma(a)` there `CHISQ.INV(1E-16,1)` and
  `CHISQ.INV(0.001,1)` are 2 ulps off Excel's 1.5707963267948967E-32 and 1.57079714926249E-6 (gen-m9), with this
  prefactor they are Excel's doubles. The kept-search quantiles moved by 1 to 2 ulps with the cdf under them: the
  merge fixed 56 more quantile cells (gen-m9) and left 6 at 1 to 2 ulps (`CHIINV(0.95,7)` Excel 2.167349909298057,
  ours 2.167349909298058; `CHISQ.INV.RT(0.1,7)`, `GAMMA.INV(0.05,2,30)`; T-2367).

- `1-1E-16` is a different double from 1 (`NORM.INV(1-1E-16,0,1)` = 8.209536151601386), but `1-1E-17` is 1 and gives `#NUM!`.
- Excel's CHISQ.INV and GAMMA.INV are exact in the lower tail (`CHISQ.INV(1E-100,1)` = 1.5707963267948966E-200, `GAMMA.INV(1E-300,1,1)` = 1E-300, `CHISQ.INV(1E-300,1)` = 0) and CHISQ.INV.RT is `#NUM!` for p below about 1E-15 with df = 1 but finite for df = 50 (`CHISQ.INV.RT(1E-300,50)` = 1592.68...). Before T-1634 ours gave `#NUM!` for CHISQ.INV(p<=1E-200,1), GAMMA.INV(p<=1E-20,1,1) and CHISQ.INV.RT(p<=1E-20,df) (Excel: values), and wrong digits elsewhere in the tails (`CHISQ.INV(1E-300,50)` 0.0033 vs 2.04E-11; `CHISQ.INV(1-1E-16,50)` 179.80 vs 180.68; `GAMMA.INV(1E-300,100,1)` 1.0165 vs 0.0380) (fixed by T-1634, below).
  The lower tail is not always correctly rounded at a shape below 1 (T-3012). `GAMMA.INV(1E-16,0.25,1E+15)`
  (gen-m9/984cba67bbe6972b.case.json Calc!A43, excel-graph) is 6.749697893111757E-50 in Excel. The exact quantile
  (p·Γ(1.25))^4·1E15 is 6.7496978931117296E-50 (40-digit Decimal), and ours is 6.74969789311173E-50. So Excel is
  4.1E-15 relative high, about 1E-15 relative in P at a = 0.25. Not yet known: whether this is the crossing of
  Excel's own GAMMA.DIST cdf (T-3012 probes it, T-3013 matches it).
  T-3012 evidence (probes/t3012-gamma-inv-small-shape, probes/t3012-gamma-dist-at-quantile, excel-graph):
  - Excel's GAMMA.INV is not the crossing of its own GAMMA.DIST. `GAMMA.DIST(x,a,1,TRUE)` at Excel's
    `GAMMA.INV(p,a,1)` quantile (second case, column B, exact relative error in P against the decimal p; the
    `cdf/p-1` column D is zero-snapped by Excel and proves nothing) is within 6E-16 of p at a = 0.05, 0.25 (except
    row 8, +9.7E-16) and 0.5, and further off at a = 0.1 (row 3, p = 1E-30: -8.5E-15), a = 0.75 (row 16, p = 1E-100:
    +2.6E-14; row 17 -3.2E-15; row 18 -1.9E-15; row 19 +1.2E-15; row 20 -1.3E-15) and a = 0.9 (row 21 +1.9E-15,
    row 22 +1.1E-15, row 23 -4.7E-15, row 25 -3.0E-15). No shape is exact at every p. The sign is positive at a = 0.05, negative at a = 0.1 and changes with p
    at a = 0.25, 0.5, 0.75 and 0.9.
    One ulp of x moves the cdf by a quarter ulp at a = 0.25, so columns A..C agree there; at a = 0.5 and 0.9 they
    differ (rows 15, 24, 25). The errors of 6 to 205 ulps of P at a = 0.1, 0.75, 0.9 (rows 3, 16-23) are more than one ulp of x
    can explain, so the quantile is Excel's own inversion, not a crossing; the same holds at a = 0.05 (+5.5E-16 and
    +5.9E-16 with A = B = C, rows 1-2) and row 8 of a = 0.25, about 30 to 90 ulps of x from a crossing. Rows 15 (a = 0.5) and 25 (a = 0.9)
    are bracketed: at row 15 Excel returns the first x whose cdf reaches p, at row 25 the last x below p (C25, one
    ulp of x higher, is above p; Excel's cdf jumps 3.6E-15 across that ulp).
  - The scale is a plain final product: in all 36 rows of the first case column B equals the double product
    A * 1E+15 exactly (`GAMMA.INV(1E-16,0.1,1)` = 6.073048362407876E-161 and `GAMMA.INV(1E-16,0.1,1E+15)` =
    6.0730483624078755E-146 are the same value to the rounding of the product, row 10).
  - Range: `GAMMA.INV(p,0.05,1)` is 0 for p <= 1E-30 and `#DIV/0!` at p = 1E-16 (row 4); shape 0.1 gives 0 for
    p <= 1E-100 and shape 0.25 / 0.5 / 0.75 / 0.9 give 0 for p = 1E-300 (0.25 also at 1E-100).
  - GAMMA.DIST's cdf at shape 0.25 (second case, rows 27-35): for x <= 1E-30 it equals
    `EXP(0.25*LN(x))/EXP(GAMMALN(1.25))` at x = 6.7497E-65 and 1E-30 and is 1 ulp above it at x = 1E-65; it never
    equals `x^0.25/GAMMA(1.25)` (2 ulps above at 1E-65, 1 ulp above at 6.7497E-65 and 1E-30). From x = 1E-10 up it includes the series' second
    term (0.0034888228355002 against 0.0034888228355700 for the one-term forms), so only the x^a / Gamma(a+1) factor
    is comparable. The `cdf/p-1` columns are zero-snapped by Excel's subtraction rule and prove nothing; compare the
    raw cdf values.
  - T-3013 should therefore not expect a crossing-based fix to reproduce Excel at a = 0.1, 0.75, 0.9; only the
    exp(a ln x) form of the leading term is indicated.
  - GAMMA.DIST's cdf at a shape below 1 and x below 1 is Excel's leading factor `EXP(a*LN(x))/EXP(GAMMALN(a+1))`
    times e^-x and the series, with a·LN(x) rounded to a double (T-3060, implemented in
    `incomplete_gamma::excel_small_shape_lower`). That rounding is Excel's own error: up to 1.2E-14 of P where
    |a ln x| is near 230 (`GAMMA.DIST(4.147567013992206E-134,0.75,1,TRUE)` = 1.000000000000026E-100 against the true
    1.0000000000000131E-100, A16), and 1E-15 to 4E-15 at the probe's other visibly-off cells. Against a 50-digit
    mpmath cdf, Excel's value is the exact cdf with its exponent replaced by the double a·LN(x) to within 0.1 ulp of
    that exponent at every probed cell from x = 6E-301 to 1E-5 (rows 1-10, 16-25, 27-31 of the second case; shapes
    0.05 to 0.9). With the engine's EXP, LN and GAMMALN it is bit-exact or 1 ulp off at those cells (2 to 3 at row
    24 and A32). Region implemented: a shape in (0, 1) other than 1/2 (which takes the ERF kernel), a normal x/beta
    below 1 and a·LN(x/beta) of -16 or less, where the double exponent's ulp is 2^-48 or more and its rounding reaches
    the 15th digit. Nearer 0 Excel still takes the form, but the forms are a few ulps apart and the engine's 1-ulp
    residue (below) would lose A31 `GAMMA.DIST(1E-5,0.25,1,TRUE)` and C25, which are bit-exact on the accurate pair,
    so the pair is kept there. Unprobed, so also kept on the pair: x of 1 or more, CHISQ.DIST, CHIDIST, POISSON and
    the quantiles. External check: gen-m9/a013a45a8fbcab4f A257 `GAMMADIST(1E-300,(3.14159/Data!D4),2,Data!D5)`
    becomes Excel's 3.743699731362489E-10 (7 ulps off before).
  - Evidence trap: formula literals are truncated (not rounded) to 15 significant digits (numbers.md "Formula number literals are read
    at 15 significant digits"), so the probe's 17-digit x values are not what Excel evaluated. `GAMMA.DIST
    (7.414417701862728e-12,0.9,1,TRUE)` (A24) is GAMMA.DIST at 7.41441770186272E-12, while B24 and C24 (one and two
    ulps of x higher) both read as 7.41441770186273E-12 and have the same value in Excel. Fitted at the 17-digit x,
    the form seemed to miss A20 and A23-A25 by 10 to 58 ulps (the gap triage's reading); at the 15-digit x it fits
    them. Fit a probe's cells at the literal as Excel reads it.
  - Still open (T-3060 residue): D20, D23 and D25 (`GAMMA.DIST(..)/1E-k-1` of B20, B23 and B25) stay visibly off.
    They magnify a few ulps of B to 1E-16 absolute: B23 is 1 ulp high with the form, B20 (a·LN(x) = -11.6) is 1 ulp
    low on the pair and with the form, and B25 (-11.55) is 2 ulps high on the pair and exact with the form, but at
    that exponent the form loses C25, one 15-digit unit of x away. The same 1-ulp residue is in the worksheet form
    itself: ours is 1 ulp off Excel's `EXP(0.25*LN(6.749697893111757E-65))/EXP(GAMMALN(1.25))` (B28; also B30-B32),
    which Excel's GAMMA.DIST equals at that x (A28). So the residue is in the engine's EXP, LN, GAMMALN or quotient
    model, not the form. None of 108 orderings tried (the quotient plain, rounded twice or as a product with
    EXP(-GAMMALN); e^-x by Excel's EXP or libm; the series with and without its tail sum, or as 1 + aS) and no x87
    extended chain (EXP and the quotient unrounded) fits every cell.
- Excel's BETA.INV is erratic where the shape is extreme: `BETA.INV(1E-300,0.5,100)` = `BETA.INV(1E-200,0.5,100)` = 2.7050408442680296E-151 (the true quantile underflows to 0, ours 0); `BETA.INV(p,0.01,0.01)` is `#NUM!` for every p; `BETA.INV(1E-200,1000,1)` = 0.2064 and `BETA.INV(1E-20,1000,1)` = `#NUM!`, neither near the true quantile 0.9549925860214368. Ours is 0.9632618940201478 since T-3007 (the crossing of a cdf whose w1 is now the complement of w, "Incomplete beta at a shape of 1 outside the closed forms"). These are not implemented.
- Open (inferred, not probed directly): Excel's `BETAINV(1E-15,1E+8,1)` is not the closed form `p^(1/alpha)`. gen-m9 5ded7594e597e31e A177, `IPMT(BETAINV(1E-15,1E+8,1),1.5,77,75622)`, is -75621.97390807692 in Excel, which implies a quantile of about 1-3.45031E-7 against the closed form's 0.9999996546122957 (1-3.45388E-7), 3.6E-10 relative apart (T-2587). Probe and fix: T-2936.
- Ours: `statrs`' `Beta::inverse_cdf` panics from p about 1E-200 and does not finish for `BETA.INV(1E-100,1000,1)` or for any shape of 1E8 or more, at any p. Since T-2119, `BETA.INV` uses the capped search only as the start for shapes up to 1E5 and p of 1E-15 or more. It then refines to the crossing of BETA.DIST's own cdf (the T-0970 incomplete beta), and with no start it bisects that cdf. Nine BETADIST_BETAINV.xlsx cells are 1-4 ulp off on it (T-2633). A sweep of NORM.INV, NORM.S.INV, LOGNORM.INV, CHISQ.INV, CHISQ.INV.RT, GAMMA.INV and CONFIDENCE.NORM over the same p found no panic or slow case.

## NORM.S.INV is AS 241 PPND16 in the tails; the central region is unfitted (T-1746)
Rule: for |p - 0.5| > 0.425, NORM.S.INV(p) (and NORMSINV, NORM.INV(p,0,1)) equals Wichura's
AS 241 PPND16 evaluated in doubles: the published coefficients by Horner, r = sqrt(-ln(p)) below
and sqrt(-ln(1-p)) above (not `log1p(-p)`), then r - 1.6 (r <= 5) or r - 5 and the tail rational.
It is bit-exact on 611 of 616 tail cells, p from 5E-309 to 1-7E-16. The 5 misses are two
upper-tail p next to the split, 0.925 and NORM.S.DIST(1.5), each 1 to 2 ulp off.
Centrally (|p - 0.5| <= 0.425) no candidate fits (central bit-exact counts of 408 cells in brackets):
- Excel is up to 31 ulp from the correctly rounded inverse. NORM.S.INV(0.16) is
  -0.9944578832097497, where the correctly rounded value is ...7531.
- The output is often, not always, mirrored: 106 of the 170 (p, 1-p) probe pairs give exactly
  opposite values, but NORM.S.INV(0.08) is 12 ulp off while NORM.S.INV(0.92) is correctly
  rounded. The rounding of 1-p explains at most about 2 ulp of that.
- It is not PPND16's central rational q*A(r)/B(r) by Horner [82], Horner with fused multiply-adds
  [100] or the ascending power sum [87].
- It is not statrs/Boost erf_inv, which is the best central candidate [164]; the correctly rounded
  value gets [126].
- It is not one Newton or Halley step from PPND16 [85 to 105]. On 232 of 341 probe points Excel's
  own NORM.S.DIST(PPND16(p)) is already exactly p, yet Excel returns a different x on 177 of those.
- It is not a Newton solve on 0.5*erfc(-x*sqrt(0.5)) from PPND7, Acklam, Moro or 0, stopping at
  |dx| <= 1e-15|x| or at dx = 0 [61 to 79].
- It is not a bisection on Excel's own NORM.S.DIST (the engine's `normal_cdf`, bit-exact with it since T-1916),
  checked against the 341 central cells (T-0925, 2026-10-25, with a throwaway engine test that is not committed: it
  calls `special::normal_cdf` on each variant). Variants tried: fixed brackets from [-1, 1] to
  [-100, 100]; brackets of 1E-12 to 1E-2 around PPND16; widths of 0 to 1E-13, absolute or relative; returning lo,
  the midpoint or hi. None matched more than 68 cells. A Newton solve on that CDF from PPND16, stopping on |dx| or
  on |CDF - p| at 1E-17 to 1E-10, matched at most 68 too. statrs' closed form gets 164.
- NORM.S.DIST(NORM.S.INV(p)) equals p on only 172 of 341 points. Probe t1759-norm-s-inv-roundtrip
  (T-1759) adds NORM.S.DIST at Excel's x and at its neighbouring doubles:
  - x is a crossing of Excel's own CDF at 234 of 340 central points;
  - the CDF is flat across the three doubles at 36;
  - x misses by 1 to 32 ulps of p at 70, 52 of them below.
  - At 53 of the 63 adjacent rows, a neighbouring double gives NORM.S.DIST exactly p, yet Excel
    returned its own x. So the stop rule neither minimises |CDF − p| nor stops at CDF = p.
  - NORM.S.INV recomputed from a constant p reproduces x at all 340 rows.
  This matches Microsoft's description of NORM.S.INV as an "iterative search" on NORM.S.DIST and
  has the same shape as BETA.INV (T-1745). Its start and stop rule are not fitted.
Consequence: the best kernel is a hybrid, PPND16 in the tails and statrs centrally (implemented, T-1916:
`normal.rs` `ppnd16_tail`, for NORM.S.INV and NORM.INV; the central region stays with T-0925). In the tails
NORM.INV(p,m,s) = m + z*s in plain doubles with z = PPND16(p): `NORM_DIST.xlsx` `NORM.DIST NORM.INV`!F4, F10, F11,
F16, F17, F24, F25, F42, F58, F68, F69 and F77 all pass with it; centrally statrs' own `Normal(m, s)` composition stays. It is bit-exact
on 775 of 1,024 sweep cells, against 341 for the engine's statrs alone (fit `norm-s-inv`).
Side finding for T-0968, now the fit `norm-s-dist` (T-1759): over all 1,773 NORM.S.DIST cells,
`0.5*erfc(-x*sqrt(0.5))` (glibc) is bit-exact on 1,103, Cody's rational approximations (R's
pnorm_both) on 991, `0.5*erfc(-x/sqrt(2))` on 869, and the engine on 879. On the T-1746 probe
alone, NORM.S.DIST(x,TRUE) equals `0.5*erfc(-x*sqrt(0.5))` (glibc erfc) at 260
of the 341 PPND16(p) points of the probe (column D) and at 240 of Excel's own NORM.S.INV results
(column B). The `0.5*erfc(-x/sqrt(2))` form matches 199 and 203.
Input handling seen on the way:
- `NORM.S.INV(0.9999999999999999)` is PPND16(0.999999999999999). Excel reads the 16-digit
  literal to 15 significant digits.
- `NORM.S.INV(2.3E-308-1E-309)` is PPND16(2.3E-308): the subnormal literal 1E-309 counts as 0
  (numbers.md, "A subnormal divisor is zero"). The difference itself is normal; had it flushed,
  the result would be #NUM!.
Evidence: probes-numeric/t1746-norm-s-inv-central (341 literal p over the central region, with
NORM.S.DIST of each result and of PPND16(p)), m9-norm-s-inv, t1048-normsinv, t1048-norminv. Run
it with `uv run rustsheet-fitlab norm-s-inv` (`fitlab/kernels/norm_inv.py`).
Status: VERIFIED (tails); the central algorithm is UNVERIFIED
Tasks: T-1746, T-1759, T-0925, T-0968

## LOGNORM.INV is EXP of NORM.INV's quantile (T-0925)
Rule: LOGNORM.INV(p, m, s) and LOGINV are EXP(NORM.INV(p, m, s)), on NORM.INV's hybrid (PPND16 in the tails, see above). In the tails it is EXP(m + z*s), with z
the PPND16 quantile. `LOGNORM.INV(1E-20,0,1)` = 9.493291347224372E-05 (probes-numeric/t1543-inverse-cdf-tiny-p
A16; statrs' `LogNormal::inverse_cdf` gave ...338E-05). With it, 20 ironcalc-calc LOGNORM.xlsx lines and 17 gen-m9
LOGNORM.INV, LOGINV, CONFIDENCE.NORM and CONFIDENCE lines pass, and no cell regresses. The central LOGNORM.INV
cells, such as `LOGINV(0.2,250,30)` (gen-m9/8fec7b207f4dd753 A55), wait on the central fit above.
Evidence: probes-numeric/t1543-inverse-cdf-tiny-p A16 (excel-graph); ironcalc-calc STATISTICAL/LOGNORM.xlsx (Excel-authored
cached values; 113 -> 133 of 147 cells pass); gen-m9 LOGNORM.INV, LOGINV, CONFIDENCE.NORM and CONFIDENCE cells (T-2255).
Status: VERIFIED (tails); the centre waits on NORM.S.INV's
Tasks: T-0925

## AVERAGEA, MAXA and MINA over an array
Rule: an array value's elements are not read like range cells. AVERAGEA skips a boolean element and
counts a text element, numeric text and the empty string included, as 0; MAXA and MINA skip both and
read only the numbers. `AVERAGEA({1,"a",TRUE})` is 0.5, `AVERAGEA({3,TRUE})` and `{3,FALSE}` 3,
`{3,"a"}`, `{3,"7"}` and `{3,""}` 1.5, `{"a",TRUE}` 0, `{"a"}` 0, `{TRUE}` `#DIV/0!`;
`MAXA({0.5,TRUE})` is 0.5, `MAXA({-1,"a"})` and `MAXA({-1,"5"})` -1, `MINA({2,TRUE})` and
`MINA({2,"a"})` 2. With no value read, MAXA and MINA are 0, as MAX is: `MAXA({"a"})`,
`MINA({TRUE,"b"})`, `MAXA(Z1:Z3)` and `MINA(Z1)` over blanks. An array from IF behaves the same
(`AVERAGEA(IF(1,{3,TRUE}))` is 3), while a direct boolean argument still counts
(`AVERAGEA({3,TRUE},TRUE)` is 2), direct non-numeric text is `#VALUE!` (`AVERAGEA({3,"a"},"a")`) and
direct date text is its serial (`AVERAGEA("1/2/2020",1)` is 21916.5). A reference reads text as 0 and
booleans as 1/0 (coercion.md "MAXA / MINA: single-cell references are read like ranges"), a cached
range of a linked workbook too.
An array computed from a range follows the same array rule: over A1:A4 = 4, TRUE, "x", 2, `AVERAGEA`, `MAXA` and `MINA`
of `FILTER(A1:A4,{1;1;1;1})`, `SORT(A1:A4)`, `IF({1;1;1;1},A1:A4)`, `IF(A1:A4>0,A1:A4)` and `CHOOSE({1,1},A1:A4)` are
2, 4 and 2 (the plain range gives 1.75, 4 and 0). We match all of them since T-1508 (lookup.md, "CHOOSE over an index array broadcasts its value arguments").
Evidence: probes/stat-a-computed-arrays (18 cells, excel-graph, 2026-10-14, T-1559); probes/averagea-array-constant-elements (25 cells, excel-graph, 2026-10-14);
probes/legacy-value-argument-intersection-b AI2, AJ2.
Status: VERIFIED (excel-graph)
Implemented in: `Model::for_each_value_a` (statistical/count_and_average.rs), `ArrayText`.
Tasks: T-1551, T-1555, T-1559

## Thin distribution functions: argument coercion, truncation and error edges (T-1049)
Rule (all from excel-graph sweeps `probes-numeric/t1049-*`; F.TEST scalars implemented by T-2093):
Implemented in: `functions/statistical/hypegeom.rs`, `weibull.rs`, `phi.rs`, `fisher.rs::fn_f_test` (F.TEST/FTEST number or text argument, T-2093), `fisher.rs::f_dist_of` (F.DIST density at x = 0, T-2095) (the HYPGEOM.DIST sample_s, zero-count, negative-fraction and TRUE-sample_s rules, the WEIBULL.DIST x = 0 density and TRUE x, PHI of a boolean)
Tasks: T-2035 (partial), T-2093, T-2095
- **F.TEST scalars:** F.TEST and FTEST with a number or text argument are `#DIV/0!` (T-2093; fisher.rs).
- **Truncation:** HYPGEOM.DIST truncates all four counts toward zero (`HYPGEOM.DIST(1.9,4,8,20,TRUE)`
  = `(1,4,8,20)`, and `(0.9,...)` = `(0,...)`; `hypgeomdist`), and a negative fraction (-0.5) is
  `#NUM!`. T.TEST truncates `tails` and `type` (`T.TEST(...,1.9,2.9)` = tails 1, type 2; `ttest`).
  F.DIST truncates df (`F.DIST(1,3.9,7.9,TRUE)` = df 3, 7; `fdist`) but `F.DIST(1,1.5,7,TRUE)` = 0.649... is a
  value, not an error (df 1.5 truncating to 1; no df=1 control cell was probed).
- **Coercion:** a blank cell is 0 for every scalar argument (`F.DIST(Z99,3,7,TRUE)` = 0, `PHI(Z99)` =
  0.3989..., a blank cumulative flag is FALSE); TRUE is 1; a numeric text ("2") is accepted and non-numeric
  text is `#VALUE!`. The cumulative flag is any number (nonzero = TRUE, `...,2)` = cumulative), "TRUE" text
  is TRUE, other text `#VALUE!`. `PHI("")` is `#VALUE!` but `PHI(Z99)` is fine; `PHI({0,1,2})` lifts over the array and spills
  (`phi` A37:C37 = PHI(0), PHI(1), PHI(2)). `PHI(1E300)` and `PHI(-1E300)` are `#NUM!`, while `PHI(38.5)` underflows to 0.
- **Density at 0:** `F.DIST(0,1,7,FALSE)` and `CHISQ.DIST(0,1,FALSE)` are `#NUM!` (the density is
  infinite), the df=2 value is 1 (F) and 0.5 (CHISQ), higher df give 0; the F.DIST rule holds for df2 = 1 and 100 and after df1 truncates (`F.DIST(0,1.5,3,FALSE)` = `#NUM!`, `F.DIST(0,2.9,7,FALSE)` = 1; probes-numeric/t2095-fdist-zero, T-2095); `WEIBULL.DIST(0,0.5,3,FALSE)`
  is 0, not `#NUM!`, and so is the shape-1 density at 0, which is mathematically 1/lambda:
  `WEIBULL.DIST(0,1,3,FALSE)` = 0 (`weibulldist` A27; `WEIBULL(0,1,1,FALSE)` = 0 too, probes-numeric/t1048-weibull A16,
  gap triage 2026-10-19). The density at x = 0 is 0 for every probed shape. Implementing these edges is T-2035 (HYPGEOM.DIST, WEIBULL.DIST, PHI) and T-2095 (F.DIST). `F.DIST.RT(0,...)` and `CHISQ.DIST.RT(0,...)` are 1.
- **Boundary probabilities:** `CHISQ.INV(0,3)`, `F.INV(0,3,7)` are 0 and p = 1 is `#NUM!`; the RT inverses
  are the mirror (`p=1` -> 0, `p=0` -> `#NUM!`). TRUE as p is 1 (`CHISQ.INV(TRUE,3)` = `#NUM!`).
  `CHISQ.INV(0.5,1E10)` = 9999999999.333334 (not probed above 1E10; the T.INV and F limits are in T-1043 and T-1530).
- **HYPGEOM.DIST domain:** `#NUM!` for N = 0 (even `(0,4,8,0)`; negative N not probed), for n > N (`(1,25,8,20)`), M > N, sample_s above
  min(n,M) is not an error but 0 / 1 (`(5,4,8,20)` = 0 density, 1 cumulative), and N = 1E10 is `#NUM!` while N =
  2E9 works. `(0,0,0,1,FALSE)` = 1. Zero n or M gives density 1 at s = 0 and 0 above.
- **Test functions:**
  - F.TEST: `#DIV/0!` for fewer than two points on either side, constant data, scalars, an empty range
    or text-only input (`F.TEST("a",B1:B5)`); unequal sizes work; the two-tailed p is twice the smaller F tail, so it never exceeds 1
    (`F.TEST(C1:C6,B1:B8)` and FTEST: 0.9195979047175995, probes/stat-argument-modes H248/H251, T-1292); text and booleans inside array
    constants are skipped (`{1,2,"a",4}` differs from `{1,2,3,4}`).
  - Z.TEST: an empty range is `#N/A` (`Z.TEST(Z1:Z5,4)`, with or without sigma); one point or a
    scalar is `#DIV/0!` even with sigma given (`Z.TEST(7,4,1)`, `D1:D1`); constant data is `#DIV/0!`
    without sigma and a value with sigma (`Z.TEST(D1:D3,5,1)` = 0.5); sigma <= 0 is `#NUM!`; text in
    a range or array constant is skipped; a text first argument is `#VALUE!` (`Z.TEST("a",4)`).
  - CHISQ.TEST: scalars and single cells are `#N/A`; `#N/A` for unequal element counts; a column
    against a row with the same count is accepted; constant data gives 1 (no `#DIV/0!`);
    `#DIV/0!` only for an empty range or a zero expected value; a negative expected value is `#NUM!`.
    A side holding no number (blank, text or a boolean in a range) is `#DIV/0!`, implemented in T-2094 (t1049-chisqtest A23/A24,
    m9-legacy-chitest A28, gen-m9 0a9226016e8f5f33 A333; checked after the `#N/A` size test; numbers on both sides never paired
    stay 1, 8dd2312c96effb1b A343).
    Text or TRUE in the actual array returns exactly the all-numeric result (`chisqtest` A34/A35 = A29).
  - T.TEST: type 1 (paired) with unequal sizes is `#N/A`; types 2 and 3 accept unequal sizes;
    tails outside {1,2} or type outside {1,2,3} is `#NUM!`; `#DIV/0!` for one point per side, zero variance in both
    samples (types 2/3) or in the paired differences (type 1); type 3 with one constant sample still returns a value; a text cell in a range is skipped (type 1 still returns a value, `ttest` D5).
Evidence: `probes-numeric/t1049-{fdist,fdistrt,finv,finvrt,ftest,chisqdist,chisqdistrt,chisqinv,
chisqinvrt,chisqtest,hypgeomdist,weibulldist,ztest,phi,ttest}` and their `*.oracle.json`
(`hypgeomdist` has 61 cells for the T-0977 fit).
Status: VERIFIED (excel-graph observation)
Tasks: T-1049, T-2035

## CORREL, PEARSON and RSQ over arrays of different sizes, and their last digit
Rule: CORREL, PEARSON and RSQ of two arrays of different sizes are `#N/A`, and of the same 10-value array twice not exactly
1: CORREL and PEARSON give 1.0000000000000002 and RSQ 0.9999999999999998 (COVARIANCE.P of it is 161.7225, exact).
Evidence: GROUPBY passing these functions by name gives each group's values and all values
(probes/groupby-pivotby-functions G!B421:B426, -functions2 G!B281:B286, G!B301:B306, G!B326; PIVOTBY functions2 P!A121),
excel-graph, 2026-10-16, T-1325. The plain different-size call is `#N/A` too (`CORREL({2,4,5},{1,2})`, probes/regression-pairs-a L17, T-1620).
Ours: matches: different sizes are `#N/A` (T-1292) and the last digits follow from Excel's formula, the covariance and
standard deviations over n for CORREL and PEARSON and over n - 1 for RSQ (T-1723; "Paired regression functions" below).
Status: VERIFIED (web)
Tasks: T-1707, T-1723

## LINEST, LOGEST, GROWTH: probe findings
Rule (excel-graph, 2026-10-16, T-1621; spilled cells are evidence too; cell ids are in the Sheet1 sidecars):
- Defaults: `const` omitted or empty is TRUE; `stats` omitted or empty is FALSE (one row: `LINEST(y,x,TRUE,)` and `LINEST(y,x,TRUE)` fill one row of two cells: K1:L1 and K22:L22). A nonzero number counts as TRUE for both; text for either is `#VALUE!`. `known_x's` omitted is 1,2,3,...
- `const` FALSE: LINEST's intercept is 0 (LOGEST's multiplier is 1) and, with stats, the intercept's standard error is `#N/A`. Stats block (5 rows) for more than one x column: rows 1-2 hold one coefficient/se per x column plus the intercept; in rows 3-5 (r2, F, ss_reg) the cells right of the second column are `#N/A`.
- Exact fit (ss_resid 0): with stats, F is `#NUM!`; `LINEST(B15,A15,TRUE,TRUE)` (one point) is `{0,9; 0,0; 1,0; #NUM!,0; 0,0}`, LOGEST's is `{1,9.000000000000002; ...}`. Near-exact fits are not snapped to that case: ss_resid 4.0e-33 gives F = 3.11e34 (LINEST K106), 1.8e-30 gives 1.36e32 (K190).
- Shapes: unequal y and x lengths, a row-vector y with a column-vector x and a scalar x with a range y are `#REF!`; `LINEST(5,A2:A9)` is a 1x9 row `{0,...,0,5}` (the 8 x observations are read as 8 variables). `LINEST(B2:B9,TRANSPOSE(A2:C9),TRUE,TRUE)` (column y, 3x8 x) is `#REF!`. A text cell in x (H2:H9 holding "a") is `#VALUE!` for all three, and so is `LINEST(y,x&"")` (`x&""` for LOGEST and GROWTH is unprobed); text in y is unprobed. LOGEST and GROWTH of a non-positive or zero y are `#NUM!`.
- Collinear x: `LINEST(B2:B9,A2:E9,TRUE,TRUE)` has y itself in x (B), so the fit is exact; Excel drops the redundant column A (coefficient 0, se 0, df 3 not 2) and E's coefficient is 1.43e-16 with se 8.3e-17. A probe with E = 2A and no y in x is unprobed.
- Dynamic arrays: GROWTH with `new_x's` omitted or equal to known_x's returns an array of y's size, also with const FALSE and no new_x's; a scalar new_x gives a scalar; the result shape follows new_x's. A new_x with the wrong number of columns is `#REF!`; boolean or text new_x is `#VALUE!`. `#SPILL!` for a blocked spill is unprobed here.
- Exactly representable fits come out inexact: LOGEST({1,2,4,8},{1,2,3,4}) gives 1.9999999999999998 and 0.5000000000000001, GROWTH({1,2,4,8},{1,2,3,4},{5,6}) gives 15.999999999999996 and 31.999999999999968, GROWTH(5,5,5) is 4.999999999999999: the oracle's evaluation order is evidence for T-1741, T-1742, T-1743.
Evidence: probes/linest-modes, logest-modes, growth-modes.
Ours: the LINEST and LOGEST shapes, stats block and check order match since T-1741 (section "LINEST and LOGEST: shapes,
one point, collinear columns, exact fits and check order"); the last-digit differences remain (T-2006).
FORECAST.ETS.SEASONALITY/STAT/CONFINT could not be lifted: excel-graph is `#N/A` for every valid call (section above), so cases would be junk evidence; the three stay below the floor until desktop values exist (T-1553).
Status: VERIFIED (excel-graph) for the stated cases; the unprobed items are marked
Tasks: T-1621, T-1741, T-1742, T-1743, T-2006 (shared least-squares kernel)

## Paired regression functions: single points, zero variance, text and large offsets
Rule: CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, COVAR and FORECAST(.LINEAR) pair their two arrays cell by cell and
skip a pair when either cell of a range is text, blank or boolean (COVAR too: `COVAR(D1:D6,B1:B6)` with D2 blank 2.52,
`COVAR(A1:A6,C1:C6)` 2.6399999999999997). A text scalar argument is `#N/A` (`CORREL(A1:A6,"a")`, `COVAR(A1:A6,"a")`).
COVARIANCE.P and COVARIANCE.S skip pairs the same way (`COVARIANCE.P(D1:D6,B1:B6)` 2.52, `COVARIANCE.S(D1:D6,B1:B6)` 3.15, `COVARIANCE.S(J1:J6,B1:B6)` 4.3) and give `#N/A` for a text scalar; with every pair skipped (`{1,2,3}` against `{TRUE,FALSE,TRUE}`) both are `#DIV/0!` (probes/covariance-pairs-skip, excel-graph, 2026-10-19, T-1724). Ours: matches for COVAR, COVARIANCE.P and .S (T-1724); a typed text in T.TEST's samples is unprobed and stays `#VALUE!`.
RSQ is not CORREL squared in its last digit: `CORREL(J1:J6,B1:B6)` is 1 while `RSQ` of the same pair is
1.0000000000000004, and `CORREL(A1:A6,C1:C6)` 0.9999999999999998 while RSQ is 1 (probes/regression-pairs-a L23/N23,
L10/N10). COVAR of an x or y with zero variance is 0, not `#DIV/0!` (probes/regression-pairs-b L6, L7).
Unequal sizes and single points follow "Array arguments given a scalar, a single cell or an error" above. Further rules:
- Zero variance: in known_y, SLOPE is 0, STEYX 0 and INTERCEPT the mean, while CORREL, PEARSON and RSQ are `#DIV/0!`
  (pairs-a O7 vs L7; pairs-b M7, N7); in known_x, SLOPE, FORECAST and the correlations are `#DIV/0!` but TREND returns the mean
  (`TREND(A1:A6,G1:G6)` first 4.5, trend O20).
- Two points: STEYX is `#DIV/0!` (pairs-b M4). COVAR with every pair skipped is `#DIV/0!` although COVAR of a single pair is 0
  (`COVAR({1,2,3},{TRUE,FALSE,TRUE})`, pairs-b L18; single pair L3, L25, L27).
- RSQ is not clamped to 1: `RSQ(J1:J6,B1:B6)` is 1.0000000000000004 (pairs-a N23), as is `RSQ({1,2;3,4},{4,3;2,1})` (N28).
- Large offset (10000000001...) keeps its variance: `CORREL(I1:I6,B1:B6)` 0.9402561526801988, `FORECAST(7,A1:A6,I1:I6)`
  -7285714278.784956, `SLOPE(A1:A6,I1:I6)` 0.7285714285713528, `INTERCEPT(A1:A6,I1:I6)` -7285714283.884956, `STEYX(I1:I6,B1:B6)` 0.822308017832176 (pairs-b M21), `STEYX(A1:A6,I1:I6)` 0.5277986629120525 (M22). A two-pass sum of
  squares about the means reproduces these bits; for r and r² the deviations' sums are then divided before the square
  roots, CORREL and PEARSON by n and RSQ by n - 1: `r = (Sxy/n) / (sqrt(Sxx/n) * sqrt(Syy/n))` and
  `RSQ = ((Sxy/(n-1)) / (sqrt(Sxx/(n-1)) * sqrt(Syy/(n-1))))^2`, with Sxx, Syy, Sxy the sums about the means (T-1723:
  this fits every number cell of the six functions in pairs-a and -b to the bit, ULP cells included; plain
  `Sxy/sqrt(Sxx*Syy)` misses 4 of 17 CORREL cells). SLOPE is `Sxy/Sxx`, INTERCEPT `ȳ - SLOPE*x̄` and STEYX
  `sqrt((Syy - Sxy*Sxy/Sxx)/(n-2))`, evaluated in that order.
- A typed text or boolean argument is one value (numeric text included; whether it counts as a number is not observable
  with one value on a side), like a single cell holding one:
  `CORREL("a",5)`, `SLOPE("1",5)`, `STEYX({1},"a")` and `FORECAST(1,"a","a")` are `#DIV/0!`, `CORREL(A1:A3,TRUE)` and
  `CORREL("a",{1,2})` `#N/A` (unequal sizes), and so is `CHISQ.TEST("a","a")` (probes/regression-pairs-text-scalar, T-1723).
- FORECAST x: TRUE is 1 (2.571428571428571), "7" is 7, "" is `#VALUE!` (forecast L6, L10, L12); an array x spills one
  forecast per element (`FORECAST.LINEAR({7,8},A1:A6,B1:B6)` 7.2, 7.9714285714285715 in P7, Q7), a range x that
  collides is `#SPILL!`; unequal sizes are `#N/A`.
- Last digits: `FORECAST(0,A1:A6,B1:B6)` 1.7999999999999998, `CORREL(A1:A6,C1:C6)` 0.9999999999999998 (both follow from the formulas above).
- STEYX on an exact fit: the difference `Syy - Sxy*Sxy/Sxx` is not snapped to 0; when it rounds below zero STEYX is `#NUM!`
  (`STEYX({1.8,1.9,...,2.4},{1,...,7})`), when it rounds above zero a tiny number (`STEYX({0.1,...,1.1},{1,...,11})`
  4.967053731282552E-09), and exactly 0 when it cancels (probes/steyx-exact-fit, excel-graph 2026-10-20, T-1723). The fit
  matches only with the probe's literals read to 15 significant digits (its `0.30000000000000004` taken as 0.3), as the engine reads them.
Evidence: probes/regression-pairs-a, regression-pairs-b, regression-forecast (excel-graph, 2026-10-16, T-1620); the
error cells in these cases are `#DIV/0!`, so whether a range error propagates or is replaced is unprobed.
Ours: the unequal-size `#N/A` matches since T-1292, single points since T-1401; CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX and FORECAST compute from the sums about the means in Excel's order, a typed text or boolean argument is one value with no number, and STEYX of a negative difference is `#NUM!` (T-1723). COVAR with text, blank or boolean cells (`#N/A` in ours) is still open.
Status: VERIFIED (web)
Tasks: T-1723, T-1724, T-1725 (overlaps T-1292, T-1707, T-1401, T-1290)

## TREND: defaults, shapes, single points and error inputs
Rule: TREND treats an empty known_x or new_x argument as the default (`TREND(A1:A6,B1:B6,,FALSE)` fits the known x's: 6 rows,
first 1.1868131868131868; `TREND(A1:A6,,K1:K2)` is 2 rows); a single point is that point (`TREND(5)` 5, `TREND(A1:A1,B1:B1)` 2);
array constants work (`TREND({2,4,5},{1,2,3})` 1x3 starting 2.1666666666666665; a 2-D known_y gives a 2-D result,
`TREND(A1:B3,H1:I3)` 3x2 starting 3.6666666665000003). Text, blank, boolean or error cells in known_y or known_x are `#VALUE!`; unequal
sizes are `#REF!` (trend L17), as is a text scalar known_x. An error cell in new_x is `#VALUE!` for that element only and
keeps the shape (`TREND(A1:A6,B1:B6,F1:F2)`: 2 rows, first 3.3428571428571425, SUM `#VALUE!`; trend L37-O37); a text or blank new_x
makes the whole result `#VALUE!` (`TREND(A1:A6,B1:B6,D1:D2)`, trend L39-O39). With column-shaped known data a row-shaped new_x
gives a row result (`TREND(A1:A6,B1:B6,{7,8})` is 1x2, SUM 15.17142857142857, trend L6-N6; ours 1 column, SUM 7.2). The multi-column known_x cases of the probe (L32, L33) hit text and blank cells, so the
plain multi-column shape is unprobed.
Shapes (Excel's LINEST rules): known_x of known_y's shape is one variable read element by element, and new_x may then have
any shape, the result taking it (2-D y included); else a column y takes one variable per column of x (new_x needs that
many columns; the result is a column), a row y one per row (the result is a row); anything else is `#REF!`. A single y
is both, so `TREND(-1,Data!D8:D9)` is one point of two variables, -1 (gen-m9 0bbab6206b5c9a07 A44), while
`TREND(Data!C4,Data!A3:D12)` is `#REF!` (41248d1cb153c66e A54). A singular fit drops the collinear x columns, as LINEST
does ("LINEST, LOGEST, GROWTH: probe findings"), so a zero-variance x gives the mean of y (`TREND(A1:A6,G1:G6)` 4.5, trend O20);
which near-collinear columns are dropped (ours: a Gram-Schmidt residual under 1E-10 of the column, the last column
first, only once the plain solve fails) is UNVERIFIED (T-2569); the last-first order is LINEST's (K106, T-1741).
Order of the checks (gen-m9 T-2256 cells): (1) an argument that is an error value, in argument order, const included
(`TREND(<#NUM!>,Data!B8,Data!A5:C11,NA())` `#NUM!`, 3b7990bd4e3307ba A345; `TREND(Data!B4:C10,Data!A11:B12,<#VALUE!>)`
with unequal shapes `#VALUE!`, 11eb3ab4f4f8104c A326; `TREND(Data!C7:D12,0.2,<#N/A>,<bool>)` `#N/A`, c2684066d8df263e
A124), then a typed or computed text const that is no boolean (`TREND(Data!A2:D6,Data!C2:E4,Data!E2:E6,"x")` `#VALUE!`,
f82473275380882e A100; `GROWTH(..,"1")`, `GROWTH(..,"")`); (2) the shapes (`#REF!`): a text scalar known_x is one value
(`TREND(Data!B2:E10,Data!$B8)` `#REF!`, 73f053591a54bc43 A53), and an error *element* waits (`TREND(Data!D8:D12,
BESSELY(MUNIT(1),<negative>))` `#REF!`, a4090567758520df A333; a 1x3 array of `#N/A` against a scalar x `#REF!`,
3b7990bd4e3307ba A97); (3) a const cell that is no boolean (`TREND(Data!C10:E11,..,Data!$A12)` with A12 = "abc" is `#REF!`,
bbab6562e88b70e8 A13); (4) a blank, text, boolean or error element of known_y or known_x, `#VALUE!` (`TREND(-1,Data!E11:E12)`
with a blank and "1.5" `#VALUE!`, 0620505280d2ea6e A357; `TREND(Data!A1:A2)` with A1 blank `#VALUE!`, 46ac3ef5e61583ca A341;
a one-cell blank known_x too, 165db58d1540f014 A91); (5) new_x's shape, after the data (`TREND(N(0.3),Data!C3:C9,
Data!B8:D10)` is `#VALUE!` from C3:C9's text, 80586b85 A265), then new_x's blank or text cells.
GROWTH reads known_y in row-major order and its first element that is no positive number decides: a blank, text, boolean or
error is `#VALUE!` at the data step, a non-positive number `#NUM!` once known_x's cells are checked (`GROWTH(Data!D6:D10)`
holding 1.005, 0, a blank `#NUM!`, 58e3253bbd3a7a82 A291; `GROWTH(Data!A2:A12)` holding "1.5", .., -0.0001 `#VALUE!`,
40425341c5034eed A158; `GROWTH(Data!D8:E9)` holding a blank, 1, -0.5 `#VALUE!`, A383; `GROWTH(0,Data!A7)` with A7 = "abc"
`#VALUE!`, fec39efaba9284c3 A35), before new_x's text (`GROWTH(Data!A2:A4,Data!A2:A4,<text cells>)` `#NUM!`,
d8266bc15533702a A396). (`GROWTH(PHI(-1E+300),Data!C11:D12,..)`, a 0 y against a 2x2 x, is `#NUM!`, not the size `#REF!`,
abc2939ebd1e2365 A187: unexplained, ours `#REF!`, T-2569.)
An array const lifts, before the other arguments' errors: each element is the call with that const (its top-left value),
so `TREND(..,SORTBY(Data!A1:D12,..))` and `GROWTH(LOOKUP(NA(),..),..,GROUPBY(..))` spill (gen-m9 80586b85 A61,
165db58d1540f014 A239), while a multi-cell range const does not (`TREND(..,OFFSET(Data!B6:C12,-1,3))` `#VALUE!`,
43c6227e0ff368c7 A165). The values of the lifted elements (ours: each element's top-left value) are UNVERIFIED (T-2569).
Evidence: probes/regression-trend (excel-graph, 2026-10-16, T-1620); gen-m9 (excel-graph, 2026-10-22/23, T-2255, T-2256).
Ours: matches since T-1726 (`trend_or_growth` in functions/statistical/trend.rs), but for the last-digit differences of the
shared least-squares solve (T-2006) and A187 above. A single point without a constant term (`TREND(5,2,4,FALSE)`) is a fit
through the origin, UNVERIFIED (T-2569). LINEST and LOGEST of one point are accepted by Excel (linest-modes K141, logest-modes
K162), and ours since T-1741.
Status: VERIFIED (excel-graph)
Tasks: T-1726 (overlaps T-1402, T-1743)

## Order-statistic and deviation functions: edge behaviour (T-1622)
Rule: `AVEDEV(1,)` and `DEVSQ(1,)` are 0.5, so an empty argument counts as 0 in both. `QUARTILE(TRUE,1)` and
`QUARTILE.INC(TRUE,1)` are 1 (a typed boolean is a one-element array; untested for QUARTILE.EXC and PERCENTILE).
DEVSQ equals the two-pass sum of squared deviations about the mean, in argument order, not the one-pass
`Σx² − (Σx)²/n`: `DEVSQ(1E15,1E15+1)` is exactly 0.5, `DEVSQ(0.1,0.2,0.3)` is 0.019999999999999997, `DEVSQ(L1:L6)`
is 48.833333333333336. Every other cell of the probe (MODE/MODE.MULT shape, QUARTILE/PERCENTILE/PERCENTRANK edges)
matches the engine.
Evidence: probes/stat-order-floor-batch A274, A299 (empty argument), A102, A128 (boolean), A277, A278, A280, A290,
A294, A295, A300 (DEVSQ; a two-pass reproduces all seven bit-exactly); excel-graph, 2026-10-17.
Status: VERIFIED (excel-graph).
Ours (T-1748): AVEDEV/DEVSQ count the empty argument as 0 (`for_each_value`); the data argument of PERCENTILE*, QUARTILE*,
PERCENTRANK* reads a typed boolean as a one-element array and typed text as a number (a single cell holding either is no number, see "A lone text, boolean or empty cell as the data of QUARTILE..."). DEVSQ is one-pass (devsq.rs, T-1749).
T-1748 sweep (probes-numeric/t1748-typed-boolean-data, excel-graph): PERCENTILE, PERCENTILE.INC/.EXC(TRUE,0.5), QUARTILE.EXC(TRUE,2),
PERCENTRANK/.EXC(TRUE,1), SMALL/LARGE(TRUE,1) and QUARTILE(TRUE,0) are all 1, QUARTILE(FALSE,1) is 0, `QUARTILE("1",1)` is 1 (direct
text coerces), `SUMSQ(1,)` is 1 (empty skipped or counted as 0 alike).
Tasks: T-1748, T-1749

## SKEW, SKEW.P, KURT, HARMEAN, STANDARDIZE, SUMX* and MDETERM edge pools (T-1623)
Rule (Excel, evidence below): SKEW.P of fewer than three numbers is `#DIV/0!` (`SKEW.P(A1:A2)`, and a range that
leaves two numbers after text, booleans and blanks are skipped; a single value and three equal values are `#DIV/0!`
too). HARMEAN of a range with no numbers (`HARMEAN(D3:D4)`, two blanks) is `#N/A` (not `#DIV/0!`), unlike SKEW, SKEW.P and KURT of the same
blanks, which are `#DIV/0!`. STANDARDIZE reads a direct `TRUE` as 1 (`STANDARDIZE(TRUE,3,2)`
is -1), and a range argument is inferred to lift element-wise: `STANDARDIZE(A1:A2,3,2)` is `#SPILL!` with data below it, which shows
Excel returned an array, but no element value was observed (T-1750 probes it unblocked).
MDETERM of the singular matrix [1,2,3;4,5,6;7,8,9] is 6.661338147750939e-16, not 0, so Excel's elimination leaves a
rounding residual. KURT with a direct `TRUE` argument is -1.4331834319526635 for 3,5,2,8,9,4,7,1 plus TRUE, ours
is -1.4331834319526626 (about 9e-16, roughly 4 ulp, off). The other 132 cells of the case (SUMX2PY2, SUMXMY2, SUMX2MY2 size mismatches and text, boolean or
blank pairs, MDETERM non-square, text and 1x1, ECMA.CEILING sign and zero-significance cases, SKEW/KURT/HARMEAN too-few
and non-positive values) already match.
Other results a later fix must keep: `MDETERM({1,2,3;4,5,6;7,8,10})` is -2.9999999999999982 (a second elimination
residual beside the singular case, T-1751), `MDETERM(5)` is 5 while `MDETERM("5")` is `#VALUE!`,
`ECMA.CEILING(-7,0.3)` is -6.8999999999999995, `ECMA.CEILING(1E+300,1E-10)` is `#NUM!`, and `STANDARDIZE(5,3,1E-300)` is
1.9999999999999998e+300. The SUMX2PY2/SUMXMY2/SUMX2MY2 scalar text and boolean results are in coercion.md.
Literal `1E+308` inside a formula makes Graph reject the whole workbook (`createSession` 501 `unsupportedWorkbook`),
as does a wrong argument count (`STANDARDIZE(5,3)`, a three-range `SUMX2PY2`); `ECMA.CEILING` and `SKEW.P` must be
written `_xlfn.`-prefixed in the file.
Evidence: probes/stats-evidence-floor-t1623 (excel-graph, 2026-10-17).
Status: VERIFIED (excel-graph).
Ours: the SKEW.P mismatches (Q16, Q19) were fixed by T-1404, HARMEAN of no numbers (`#N/A`, Q50) by T-1753, and the direct TRUE in STANDARDIZE's x (Q58, and L14 of probes/stat-later-argument-lift) by T-1755; the others above are open.
Tasks: T-1750, T-1751, T-1752, T-1753, T-1754, T-1755

## AGGREGATE 14-19: k is read before the array, a referenced error k is `#VALUE!` (T-2281)
Rule: functions 14-19 read k as a number before they read the array, so a text k is `#VALUE!`
and an error k its own error even where the array holds an error or no numbers, while a blank,
TRUE or "2" k is coerced and the array decides (no numbers: `#NUM!`). A k that is a reference,
written or returned by OFFSET, INDIRECT or IF, reads its top-left cell (a multi-cell range does not
lift), and a referenced cell holding an error is `#VALUE!`, not that error (options 0, 4 and 6
probed; options 1, 2, 3, 5 and 7 too, T-2336: `AGGREGATE(14,o,A1:A5,D1)` with D1 `=NA()` is `#VALUE!`
for every one of them, probes/aggregate-error-k-picked-ref Calc!A1:A5). Under function 9, a fourth argument that
is IF or CHOOSE returning a reference is summed as that reference: `AGGREGATE(9,0,A1:A2,IF(TRUE,B1))` and the
`CHOOSE(1,B1)` form give 13, the same as plain `B1` (Calc!A6:A8; B1 = 10, A1:A2 = 1, 2). Other functions in 1-13 are
assumed alike.
Cells (probes/aggregate-range-k-first-cell, excel-graph 2026-10-21; Data A1:A5 = 1..5, B1 "x",
B2 blank, B3 0.5, B4 2, C1:C2 no numbers, D1 `=NA()`, E1 `=NA()`, E2 2):
- `AGGREGATE(18,3,C1:C2,B1:B3)`, `AGGREGATE(14,6,C1:C2,"x")`, functions 14-19 over C1:C2 with
  k B1:B3: `#VALUE!` (A3, A5:A11); `EDATE(…,-21)` `#VALUE!` (A19, and gen-m6/03333cc38ec06a14
  Calc!A120). `AGGREGATE(14,4,E1:E2,"x")` `#VALUE!`, not the array's `#N/A` (C8).
- `AGGREGATE(18,3,C1:C2,0.5)`, k -1, k B2:B3 (blank), k B3:B4 (0.5), k "2", k TRUE: `#NUM!` (A4,
  A12, A20, A21, A25, A26); k `1/0`: `#DIV/0!` (A24); `AGGREGATE(14,6,A1:A5,NA())` `#N/A` (C9).
- `AGGREGATE(14,6,A1:A5,B1:B3)` `#VALUE!` (A2), `AGGREGATE(16,6,A1:A5,B3:B4)` 3 (A14),
  `AGGREGATE(14,6,A1:A5,B4:B5)` 4 and `ROWS(…)` 1 (A16, A17), `AGGREGATE(14,6,A1:A5,"2")` 4 (A28);
  k `OFFSET(B4,0,0)`, `OFFSET(B4,0,0,2,1)`, `INDIRECT("Data!B4")`, `IF(TRUE,B4)`: 4 (C5, E1, C12,
  C13), and `OFFSET(B1,0,0)` over C1:C2 `#VALUE!` (C7).
- k D1 or D1:D2 over A1:A5 or C1:C2 under options 0, 4 and 6, and k `INDIRECT("Data!D1")` or
  `IF(TRUE,D1)`: `#VALUE!` (A27, A29:A31, C1:C4, C6, C10, C11, C14).
- An array k still lifts: `AGGREGATE(14,6,C1:C2,{"x";1})` spills `#VALUE!`, `#NUM!` (A22:A23).
Engine: `Model::aggregate_k` (crates/rustsheet-engine/src/functions/aggregate.rs); `fn_aggregate`
keeps a fourth argument IF or CHOOSE picks a reference for as that reference.
Status: implemented (T-2281)

## F.TEST / FTEST: larger variance on top, Excel's rounded powers, an accurate remainder (T-1927, T-2361)
Rule (fit, UNVERIFIED as Excel's algorithm): F.TEST takes f = the larger two-pass sample variance
over the smaller (df_big, df_small its and the other sample's n - 1), so it is exactly symmetric
in its arguments, and its two-tailed p is `2*MIN(u, 1-u)` for the upper tail
u = I_x(df_small/2, df_big/2), x = df_small/(df_big*f + df_small), y = df_big*f/(df_big*f + df_small).
x never exceeds the mean a/(a+b) there, so the tail is computed without a reflection, as
`exp(a*ln x) * exp(b*ln y) * R` with R = I_x(a,b)/(x^a y^b): the two powers carry Excel's
exp-of-a-product rounding (where a ln x is near -100 to -300 the exact p is tens of ulps, up to
179, from Excel's, and the model is within a few), while R itself is close to correctly rounded. The closed forms
of the incomplete beta hold: b = 1 is exp(a ln x) and a = 1 (df_small = 2) is -expm1(b ln1p(-x))
below x = 0.1, which is why the a = 1 deep tails are the correctly rounded value.
Since T-2977 a tail outside the series (a + b above 200, or a shape that is not a half integer) takes the bratio port
in front of today's kernel (`bratio::beta_dist_lower_tail`); a shape of 1 keeps the forms above, unswapped.
A variance that overflows is `#NUM!`: `F.TEST({1E+200,1E+200,-1E+200},Data!A11:B12)` (gen-m9/
7674c89c824aca43 A39, f2bd5f380ec002bf A210); the engine also returns `#NUM!` for two finite
variances whose ratio overflows, `FTEST({1E-160,2E-160},{1E+150,2E+150})` (unprobed, T-2571).
Not the model: VAR.S's one-pass switch for the variances, f = v1/v2 with the minimum of both tails,
and a single exp(a ln x + b ln y + ln G).
Fit (T-2361, 188 distinct inputs from the cells on file: gen-m9 cells whose arguments are plain
ranges, probes stat-argument-modes and ftest-decomposition, probes-numeric m9-legacy-ftest and
t1049-ftest; bit-exact / within 4 ulp / visibly off):
- exact p, two-pass f: 53 / 128 / 40; exact p with VAR.S's one-pass switch: 30 / 124 / 46;
- Cephes-style double kernel (exp-log powers), f = v1/v2, smaller of the two tails: 44 / 146 / 24;
- larger variance on top, R as Gamma ratio times Cephes' double continued fraction: 63 / 156 / 17;
- the same with R = Gamma ratio times the double power series 2F1(a+b,1;a+1;x): 74 / 161 / 11;
- the same with R correctly rounded (the model above): 90 / 170 / 6.
Excel's residual to the model runs from -12 to +14 ulps, mostly within 2, on both sides, so
Excel's own R is a double computation that is not identified yet (open: T-2571).
T-2571 (committed fit `uv run rustsheet-fitlab f-test` and `f-test-structure`, tools/src/rustsheet_tools/fitlab/kernels/ftest.py,
291 numeric cells on file with plain-range arguments, a + b <= 200; bit-exact / within 4 ulp / max ulp): the Python
model with R correctly rounded 142 / 267 / 26 (the engine before the closed form below 150 / 267 / 26, after it 160 / 270 / 26); R as the double 2F1 series over a Gamma-ratio
B 124 / 264 / 26, backward 115 / 270 / 27, forward over the recurrence B 104 / 262 / 27; with R correctly rounded,
exp(a ln x + b ln y) 118 / 240 / 101, libm pow 116 / 221 / 180, x and y from the variances without f 111 / 251 / 99, VAR.S's
one-pass variance 90 / 232. Every R variant keeps the same worst cells (26, 14, 13 and 12 ulp), so the visibly-off
residual is probably not in R. Further scores (T-2571): ln of the larger side by log1p 140 / 271 / 24, pairwise variance
117 / 259, ln as a difference 105 / 203; A&S 26.6.4 for an even df_big (positive series, `f-test-as`) 126 / 251 at best; the finite sums
from the asin base (`f-test-finite`) fail in deep tails (cancellation) but get several body cells the model misses.
One degree of freedom on each side (a = b = 1/2) is a closed form in Excel: `2*asin(sqrt(x))/pi` is bit-exact on 13 of
the 16 such cells on file ((2/pi)*asin 11, 2/pi*atan(sqrt(x)/sqrt(y)) 10, the series model 3); the engine uses it
since T-2571 (6 gen-m9 cells fixed, among them `F.TEST({0.02,0.027},Data!D6:D7)`, gen-m9/8f2ac25945af87da A241, and
gen-m9/c468f5046aadfbbc A192). Whether Excel's other half-integer shapes go through A&S's theta forms is open (T-2681).
The residual is not in the variance ratio f (T-2571): the two-pass double variances are within 3 ulp of the exact
ones on all but 14 of the 291 cells (and on those Excel follows the double f), the tail at the exact f is further from
Excel (42 bit-exact, up to 173 ulp), and on the worst cells the f that would give Excel's p lies 6 to 70 ulps from the
exact f while the double f is within 3: the residual is in the tail's evaluation. Numerical
Recipes' `betai` in doubles (exp(lnB + a ln x + b ln y) or with three separate exps, `betacf`, with or without the
symmetry switch, ln y or ln(1 - x), libm or correctly rounded lgamma) scores at best 132 / 235 / 29, and ln y as
log1p(-x) under the model 146 / 272 / 24: none beats the model. These last scores (NR `betai`, log1p(-x), the exact-f inversion) and the closed form's
13 / 16 against (2/pi)*asin 11 and atan 10 were run as ad hoc scripts; T-2681 committed them (`uv run rustsheet-fitlab
f-test-half` and `f-test-nr`, tools/src/rustsheet_tools/fitlab/kernels/ftest_adhoc.py). They reproduce the closed forms
(13 / 16, 11, 10, the series 3) and log1p(-x) (146 / 272 / 24); the model at the exact variance ratio scores 141 / 259 (max
311 ulp); NR `betai` with libm lgamma scores at best 106 / 198 / 51 max ulp (three exps, ln(1 - x)), so T-2571's 132 / 235 came
from a variant the committed fit does not have (its correctly rounded lgamma, unrecorded).
Both shapes above 1 (df 3 or more on each side; rule from a fit, PARTIAL): F.TEST's tail is TOMS 708 bratio with
Excel's pieces, T-2464 r2-bratio's integrated model (`docs/research/t2464/r2-bratio/bratio_excel.py`, `BEST`), not the
remainder model above. After bratio's swap on lambda = (a+b) y - b (a > b) or a - (a+b) x, with b0 below 40:
- b0 x0 <= 0.7: `bpser` as w3 found it, s = 1/a0 + sum c_n/(a0+n), c_n = c_(n-1) ((1/2 - b0/n) + 1/2) x0, every step an
  x87 op stored to a double, summed until s stops changing, times EXP of the unstored fl64(fl(a0 LN x0) - ln B), with ln B
  fl64(GL(a0) + fl64(GL(b0) - GL(a0+b0))) (GL Excel's GAMMALN; ln Gamma(1/2) as fl64(gamln1(1/2) - LN(1/2))) up to a
  shape of 8 and the accurate ln B above;
- otherwise, with x0 <= 0.7 (TOMS L140): `bup` from the shape b0 - n (1/2 or 1) up by n = the integer part of b0, its
  prefactor Excel's beta density at y0 times y0 x0 over the shape (BETA.DIST's density for a shape of 1/2 or 1, T-2513's
  model M2; with r2-bratio's w1 model M1 instead the F.TEST hybrid is 192 bit-exact, not 205) and its ratios stored, plus `bpser` at that shape;
- p = 2 min(w, (1/2 - w) + 1/2).
On the 291 cells of the `f-test` fit the 113 with both shapes above 1 score 68 / 110 / 1 visibly off under bratio against
31 / 96 / 7 for the remainder model; with a shape of 1/2 bratio's `bpser` is worse (28 / 88 / 23 visibly off of 116
against 75 / 114 / 0), so the model stays there. What decides: w3's series with the unstored EXP argument (plain TOMS
`bpser` 38 / 76 / 24, the argument stored to a double 47 / 80 / 22) and bup's density prefactor (TOMS `brcmp1` 51 / 100 /
8, a correctly rounded density 64 / 107 / 2); the arithmetic switches move at most one cell. The engine's port
(below, with M2) is 76 / 110 / 1 on those 113 and 205 / 284 / 1 on all 291 against 160 / 270 / 7 before: 7 of the 8 cells
T-2571 left visibly off now have Excel's 15 digits (gen-m9/58e3253bbd3a7a82 A141, 8f2ac25945af87da A119 and
a7bf0d8d81d33ba6 A81 bit-exact; fec39efaba9284c3 A36 1 ulp, caa1eed48b1883fe A380, f34f66fb7f81e673 A4 and the FILTER cell
bcb24385baafb2f0 A377 3 ulp). Still visibly off:
`F.TEST(Data!B6:E12,Data!A1:D10)`, gen-m9/b6593d7dd13f2dac A306 (Excel 1.23966888990022E-69, ours 25 ulp off; data with
123456789.123 among values near 1; T-2829). 8 cells move from bit-exact to 1 to 3 ulp (T-2830). Evidence and per-branch
tables: docs/research/t2681/README.md, `score_bratio.py`.
- F.TEST is 2·F.DIST.RT at Excel's own f, to the bit: at gen-m9/b6593d7dd13f2dac A306's data
  `F.TEST(..)` and `2*F.DIST.RT(VAR.S(..)/VAR.S(..),23,12)` are both 1.23966888990022E-69. With both shapes above 1,
  Excel's bpser stores fl(a·LN x) as the port does: when the sample is scaled to move x by up to 1E5 ulp, the gap
  between ours and Excel stays a constant of the shapes (5.3E-15 relative at a = 6, b = 11.5, 1.4E-15 at b = 11).
  That constant is Excel's ln B(a, b): bpser's L is a stored double an integer number of ulps off fl(accurate ln B)
  (-3 at a = 6, b = 11.5, which is A306's whole miss; -1 at (6, 11); 0 at (2.5, 11), gen-m9/a790a4d4a6a82756 A289).
  On a grid of 99 F.TEST cells (a 2.5 to 8, b 8.5 to 19.5, x near 1E-12) every L is within 0.2 of an integer k ulps,
  k from -3 to +2 (|k| <= 1 at a = 2.5, 4, 4.5 and 5, 2 at a = 3, 3.5, 6, 7 and 8, -3 only at (6, 11.5)); the same
  test on L + ln a, L + ln b or L + lnGamma(a + b) gives random fractions.
  No form tried gives the map: accurate 45 of 99 (shipped), TOMS betaln 49, TOMS's a-reduction with w = prod a/(a + b)
  55, Excel GAMMALN sums 15 to 29 (docs/research/t2829/ln_beta_map.py; probes/t2829-ftest-a306-shapes and
  t2829-ftest-ln-beta-grid, excel-graph; open as T-2924). A306 stays 25 ulp low until that form is found.
  Ruled out by structure (T-2924, docs/research/t2924/structure.py):
  - Any sum of double lnGamma values, each within 2 ulp, in seven orders. lnGamma(b) and lnGamma(a + b) lie on a
    grid 4 ulp(L) wide, so 16 or more cells are unreachable in every order.
  - For integer a, TOMS's a-reduction fl(LN(w) + A(b)) with any w product, the exact ln of the product included:
    no single A(b) fits more than 1 of 11 b values.
  - Any per-argument error f(a) + h(b) + q(a + b) added to ln B and rounded once. At most 94 of 99 cells fit such
    an error, against 82 for random k.
  So Excel forms ln B jointly in (a, b), finer than a double lnGamma grid. Blocked as time-boxed (decision 0056). BETA.DIST's
  bpser cells with a shape above 8 were not scored.
  The F.DIST.RT cells of probes/t2829-ftest-tail-over-b (f = 1E12) are off ours by these k plus whole steps of the
  fl(a·LN x) grid (a = 6, b = 11.5: -143 ulp = +5.4E-15 - 2.8E-14 relative), and `2*F.DIST.RT(..,22,12)` differs from
  F.TEST at the b = 11 twin by such a step, so F.DIST.RT forms x or z differently there; F.TEST is the clean probe.
Evidence: the F.TEST cells of gen-m9 (excel-graph, T-2255 sweep), probes/stat-argument-modes
H248:H251, probes/ftest-decomposition A7:F13, probes-numeric m9-legacy-ftest and t1049-ftest.
Earlier findings kept: F.TEST is not `2*MIN(F.DIST(f,..,TRUE),F.DIST.RT(f,..))` of Excel's own
F.DIST (probes/ftest-decomposition A8 0.7947858780819459 against F.TEST's 0.7947858780819458).
Engine: `f_test_p` in crates/rustsheet-engine/src/functions/statistical/fisher.rs: `f_test_bratio_p`
(f_test_tail.rs, T-2681) for both shapes above 1 with a + b <= 200 where it reaches (not TOMS `bgrat`, `bfrac`
or `basym`), else `f_test_upper_tail` and `half_integer_beta::half_integer_remainder` (R in double-double for a + b <= 200,
`beta_reg_complement` above).
Status: implemented as a fit (T-2361, T-2681); bratio for both shapes above 1 is PARTIAL (one cell visibly off, T-2829),
Excel's algorithm for R at a shape of 1/2 is UNVERIFIED.

## Paired functions: a size mismatch is `#N/A` before an error cell (T-1927)
COVARIANCE.S, COVARIANCE.P, COVAR and a paired T.TEST (type 1) of arguments with different cell
counts are `#N/A` even when a cell holds an error: `COVARIANCE.S(H1:H3,I1:I4)` with H2 `=1/0` is
`#N/A`, as is `COVARIANCE.P(I1:I4,H1:H3)` with the order swapped; with equal sizes the error wins
(`COVARIANCE.S(H1:H3,I1:I3)` `#DIV/0!`). CORREL, PEARSON and RSQ already check sizes first
(`#N/A`). The tests that allow unequal sizes return the error: `T.TEST(H1:H3,I1:I4,2,2)` and
`F.TEST(H1:H3,I1:I4)` are `#DIV/0!`. Evidence: probes/ftest-decomposition G1:G13 (excel-graph,
2026-10-22). An error inside an array constant was unprobed here (`COVARIANCE.S({1,#DIV/0!,3},{2,4,5,7})`);
the gen-m9 sweep answers it for equal sizes: see "An error element of an array argument is that error (T-2255)" below.
Engine: `Model::paired_samples` (`PairedSamplesError::sizes_differ`) in
crates/rustsheet-engine/src/functions/statistical/paired.rs.
Status: implemented (T-1927)

## FORECAST.ETS family on excel-graph: argument errors, text and lifting decide before the #N/A (T-2255)
Rule: although excel-graph never computes the family (the section "FORECAST.ETS family: excel-graph never computes it"),
it reads the arguments first, so these results are web evidence that does not depend on the algorithm. An error value in
a scalar argument (target_date, seasonality, data_completion, aggregation, confidence_level, statistic_type) or a direct
error in values or timeline is the result, and the first such argument in argument order wins: `FORECAST.ETS(ROUNDUP(0.1,NA()),
Data!C2:E7,..)` is `#N/A` (gen-m9/22656c225ff4f22a Calc!A1), `FORECAST.ETS.SEASONALITY(Data!A6:D6,SMALL(..)<#NUM!>,1)`
`#NUM!` (A32), `FORECAST.ETS.SEASONALITY(Data!D7:D9,Data!D7:E12,1,<#N/A>)` `#N/A` (A28); a text scalar is `#VALUE!` in
its turn, before a later argument's error (`FORECAST.ETS(Data!C$4,BYROW(..)<#CALC!>,..)` with C4 = "ABC" is `#VALUE!`,
A69; a text statistic_type, A131; a text confidence_level read from a cell holding "#N/A" as text, A150); an array or
multi-cell range in a scalar argument lifts, so the anchor is `#SPILL!` in the generated layout
(`FORECAST.ETS.STAT(..,(AVERAGEIF(..)+OFFSET(Data!B6:D10,5,-1)))`, A31). Of the 1,173 top-level FORECAST.ETS* cells of
gen-m9 whose web value is not that plain `#N/A`, the first error argument decides 967, a text scalar 36, a lifted
argument 54; 116 have an error element inside an array argument or an inner function that diverges from ours.
A web `#N/A` with no error argument stays undecidable on the web (T-1553).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), e.g. 22656c225ff4f22a Calc!A1, A28, A31, A32, A69, A131, A150;
1782efcecd5d4b1b A27, 3462856259a37891 A28 (the earlier section's two `#VALUE!` cells are this rule).
Ours: since T-2424, the argument step (`statistical/forecast.rs` `forecast_ets_lifted`). Every scalar argument lifts and
reads as a number in turn, and `values` and `timeline` are evaluated once for a direct error. A reference to an
error cell, even a one-cell one, is not a direct error here: the call stays `#N/IMPL!`, and no oracle cell covers it yet
(the gen-m9 Data sheets hold "#N/A" only as text; T-2425 probe). The first failure in
argument order decides, and only a call with none stays `#N/IMPL!` (T-2425 the values, after T-1553). FORECAST.ETS.SEASONALITY
lifts its data_completion and aggregation too. Counts: the task names 1,215 known-failure lines (1,196 from T-2255
and 19 from T-2256); with the 19 visibly-off lines of T-2257 and T-2258 it owns 1,234 cells. 656 of them leave the
baseline with T-2424, 481 had already passed on main before it, and 97 still differ. The 97 are:
- 84 are web `#N/A` with no error argument, or with an error element inside `values` or `timeline`, which this step does
  not read;
- 13 differ in an inner function (GETPIVOTDATA, KURT, XIRR, MODE.MULT, NPV at rate -1, SYD, PROB and others), not in this
  step.
Status: VERIFIED (excel-graph) for the argument step
Tasks: T-2028, T-2424, T-2425, T-1553

## QUARTILE reports quart's error before empty data (T-2455)
Rule: `QUARTILE(B5:B6,SUMIFS(C1:E1,B11:D12,1))` with no number in B5:B6 is the quart's `#VALUE!` (the SUMIFS of mismatched
ranges, A58), not the empty-data `#NUM!`; with a valid quart it is `#NUM!` (`QUARTILE(B5:B6,1)`)
(probes/t2455-transpose-single-value A59, A60, excel-graph, 2026-10-23; gen-m9 899f9555e67da2d6 A344 through LINEST).
QUARTILE.INC is the same (A83).
Ours: since T-2455 (`fn_quartile_inc`, QUARTILE and QUARTILE.INC); QUARTILE.EXC keeps the empty-data check first (unprobed).
Status: VERIFIED (excel-graph)
Tasks: T-2455

## F.TEST of a single value (T-2455)
Rule: F.TEST takes any single value as a sample of at most one, so it is `#DIV/0!` whatever the value's type: `FTEST(5,{1;2;3})`,
`FTEST({1;2;3},SUM(4))`, `FTEST(TRANSPOSE(5),{1;2;3})`, `FTEST(B1,{1;2;3})` with B1 = 5, `FTEST("5",..)`, `FTEST("abc",..)`,
`FTEST(TRUE,..)`, `FTEST(5,6)` and `FTEST({1;2},5)` (probes/t2455-transpose-single-value A8:A36, excel-graph, 2026-10-23); a direct
error is that error (`FTEST(1/0,..)` `#DIV/0!`, A29). A one-cell reference to a blank cell is `#VALUE!` (next entry): `FTEST(B10,{1;2;3})`,
`FTEST(5,B10)`, `FTEST(B1,B10)`, `FTEST(B10,B1)` (A30:A39) and gen-m9 bbdfc06cae0e0d71 A34 `F.TEST(Data!B9:B9,<number>)`, while a
blank two-cell range is `#DIV/0!` (`FTEST(B10:B11,{1;2;3})`, A33).
F.TEST checks its samples in order, each one's error or blank one-cell reference before the next: `FTEST(1/0,B10)` is
`#DIV/0!` and `FTEST(NA(),B10:B10)` `#N/A` (A77, A78), so for F.TEST the blank does not beat an earlier sample's error; a
computed blank (`FTEST(IF(TRUE,B10),{1;2;3})`) is `#VALUE!` too (A79). An omitted sample (`FTEST(,{1;2;3})`) is unprobed
(ours: the one-value `#DIV/0!`).
Ours: since T-2455 (`fn_f_test`, `f_test_sample`, `is_blank_one_cell` in functions/statistical/fisher.rs). The paired
functions' blank-cell `#VALUE!` followed in T-2426.
Status: VERIFIED (excel-graph)
Tasks: T-2455, T-2426

## LINEST and LOGEST report error values first (T-2455)
Rule: LINEST and LOGEST first report an argument that is itself an error value, in argument order (known_y, known_x, const,
stats), and only then check the data: an error element of a range or array, a single value that is no number, too few points,
unequal rows (`#REF!`) and text. So `LINEST({1;2},1/0,"x")`, `LINEST(1/0,{1;3},"x")`, `LOGEST({1;2},1/0,"x")` and
`LINEST({1;2},1/0,UNIQUE(1,"1"))` are `#DIV/0!` (probes/t2455-transpose-single-value A4, A6, A25, A27; gen-m9 4e55500bb8ad3cd8
A178), while an error const or stats beats the size and point checks: `LINEST(Data!C3:C7,Data!B5:B10,NA(),"1")` is `#N/A`, not
`#REF!` (gen-m9 05d44f6dd6cfc567 A195), `LINEST(5,{1;2},NA())` and `LINEST(INT(999999999999999),42,NA(),TRUE)`
`#N/A` (A50), `LINEST(<TRUE cell>,0.5,"FALSE",NA())` `#N/A` (c6cfda6bd10cae93 A110), and an error element of x comes after
them (`LOGEST(Data!E$11,IF(MAP(..)),(MODE.MULT(..)<"1.5"))` is the const's `#N/A`, 35e8168aae2b00f8 A400). A text const or
stats is `#VALUE!`: `LINEST({1;2},{1;3},"x")` (A7), `LINEST({1;2},{1;3},TRUE,"x")` (A26), and `UNIQUE(1,"1")` as const (A5).
A computed blank known_y is `#VALUE!` before known_x's and const's errors (`LINEST(IFERROR(B10,1),1/0)` and
`LINEST(IFERROR(B10,1),{1;2},NA())` with B10 blank, A68, A84; `LOGEST(IFERROR(B10,1),PEARSON(..)<#N/A>)` A66, gen-m9
623663cacea53bbc A144), while a blank reference and a computed text or boolean y wait (`LINEST(B10,1/0)`, `LINEST(5,1/0)`,
`LINEST("abc",1/0)`, `LINEST("5",1/0)` and `LINEST(IF(TRUE,"abc"),1/0)` are `#DIV/0!`, `LINEST(TRUE,{1;2},NA())` `#N/A`;
A62, A67, A73:A76).
Ours: the order since T-2455 (`read_regression_inputs`, `validate_regression_inputs`, `read_matrix` in
functions/statistical/linest.rs). An error cell in known_y waits too (`LINEST(B20:B21,1/0)` with B20 `#N/A` is `#DIV/0!`,
`LOGEST(B20:B21,{1;2},NA())` `#N/A`, A89, A94), an omitted known_y is `#VALUE!` (`LINEST(,1/0)`, A90), and TREND and GROWTH,
which share the reader, follow the same y-then-x order (`TREND(B20:B21,1/0)`, `TREND("abc",1/0)`, `GROWTH(B20:B21,1/0)` and
`GROWTH("abc",1/0)` are `#DIV/0!`, A85:A88). A defined name for a blank cell as known_y is unprobed (ours: the computed-blank
`#VALUE!`). A text const or stats is read as a boolean (0.5 for A7, A26), T-2556.
Status: VERIFIED (excel-graph)
Tasks: T-2455, T-2556

## A one-cell reference to an empty cell is #VALUE! in the paired functions and F.TEST (T-2255)
Rule: COVAR, COVARIANCE.P, COVARIANCE.S, CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX, FORECAST, FORECAST.LINEAR,
CHISQ.TEST, CHITEST, T.TEST, TTEST, F.TEST and FTEST are `#VALUE!` when an array argument is a one-cell reference to an
empty cell (`Data!C4`, `Data!E9:E9`), before the size check and before the other argument's error: `COVARIANCE.P(Data!D9:E10,
Data!C4)` (27601819397dd56e A257), `COVAR(Data!B3:D6,Data!$B6)` (8dd2312c96effb1b A165), `CHITEST(Data!C1:E4,Data!A2)`
(f2bd5f380ec002bf A84), `F.TEST(Data!C8:C8,Data!Z1:Z3)` (b4173fe25109a238 A348), `CORREL(Data!E9:E9,{100000000.1,
100000000.2,100000000.3})` (fec39efaba9284c3 A255, not the size `#N/A`), `INTERCEPT(Data!D1,AVERAGEIF(..)<#DIV/0!>)`
(623663cacea53bbc A178, not `#DIV/0!`), each referenced cell empty; nested the same (`MONTH(CORREL(Data!D10,Data!A7))`,
fcf11f2ec6f7a59d A9, the T-2051 question). A one-cell reference holding a number, text or a boolean stays one value
("Array arguments given a scalar, a single cell or an error" above; `FTEST(6,Data!A$11)` with A11 = "x" is `#DIV/0!`,
35e8168aae2b00f8 A253). This contradicts the earlier inference that an empty cell is one value with no number.
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 109 cells listed in T-2426.
A reference returned by IF is `#VALUE!` too (`FORECAST(Data!$D2,IF(ISNUMBER(Data!E2),Data!A10,Data!D$5),LEN("ß"))`, 4250f880ab640c49 A294); a blank
from IFERROR or LET is UNVERIFIED for the paired functions (F.TEST: `#VALUE!`, A79). PROB follows the same rule (see its entry, T-2468).
Ours: F.TEST and FTEST since T-2455 (`is_blank_one_cell`); the paired functions since T-2426 (`blank_one_cell_as_error` in
functions/statistical/paired.rs, called by `fn_get_two_matrices` and `paired_samples`).
Status: VERIFIED (excel-graph)
Tasks: T-2426, T-2051

## Paired functions pair two ranges of equal count whatever their shapes (T-2255)
Rule: CORREL, PEARSON, RSQ, SLOPE, STEYX, INTERCEPT, FORECAST, FORECAST.LINEAR, CHISQ.TEST and CHITEST accept two arrays with the same number of cells
and different shapes, pairing them in reading order; only a different count is `#N/A`: `CORREL(Data!B11:E12,Data!D9:E12)`
(2x4 against 4x2) is 1 (2a18b2f5a4dc43ac A370), `CORREL(Data!B10:D12,Data!C3:C11)` (3x3 against 9x1) -0.9888852938996431
(a54c738c697617c3 A291), `INTERCEPT(Data!E5:E8,Data!B3:C4)` -0.15000001236262528 (f2bd5f380ec002bf A94),
`CHITEST(Data!D10:E12,Data!A9:C10)` 0 (0a9226016e8f5f33 A291), `FORECAST.LINEAR(x,Data!C8:E9,Data!C7:D9)` `#DIV/0!`
(7674c89c824aca43 A72). Which cell pairs with which (reading order, row by row) follows from the values only where they
differ; UNVERIFIED beyond these cells.
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 17 cells listed in T-2431; T-2256: `RSQ(Data!A4:C7,Data!C2:D7)` 0.3991684810183792 (329b177b5d5165ad A104), `SLOPE(Data!A7:D9,Data!D3:E8)` 1.0000000246913243 (334f3dc9ca937f10 A63), `PEARSON({3,3,3,3},Data!B8:C9)` `#DIV/0!` (38e4fc309fe199ea A26), `STEYX(Data!C8:E10,Data!C1:C9)` `#DIV/0!` (7980ffb6095a340d A352); T-2436 probe: `FORECAST(<FALSE>,Data!C1:D3,Data!C10:E11)` (3x2 against 2x3) `#DIV/0!` (b4173fe25109a238 A78).
Ours: pairs by cell count in reading order (T-2431). For CHISQ.TEST and CHITEST of different shapes the degrees of freedom come from the first array's shape: UNVERIFIED, no oracle cell separates the choices (T-2650 probes `CHISQ.TEST({1;2;3;4},{2,2;2,2})` against `CHISQ.TEST({1,2;3,4},{2;2;2;2})`, df 3 against 1).
Status: VERIFIED (excel-graph) for the cited cells

CHISQ.TEST/CHITEST have no per-cell sign check: a negative actual (`CHITEST(Data!D7:D9,Data!C10:E10)`, -1 against 1, 0.1353352832366127 = exp(-2), c6cfda6bd10cae93 A375) and a negative expected cell with a positive summed statistic (`CHITEST(Data!B5:E7,Data!B7:D10)` is 0, f2bd5f380ec002bf A314) are numbers; only a negative summed statistic is `#NUM!` (probes-numeric/m9-legacy-chitest A21, `CHITEST(G1:G3,M1:M3)`, G={1,2,3}, M={5,6,-1}).
Evidence: those three cells, excel-graph. Status: VERIFIED for them; a single negative expected with a positive total and all-numeric inputs is unprobed (T-2650).
Rule (T-2094): a side holding no number (blank, text or booleans in a range) is `#DIV/0!`: `CHISQ.TEST(Data!A10:B12,Data!A8:C9)` (0a9226016e8f5f33 A333, the actual side has only text, a boolean and blanks). Numbers on both sides that are never paired stay 1 (8dd2312c96effb1b A343). Still UNVERIFIED: blank against blank of equal shape beyond these cells (T-2650).

## An error element of an array argument is that error (T-2255)
Rule: an error inside an array constant or array value given to LARGE, SMALL, QUARTILE.EXC, PERCENTRANK.INC,
PERCENTRANK.EXC, MODE.MULT, COVARIANCE.S, CORREL, F.TEST and FTEST is the result (not `#VALUE!`), also when the other side
has one point or the error is not the k-th value: `LARGE({1,#N/A,3},3)` and `LARGE({1,#N/A,3},1)` are `#N/A`
(165db58d1540f014 A286, abc2939ebd1e2365 A135), `COVARIANCE.S({1,#N/A,3},{-1,0,1})` `#N/A` (2801c1c11c21e134 A139),
`CORREL({1,#N/A,3},Data!A11:C11)` `#N/A` (a54c738c697617c3 A352), `F.TEST({-1,0,1},{1,#N/A,3})` `#N/A`
(165db58d1540f014 A337), `FTEST({1,#N/A,3},AVERAGE(0.2))` `#N/A`, not the one-point `#DIV/0!` (9cca17438a7e6f8a A43).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255); cells in T-2430 and T-2093.
Another argument's error beats the array's error element: `PERCENTILE({1,#N/A,3},(<FALSE>/"x~*"))` and `COVARIANCE.S({1,#N/A,3},RANK(<#VALUE!>))`
are `#VALUE!`, as is `TRIMMEAN({1,#N/A,3},{"a"})` (the percent does not coerce); an error value as the first (data) argument still comes first
(gen-m9/31f09e6c48ca245c A14, 165db58d1540f014 A353, 9735c5ab72fa5ecf A36). That holds for an element's error that an inner call has already
returned as a value: `SMALL((SMALL({1,#N/A,3},2)/1.005),Data!C8)` with Data!C8 = `a*` is `#N/A`, not k's `#VALUE!` (gen-m9/1b31092a0d79e5f1
A363). Ours gives `#VALUE!` because the inner call's tag stays on the error (T-3080).
Probed on excel-graph (T-2636, probes/array-error-element-order): PROB (`PROB({1,#N/A,3},{0.2,0.3,0.5},1)`, an element in
either data argument, with a limit error or an extra upper limit) and SLOPE, INTERCEPT, STEYX, RSQ, FORECAST.LINEAR and
CHISQ.TEST with the element in either position are all `#N/A`; with an element error in each data argument the first
argument's wins (`SLOPE({1,#N/A,3},{1,#DIV/0!,3})` is `#N/A`). A scalar bins error beats a data-array error element
(`FREQUENCY({1;#DIV/0!;3},NA())` is `#N/A`), but an error element of FREQUENCY's *bins* array is skipped like text:
`ROWS(FREQUENCY(0,{1,#VALUE!,3}))` is 3, `INDEX(FREQUENCY(0,{1,#VALUE!,3}),1)` is 1, and
`ROWS(FREQUENCY(0,ABS({13,15,17,""}-13)))` is 4 (A1, A2, A4), as desktop Excel's cached value for corpus-bench/53062 shows.
A typed or computed boolean as data or bins, and numeric text `"2"` as a bin, are skipped (A8-A11). `FREQUENCY({1;2},{#N/A})` is
`#SPILL!` on the web (A3): bins made only of errors are no numeric bin, so the two-row `{0;n}` result spills into the occupied
A4 (as gen-m9/06b4e8f5de409d5e A239). Still unprobed: an error cell in a bins range (ours: that error), a data-array error element
against an error cell in a bins range, and an empty argument in the other position of the paired functions (T-2636 follow-up).
Ours: T-2430 implements it for LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN, MODE*, Z.TEST and the paired functions
(COVAR, COVARIANCE.*, CORREL, PEARSON, T.TEST); PROB since T-2636; FREQUENCY's data array since T-2427 and its bins array
skips an error element since T-2636; F.TEST and FTEST since T-2692 (`f_test_sample`, fisher.rs). The other argument's error winning is verified only for
PERCENTILE, TRIMMEAN and COVARIANCE.S and applied to every function above.
Status: VERIFIED (excel-graph)

## F.TEST: an error argument is that error; overflowing squares are #NUM!, KURT's #DIV/0! (T-2255)
Rule: F.TEST and FTEST return an error argument's error (`F.TEST(Data!B6:B11,INDEX(Data!B6:E6,5))` `#REF!`,
05d44f6dd6cfc567 A51; `F.TEST(INDIRECT("x~*"),Data!D9:D10)` `#REF!`, 58e3253bbd3a7a82 A377; `F.TEST(Data!B6:E12,
-HOUR(123456789.123))` `#NUM!`, eced4bb6a0234c9e A258), the first argument's first. Data whose sum of squares overflows
(`{1E+200,1E+200,-1E+200}`) make F.TEST `#NUM!` (`F.TEST(Data!A3:A10,{1E+200,1E+200,-1E+200})`, 2a18b2f5a4dc43ac A130;
four more cells) and KURT `#DIV/0!` (`KURT({1E+200,1E+200,-1E+200},Data!E4:E12)`, a61e547a1cab994c A14; five more).
Explained since by T-2436 ("ERROR.TYPE and CELL results are #NUM! in the paired functions" below): three cells are `#NUM!` where a number against a range gives `#DIV/0!` elsewhere:
`FTEST(Data!C3:C9,ERROR.TYPE(#REF!))` (4250f880ab640c49 A325), `F.TEST(CELL("col",Data!$B$12),(PI()+FILTER(..)))`
(623663cacea53bbc A400), `F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})` (06b4e8f5de409d5e A261); owned by
T-2576 and T-2577.
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255).
Ours: `#VALUE!` for an error argument (T-2093); the F.TEST overflow `#NUM!` (fisher.rs) and, since T-2432, KURT's `#DIV/0!`
("Overflowing squares (T-2432)").
Status: VERIFIED (excel-graph); the INDIRECT cell's `#NUM!` (06b4e8f5de409d5e A261) stays UNVERIFIED (T-2577)

## FREQUENCY: one row per bin plus one, even for one bin or a text bin; an error argument is that error (T-2255)
Rule: FREQUENCY with a single bin (a number, a one-cell reference, a cell holding text or a boolean) or a single text or
boolean data value still returns its bins-plus-one column, so in the generated layout (the next formula one row below)
the anchor is `#SPILL!`: `FREQUENCY(Data!B11:C12,Data!B3:B3)` (0a614cfe974b59c3 A64), `FREQUENCY(Data!E5:E7,Data!D7:D7)`
(589f42e67f4b4b8d A384), `FREQUENCY(-0.5,Data!A4)` with A4 = TRUE (8fec7b207f4dd753 A245), `FREQUENCY(Data!C$7,Data!A7)`
with C7 = " x " (3e98b26cf6713e77 A249). The row count for a text or boolean bin is settled by probes/frequency-bins
(below). An error argument is the result: `FREQUENCY((0*CHITEST(..)),(-2.5))`
`#N/A` (05d44f6dd6cfc567 A65), `FREQUENCY(Data!B6:D10,YEAR(VDB(..)))` `#NUM!` (b4173fe25109a238 A184); which wins when
both data and bins are errors is settled by probes/frequency-bins below.
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 142 cells listed in T-2427.
Resolved by probes/frequency-bins (excel-graph, 2026-10-24): with no numeric bin (a text, boolean or blank cell or a
typed text or boolean) the result is two rows, 0 then every numeric data value; text in data, and text or booleans in bins, are
skipped (`{1;2}` against `"a"`, TRUE, a blank cell, `{"a";"b"}`; `FREQUENCY("a",{1;2})` is three rows of 0 counts). Error
precedence: a scalar error argument (data, then bins) beats an error element of either array, and within arrays data
comes first: `FREQUENCY({1;#DIV/0!},NA())` and `FREQUENCY(NA(),{1;#DIV/0!})` are `#N/A`, `FREQUENCY(1/0,NA())` is
`#DIV/0!`, `FREQUENCY({#N/A;1},{#DIV/0!;1})` is `#N/A`.
Probe cells B4 and B7 confirm the T.TEST entry's order.
UNVERIFIED in ours, each skipped with no oracle cell (T-2636 probes them): a typed or referenced text that reads as a
number (`FREQUENCY({1;2},"2")`), a typed or computed boolean as data, and a computed boolean bin. Also unprobed: a
data-array error element against an error cell in a bins range.
Ours: implemented, T-2427, only as far as the probe goes. An error element of the data array is the result, and a scalar
bins error beats it. An error element of the bins array keeps `main`'s `#VALUE!`: no oracle cell shows it, and desktop
evidence points elsewhere (corpus-bench/53062 caches a number with a `#VALUE!` bins element; T-2636 probes it). An error
cell in a data range keeps `main`'s data-first order against a scalar bins error, UNVERIFIED (T-2636). The cells still off are other functions' gaps (ODDFYIELD, FORECAST.ETS, EXPAND, DB, NPER, ...).
Status: VERIFIED (excel-graph) except the noted UNVERIFIED parts

## Booleans from a reference or a formula are numbers in the F, beta, chi-squared and hypergeometric functions (T-2255)
Rule: a TRUE or FALSE that comes from a cell or a formula (not typed in the call) reads as 1 or 0 in F.INV, FINV,
F.INV.RT, BETAINV, CHISQ.INV and HYPGEOMDIST, as T-1849 found for array elements: `F.INV(Data!A8,6,2)` with A8 = FALSE
is 0 (285d87cea1aaa73c A324; also 40425341c5034eed A91, 58e3253bbd3a7a82 A109), `F.INV.RT(..,4,Data!$B12)` with
B12 = TRUE is 562.0832962779946 (c6cfda6bd10cae93 A183), `HYPGEOMDIST(1,1,Data!D7,10)` with D7 = TRUE is
0.10000000000000002 (ee2497a5e683c5d9 A356), `HYPGEOMDIST(1,IFERROR(..,ISERROR(#N/A)),6,19)` 0.3157894736842105
(b6593d7dd13f2dac A133), `CHISQ.INV(0.001,IFNA((Data!A9<>""),..))` 1.57079714926249E-06 (146b46e74f2dac3b A45). An error
in a later argument then wins over the boolean (`BETAINV(Data!C$5,(1E15/Data!C1),1.999)` with C5 = TRUE and C1 empty is
`#DIV/0!`, 165db58d1540f014 A257). The T-2256 sweep adds WEIBULL, WEIBULL.DIST and STANDARDIZE: `WEIBULL(38.5,44439,Data!C6,0.5)`
with C6 = FALSE is `#NUM!` (beta 0; d37313406b9f6e55 A380), `WEIBULL.DIST(0.2,Data!B1,0.01,..)` with B1 = FALSE `#NUM!`
(928d08e3e1eecb5f A303), `STANDARDIZE(-40,Data!E5,-1)` with E5 = FALSE `#NUM!` (sd -1; 334f3dc9ca937f10 A219) and
`STANDARDIZE(-1E-15,1E-15,Data!A7)` with A7 = TRUE -2E-15 (928d08e3e1eecb5f A249).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 90 cells listed in T-1849; T-2256 (2026-10-23), 28 more cells.
The BETA.DIST/BETA.INV bounds A and B read a boolean the same way (`BETADIST(1E+10,Data!A2,0.5,0.2,Data!A2)` with A2 = TRUE is `#NUM!`, 165db58d1540f014 A52; `BETAINV(1-1E-10,1E+10,1E-10,Data!D2)` with D2 = TRUE `#NUM!`, bc40d37cb4d8301d A145). A TRUE typed directly in alpha, beta, the HYPGEOM counts, WEIBULL and the STANDARDIZE mean and standard_dev has no oracle cell: UNVERIFIED (T-2559 probes it).
Ours: matches since T-1849 (the kernels use `cast_number_ref`, so a boolean reads as 1 or 0).
Status: VERIFIED (excel-graph) except the typed-direct reads named above

## HYPGEOMDIST: a sample below the possible range is 0; a negative or huge count is #NUM! (T-2255)
Rule: HYPGEOMDIST and HYPGEOM.DIST return 0 when sample_s is below number_sample - (number_pop - population_s), the
smallest possible count (`HYPGEOMDIST(3,5,5,6)`, `HYPGEOMDIST(0,5,6,9)`, `HYPGEOMDIST(2,5,10,11)` are 0: c468f5046aadfbbc
A85, 4bad303d32cc3f46 A267, 4250f880ab640c49 A339; `HYPGEOM.DIST(1,2,7,7,FALSE)` 0, a7bf0d8d81d33ba6 A315); any negative
count, however small, is `#NUM!` (`HYPGEOMDIST(3,-0.025,5,9)`, 7674c89c824aca43 A386; `HYPGEOMDIST(5,1,-5E-20,2)`,
589f42e67f4b4b8d A2); so is a count of about 1E15 (`HYPGEOMDIST(1E15,2,2,17)`, a612e89321a6df9c A395;
`HYPGEOMDIST(2,3,8,1000000000000017.1)`, ee2497a5e683c5d9 A264; `HYPGEOM.DIST(3,4,1,753749999999999.8,FALSE)`,
a013a45a8fbcab4f A352). The threshold is 2^31 - 1, the integer-count functions' limit (PERMUT, COMBIN, BINOM.DIST's
trials): a count of 2147483647 or more in any of the four arguments is `#NUM!`, even a sample_s above the possible
range, while 2147483646.9 computes (`HYPGEOMDIST(1,2,3,2147483646)` 2.7939677238464347E-09, `HYPGEOMDIST(2147483646.9,2,2,17)`
0; `HYPGEOMDIST(1,2,3,2147483647)`, `HYPGEOMDIST(2147483647,2,2,17)`, `HYPGEOMDIST(1,2,3,2147483647.0000002)` `#NUM!`).
Every argument is read first: a text cumulative is `#VALUE!` beside a huge or negative count
(`HYPGEOM.DIST(1E+15,2,3,10,"abc")`, `HYPGEOM.DIST(1,-0.5,3,10,"abc")`), while the text "true" and a numeric -1 or 0.5
are a cumulative TRUE (`HYPGEOM.DIST(1,2,3,10,"true")` 0.9333333333333333). A negative fraction in population_s or
number_pop is `#NUM!` too (`HYPGEOM.DIST(1,2,-0.5,10,TRUE)`, `HYPGEOMDIST(1,2,3,-0.5)`).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 30 cells listed in T-2429; probes/t2429-hypgeom-huge-count A1-A54
and probes/t2429-hypgeom-threshold A1-A47 (excel-graph, 2026-10-25); probes/t2429/<case>, each owned gen-m9 cell's
HYPGEOM call and its arguments on that case's Data sheet.
Ours: since T-2429, `#NUM!` for a negative or huge count (`limits::COUNT_LIMIT_EXCLUSIVE`), checked on every count
before truncation; the impossible sample is 0 for the pmf and the cdf since T-2699 (hypegeom.rs
`hyp_geom_dist_of`; probes/t2429-hypgeom-huge-count A5, A6 and their rows, `HYPGEOMDIST(1,N,3,N)`, are its cells).
T-2700 confirmed the sign-before-truncation order for the HYPGEOM counts.
Status: VERIFIED (excel-graph)

## A single cell holding a boolean is no number in the PERCENTILE and QUARTILE data (T-2429)
Rule: as a single cell holding text ("A single cell holding text is no number" above), a single cell holding TRUE is no
number in the data of QUARTILE, QUARTILE.INC/.EXC and PERCENTILE, PERCENTILE.INC/.EXC: the data is empty and the call
`#NUM!` (`QUARTILE.EXC(A1,1)`, `QUARTILE(A1,1)`, `PERCENTILE.INC(A1,0.5)` with A1 = TRUE), after quart's or k's own
error (`QUARTILE.EXC(A1,NA())` `#N/A`, `QUARTILE.EXC(A1,1/0)` `#DIV/0!`, `QUARTILE.EXC(A1,"a")` `#VALUE!`), while a typed
TRUE stays the one value 1 ("QUARTILE(TRUE,1)", but `QUARTILE.EXC(TRUE,1)` is `#NUM!` for one value). PERCENTRANK of it is
`#N/A` and `MEDIAN(A1)` `#NUM!`. So `QUARTILE.EXC(Data!D7,HYPGEOMDIST(2,3,7,2^32))` with D7 TRUE is `#NUM!`
(gen-m9/38e4fc309fe199ea A5).
Evidence: probes/t2429-quartile-exc-no-number A1-A24 (excel-graph, 2026-10-25).
Ours: since T-2429 (`percentile_numbers`); `#VALUE!` before.
Status: VERIFIED (excel-graph)

## GAUSS of a tiny argument is x times phi(0) (T-2255)
Rule: GAUSS(z) keeps its leading term for tiny z: `GAUSS(1E-300)` is 3.989422804014327E-301 and `GAUSS(-1E-300)`
-3.989422804014327E-301 (gen-m9/4bad303d32cc3f46 A225, 05d44f6dd6cfc567 A233), `GAUSS(1E-15)` 3.989422804014328E-16
(a54c738c697617c3 A235), so it is not NORM.S.DIST(z) - 0.5 in doubles (that gives 0 and 4.440892098500626E-16).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255), 8 cells listed in T-2434.
Cut: the leading term is exact only for tiny z. `GAUSS(1E-10)` is 3.9894310077670525E-11 and `GAUSS(4E-15)`
1.5543122344752192E-15, the cancelled Phi(z) - 0.5 (gen-m9/68d8ffb69ff739d8 A262, 146b46e74f2dac3b A46), so the switch
lies in (1E-15, 4E-15); ours uses z * phi(0) below 2E-15 (T-2434).
Ours: 7 of the 8 cells match; `GAUSS(1E-15)` is 3.9894228040143274E-16, one ulp below Excel (follow-up).
Status: VERIFIED (excel-graph)

## Scalar distributions at extreme arguments (T-2255)
Rule (each from one gen-m9 cell, excel-graph, 2026-10-22): `BINOM.DIST(0.1,2,1,0)` and `BINOM.DIST(0,3,1-1E-17,FALSE)`
(p = 1, fewer successes than trials) are `#NUM!` (05d44f6dd6cfc567 A344, a013a45a8fbcab4f A206); a slightly negative
number_s is `#NUM!` (`BINOM.DIST(-0.0125,10,1E-16,FALSE)`, 85f48d5cdfb24555 A35; `BINOMDIST(-0.0001,9,0.999999,..)`,
caa1eed48b1883fe A384); trials 2^32 is `#NUM!` (`BINOM.DIST(0.3,4294967296,1E-16,1)`, bc40d37cb4d8301d A56); BETAINV
with a tiny alpha is `#NUM!` (`BETAINV(1E-20,1E-300,30)`, `BETAINV(0.025,1E-15,2.5)`, `BETAINV(0.05,0.002,1)`:
3e98b26cf6713e77 A151, 9cca17438a7e6f8a A240, 8fec7b207f4dd753 A125) and so are `BETAINV(1E-200,0.1,1E+5)` (4ab2ba98d26c8a5e
A104) and `BETAINV(1-1E-16,2,0.01)` (a5828d3925575713 A315); `GAMMA.DIST(1E+307,100,0.5,FALSE)` and
`GAMMA.DIST(1E+307,30,1E+5,0)` are 0 (2801c1c11c21e134 A99, b6593d7dd13f2dac A231) while `GAMMADIST(1E-300,10000000001,3,
FALSE)` and `GAMMADIST(2,1.85E+76,1E+10,"TRUE")` are `#NUM!` (2801c1c11c21e134 A241, 40425341c5034eed A265);
`FDIST(1E-300,6,10000000000.5)` is `#NUM!` (c6cfda6bd10cae93 A363); `CRITBINOM(1E15,0.5,0.001)` and
`BINOM.INV(1E15,5.2E-16,0.5)` are `#NUM!` (a61e547a1cab994c A127, ee2497a5e683c5d9 A345); `LOGNORMDIST(1E+300,1,
5.277777777777777E-301)` is `#NUM!` (a013a45a8fbcab4f A97); `EXPON.DIST(7,102,FALSE)` is 0, not the subnormal
8.36257629135218E-309 (165db58d1540f014 A65, numbers.md "Subnormal results become 0").
Ours: matches since T-2435, except the GAMMA.DIST density and cdf at a huge shape (T-2982) and the
F.INV.RT far tail (T-2808); the rules T-2435 settled are the four sections below. A negative
BINOM.DIST/BINOMDIST number_s, one in (-1, 0) included (`BINOM.DIST(-0.0125,10,1E-16,FALSE)`), is `#NUM!` since T-2700 (the sign
is checked before truncation). Since T-2621 a BETA.INV quantile of 0 or 1 is `#NUM!`, which settles
`BETAINV(0.025,1E-15,2.5)`, `BETAINV(0.05,0.002,1)`, `BETAINV(1E-200,0.1,1E+5)` and `BETAINV(1-1E-16,2,0.01)`;
`BETAINV(1E-20,1E-300,30)` stays a number (ours 3.333333333333333E-302, the search's floor rather than an underflow to 0).
Status: VERIFIED (excel-graph), one cell each

## BINOM.DIST's pmf at k = 0 is #NUM! when p = 1 (T-2435)
Rule: `BINOM.DIST(0,n,1,FALSE)` and `BINOMDIST(0,n,1,FALSE)` are `#NUM!` from one trial on, while the
cdf there is 0 and `BINOM.DIST(0,0,1,FALSE)` is 1. This is Excel's closed form at k = 0 taking
LN(1 - p) = LN(0) ("BINOM.DIST pmf: closed forms at k = 0 and k = n"): the pmf away from k = 0
stays a number (`BINOM.DIST(1,2,1,FALSE)` is 0, `BINOM.DIST(2,2,1,FALSE)` is 1), p = 0 is a number
at every k, and BINOM.DIST.RANGE is 0 rather than `#NUM!` (`BINOM.DIST.RANGE(2,1,0)`).
Evidence: probes/t2435-dist-extremes A1:A13, A113, A114 (excel-graph, 2026-10-26); gen-m9
05d44f6dd6cfc567 A344 (`BINOM.DIST(0.1,2,1,0)`, the count truncated to 0) and a013a45a8fbcab4f A206
(`BINOM.DIST(0,3,1-1E-17,FALSE)`, whose p is 1 to the double).
Ours: matches since T-2435 (functions/statistical/binom.rs `binom_dist_of`).
Status: VERIFIED (excel-graph)
Tasks: T-2435

## BINOM.INV and CRITBINOM take BINOM.DIST's trial limit (T-2435)
Rule: the 2^31 - 1 trial limit of "BINOM.DIST is #NUM! from 2^31 - 1 trials" holds for the inverses
too: `BINOM.INV(2147483647,0.5,0.5)` and `CRITBINOM(2147483647,0.5,0.5)` are `#NUM!`, as is
`BINOM.INV(1E+10,0.5,0.5)`, while 2^31 - 2 trials give a count (1073741823), the count for
2147483646.9 as well (the trials truncate first).
Evidence: probes/t2435-dist-extremes A15:A19 (excel-graph, 2026-10-26); gen-m9 a61e547a1cab994c A127
(`CRITBINOM(1E+15,0.5,0.001)`) and ee2497a5e683c5d9 A345 (`BINOM.INV(1E+15,5.2E-16,0.5)`).
Ours: matches since T-2435 (`TRIALS_LIMIT` in functions/statistical/binom.rs now also in
`binom_inv_of`). The count itself at a huge trial count is ours to fix: `BINOM.INV(2147483646,0.5,
0.5)` is 1073744668 against Excel's 1073741823 (statrs' binomial cdf; T-2809).
Status: VERIFIED (excel-graph)
Tasks: T-2435, T-2809

## Normal densities are #NUM! once the square overflows (T-2435)
Rule: NORM.DIST, NORMDIST, NORM.S.DIST and PHI report `#NUM!` for a density whose standardized
value squares past the double range, so from |z| = 2^512 on, instead of the limit 0 they give
just below it: `NORM.S.DIST(2^512,FALSE)`, `NORM.S.DIST(1.35E+154,FALSE)`, `PHI(1E+200)`,
`NORM.DIST(1E+155,0,1,FALSE)` and `NORM.DIST(3,1,2^-512,FALSE)` are `#NUM!`, while
`NORM.S.DIST(1.3E+154,FALSE)`, `NORM.S.DIST(1.3407807929942596E+154,FALSE)` (SQRT of the largest
double), `PHI(1E+154)` and `NORM.DIST(1E+154,0,1,FALSE)` are 0. The sign does not matter
(`NORM.S.DIST(-1E+200,FALSE)` is `#NUM!`), and the cdf stays 1 or 0 at any magnitude
(`NORM.S.DIST(1E+300,TRUE)`, `NORM.S.DIST(-1E+300,TRUE)`).
Evidence: probes/t2435-dist-extremes A42:A58, A82:A84, A119:A125 (excel-graph, 2026-10-26);
probes/t2475-stat-subexpressions P!A572 (`NORMDIST(1E+300,0.3,2.5,0)`, T-2475); gen-m9
8dd2312c96effb1b A228, 4ab2ba98d26c8a5e A341 and 165db58d1540f014 A45 (`KURT(PHI(1E+300))`).
Ours: matches since T-2435 (`special::normal_density_overflows`, shared by NORM.DIST, NORM.S.DIST
and PHI).
Status: VERIFIED (excel-graph)
Tasks: T-2435

## BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)
Rule: BETA.DIST's density is `#NUM!` at an x of exactly 0 when alpha <= 1, and at an x of exactly 1
when beta <= 1 — and Excel tests the *given* x against 0 and 1, not the ends of the [A, B] interval,
so `BETA.DIST(0,0.5,2,FALSE,-1,1)` and `BETA.DIST(1,2,0.5,FALSE,0,5)` are `#NUM!` at an interior
point, while at the interval's own ends `BETA.DIST(2,0.5,2,FALSE,2,5)` and
`BETA.DIST(5,2,0.5,FALSE,2,5)` are 0. A shape just above 1 gives 0 (`BETA.DIST(0,1.0000001,2,
FALSE)`), a shape of exactly 1 is `#NUM!` (`BETA.DIST(0,1,2,FALSE)`, whose density is 2), and the
cdf is unaffected (`BETA.DIST(0,0.5,1E+5,TRUE)` and `BETADIST(0,0.5,2)` are 0). At an interval's own
end Excel gives 0 instead, at a shape of exactly 1 as well as a diverging one, so the density steps
down to 0 at the end: `BETA.DIST(2,0.5,2,FALSE,2,5)`, `BETA.DIST(5,2,0.5,FALSE,2,5)`,
`BETA.DIST(2,1,2,FALSE,2,5)`, `BETA.DIST(2,1,2,FALSE,2,3)` and `BETA.DIST(5,2,1,FALSE,2,5)` are all
0, while one step inside `BETA.DIST(2.0000001,1,2,FALSE,2,5)` is 0.6666666444444445. Which error wins when `B - A` leaves the double range is
unprobed: `BETA.DIST(0,2,3,TRUE,-1E308,1E308)`'s `#DIV/0!` ("probes/stat-overflow-arguments") is
decided before this rule, and excel-graph refused a workbook holding `BETA.DIST(0,0.5,2,FALSE,
-1E308,1E308)` (FileCorruptTryRepair), so no cell pins it.
Evidence: probes/t2435-dist-extremes A59:A67, A103:A107, A126:A129, A132:A140, A143:A146
(excel-graph, 2026-10-26); gen-m9 35e8168aae2b00f8 A48 (`AVERAGEA(BETA.DIST(0,0.5,1E+5,FALSE))`).
Ours: matches since T-2435 (functions/statistical/beta.rs `beta_dist_of`), the 0 at an interval's own
end included, at a shape of exactly 1 too; the last digits of the density at a tiny x and a huge
shape (A66) are T-2811's. Our end test is on the scaled t, so it also fires where x differs from A
but x - A cancels to 0 (`BETA.DIST(1E+300+1,1,2,FALSE,1E+300,1E+301)`): unprobed, T-2811.
Status: VERIFIED (excel-graph)
Tasks: T-2435

## The F cdf is #NUM! where df2/(df1 x) overflows (T-2435)
Rule: Excel forms the incomplete beta's argument as u = 1/(1 + df2/(df1 x)) ("Incomplete beta is
TOMS 708 `bratio`"), and where that quotient leaves the double range — a tiny x against a huge df2 —
every F cdf is `#NUM!`: `FDIST(1E-300,6,1E+10)`, `F.DIST(1E-300,6,1E+10,TRUE)`,
`F.DIST.RT(1E-300,6,1E+10)` and `FDIST(1E-300,1,1E+10)`, while one power of ten either way keeps the
quotient finite and gives 1 (`FDIST(1E-300,6,1E+9)`, quotient 1.67E+308, and `FDIST(1E-290,6,1E+10)`),
one power of ten the other way is `#NUM!` again (`FDIST(1E-301,6,1E+10)`, `FDIST(1E-302,6,1E+10)`).
A subnormal x never reaches the quotient — number text below the smallest normal double reads as 0,
so `FDIST(1E-308,6,1E+8)` is 1 (numbers.md "Subnormal results become 0"). The density is 0 there, not `#NUM!`
(`F.DIST(1E-300,6,1E+10,FALSE)`), and
a huge x whose df1 x overflows stays `#DIV/0!` for the cdf ("probes/stat-overflow-arguments").
Evidence: probes/t2435-dist-extremes A35:A41, A85:A91, A130, A131, A141, A142 (excel-graph,
2026-10-26); gen-m9 c6cfda6bd10cae93 A363 (`FDIST(1E-300,6,10000000000.5)`).
Ours: matches since T-2435 (functions/statistical/fisher.rs `f_beta_ratio_overflows`).
Status: VERIFIED (excel-graph)
Tasks: T-2435

## BETAINV is #NUM! where its quantile underflows, but 0.5 at a tiny alpha and a high p (T-2435)
Rule: BETAINV and BETA.INV are `#NUM!` wherever the quantile underflows to 0, which a tiny alpha
reaches quickly because the lower tail is x^alpha: at p = 1E-20, alpha 0.1 gives
2.05508004608823E-202 and alpha 0.5 gives 2.6399006310995205E-42, while alpha 0.05, 0.02 and 0.01
are `#NUM!` (their (p a B(a,b))^(1/a) is below 1E-308); `BETAINV(0.5,1E-3,1)` is
9.332636185032827E-302 = 0.5^1000 and `BETAINV(0.5,1E-5,1)` is `#NUM!`. Unexplained: with alpha
1E-300, where every positive x has cdf 1 to the double, p = 0.5 and p = 1E-20 are `#NUM!` but
p = 0.9 and p = 0.99 are exactly 0.5, the midpoint of [A, B] (`BETAINV(0.9,1E-300,30)`,
`BETAINV(0.99,1E-300,30)`), so Excel's search stops at its start there.
Evidence: probes/t2435-dist-extremes A68:A76, A108:A112 (excel-graph, 2026-10-26); gen-m9
3e98b26cf6713e77 A151 (`BETAINV(1E-20,1E-300,30)`).
Ours: the search's floor rather than 0 (`BETAINV(1E-20,1E-300,30)` is 3.333333333333333E-302), and
0.5 is not reproduced; T-2808.
Status: VERIFIED (excel-graph)
Tasks: T-2435, T-2808

## F.INV.RT saturates at 3.696066238512854E+150/df1 in the far tail (T-2435)
Rule: for a probability far in the tail and a df2 below 2, Excel's F quantile search stops at a
value that depends only on df1: `F.INV.RT(1E-200,1,1.9)`, `F.INV.RT(1E-200,1,1.99)`,
`F.INV.RT(1E-200,1,1.999)` and `F.INV.RT(1E-280,1,1.999)` are all exactly 3.696066238512854E+150,
and `F.INV.RT(1E-200,2,1.999)` is half of it, 1.8480349672907807E+150. A df2 of exactly 2 takes the
closed form instead (`F.INV.RT(1E-200,1,2)` is 1.0000000000000202E+200), and df2 = 1 gives
4.052847345693495E+199 at p = 1E-100.
Evidence: probes/t2435-dist-extremes A77:A81, A115:A118 (excel-graph, 2026-10-26); gen-m9
abc2939ebd1e2365 A11 (`CHIDIST(F.INV.RT(1E-200,1,1.999),10)` is 0, so the quantile is finite),
a612e89321a6df9c A23 and 01523d247594952e A377 (the far tail's last digits).
Ours: `#NUM!` where the quantile saturates, and 7 ulp off at `F.INV.RT(1E-20,3,9)`; T-2808.
Status: VERIFIED (excel-graph)
Tasks: T-2435, T-2808

## GAMMA.INV and CHISQ.INV at a huge shape: #NUM! from about 7E9, then a Wilson-Hilferty closed form (T-2982)
Rule: GAMMA.INV (and GAMMAINV, CHISQ.INV, CHISQ.INV.RT, CHIINV, with shape df/2 and scale 2) is close to the true
quantile up to a shape of 5E9 (`GAMMA.INV(0.5,5E9,1)` = 4999999999.666667, `CHISQ.INV(1E-16,10000000000.5)` =
9998837266.383266), mostly `#NUM!` at 7E9 and 1E10 (`GAMMA.INV(p,1E10,1)` is `#NUM!` for 17 of 18 p from 1E-300 to
1-1E-15; only p = 1E-30 gives 9998853641.00538; `CHISQ.INV.RT(1E-10,10000000000.5)` and `(0.001,..)` are `#NUM!`
while `(0.5,..)` = 9999999999.333334 and `(0.999,..)` = 9999562980.855858), and from 2E10 on the true quantile times a
constant c = 1.0002658235505755, at every shape up to 1E300 and every scale: `GAMMA.INV(0.5,a,1)` / a is c for a = 1E11
to 1E300 (A9-A23; `GAMMA.INV(0.5,1E13,1)` = 10002658235505.422 = c (1E13 - 1/3)), `GAMMA.INV(0.5,1E13,b)` is b times it
for b = 1E-300 to 1E280 (A78-A85), and `GAMMA.INV(0.5,1E300,1E8)` = 1.0002658235505756E+308 (A86). The tails are not
exactly c times the true quantile: at shape 1E13 and p = 1E-300 Excel's 10002541064587.973 is 12,972 below c times
it and 2,600 above the true quantile at shape c·1E13 (A24). The whole factor is a Wilson-Hilferty
closed form with constants of Excel's own: c^(1/3) - 1 is 0.0000886 to 13 digits, and from shape 2E10
Excel's value is a·(1 + 0.0000886 - 1/(9a) + z·s/(3·sqrt a))^3, z the normal quantile of p and s =
1 - 2.2106E-5 (fitted on the 1E13 sweep A25-A41, all within 7 ulps of it; at 1E11, A61-A77, a term
in z^2 of about 0.0002·z^2 absolute is left, 100 to 1,500 ulps). No search: the value has no
(z^2 - 1)/3 skew of its own beyond the cube's. CHISQ.INV and CHIINV differ at the
same df: `CHISQ.INV(0.5,2E13)` is `#NUM!` while `CHIINV(0.5,2E13)` = `CHISQ.INV.RT(0.5,2E13)` = 20005316471010.844 (A91-A93).
The cdf itself has no such factor (`GAMMA.DIST(1E12,1E12,1,TRUE)` is correctly rounded, probes/incomplete-gamma-cap E9),
so the factor is the inversion's.
Evidence: probes/t2982-gamma-inv-huge-shape A1-A95 (excel-graph, 2026-10-28); probes/incomplete-gamma-cap E5, E14, E15,
E19; gen-m9 0a614cfe974b59c3 A327, 4e55500bb8ad3cd8 A341, 7bd219ec0c0c5e22 A239, a013a45a8fbcab4f A338, d8266bc15533702a
A296 (each within about 1E-11 of c times the true quantile). excel-graph refused (FileCorruptTryRepair) workbooks holding
`GAMMA.DIST(2,a,1E10,TRUE)` for a from 1E40 to 1E100 and `GAMMA.DIST(x,1E10..1E11,1,FALSE)` at a tiny x, so those cells
were dropped from the probe (T-2982).
Ours: the closed form from shape 2E10 for GAMMA.INV and GAMMAINV (`incomplete_gamma` `wilson_hilferty_quantile`);
below it statrs' search, `#NUM!` from a shape of about 1E9, where Excel has values to 5E9. CHISQ.INV.RT and CHIINV
stay `#NUM!` above 1E10 degrees of freedom (`chisq.rs` `MAX_DEGREES_OF_FREEDOM`), where Excel takes 2E13 (A92, A93):
both are T-3070's.
Status: VERIFIED (excel-graph) for the factor at p = 0.5 and the #NUM! band; the closed form is a fit (7 ulps at 1E13)
Tasks: T-2982

## GAMMA.DIST at a huge shape: #NUM! for a density at a tiny x, for the cdf above about 1E+76 (T-2435)
Rule: GAMMA.DIST's density is `#NUM!`, not 0, where the shape is large and x/beta tiny:
`GAMMA.DIST(1E-300,1E+9,3,FALSE)` and `GAMMA.DIST(1E-300,1E+10,3,FALSE)` are `#NUM!`, while a shape
of 1E+8 at the same x, and a shape of 1E+9 at x = 1E-10 or 1, are 0; the cut sits between
(alpha - 1)|LN(x/beta)| of 6.9E+10 (0) and 6.9E+11 (`#NUM!`). The cdf is 0 for shapes up to 1E+40 at
x/beta = 2E-10 but `#NUM!` at 1.85E+76 (`GAMMA.DIST(2,1.85E+76,1E+10,TRUE)`,
`GAMMADIST(2,1.85E+76,1E+10,TRUE)`).
Evidence: probes/t2435-dist-extremes A25:A34, A92:A102 (excel-graph, 2026-10-26); gen-m9
2801c1c11c21e134 A241 (`GAMMADIST(1E-300,10000000001,3,FALSE)`) and 40425341c5034eed A265
(`GAMMADIST(2,1.85E+76,1E+10,"TRUE")`).
Ours: 0 for each `#NUM!` above (T-2982); `GAMMA.DIST(1E+10,1E+10,1,TRUE)` is Excel's 0.5000013298076014 since
T-1891 (Temme's expansion).
Status: VERIFIED (excel-graph), one cell per shape at the cuts
Tasks: T-2435, T-1891, T-2982

## LINEST and LOGEST: shapes, one point, collinear columns, exact fits and check order (T-1741)
Rule: LINEST and LOGEST line known_x up with known_y as TREND does ("TREND: defaults, shapes, single points and error
inputs"): the same shape is one variable read element by element (row arrays and `TRANSPOSE` results included:
`LINEST({2,4},{1,2})` is `{2,0}`, `LINEST(TRANSPOSE(B2:B9),TRANSPOSE(A2:A9),TRUE,TRUE)` the column fit), a column y takes
one variable per column of x, a row y one per row, anything else is `#REF!` (`LINEST({1,2,3,5},{1;2;3;4})`); an empty
known_x is the default 1, 2, 3, ... (`LINEST(B2:B9,,TRUE,TRUE)`). One point is accepted (K141, K204). A column that is,
to rounding, a combination of the intercept and the later columns is dropped: coefficient 0, standard error 0, and the
degrees of freedom and F count only the kept columns. The last column is kept first: with E = 2A,
`LINEST(B2:B9,A2:E9,TRUE,TRUE)` drops A (its se is exactly 0, the others are noise), df 3, F 3.1e34 = (ss_reg/4)/(ss_resid/3).
One point drops its x (`LINEST(B15,A15,TRUE,TRUE)` is `{0,9; 0,0; 1,0; #NUM!,0; 0,0}`), and one point of 8 variables
drops them all (`LINEST(5,A2:A9)` is `{0,..,0,5}`). With no degree of freedom left, se_y and the standard errors are 0 and
F is `#NUM!` (`LINEST(A12:A13,B12:B13,TRUE,TRUE)`, two points: `{0.5,-1.5; 0,0; 1,0; #NUM!,0; 0.5,0}`); an exact fit with
degrees of freedom left has F `#NUM!` too (`LINEST({2,4,6},{1,2,3},TRUE,TRUE)`, K249).
Order of the checks (gen-m9 cells): (1) an argument that is an error value, in argument order (section "LINEST and
LOGEST report error values first"), and a typed or computed const or stats text that is no boolean, `#VALUE!`
(`LINEST(UNIQUE(Data!D9:E9),Data!B7,TRUE,"1")`, `LOGEST(Data!C6:E9,Data!D$6,1,"")` against unequal shapes,
a013a45a8fbcab4f A276, fd9fa22d212080d6 A201), while a text cell waits for the shapes (`LINEST(Data!D10:E10,Data!B1:B7,
<"x~*" cell>,<#N/A>)` is `#N/A`, bcb24385baafb2f0 A40); (2) the shapes, `#REF!`, as TREND (`LINEST(Data!B11:E12,
(BYROW(..)+HOUR(0.5)),2,-1)` is `#REF!` though its x holds errors, 05d44f6dd6cfc567 A317, and six more cells); (3) an
error element of known_y or known_x, `#VALUE!` (`LINEST(Data!A1:C6,ROUND(GROUPBY(..)),..)`, 589f42e67f4b4b8d A100;
`LOGEST(<number>,IF(SORT(IF({1,0,1},1,NA())),..),..)`, a scalar y against a 1x3 x holding `#N/A`, d8266bc15533702a A117);
(4) a const or stats cell, or a LAMBDA result that is a reference to one, holding text that is no boolean, `#VALUE!`
(`LOGEST(Data!A10:B10,Data!B8:B8,-1,<"a" cell>)` and `LOGEST(Data!A10:A11,-0,LAMBDA(acc,Data!B3)(..))` with B3 = " x "
are `#REF!`, f34f66fb7f81e673 A179, bbdfc06cae0e0d71 A324); (5) a blank, text or boolean element, `#VALUE!`
(`LINEST(B2:B9,A2:A9&"")`, K113), LOGEST's non-positive y `#NUM!` as GROWTH's. An array const or stats lifts:
`LOGEST(Data!C$10,0.1,EXPAND(Data!A10:C11,2,4))` (165db58d1540f014 A327) and `LINEST(..,..,..,HSTACK(..))` as stats
(165db58d1540f014 A366, e6cf3b20a014e6a3 A89) are `#SPILL!`. Excel lifts before the data arguments' error values
(`LINEST(<#DIV/0!>,..,..,MAKEARRAY(2,3,..))` is `#SPILL!`, c2974c061384fdd9 A89), while
`LOGEST(Data!C6:E12,<#NUM!>,TRIMRANGE(..),TRUE)` is `#NUM!` (0a614cfe974b59c3 A342), presumably because TRIMRANGE is a
reference there; ours reports the data error first (T-2574).
The collinear drop rests on one probe, K106 (A's se exactly 0 there): which column of another collinear pair is kept,
with y not in x, is unprobed (T-2569).
Evidence: probes/linest-modes (excel-graph, 2026-10-16, T-1621); gen-m9 (excel-graph, 2026-10-22/23).
Ours: since T-1741 (`linest_or_logest` in functions/statistical/linest.rs, shared readers in regression.rs). The
noise digits of exact and collinear fits (K106, K190, K197: Excel 1.4e-16 where ours is 0, F 1.4e32 where ours is
`#NUM!`) and the two-point K134 (Excel exactly 0.5 and -1.5) come from Excel's solve and stay with T-2006; the other
leftover gen-m9 cells are T-2574's.
Status: VERIFIED (excel-graph)
Tasks: T-1741, T-2006, T-2556, T-2574

## LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)
Rule: confirmations and additions to "LINEST, LOGEST, GROWTH: probe findings": a scalar known_y with no known_x is one
point, so LINEST's anchor is 0 and LOGEST's 1 (`LINEST(2)`, 8d3588d8df0812df A186; `LOGEST(0.3)`, 8dd2312c96effb1b A329;
42 LINEST and 33 LOGEST cells); LOGEST of a non-positive known_y is `#NUM!`, also when the range holds text and booleans
too (`LOGEST(-SUBTOTAL(5,..))` = LOGEST(-0.1), c468f5046aadfbbc A282; `LOGEST(Data!C6:E11)` with E6 = -0.0001 and C7 = "1.5",
58e3253bbd3a7a82 A250), so the sign check comes before the text check; a scalar known_y with a multi-cell known_x is
`#REF!` (`LINEST(CHISQ.DIST(..),Data!B5:C10)`, bcb24385baafb2f0 A284), and so is a range known_y with a one-cell text
known_x (`LINEST(Data!B1:D11,Data!E10)` with E10 = "ü", c18dbf21f11093f3 A182), the size check before the text check;
a range known_y with text is `#VALUE!` (`LINEST(Data!D4:D5)`, 2a18b2f5a4dc43ac A273; whether it spills is not shown, B273 is unchecked).
Evidence: gen-m9 (excel-graph, 2026-10-22, T-2255); cells in T-1741, T-1742, T-1743.
Ours: matches since T-1741 (section "LINEST and LOGEST: shapes, one point, collinear columns, exact fits and check order").
Status: VERIFIED (excel-graph)


## LINEST, LOGEST and GROWTH: cells left after T-1741 (T-1742)
Rule: of the 189 gen-m9 LOGEST cells of T-1742, T-1741's shared shape and check-order code fixes 184; the other five are
decided elsewhere. `LOGEST(T.DIST.2T(Data!B3,6),Data!C9:C12,<FALSE>)` (the const is an IF that is FALSE; one point of four variables, x = 0, -1, -2.5, 0.2,
no constant) is 1 in Excel (285d87cea1aaa73c A293), so Excel drops the last column (0.2) where ours keeps it and fits
m = 2.7E-40. Together with linest-modes K106 (E = 2A keeps E) this fits a solve that keeps the column of largest norm
first (pivoted QR), which is UNVERIFIED (T-2569). `LOGEST(Data!A10:B11)` 2.220601291150046, ours ...0514, is the solve's
digits (589f42e67f4b4b8d A193, T-2006), as are logest-modes K85 and the exact-fit stats K169:L173 and K197:L201 (Excel's
residual is exactly 0, so se is 0 and F `#NUM!`). The remaining three are other functions: TRIMMEAN's order (a5342a7e24fd803e
A288, the next entry), and PRICEMAT with an error yld beside a LOGEST basis (bc04c1f478c9896c A75, Excel `#VALUE!`, ours
`#NUM!`) and LOGEST over an HLOOKUP whose range_lookup refers to a text "TRUE" cell (b8ee55934c5b824c A161, Excel `#VALUE!`,
ours the size `#REF!`), both UNVERIFIED (T-2615). A blank one-cell known_x reference beside a range known_y is the size
`#REF!` in Excel, as in ours (`LOGEST(Data!B9:D12,Data!A$9)`, 05d44f6dd6cfc567 A30; `LINEST(Data!C6:E12,Data!A11)`,
623663cacea53bbc A210; `GROWTH(Data!A11:D12,Data!C12)`, fec39efaba9284c3 A265).
GROWTH (T-1743): of its 75 gen-m9 cells, T-1726's and T-1741's shared code fixes 71. The other four are decided
elsewhere. `GROWTH(DISC(39060,Data!A$4,Data!C6,(Data!D$7/1E-15)),..)` is `#NUM!` (05d44f6dd6cfc567 A34): GROWTH reports
its known_y's error first, so Excel's DISC there, with maturity 1E+15, a text price and an error redemption, is
`#NUM!` (inferred: no other argument can give `#NUM!`; T-2484's probe confirms it), where ours is `#VALUE!` (T-2484). `WEIBULL(PROB(-42,GROWTH(Data!B10:C11,Data!B$3),..),..)` is `#REF!` (41248d1cb153c66e
A12): the GROWTH is the size `#REF!` in both, and Excel's PROB returns that prob_range error before the scalar x_range's
`#VALUE!` (T-2468). A 0 y against a 2x2 x is abc2939ebd1e2365 A187 (T-2569), and `GROWTH(Data!D10:E10)`
0.19999999999999996, ours 0.2000000000000001, is the solve (8fec7b207f4dd753 A75, T-2006), as are the growth-modes
numerics past the 15th digit (K145:K152, K157 and K169 with a multi-column x; K3, K15, K27, K123 and K219).
Evidence: gen-m9 (excel-graph, 2026-10-22); probes/logest-modes and probes/growth-modes (excel-graph).
Ours: as described; T-2006, T-2468, T-2484, T-2569, T-2614, T-2615.
Status: VERIFIED (excel-graph) for the values; the pivot and the four cross-function causes (PRICEMAT, HLOOKUP, DISC, PROB) UNVERIFIED
Tasks: T-2006, T-2468, T-2484, T-2569, T-2614, T-2615

## TRIMMEAN: an error percent before data with no number (T-1742)
Rule: TRIMMEAN returns an error-valued percent before it reports data with no number: `TRIMMEAN(Data!E11:E11,<#N/A>)`
with E11 = TRUE is `#N/A`, not `#NUM!` (a5342a7e24fd803e A288, the percent `COVARIANCE.P(8 cells,3 cells)-LOGEST(..)`).
Evidence: gen-m9 (excel-graph, 2026-10-22).
Ours: `#NUM!` (the data first); `TRIMMEAN(1,NA())` is already `#N/A`; T-2614.
Status: VERIFIED (excel-graph), one cell
Tasks: T-2614

## PROB: an error data argument first, one value on each side is #DIV/0!, unequal counts #N/A (T-2256)
Rule: PROB checks its data in this order, each step before the next: x_range given as an error value is that error, and a
blank one-cell x_range `#VALUE!`; then prob_range the same way (a blank x_range beats prob_range's error:
`PROB(<blank>,NA(),1)` and `PROB(<blank>,1/0,1)` are `#VALUE!`, `PROB(NA(),<blank>,1)` `#N/A`, probes/t2707-prob-orders
A9:A12; `PROB(2,SINH(1E+8),DAY(..)<#VALUE!>)` is `#NUM!`, d8f44d556674c2fd A40;
`PROB(<#CALC!>,Data!A4:A7,<#VALUE!>)` `#CALC!`, eccd03df42e40006 A16; the blank's `#VALUE!` as in the paired functions,
T-2426); then an error lower or upper limit (T-1813); then x_range and prob_range of
different **cell** counts are `#N/A`, blanks, text and booleans counted (`PROB(Data!C7:D7,1E-15,2)`, a number and a
blank against one number, is `#N/A`, 787ba8d801105a60 A108; `PROB(Data!C1:E6,"b",1.3)` 18 against a typed text,
329b177b5d5165ad A206; `PROB(Data!C2:D11,Data!C5:E8,..)`, 20 against 12 cells, 873f30d2bf76f8db A300;
`PROB((7/-(-2.5)),Data!A8:D8,1.005)`, 1 against 4, 5be29fa316931f05 A215), ahead of an error element of an array on either
side (`PROB(0,IF({1,1,1,1},1/0),0)` and `PROB(IF({1,1},1/0),{0.5,0.5,0},1)` are `#N/A`, t2707 A54, A56; for an error cell
of a range it is inferred from the paired functions' `COVARIANCE.S(H1:H3,I1:I4)`, UNVERIFIED for PROB); with equal counts an
error cell or element is the result, x_range's first (`PROB(<1;#DIV/0!;3>,<0.2;#N/A;0.5>,1)` is `#DIV/0!`, A19; A20, A21,
A55), and a direct error value on the other side still comes first (A16:A18). Equal counts pair cell by cell, row-major
whatever the shapes, and a pair counts only when both cells are numbers: `PROB(Data!D3:D8,Data!C5:C10,0.3,0.3)` is 0
because the probabilities -1 and 3.14159 sit beside a blank and a text and are dropped, leaving 0.3, 0.2 and 0.5
(e6cf3b20a014e6a3 A229, T-2436 probe). Text and booleans inside an array constant are dropped the same way, numeric
text too (`PROB({1;"a";3},{0.5;0;0.5},1)` is 0.5, `PROB({1;"a";3},{0.2;0.3;0.5},1)` `#NUM!` for the remaining sum 0.7,
`PROB({1;2;3},{0.5;"0.3";0.5},1)` 0.5, `PROB({1;2;3},{0.2;TRUE;0.8},1)` 0.2; A24:A34).
No pair of numbers is `#DIV/0!`: a single text or boolean against a number
(`PROB(Data!C$4,13,-2.5)` with C4 TRUE, 334f3dc9ca937f10 A82; `PROB(<"#N/A" text cell>,1.2,3.8,1)`, 583ca2baea0bb30a
A182; a typed boolean `PROB(-5.99,TRANSPOSE(" x "<>Data!A$11),0,3.14159)`, 5cebac1f02bf30f2 A372). A pair of numbers
then goes through the probability checks (`#NUM!` for a probability outside [0, 1] or a sum other than 1:
`PROB(COLUMNS(Data!C5:E10),0.1,-1,-1)` is `#NUM!`, 0bbab6206b5c9a07 A322; `PROB(Data!B3:E3,Data!E7:E10,1,Data!E7)`, 4
against 4, 873f30d2bf76f8db A72). The `#DIV/0!` of `PROB((ERF(1E-10)+(-0.5)),EFFECT(10,11),PV((-0.9999999999),161,..))`
(6e18550e9fc9c1b9 A394) and `PROB(Data!C6:C9,..,PV((-0.9999999999),78,..))` (92d4fa29e5510f1e A70) is then the lower
limit's: Excel's PV at a rate of -0.9999999999 over 78 or 161 periods would be `#DIV/0!` (ours `#NUM!`; inferred,
UNVERIFIED, the PV itself is not probed). These are direct error values; an error inside a range still loses to a
lower/upper error (T-1813, section "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments"). Whether a
lower_limit error beats unequal counts rests on a cell whose lower_limit is an inner YIELD (d37313406b9f6e55 A142,
`#NUM!`, which ours now matches); no probe cell separates them (t2707 A57 has the same error on both sides): UNVERIFIED.
A bare typed text, numeric text or boolean on either side is one cell with no number, so `#DIV/0!` against a single
number (`PROB(1,"a",1)`, `PROB("1",1,1)`, `PROB(-6,1=1,0,3)`, `PROB(1,TRUE,1)`; t2707 A1:A8).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256): the top-level PROB cells of T-2468 and T-2426; probes/t2436 (T-2436);
probes/t2707-prob-orders (excel-graph, 2026-10-29, T-2707).
An earlier reading of this entry ("a single value on both sides is `#DIV/0!`") was wrong: a pair of numbers is checked
as usual (T-2468).
Ours: since T-2468 (`statistical/prob.rs`) an error value or a blank one-cell reference as x_range, then as prob_range
(T-2707), comes before the limits; the data pairs by position (`paired_values`), unequal cell counts are `#N/A` before an
array's error element (T-2707), no pair of numbers `#DIV/0!`, each after the limits.
Status: VERIFIED (excel-graph) except the noted UNVERIFIED parts

## T.TEST: an error sample first, then tails and type errors, then their range check (T-2256)
Rule: T.TEST and TTEST return array1's error when array1 is an error value, then array2's (`T.TEST({1,2},-CEILING(..)
<#VALUE!>,Data!C10,1)` is `#VALUE!`, 73f053591a54bc43 A280; `T.TEST({1;2;"";4},STDEV(Data!C$10)<#DIV/0!>,2,0)` `#DIV/0!`
before the type 0 `#NUM!`, bbab6562e88b70e8 A338; `TTEST(MATCH(..)<#N/A>,..,7,3)` `#N/A` before tails 7), then a one-cell
reference to an empty cell `#VALUE!` (T-2426), then an error in tails or type (`TTEST(5,-0.005,Data!E$11,<#VALUE!>)` is
`#VALUE!` with tails FALSE, b9a32ce9a1fd0d71 A274; `T.TEST(Data!A6:D11,Data!A1:A12,-2.5,<#VALUE!>)` `#VALUE!`, 80586b859b73c87d
A32), and only then the tails/type range check (`#NUM!`). A typed empty text as a sample is no number (`TTEST(IFERROR(
#VALUE!,""),Data!B11:E11,1,3)` is `#DIV/0!`, 46ac3ef5e61583ca A344). The data-error-first order holds for direct error values;
an error inside a range still loses to a bad tails/type (T-1813). Unexplained: `TTEST(CELL("col",Data!D4),Data!A5:B10,2,2)`
is `#NUM!` (a5342a7e24fd803e A15), where one value against twelve was `#DIV/0!` in ours: UNVERIFIED (the cell now passes).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); cells in T-2469.
Ours: the order above (an error-value sample, then tails and type errors, then their range checks, then an error inside
a range; T-1813, T-2256, T-2469). It matches the computed empty text, a text sample being an array of one cell with no
number (T-2469). A typed text
literal (`T.TEST("",A1:A4,1,3)` `#DIV/0!`, `T.TEST("a",..)`) and a typed or computed boolean sample take the same path in
ours (as in COVARIANCE.P/.S, probes/t2436/computed-scalar A35:A37): UNVERIFIED for T.TEST (T-2853 probes them).
Status: VERIFIED (excel-graph) except the noted cell

## Z.TEST: an error array first; one data value is #DIV/0!; a lone non-number cell is #N/A (T-2256)
Rule: Z.TEST and ZTEST return the array's error when the array argument is an error value, before x's and sigma's
(`ZTEST(<#VALUE!>,STDEV.S((-0.0001))<#DIV/0!>)` is `#VALUE!`, 7fd0a98183c6babf A54; `ZTEST(BYCOL(..)<#CALC!>,2.5,<#VALUE!>)`
`#CALC!`, 607e85cfee83a689 A73; `ZTEST(COS(1E+16)<#NUM!>,Data!E2)` `#NUM!`, 334f3dc9ca937f10 A282). An array of one number
(a scalar, `{5}` or a one-cell reference holding a number) is `#DIV/0!`, before a sigma of 0 or below (`ZTEST({5},Data!$B6,
-123456789.123)`, 38e4fc309fe199ea A87; `ZTEST(0.5,-1,-2.5)`, 9735c5ab72fa5ecf A158; `ZTEST(Data!C3,Data!C2,-16586.85..)`,
0bbab6206b5c9a07 A231). An array argument with no number is `#N/A`, as "Test functions" says of an empty range, whether
it is a range (`ZTEST(Data!C7:C7,..)` with "abc", 5919660b7bb4d195 A364; `ZTEST(Data!C2:C2,..)` with FALSE, 9e2225ead8a6a07b
A75; `ZTEST(Data!Z1:Z3,TIMEVALUE("6:00"))`, 928d08e3e1eecb5f A159) or a single-cell reference without a colon holding text,
a boolean or nothing (`ZTEST(Data!B11,Data!C2)` with "x~*" and "1", c2684066d8df263e A190; `ZTEST(Data!A$4,-1,1E-10)` with
FALSE, 873f30d2bf76f8db A333; `ZTEST(Data!$A4,..)` with A4 empty, 201d67f1d59436ed A99). Two `#DIV/0!` cells over ranges with
no number, `ZTEST(Data!B5:C5,0.04)` (B5 empty, C5 FALSE; its x is `SUMXMY2(UNIQUE(Data!D4),-0.2)`, 7980ffb6095a340d A294)
and `ZTEST(Data!B10:B10,..)` (its sigma is `PV(-0.9999999999,167,-43,1E+300)`, a63d1c7d6f96c56d A324), are explained by
their x or sigma argument, not by the range: UNVERIFIED, T-2470 (which owns them) traces them.
The error array comes first in every element of a lifted call (`Z.TEST(NA(),{1,2})` spills two `#N/A`,
`Z.TEST(NA(),Data!A1:A2)` too), and it is only an error *value*: an error inside a range or array, or in a one-cell
reference, still comes after x (`Z.TEST(Data!B2,1/0)` and `Z.TEST(Data!B2:B3,1/0)` with B2 `#N/A`, and
`Z.TEST({1,2,#N/A},1/0)`, are `#DIV/0!`; "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments").
`ZTEST(PERCENTRANK.EXC(..,LCM(<text>))<#VALUE!>,XIRR(..)<#NUM!>)` is `#VALUE!` (gen-m9/c7664e494e418395 A204).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); cells in T-2470; probes/t2428-ztest-error-data A1-A14 (excel-graph,
2026-10-24, direct).
Ours: the error array value first, per element, since T-2428 (`statistical/z_test.rs`); one value is `#DIV/0!` before
a sigma of 0 or below, and a plain single-cell reference holding text, a boolean or nothing is `#N/A`; T-2470. The
`#DIV/0!` cells 7980ffb6095a340d A294 and a63d1c7d6f96c56d A324 are still untraced.
Status: VERIFIED (excel-graph)

## MODE: a one-cell range reference to an empty cell is #VALUE! (T-2256)
Rule: MODE, MODE.SNGL and MODE.MULT are `#VALUE!` for a one-cell range reference to an empty cell (`MODE(Data!C11:C11)`,
775d053de498b8c7 A352; `MODE.SNGL(Data!B8:B8)`, a63d1c7d6f96c56d A381; `MODE.MULT(Data!A10:A10)`, 5cebac1f02bf30f2 A183), as
for a blank single-cell reference ("Statistical function argument modes", A23), also beside other arguments that hold a mode
(`MODE(Data!A10:A10,Data!A9:A12,Data!E10:E10)` with E10 empty is `#VALUE!`, 329b177b5d5165ad A172).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); 36 cells in T-2471, plus probes/t2436/ecc87f80077959e0 (`MODE.MULT(Data!D10:D10,Data!D1:D6)`)
and probes/t2475-stat-subexpressions P!A96. UNVERIFIED: a one-cell reference returned by INDEX, OFFSET or IF, or a defined name (ours: `#VALUE!` too).
Ours: a one-cell range reference to an empty cell is `#VALUE!` like the plain reference, since T-2471 (`mode_functions.rs`).
Status: VERIFIED (excel-graph)

## Order statistics: the scalar argument's error before the no-number result; PERCENTRANK's #N/A before significance (T-2256)
Rule: PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN and SMALL return an error k, quart, x or percent before the result of
data with no number (`QUARTILE(Data!C10,IFNA(1/0,..))` with C10 empty is `#DIV/0!`, 41248d1cb153c66e A112;
`PERCENTRANK.INC(Data!Z1:Z3,STDEVA(Data!A1))` `#DIV/0!`, c4635963fc926284 A103; `PERCENTILE.INC(Data!D10:D11,<#VALUE!>)` with
text data `#VALUE!`, 41248d1cb153c66e A153; `TRIMMEAN(Data!C7:C7,<#DIV/0!>)` `#DIV/0!`, a63d1c7d6f96c56d A72), as PERCENTILE.EXC
already did (T-1403). PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC check x against the data (`#N/A` outside it, or for no
number) before significance (`PERCENTRANK(Data!E1:E5,1963,-2.5)` is `#N/A`, 139684e68fb2fc3b A324;
`PERCENTRANK.EXC({1,..,10},-0.0001,0)` `#N/A`, 09b47629f54b9817 A58).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); 62 cells in T-2472.
Ours: matches (T-2472).
UNVERIFIED: whether a significance that is itself an error or text (`PERCENTRANK(C1:C5,1963,"a")`) beats the out-of-range x `#N/A`; ours reports the significance's error first.
Status: VERIFIED (excel-graph)

## PERCENTRANK: a huge significance is #DIV/0!; truncation does not lose a digit at a cut (T-2256)
Rule: a significance of 309 or more (10^significance overflows) makes PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC `#DIV/0!` (2147483648,
7fd0a98183c6babf A202; 5616037595088.403, 941c409532603d32 A168; 1E+15, 91e167cd8d7eb652 A9; 9.18E+36, 2138b909d8f810ea A333),
where a significance below 1 is `#NUM!`. The cut is where 10^significance overflows: 308 digits computes and 309 or more
is `#DIV/0!` (probes/t2473-percentrank-significance A1-A15, B1-B15, C1-C15). The truncation to significance
digits does not drop a digit when the rank lies within rounding of a cut: `PERCENTRANK.EXC({1E+15,1,-1E+15},0.3)` is 0.5
(c7664e494e418395 A252), `PERCENTRANK.INC(Data!D4:E12,0)` 0.5 (d37313406b9f6e55 A303), `PERCENTRANK.INC(Data!A1:C3,0.2,2)` 0.36
(ede0323c389fc703 A43) and 0.603 at 3 digits (A157), where a plain floor of the double rank gives 0.499, 0.35 and 0.602.
The fit (t2473 D1-D24): the rank times 10^significance is cut to 15 significant digits, then floored and divided back
in doubles, so 2/3 at 15 digits is 0.666666666666667 (rounded up by the cut), at 17 digits 0.6666666666666671, and 1/7 at
20 digits 0.14285714285714302, while a rank of 0.499999999999999 (15 digits) still floors to 0.499. A typed literal of 16
nines is cut to 15 digits on entry, so `PERCENTRANK({0,1},0.9999999999999999,3)` is 0.999.
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); cells in T-2473; probes/t2473-percentrank-significance (excel-graph,
2026-10-25).
Ours: since T-2473 (`percentrank_of`, `decimal_round::cut_to_15_digits`, 10^significance correctly rounded).
UNVERIFIED: how the `#DIV/0!` orders against an x outside the data (`#N/A`) and a single value (1); ours checks it after
both. The direction of an exact 16th-digit tie in the cut (ours away from zero, as FLOOR's) is unobserved, as no probed
rank reaches one.
Status: VERIFIED (excel-graph) for the cited cells

## Scalar distributions at extreme arguments, part 2 (T-2256)
Rule (gen-m9, excel-graph, 2026-10-23):
- `TDIST` and `T.DIST.2T` of x = 1E+300 or 1E+307 are `#NUM!` (`TDIST(1E+300,8,1)`, 9735c5ab72fa5ecf A33; `T.DIST.2T(1E+307,3)`,
  775d053de498b8c7 A26; 12 cells), not 0; `T.DIST.2T(1E+15,1)` is still a number (46ac3ef5e61583ca A236). The cut is UNVERIFIED.
- POISSON and POISSON.DIST do not reject a negative mean when x truncates to 0: the result is EXP(-mean)
  (`POISSON(0,-1,0)` = 2.718281828459045, 607e85cfee83a689 A189; `POISSON.DIST(0,-42,..)` = 1.739274941520501E+18,
  c7664e494e418395 A189; `POISSON(0.3,-0.5,TRUE)` = 1.6487212707001282, b9a32ce9a1fd0d71 A221; 9 cells), and
  `POISSON.DIST(2E+29,7,-1)` is 1 (5919660b7bb4d195 A88; owned by the residual probe T-2475).
- NEGBINOMDIST and NEGBINOM.DIST: a number_f in (-1, 0) is `#NUM!` before truncation (`NEGBINOMDIST(-0.5,8,0.5)`,
  d208bbafc3bc1997 A231; -1E-06, 3b7990bd4e3307ba A312), and a probability that underflows the result is 0, not `#NUM!`
  (`NEGBINOMDIST(7,123456789.123,1E-200)`, 5919660b7bb4d195 A124; `NEGBINOM.DIST(9,4,1E-300,TRUE)`, bcb1105c61a2649d A24).
- PERMUTATIONA truncates number_chosen toward 0 before its sign check (`PERMUTATIONA(10,-0.75)` and `(7,-0.1)` are 1,
  0620505280d2ea6e A280, 5395b8a427764a3f A385), and a number of about 1E+15 or more is `#NUM!` even with number_chosen 0
  (`PERMUTATIONA(1000000000000000,0)`, 775d053de498b8c7 A152; `PERMUTATIONA(FACT(18),2)`, b9a32ce9a1fd0d71 A378).
Ours (T-2474): as Excel for every T-2474 cell. The cumulative t functions (T.DIST with cumulative TRUE, T.DIST.RT,
T.DIST.2T, TDIST) are `#NUM!` where `x²` overflows (|x| past about 1.34E+154; the cut, and that a negative x of that size is `#NUM!` too, are UNVERIFIED), the negative mean
follows "POISSON at x 0 with a negative mean", the tiny-p NEGBINOM is 0 (the binomial walk renormalizes before a ratio
p/q past 2^53 overflows it), and PERMUTATIONA takes PERMUT's limit, a number of 2^31 - 1 or more is `#NUM!` (UNVERIFIED cut).
Since T-2700 a NEGBINOM.DIST/NEGBINOMDIST number_f in (-1, 0) is `#NUM!` as in Excel (the sign is checked before truncation; bonus cell
b9a32ce9a1fd0d71 A195, `NEGBINOMDIST(Data!$D$6,7,..)` with D6 = -0.5). The PERMUTATIONA truncation cells are T-2701's. Probe of the cuts: T-2752.
Status: VERIFIED (excel-graph), the cuts UNVERIFIED

## Student t quantile at p = 0 with df below 2 is #DIV/0! (T-2256)
Rule: TINV, T.INV.2T and T.INV with p = 0 and a df that truncates to 1 are `#DIV/0!`, not `#NUM!`: `TINV(0,1)`
(ede0323c389fc703 A338, 139684e68fb2fc3b A128), `T.INV.2T(Data!C1,1)` with C1 = 0 (607e85cfee83a689 A160), `T.INV(0,1)`
(928d08e3e1eecb5f A271), `T.INV(0,1.999)` (0072265d385d8ab2 A341), `T.INV.2T(FALSE,1.005)` (9bc247733a1ca9fa A36),
`TINV(0,COUNT(..))` (7f7ca726c3f68554 A280). From df 2 up, p = 0 is `#NUM!`: `T.INV(0,2)` (3fb9cecf2c23169f A120), `TINV(0,3)`
(31f09e6c48ca245c A316), `T.INV.2T(0,3)` (9e2225ead8a6a07b A102), `T.INV(0,4)` (91e167cd8d7eb652 A324); the cut is between
1.999 and 2.
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); cells in T-2096.
Ours: as Excel since T-2621 (`t_inv_of`).
Status: VERIFIED (excel-graph)

## A lone text, boolean or empty cell as the data of QUARTILE, PERCENTILE, PERCENTRANK and TRIMMEAN (T-2256)
Rule: a single-cell reference (no colon) holding text, a boolean or (for TRIMMEAN) nothing is data with no number:
QUARTILE*, PERCENTILE* and TRIMMEAN are `#NUM!` (`QUARTILE(Data!E2,3)` with E2 = "a", 7980ffb6095a340d A132;
`QUARTILE(Data!C11,0)` with TRUE, 17c110a53924beb8 A234; `PERCENTILE(Data!D5,0)` with "1", 5cebac1f02bf30f2 A247;
`TRIMMEAN(Data!B7,1.005)` with TRUE, 9735c5ab72fa5ecf A264) and PERCENTRANK* `#N/A` (`PERCENTRANK(Data!C11,1.005)` with "a",
44a8841352dc48e9 A52). This answers T-2344's single-cell question and T-2066's (`QUARTILE` of one text cell).
Evidence: gen-m9 (excel-graph, 2026-10-23, T-2256); 72 cells in T-2344.
Ours: since T-2429 a cell holding text or a boolean is no number in QUARTILE*, PERCENTILE* and PERCENTRANK* ("A single cell holding a boolean is no number in the PERCENTILE and QUARTILE data"), and since T-2621 a cell holding nothing, text or a boolean in TRIMMEAN.
T-2344 probed the same data given as a defined name, INDEX, OFFSET, IF or CHOOSE result (probes/t2344-percentile-name-index-data A1-A30, excel-graph, 2026-10-27): a reference of any of these forms holding one text (also numeric text "1") or boolean cell is no number, `#NUM!` for QUARTILE*/PERCENTILE* and `#N/A` for PERCENTRANK*, after quart's own error (`QUARTILE(INDEX(..),NA())` `#N/A`); ours since T-2344 (`percentile_numbers` reads the argument once in reference context). TRIMMEAN's name/INDEX forms and LET, IFS, SWITCH and LAMBDA picks are unprobed.
Status: VERIFIED (excel-graph)

## COVARIANCE: a boolean scalar is one value (T-2436)
Rule: COVARIANCE.P and COVARIANCE.S take a TRUE or FALSE argument, typed or computed, as an array of one value like a number,
so the size check and the one-point rule apply: `COVARIANCE.P({1,2,3},TRUE)` is `#N/A`, `COVARIANCE.S(-2.5,FALSE)` and
`COVARIANCE.S(B1,FALSE)` with B1 = 5 are `#DIV/0!` (probes/t2436/computed-scalar A35:A37), as are the sweep's
`COVARIANCE.P(Data!B3:E7,CHOOSE(1,("abc">Data!A4)))` `#N/A` (899f9555e67da2d6 A369) and
`COVARIANCE.S(YEAR(20863),HYPERLINK(..,("x~*"=Data!C9)))` `#DIV/0!` (f34f66fb7f81e673 A127). With a number in place of the
boolean ours already matches (A29:A34).
Evidence: probes/t2436/computed-scalar (excel-graph, 2026-10-24); 4 gen-m9 cells listed in T-2575.
Ours: matches (T-2575).
UNVERIFIED: COVAR (the compatibility name) shares the code path, so a boolean scalar is one value there too; no probe of it yet.
Status: VERIFIED (excel-graph); COVAR with a boolean unprobed
Tasks: T-2575

## ERROR.TYPE and CELL results are #NUM! in the paired functions (T-2436)
Rule: a number that ERROR.TYPE or CELL returns, passed straight on as a one-value argument, makes F.TEST, FTEST, FORECAST,
INTERCEPT and COVARIANCE.P `#NUM!` where the same number from anywhere else gives the one-point `#DIV/0!` (or COVARIANCE.P's
0): `FTEST({1;2;3},ERROR.TYPE(#REF!))`, `FTEST({1;2;3},ERROR.TYPE(NA()))`, `FTEST({1;2;3},CELL("col",B12))`,
`FTEST(ERROR.TYPE(#REF!),{1;2;3})`, `FORECAST(1,ERROR.TYPE(#REF!),5)`, `FORECAST(1,CELL("col",B12),5)`,
`INTERCEPT(ERROR.TYPE(#REF!),5)` and `COVARIANCE.P(ERROR.TYPE(#REF!),5)` are `#NUM!`, while `ERROR.TYPE(#REF!)+0`,
`N(ERROR.TYPE(#REF!))`, `CELL("col",B12)+0`, `SUM(4)`, `VALUE("4")`, `ROW(B4)`, `COLUMN(D1)`, `TYPE(4)`, `INDEX({4,5},1)`,
`IF(TRUE,4)` and `INDIRECT("B1")` give `#DIV/0!` (probes/t2436/computed-scalar A1:A29). This explains the three F.TEST cells
the T-2255 entry above left UNVERIFIED (`FTEST(Data!C3:C9,ERROR.TYPE(#REF!))`, 4250f880ab640c49 A325; `F.TEST(CELL("col",..),..)`,
623663cacea53bbc A400) and `FORECAST(Data!A10,ERROR.TYPE(..),STDEV.P(..))` (bcb24385baafb2f0 A252). Why Excel does this is
unknown (perhaps these two functions return an internal integer type the array readers reject).
T-2576 probe (probes/t2576-error-type-cell-paired, excel-graph, 2026-10-26; FTEST({1;2;3},x) in D, other calls in E):
- Which values carry the mark: ERROR.TYPE (also of an array, `ERROR.TYPE({#REF!})`), `CELL("col",..)` and `CELL("row",..)`;
  not `CELL("contents",..)` of a number, N, SHEET, SHEETS, COLUMNS, LEN, MATCH, AREAS, WEEKDAY, GESTEP or ISREF (D6:D26).
- What keeps it: parentheses, `LET(x,ERROR.TYPE(#REF!),x)`, `IF(TRUE,..)`, `CHOOSE(1,..)`, `IFERROR(..,0)`, a LAMBDA call
  `LAMBDA(x,x)(..)` and `INDEX(..,1)` (D1:D5, D19, D20). What drops it: `*1`, unary minus, MAX, SINGLE, `@`, and a cell
  holding the value (`FTEST({1;2;3},Z1)` with Z1 `=ERROR.TYPE(#REF!)` is `#DIV/0!`; D9, D17, D18, D21, D25, D27, D28).
- Which functions react, first array argument: COVARIANCE.S, COVAR, CORREL, SLOPE, RSQ, PEARSON, STEYX, SUMXMY2 (1 with 4)
  and FORECAST.LINEAR are `#NUM!` (E1:E16, E23); so is T.TEST's second array (`T.TEST({1,2,3},ERROR.TYPE(#REF!),2,2)`,
  E14) and both of FTEST's (E35). A second argument of COVARIANCE.P is `#DIV/0!` (`COVARIANCE.P(4,ERROR.TYPE(#REF!))`,
  while `COVARIANCE.P(4,5)` is 0; E26), INTERCEPT's known_x and FORECAST's x read it as the number (E27, E29). T.TEST's
  first sample is `#NUM!` too (`TTEST(CELL("col",Data!D4),Data!A5:B10,2,2)`, gen-m9 a5342a7e24fd803e A15), after its
  tails/type checks (gen-m9 d5fd9e05e9e6f466 A184 is the tails argument's `#CALC!`). A size mismatch comes first: `#N/A`
  (`COVARIANCE.S(ERROR.TYPE(#REF!),Data!C10:D10)`, gen-m9 abc2939ebd1e2365 A335; `RSQ(ERROR.TYPE(#NAME?),Data!C8:C11)`,
  b9a32ce9a1fd0d71 A267). UNVERIFIED: the second argument of COVAR, COVARIANCE.S, CORREL, PEARSON, SLOPE, RSQ and STEYX
  (the engine treats it as COVARIANCE.P's and INTERCEPT's), and a marked first argument against an error element of the
  second (the engine gives `#NUM!`).
- LINEST reacts: `LINEST(ERROR.TYPE(#REF!))` is `#VALUE!` (E34) where `LINEST(2)` is 0 (gen-m9 8d3588d8df0812df A186).
  CHITEST and the one-list functions are unaffected: SUM, STDEV, VAR.P, KURT, PERCENTILE, LARGE, MEDIAN, DEVSQ, AVEDEV and
  ZTEST give what 4 gives (E18:E22, E30:E33, E36:E38).
The third T-2255 cell, `F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})`
(06b4e8f5de409d5e A261), is `#NUM!` over an INDIRECT that is `#VALUE!` (references.md "INDIRECT: a 1x1 array a1 argument is
#VALUE!"), as is `COVARIANCE.P(INDIRECT("Data!A1",FILTER(..)),Data!B11:D12)` (f2bd5f380ec002bf A383); T-2577 explained
that `#NUM!`: the INDIRECT is an array of references ("An array of references is #NUM! in the paired functions" below).
Evidence: probes/t2436/computed-scalar (excel-graph, 2026-10-24), probes/t2576-error-type-cell-paired (excel-graph,
2026-10-26); gen-m9 cells in T-2576 and T-2577.
Ours: since T-2576 the same for the call itself (parentheses aside), in F.TEST/FTEST, the regression family, COVARIANCE
and T.TEST (`is_marked_number`, functions/util.rs; `evaluate_sample_arg`, functions/statistical/paired.rs); the LET/IF/CHOOSE/IFERROR/LAMBDA/INDEX forms, an array of
marked numbers, SUMXMY2 and LINEST are T-2766's (still `#DIV/0!` or a value); T-2577.
Status: VERIFIED (excel-graph) for the listed calls; mechanism UNVERIFIED
Tasks: T-2576, T-2577, T-2766

## An array of references is #NUM! in the paired functions (T-2577)
Rule: an OFFSET or INDIRECT lifted into an array of references (lookup.md "OFFSET, INDIRECT and TRIMRANGE lift
into arrays of references"), passed as a sample, is treated like the ERROR.TYPE/CELL mark above, not as its
`#VALUE!` elements: `#NUM!` as either F.TEST sample (`F.TEST(INDIRECT("B1",{1}),{-1,0,1})`,
`F.TEST({-1,0,1},INDIRECT("B1",{1}))`, `F.TEST(OFFSET(B1,{0;1},0),{-1,0,1})`, `F.TEST(INDIRECT(C1:C2),{-1,0,1})`,
`F.TEST(INDIRECT({"B1","B2"}),{-1,0,1})`) and as the first argument of COVARIANCE.P, COVARIANCE.S, CORREL, PEARSON,
SLOPE, INTERCEPT, FORECAST's known_y, T.TEST, SUMXMY2 and CHITEST (`COVARIANCE.P(OFFSET(B1,{0;1},0),{1;2})`,
`CHITEST(INDIRECT("B1",{1;1}),{1;2})`), while the literal `{#VALUE!}` is `#VALUE!` (`F.TEST({#VALUE!},{-1,0,1})`,
`COVARIANCE.P({#VALUE!;#VALUE!},{1;2})`). As COVARIANCE.P's second argument it is cells with no number:
`COVARIANCE.P(5,INDIRECT("B1",{1}))` and `COVARIANCE.P({1;2},INDIRECT("B1",{1;1}))` are `#DIV/0!`. LINEST of one
is `#VALUE!`. An array of references holding only error elements is those errors (`F.TEST(INDIRECT("B1",{0}),..)`
is `#REF!`, `F.TEST(INDIRECT("B1",{"a"}),..)` `#VALUE!`), one with a reference among errors is still `#NUM!`
(`COVARIANCE.P(INDIRECT("B1",{1;0}),{1;2})`). What drops it: `+0` (`#VALUE!`), IFERROR (its 7, `#DIV/0!`) and INDEX
of an element (the cell, `#DIV/0!`); what keeps it: LET (`F.TEST(LET(x,INDIRECT("B1",{1}),x),{-1,0,1})` is
`#NUM!`). The one-list functions read through the references: `STDEV(INDIRECT("B1",{1;1}))` is 0 and
`MAX(INDIRECT("B1",{1}))` 5. This explains both gen-m9 cells: `F.TEST(INDIRECT("Data!B3",TOCOL(2,0,Data!A5)),{-1,0,1})`
(06b4e8f5de409d5e A261) and `COVARIANCE.P(INDIRECT("Data!A1",FILTER(..)),Data!B11:D12)` (f2bd5f380ec002bf A383).
Evidence: probes/t2577-indirect-ref-array (excel-graph, 2026-10-26).
Ours: since T-2577 for a direct OFFSET or INDIRECT argument in F.TEST, the `fn_get_two_matrices` family (CORREL,
PEARSON, SLOPE, INTERCEPT, RSQ, STEYX, FORECAST, CHITEST), COVARIANCE and T.TEST (`Model::evaluate_sample_arg`,
functions/statistical/paired.rs); the LET form, SUMXMY2 and LINEST are T-2766's, STDEV T-2065's. UNVERIFIED, marked by
analogy with the ERROR.TYPE mark: the first argument of RSQ and STEYX, T.TEST's second sample, and COVARIANCE.S's and
COVAR's second argument (read as cells with no number); probing them is T-2766's.
Status: VERIFIED (excel-graph) for the listed calls
Tasks: T-2577, T-2766, T-2065

## LINEST: a known_y range with empty cells is #VALUE! (T-2436)
Rule: LINEST of a known_y range that holds empty cells beside its numbers is `#VALUE!`, not a fit of the numbers:
`LINEST(Data!E8:E12)` with E8:E10 = 1E-15, -2.5, -0.0001 and E11:E12 empty (gen-m9 b8ee55934c5b824c A394, probed alone in
probes/t2436/b8ee55934c5b824c). LOGEST, TREND and GROWTH with the same known_y are unprobed.
Evidence: probes/t2436/b8ee55934c5b824c (excel-graph, 2026-10-24).
Ours: `#VALUE!` (shared reader `Grid::numbers`, regression.rs; T-2584 found it already matched).
Status: VERIFIED (excel-graph), one cell
Tasks: T-2584

## Sub-expression probe of the untraced gen-m9 statistical cells (T-2436)
Rule: probes/t2436/<case> evaluates every sub-expression of the 122 gen-m9 cells the T-2255 and T-2258 triage could not trace,
on each case's own Data sheet (excel-graph, 2026-10-24); 28 of them already pass on main. For the other 94 the innermost
call Excel computes differently is in the owning task's Context. Besides the entries above, the probe confirms existing
entries on new forms: HYPGEOM.DIST's impossible sample is 0 for the cumulative form too (`HYPGEOM.DIST(1,3,6,7,TRUE)`, T-2429);
`PHI(1E+300)`, `NORM.S.DIST(1E+300,FALSE)`, `BETA.DIST(0,0.5,1E+5,FALSE)` and `T.DIST.RT(1E+300,7)` are `#NUM!`,
`WEIBULL(100,1E+8,99999,..)` is 0 and `WEIBULL(0.2,1E+15,0.2,"TRUE")` 0.6321205588285577, `F.INV.RT(1E-200,1,1.999)` is
3.696066238512854E+150 (T-2435, T-2474); `FISHER(1E-15/3.14159)` is 2.2204460492503126E-16 (T-2437); a lone text or boolean
cell as QUARTILE's data is `#NUM!` and as PERCENTRANK's `#N/A` (T-2344); PROB, Z.TEST, T.TEST, MODE and TRIMMEAN follow their
T-2256 order rules (T-2468, T-2470, T-2469, T-2471, T-2472); the `#N/A` pads of EXPAND, HSTACK and VSTACK and a lifted `{#N/A}`
are error elements (T-2430); FORECAST and CHITEST pair equal counts of different shape (T-2431). STANDARDIZE and HYPGEOM.DIST read a referenced FALSE as 0 (`STANDARDIZE(3,-1,<FALSE>)` and
`STANDARDIZE(<"1.5" text>,-2.5,<FALSE>)` are `#NUM!`, standard_dev 0 before x's text; matched since T-1849).
Evidence: probes/t2436 (57 cases plus computed-scalar).
Ours: see each owning task.
Status: VERIFIED (excel-graph)
Tasks: T-2436

## Inner sub-expressions of the T-2256 gen-m9 statistical cells (T-2475)
Rule: Excel gives these values for the innermost sub-expressions where the T-2475 probe found ours diverging, each read on a copy
of its gen-m9 case's Data sheet (the probe's D01..D48 sheets; errors are named, numbers are the full doubles). They are observed
values, one cell each, recorded so the fix tasks can state the rule behind each: PERMUTATIONA and COMBINA give 1 for the probed
negative argument; IRR is `#NUM!` on four flow sets where ours converges; PV, NPER, PRICEDISC, DB, ACCRINT, ACCRINTM, YIELDDISC,
YIELD, T.TEST, T.INV.2T, PERCENTRANK(.INC/.EXC), TRIMMEAN, XMATCH, STEYX and SLOPE give a different error from ours (the
argument-domain order); TAKE of a LET-bound scalar is that scalar, not a 1x1 array; CELL("contents") is read differently inside
COUNT and MODE.SNGL; and BINOM.DIST, NORMDIST, WEIBULL(.DIST), BETA.INV, POISSON.DIST, F.INV.RT, NEGBINOMDIST, STDEV.P, SKEW,
MODE.MULT, PMT and PPMT differ in value or error. NPV's rule is "NPV with a rate of -1 or 1E+300 is #DIV/0!" (financial.md).
- P!A2 `IRR(D01!C4:E7,-D01!C8)`: `#NUM!` (ours 31622775.851604085; 0620505280d2ea6e A77)
- P!A28 `PERMUTATIONA(D02!D$12,D02!A12)`: 1.0 (ours `#NUM!`; 097af1ca33638670 A358)
- P!A35 `COMBINA((-0.5),D03!$A3)`: 1.0 (ours `#NUM!`; 11eb3ab4f4f8104c A90)
- P!A39 `PV(-D03!B$1,0,-713)`: `#NUM!` (ours `#DIV/0!`; 11eb3ab4f4f8104c A109)
- P!A45 `PRICEDISC(D03!$A4,D03!D2,0.3,103,(-2.5))`: `#NUM!` (ours `#VALUE!`; 11eb3ab4f4f8104c A160)
- P!A58 `T.TEST((D03!A11+(-0.5)),(42/D03!$D6),1,(-1))`: `#VALUE!` (ours `#NUM!`; 11eb3ab4f4f8104c A169)
- P!A70 `TAKE(LET(v,D05!A$11,acc,v,y,"?b",acc),-1,-1)`: 0.0 (ours 64|1x1|0; 1b31092a0d79e5f1 A143)
- P!A75 `PERCENTRANK.INC(D05!A7:D8,2,-1e-15)`: `#N/A` (ours `#NUM!`; 1b31092a0d79e5f1 A171)
- P!A78 `WEIBULL.DIST(IRR({-100,100}),1E+8,NORMDIST(40,(D05!A11*1),MAX(D05!C1:C8,D05!C8:C10),(D05!$A$9<D05!B$`: 0.0 (ours `#NUM!`; 1b31092a0d79e5f1 A179)
- P!A92 `NPER(1E-300,-113,3037,368,1)`: `#DIV/0!` (ours `#NUM!`; 1b31092a0d79e5f1 A260)
- P!A96 `MODE.MULT(D05!A8:A8,D05!C7:C10,D05!D3:D11)`: `#VALUE!` (ours 64|2x1|2.5;1.005; 1b31092a0d79e5f1 A343)
- P!A124 `T.INV.2T(D07!$A4,1)`: `#DIV/0!` (ours `#NUM!`; 2138b909d8f810ea A344)
- P!A127 `BETA.INV(0.025,1e-15,2.5)`: `#NUM!` (ours 2.518020676354722e-31; 2138b909d8f810ea A358)
- P!A136 `PV((-1+2^-52),165,-978,224,0)`: `#DIV/0!` (ours `#NUM!`; 31f09e6c48ca245c A232)
- P!A158 `STDEV.P(D10!A9:A12,IFERROR(1/0,D10!$E6),{1,2,3,4,5,6,7,8,9,10})`: 25.559768683380703 (ours 26.354817185647274; 334f3dc9ca937f10 A226)
- P!A165 `DB(15779,385,37,D11!$B7,D11!C11)`: `#NUM!` (ours `#VALUE!`; 38e4fc309fe199ea A89)
- P!A198 `ACCRINT(34443,36178,38986,0.05/12,3587,1,5,"true")`: `#VALUE!` (ours `#NUM!`; 44a8841352dc48e9 A7)
- P!A207 `YIELDDISC(36531,45775,97,RANK.EQ(D14!A9,D14!B9:E11,0),(-1))`: `#NUM!` (ours `#N/A`; 44a8841352dc48e9 A291)
- P!A214 `YIELD(39681,45662,D14!B$8,3.14159,102,4)`: `#NUM!` (ours 19.040243161919385; 44a8841352dc48e9 A318)
- P!A216 `POISSON.DIST(((D15!E12+0.2)*(1000000000000000.0*1000000000000000.0)),((D15!D5-D15!D10)+HOUR(D15!A9))`: 1.0 (ours `#NUM!`; 5919660b7bb4d195 A88)
- P!A222 `TTEST(((D16!C6+D16!E$6)/FINV(1.5,3,0.3)),IRR(INDEX(D16!D3:E9,D16!$C7),ROUNDDOWN(1,42)),2,(INDEX(D16!`: `#VALUE!` (ours `#REF!`; 5cebac1f02bf30f2 A311)
- P!A245 `SKEW({1E+200,1E+200,-1E+200},2.5,D18!D5)`: `#DIV/0!` (ours 0.0; 6e18550e9fc9c1b9 A347)
- P!A266 `STEYX(ERROR.TYPE(SUMPRODUCT(D20!C1:E5,7)),MATCH((D20!$E$8>=D20!$E7),D20!A11:C11))`: `#NUM!` (ours `#DIV/0!`; 775d053de498b8c7 A98)
- P!A273 `SLOPE(CELL("row",D20!E3),(0.1-TIME(5,16,59)))`: `#NUM!` (ours `#DIV/0!`; 775d053de498b8c7 A112)
- P!A279 `(TAKE(D20!C8:E12,D20!B12,D20!$A7)-(D20!A$3/D20!A2))`: `#DIV/0!` (ours 64|1x1|#2; 775d053de498b8c7 A235)
- P!A317 `F.INV.RT(1-1E-10,8,7)`: 0.0012599666259138292 (ours 0.0012599666259138298; 873f30d2bf76f8db A308)
- P!A326 `MODE.SNGL(CELL("contents",D26!C$5))`: `#VALUE!` (ours `#N/A`; 92d4fa29e5510f1e A98)
- P!A330 `IRR(D26!C8:E12,1E-10)`: `#NUM!` (ours 399999999999998.7; 92d4fa29e5510f1e A236)
- P!A341 `BINOM.DIST(3,1E+15,1E-20,FALSE)`: `#NUM!` (ours 1.6666500000833278e-16; 9bc247733a1ca9fa A5)
- P!A353 `TRIMMEAN(D28!A6,D28!C2)`: `#NUM!` (ours `#VALUE!`; 9bc247733a1ca9fa A360)
- P!A396 `PERCENTRANK.EXC(D31!A9:C9,D31!B12,D31!B$3)`: `#N/A` (ours `#NUM!`; a4090567758520df A101)
- P!A407 `WEIBULL(D32!E2,D32!E2,D32!$E11,D32!$A$8)`: `#NUM!` (ours 1.0; a5342a7e24fd803e A166)
- P!A417 `NEGBINOMDIST(7,9,1E-10)`: 6.434999995495541e-87 (ours 6.434999995495502e-87; a63d1c7d6f96c56d A92)
- P!A424 `IRR(D33!C4:E8,((-0.5)*D33!D4))`: `#NUM!` (ours -0.9710878190154174; a63d1c7d6f96c56d A111)
- P!A455 `PERCENTRANK(D36!C11:E12,(-0.0001),1000000000000000.0)`: `#DIV/0!` (ours `#NUM!` for the whole TYPE/ERROR.TYPE summary, the error escaping it; bbab6562e88b70e8 A290)
- P!A462 `PMT(D37!C9,346,96301,42,1)`: -278.44797687866065 (ours -278.4479768786607; bcb1105c61a2649d A97)
- P!A491 `COUNT(D38!B2:B10,CELL("contents",D38!C8))`: 3.0 (ours 2.0; c2684066d8df263e A201)
- P!A517 `DB(79665,(-100),17,9)`: `#NUM!` (ours `#NUM!` for the whole TYPE/ERROR.TYPE summary, the error escaping it; cb61169f4c033d4b A272)
- P!A520 `SKEW(D41!D11:D12,D41!D10)`: 2.4980018054066022e-15 (ours 2.9976021664879227e-15; d208bbafc3bc1997 A262)
- P!A543 `PPMT(100,5,217,4143,669,1)`: 0.0 (ours 4.768374219565885e-5; d910f000b60e2eac A20)
- P!A549 `XMATCH(COUNTA(D45!B8),{3;1;4;1;5},(D45!$C1+D45!E8))`: 2.0 (ours `#N/A`; eccd03df42e40006 A21)
- P!A563 `IRR(D47!A6:C8)`: `#NUM!` (ours -0.8583707295763425; eedd629f3d69fdda A297)
- P!A567 `ACCRINTM(32899,36547,D48!C9,2035,3)`: `#VALUE!` (ours `#NUM!`; f82473275380882e A131)
- P!A572 `NORMDIST(1E+300,MAX(D48!C11:C12),2.5,0)`: `#NUM!` (ours 0.0; f82473275380882e A143)
Evidence: probes/t2475-stat-subexpressions (excel-graph, 2026-10-24, T-2475); the owning gen-m9 cells are listed in T-2619 to T-2622.
Ours: as given per cell (T-2619 IRR, T-2620 financial, T-2621 statistical, T-2622 TAKE, CELL, XMATCH).
Status: VERIFIED (excel-graph; the per-cell values, not yet the rules behind them)
Tasks: T-2475, T-2619, T-2620, T-2621, T-2622

## POISSON at x 0 with a negative mean
Rule: POISSON and POISSON.DIST at x 0 with a negative mean are EXP(-mean), cumulative or not, not `#NUM!`:
`POISSON(0,-1,2)`, `POISSON(0,-1,TRUE)`, `POISSON(0,-1,FALSE)` and `POISSON.DIST(0,-1,TRUE)` are 2.718281828459045,
`POISSON.DIST(0,-0.5,FALSE)` is 1.6487212707001282; at x 1 a negative mean is `#NUM!` (`POISSON(1,-1,TRUE)`).
Evidence: probes/poisson-negative-mean A1-A7 (excel-graph, 2026-10-24, T-2484); it surfaced through gen-m9
23d82b62e9a85bdd A208 (a RECEIVED maturity of `0/POISSON(0,-1,2)` is 0, not an error).
A fractional x in (0, 1) truncates to 0 the same way: `POISSON(0.3,-0.5,TRUE)` is 1.6487212707001282 (gen-m9
b9a32ce9a1fd0d71 A221, T-2256). Unprobed: a huge negative mean (EXP overflow; ours `#NUM!`).
Ours: as Excel (T-2474, which also closed T-2628).
Status: VERIFIED (excel-graph)
Tasks: T-2628

## T.TEST of a blank one-cell sample and WEIBULL.DIST overflow (T-2496 probe)
Rule (excel-graph, 2026-10-25): `T.TEST(Data!E11,Data!A6:C12,2,123456789.123)` with E11 blank is `#VALUE!`, before the
type check that ours reports as `#NUM!` (T-2469). `WEIBULL.DIST(7,1E+8,0.3,"TRUE")` is `#NUM!`, where ours returns the limit 1
(hypothesis: Excel reports the overflow of (x/beta)^alpha; T-2435).
Evidence: probes/t2496-fin-subexpressions P!A90; probes/tbill-error-order-inner-calls (the T.TEST cell).
Ours: both as Excel since T-2621 ("Statistical argument-domain errors and extreme values (T-2621)").
Status: VERIFIED (excel-graph), one cell each; the boundaries are not probed.
Tasks: T-2469, T-2435, T-2621

## Statistical argument-domain errors and extreme values (T-2621)
Rule (probes/t2475-stat-subexpressions, excel-graph, 2026-10-24; each from the probe cell named, with its gen-m9 owner):
- PERMUTATIONA's number_chosen truncates toward zero before its sign check: `PERMUTATIONA(<blank>,-0.0001)` is 1 (P!A28,
  097af1ca33638670 A358; also "Scalar distributions at extreme arguments, part 2"). COMBINA is not symmetric about it:
  `COMBINA(-0.5,-0.0001)` is 1 (P!A35, 11eb3ab4f4f8104c A90), so number truncates toward zero, but with a number of 1 or
  more a number_chosen in (-1, 0) is `#NUM!` (`COMBINA(7,1-1.005)`, gen-m5 c688ca172fe533dc A118; `COMBINA(24,-0.0001)`,
  gen-m6 44194b2fefa30de2 A222; `COMBINA(16,SIN(..))`, `COMBINA(10,TAN(..))`, gen-m5 989a3d6036b5fef3 A210, gen-m6
  9e6a1b733ee1e1e0 A124). Ours truncates number_chosen only when number truncates to 0 and floors it otherwise; why
  Excel lets the n = 0 case through is UNVERIFIED (no cell has a number in (0, 1) or a number_chosen of -1 or less with
  n = 0).
- T.TEST and TTEST: a first sample that is itself an error value comes before a bad tails or type, and before the
  other arguments' errors: `T.TEST(<#VALUE!>,42,1,-1)` is `#VALUE!` (P!A58, 11eb3ab4f4f8104c A169) and
  `TTEST(<#VALUE!>,<#REF!>,2,<#REF!>)` is `#VALUE!` (P!A222, 5cebac1f02bf30f2 A311). A blank one-cell first sample does
  the same ("T.TEST of a blank one-cell sample" above). An error inside a range still comes after tails and type
  (`fn_t_test`, probes/stat-test-lift). UNVERIFIED: a second sample that is an error value against a bad tails/type
  (ours: tails/type first).
- `T.INV.2T(<FALSE cell>,1)` is `#DIV/0!` (P!A124, 2138b909d8f810ea A344), the p = 0, df 1 rule ("Student t quantile at
  p = 0 with df below 2 is #DIV/0!").
- WEIBULL.DIST at x = 0 is 0 cumulative too, even where beta^-alpha overflows: `WEIBULL.DIST(0,1E+8,4.6269E-09,TRUE)`
  is 0 (P!A78, 1b31092a0d79e5f1 A179); and an overflowing (x/beta)^alpha is `#NUM!`, not the cdf's limit 1:
  `WEIBULL(1E+15,1E+15,<TRUE cell>,1)` (P!A407, a5342a7e24fd803e A166), confirming the T-2496 hypothesis above. Which
  form Excel overflows on ((x/beta)^alpha or x^alpha·beta^-alpha) is UNVERIFIED where the two differ, and so is the
  density (cumulative FALSE) on that overflow: both cells are cumulative, and ours gives the density `#NUM!` too.
  NORMDIST's cdf at that x (ours 1) is likewise unprobed.
- `POISSON.DIST(2E+29,7,-1)` is 1 (P!A216, 5919660b7bb4d195 A88): a cumulative count past 2^64 is not rejected.
  UNVERIFIED: the pmf there (ours `#NUM!`).
- IFERROR's fallback that is a blank cell is the empty value, as its value argument's is (logic.md "IFERROR / IFNA of a
  blank cell is an empty value"): STDEV.P counts it as 0, `STDEV.P(<100, 2, blank, blank>,IFERROR(1/0,<blank>),
  {1,..,10})` is 25.559768683380703, the population deviation of 13 values (P!A158, 334f3dc9ca937f10 A226).
- `TRIMMEAN(<blank cell>,3.14159)` is `#NUM!` (P!A353, 9bc247733a1ca9fa A360), as a lone text or boolean cell is ("A lone
  text, boolean or empty cell as the data of QUARTILE, PERCENTILE, PERCENTRANK and TRIMMEAN").
- `SKEW({1E+200,1E+200,-1E+200},2.5,<FALSE cell>)` is `#DIV/0!` (P!A245, 6e18550e9fc9c1b9 A347): the squared deviations
  overflow, and Excel answers the zero-variance error rather than the 0 that dividing by an infinite deviation gives.
  SKEW.P and KURT do the same ("Overflowing squares (T-2432)").
- `NORMDIST(1E+300,0.3,2.5,0)` is `#NUM!` (P!A572, f82473275380882e A143): the density is `#NUM!` where the squared
  z-score overflows, not its limit 0 (NORM.S.DIST's density is unprobed).
- `BETA.INV(0.025,1E-15,2.5)` is `#NUM!` (P!A127, 2138b909d8f810ea A358; the true quantile is about EXP(-3.7E+15)), with
  the T-2255 BETAINV cells above: a quantile that underflows to 0 or rounds to 1 is `#NUM!`.
Evidence: probes/t2475-stat-subexpressions P!A28, A35, A58, A78, A124, A127, A158, A216, A222, A245, A353, A407, A572
(excel-graph, 2026-10-24).
Ours: as Excel since T-2621 (`permut.rs`, `fn_combina`, `t_test_element` with `PairedSamplesError::argument_is_error_value`,
`t_inv_of`, `weibull.rs`, `poisson.rs`, `if_error_like`, `trimmean.rs`, `fn_skew`, `norm_dist_of`, `beta_inv_of`). The
three last-digit cells of the same probe (P!A317 F.INV.RT, A417 NEGBINOMDIST, A520 SKEW) are T-2791's.
Status: VERIFIED (excel-graph), one cell each; the boundaries are not probed
Tasks: T-2621, T-2791

## Overflowing squares (T-2432)
Rule: over data whose squared deviations overflow (the gen-m9 pool's `{1E+200,1E+200,-1E+200}`), the moment functions
answer their zero-variance error and the tests `#NUM!`, never the finite value that dividing by an infinite deviation
gives: KURT, SKEW and SKEW.P are `#DIV/0!` (`KURT({1E+200,1E+200,-1E+200},Data!E4:E12)`, a61e547a1cab994c A14, five more;
`SKEW.P({1E+200,1E+200,-1E+200})`, 928d08e3e1eecb5f A314, seven more; `SKEW.P({1E+200,1E+200,-1E+200},{1,1,2,2,2,3})`,
probes/t2436/899f9555e67da2d6), Z.TEST with sigma omitted is `#NUM!` (`ZTEST({1E+200,1E+200,-1E+200},COUNT(..))`,
d5fd9e05e9e6f466 A341; c4635963fc926284 A245), and so are T.TEST (`T.TEST(Data!B6:D12,{1E+200,1E+200,-1E+200},1,2)`,
a612e89321a6df9c A375, probes/t2436/a612e89321a6df9c) and F.TEST (the T-2255 entry above).
T.TEST also orders its errors (gen-m9, excel-graph, 2026-10-23): either sample that is itself an error value comes before
tails and type (`TTEST({..},<#VALUE!>,<#N/A>,3)` is `#VALUE!`, 9e2225ead8a6a07b A59; `T.TEST({..},<#DIV/0!>,<#VALUE!>,3)`
`#DIV/0!`, 3b7990bd4e3307ba A100), extending "Statistical argument-domain errors and extreme values (T-2621)" to the
second sample, and tails and type are both read before either is range-checked (`T.TEST(<number>,{..},7,<#VALUE!>)` is
`#VALUE!`, 607e85cfee83a689 A194).
WEIBULL's cdf where statrs' x^alpha·beta^-alpha is inf·0 is a number, 1 - EXP(-(x/beta)^alpha):
`WEIBULL(0.2,1E+15,0.2,TRUE)` is 0.6321205588285577 (probes/t2436/abc2939ebd1e2365 A196), `WEIBULL(100,1E+8,99999,<42>)`
is 0 ((x/beta)^alpha underflows; probes/t2436/7c20351562a2d71d A106) and `WEIBULL(1E+15,42,10000000001,TRUE)` is a
number (read through `VLOOKUP(..,TAN(3.14159))`'s `#VALUE!` inside a T.TEST, 7f7ca726c3f68554 A308; ours 1, the closed
form's value, which no oracle cell shows directly). The density there is not: `WEIBULL(3,3.14159E+15,<ACCRINTM(..) about
1.5E+6>,FALSE)` is `#NUM!` (9e2225ead8a6a07b A340), the one density cell; how Excel forms the density is UNVERIFIED (ours
keeps statrs' NaN as `#NUM!`).
Evidence: gen-m9 (excel-graph, 2026-10-22/23, T-2255, T-2256), probes/t2436 (excel-graph, 2026-10-24); the cells in T-2432.
Ours: as Excel since T-2432 (`fn_kurt`, `fn_skew_p`, `z_test_element`, `t_test_p`, `t_test_element`,
`PairedSamplesError::argument_is_error_value`, `weibull_dist_of`). T.TEST's "both read, then checked" order rests on
one cell (A194), which a plain type-before-tails order would fit too.
Status: VERIFIED (excel-graph)
Tasks: T-2432

## PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1) (gap triage 2026-10-25)
Rule (checked against Excel's values; implemented by T-1811): with the n numbers sorted as a[1..n], Excel computes
the rank `r = 1 + k*(n-1)` in doubles, takes `j = floor(r)` and `h = r - j`, and returns `a[j] + h*(a[j+1] - a[j])`.
Because the 1 is added before the fraction is taken, a tiny k is lost in `r` (`1 + 1E-20` is 1), so the result is the
smallest value exactly, and h carries the rounding of `1 + k(n-1)` rather than of `k(n-1)`. This looks wrong for tiny k
and is still Excel: `PERCENTILE.INC(Data!B1:D6,1E-20)` is 0 (the smallest number), not 1.1E-20, and
`PERCENTILE.INC(Data!B7:D8,0.1)` is -1.1102230246251565E-16, not 0. A scratch script (triage, not in the repo) applied
the rule to the 40 failing gen-m9 PERCENTILE/PERCENTILE.INC cells whose arguments are literals or Data cells: 39 match
bit for bit (the zero-based `a[i] + h*(b-a)` that the engine uses matches none of the 13 visibly-off ones). The one miss is
gen-m9/5395b8a427764a3f Calc!A269 `PERCENTILE(Data!A9:D11,1E-16)` (data -2.5, 1E-15, ...): Excel -2.499999999999999, the
rule -2.4999999999999987, which `(1-h)*a + h*b` gives instead (that form misses 4 other cells). The same rule gives
probes/stat-argument-modes H424 and H429 (`PERCENTILE(C1:C6,0.3)` over 1 and 4) as 1.9000000000000001, Excel's value.
Evidence: eval/baseline/gen-m9.txt lines of T-1811 (excel-graph, 2026-10-23), e.g. gen-m9/38e4fc309fe199ea Calc!A311,
gen-m9/607e85cfee83a689 Calc!A20 (`PERCENTILE.INC(Data!B7:E11,1E-20)` is 1E-15, the smallest value), gen-m9/eccd03df42e40006
Calc!A151 (`PERCENTILE.INC(Data!B11:E12,1E-200)` is 0), Calc!A162 (`PERCENTILE(Data!A4:B4,1E-10)` over -0.0001 and 7 is
-9.999929998994208E-05), gen-m9/a4090567758520df Calc!A28.
Status: VERIFIED (excel-graph). Implemented by T-1811; QUARTILE and QUARTILE.INC share `percentile_inc_impl` and show 0 regressions across their oracle cells. One cell is not reproduced (A269 above).
Ours: `percentile_inc_impl` (functions/statistical/percentile.rs) uses the one-based rank `1 + k*(n-1)` (T-1811). Known miss: gen-m9/5395b8a427764a3f Calc!A269 (Excel -2.499999999999999, rule -2.4999999999999987) stays in the baseline.
Tasks: T-1811
