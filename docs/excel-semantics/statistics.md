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
Status: VERIFIED (desktop cached values)
Implemented in: `functions/statistical/fisher.rs` `fn_fisher_inv`
Tasks: T-0976

## T.DIST.RT and T.DIST.2T are not 1 - T.DIST
Rule: the Student-t upper tail is computed on its own, not as `1 - T.DIST(x, df, TRUE)`.
`T.DIST(1,1,TRUE)` = 0.75 exactly, yet `T.DIST.RT(1,1)` = 0.25000000000000006 and
`T.DIST.2T(1,1)` = 0.5000000000000001, so the two tails do not add up to 1. The far tail keeps
full relative precision: `T.DIST.RT(1E6,10)` = 1.230468749943611E-56 (equal to `T.DIST(-1E6,10,TRUE)`),
where `1 - T.DIST` would be 0. Small df have closed-form-exact CDFs: `T.DIST(1,2,TRUE)` =
0.7886751345948129 = 0.5 + 1/(2*sqrt(3)) in doubles.
Evidence: `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`!D15, F15, D16, F18, D19;
`T.DIST.2T T.INV.2T`!C15.
Status: VERIFIED (observation); Excel's algorithm is UNVERIFIED (T-0969)
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
Evidence: `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` Sheet1!F2, F3.
Status: VERIFIED (observation); formulation UNVERIFIED (T-0977, sweeps T-1048 and T-1049)
Tasks: T-0977

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
Earlier negatives (T-1186): the engine's CF in 7 orders, Cody CALERF with and without its x*16 split, and the
asymptotic series in five orders, all at most 22%.
Not tried:
- fdlibm `erfc`. Its split `z` (x with the low 32 bits cleared) avoids the `fl(x*x)` error that
  probes/subnormal-function-boundaries A5 (`ERFC(26.4)`) shows, so it does not fit.
- A Chebyshev fit such as NR3 `erfccheb`. It adds a term to the exp argument, the same objection.
- MSVC's own `exp`, which is not available here.
Evidence: probes-numeric/t1040-erfc (excel-graph, 2026-10-08), scored against 50-digit mpmath and by
docs/research/t1187-erfc-candidates.py. That script re-implements every candidate above in IEEE double and prints
the counts (T-1187, 2026-10-11).
Status: OPEN (no candidate found); T-1186 stays blocked.

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
(`static_analysis_rank`) makes such a formula a dynamic-array anchor so it spills. For a literal array (`{7,1,5}`) we also spill `#VALUE!` 1x3; that is
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
Status: VERIFIED (excel-graph); literal arguments UNVERIFIED
Tasks: T-1152, T-1203, T-1208, T-1215, T-1303

## Statistical function argument modes
Rule: every scalar argument of the M9 statistical functions lifts elementwise over an array literal and over a multi-cell
range (all 302 lift cells are 2: 210 `ROWS(F(..{v;v}..))` and 92 `ROWS(F(..G1:G2..))`): distribution parameters including
`cumulative`, LARGE/SMALL `k`, PERCENTILE `k`, QUARTILE `quart`, PERCENTRANK and Z.TEST `x`, RANK `number`, T.TEST
`tails`/`type`, TRIMMEAN `percent`, FORECAST `x`, PROB `lower_limit`, FORECAST.ETS `target_date` and FORECAST.ETS.STAT
`statistic_type`. The data arguments of AVEDEV, DEVSQ, GEOMEAN, HARMEAN, KURT, SKEW, SKEW.P, MEDIAN, STDEV*, VAR* skip
text, booleans and blanks inside a range but coerce typed ones (`AVEDEV(C1:C6,5)` over 1, "2", TRUE, "x", 4, blank is
AVEDEV(1,4,5); `AVEDEV(1,"2",4,TRUE)` = 1, AVEDEV(1,2,4,1)). The A-variants (AVERAGEA, MAXA, MINA, STDEVA, STDEVPA,
VARA, VARPA) count range text as 0 and TRUE as 1 (`AVERAGEA(C1:C6,5)` = 11/6) and a typed "2" as 2. MODE, MODE.SNGL
and MODE.MULT are `#N/A` for `(1,"2",4,TRUE)`, so they do not coerce the typed TRUE the way STDEV does (unprobed which
argument is dropped, T-1291). Array arguments (CORREL, COVAR, F.TEST, T.TEST, LARGE, PERCENTILE, QUARTILE, TRIMMEAN,
Z.TEST, LINEST, ...) take array literals whole with the range's value; LARGE, SMALL, PERCENTILE*, QUARTILE*,
PERCENTRANK*, TRIMMEAN, Z.TEST, F.TEST and T.TEST skip text and booleans in a range, GROWTH, TREND, LINEST and LOGEST are
`#VALUE!` for them, and the pairing functions are `#N/A` for unequal sizes with text (size-only and text-only unprobed, T-1292). The FORECAST.ETS family is `#N/A` for a
plain 8-point series on the timeline 1..8 (T-1293). These decide the generator modes in eval/catalog/functions.yaml
(`value`, `aggregate`, `aggregate_a`, `array`; RANK `ref`, COUNTBLANK and the *IFS ranges are `range`).
Evidence: probes/stat-argument-modes (573 cells, excel-graph, 2026-10-09, T-1080).
Ours: no scalar argument lifts (`#N/IMPL!` for an array, `#VALUE!` for a range; T-1290); typed numeric text in the
A-variants and MODE (T-1291); the pairing functions are `#VALUE!` and F.TEST takes the wrong tail (T-1292); FORECAST.ETS is
`#N/IMPL!`; many distribution values are a few ulps off (the existing accuracy entries above).
Status: VERIFIED (excel-graph)
Tasks: T-1080, T-1290, T-1291, T-1292, T-1293

## Array arguments given a scalar, a single cell or an error
Rule: an array argument of the paired and single-array statistical functions takes a plain number or a single cell as an
array of one value, and an error value as that error; the usual rules then apply. For the paired functions, unequal sizes
are `#N/A` and one point is `#DIV/0!` (COVAR of one pair is 0): `TTEST(A7:C7,<number>,1,1)`, `STEYX(D4:D10,B6)`,
`COVAR(0,C6:D6)`, `COVAR(D4:D8,<number>)`, `RSQ(A4,B11:C12)`, `INTERCEPT(B9:E9,1E-15)`, `CHISQ.TEST(B4,C5:D11)`,
`FORECAST.LINEAR(x,C2:D7,<number>)` and `FORECAST.LINEAR(x,0.2,D11:D12)` are `#N/A`; `SLOPE(123456789.123,<number>)`,
`INTERCEPT(1.005,2.5)`, `FORECAST.LINEAR(x,0.2,1)` and `FORECAST.LINEAR(x,7,137.5)` are `#DIV/0!`; `COVAR(0,D6)` is a number (inferred 0;
`(B6-0.5)>COVAR(0,D6)` is TRUE). `PEARSON(D5,SEQUENCE(0))` is `#CALC!`. T.TEST checks tails and type first:
`T.TEST(B11,A8:B9,0.1,0.3)` with B11 = FALSE is `#NUM!`. GROWTH of one value is a number (`GROWTH(GAMMA(100))` =
9.332621544394404E+155), `TREND(#NUM!)` is `#NUM!`, and `GROWTH(B2:B3,<number>,..)` with B2:B3 = "?b", 0 is `#REF!` (the size
check before the text check). LARGE of an error is that error (`LARGE(<#VALUE!>,k)`, `LARGE(<#REF!>,0.1)`), and so is
PERCENTRANK (`PERCENTRANK(<#N/A>,100)` is `#N/A`). `QUARTILE(D6,7)` with D6 = " x " is inferred `#NUM!` (inside a BETAINV whose
probability is also invalid, so UNVERIFIED; T-1403 probes it), and `PERCENTILE.EXC(A8:B8,<#VALUE!>)`
with no number in A8:B8 is `#VALUE!` (k's error before the empty-data `#NUM!`). `PERCENTRANK.EXC(B11,1.005)` with B11 =
1.005 is not 0.5 (inferred `#N/A` through `VLOOKUP(..)`; UNVERIFIED, T-1403 probes it).
Evidence: gen-m9 (excel-graph, 2026-10-12, T-1289): 1782efcecd5d4b1b Calc!A10, A25, A38; 3462856259a37891 A22;
50f10c4062fb2e98 A3, A17, A31; 696af2c8cee2f57e A9; 89693ab0cafc29df A24, A30; a2e51f48717af2a1 A34; adb3fd63c7368166 A26,
A33; b58b6d7cd39d195f A2, A31; bc0901ce6193d576 A12; d244411dd90f7fbd A10, A17, A21; eefcc3989f31de76 A22;
95efb37a9874f82e A2; b5352ebdb9c5f1a1 A24; 288b99797f58f149 A3; d5bd4272655c5211 A31; 65faaeb48419087d A17. The arguments
are read from the cases' Data sheets; each cell's other operands were checked not to decide the result.
Ours: a plain number or single cell in these arguments is `#VALUE!` (one-element array literals already work:
`COVAR({0},{1,2})` is `#N/A`, `SLOPE({1},{2})` `#DIV/0!`); `LARGE(#N/A,1)` is `#N/IMPL!`; PERCENTILE.EXC checks empty data
before k; `PERCENTRANK.EXC` of a single value is 0.5.
Status: VERIFIED (web) for the listed cells; PERCENTRANK.EXC of one value UNVERIFIED
Tasks: T-1401, T-1402, T-1403

## SKEW.P of two numbers is #DIV/0!
Rule: `SKEW.P` of two numbers is `#DIV/0!`, like SKEW: `SKEW.P(E11:E11,A11:A12,A5:A7)` over -0.0001 and 1 (blank, FALSE,
TRUE and "a" skipped) is `#DIV/0!`. Whether three numbers are enough is unprobed.
Evidence: gen-m9/3b8e9078261b5bd2 Calc!A13 (`LAMBDA(v,acc,v)(SKEW.P(..),..)`, excel-graph, 2026-10-12, T-1289).
Ours: 0 (`SKEW.P(1,2)` and `SKEW.P(1,2,3)` are 0).
Status: VERIFIED (web) for two numbers; three numbers UNVERIFIED
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
Tasks: T-1293

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
index. A boolean field (`TRUE`) selects no column: DSUM, DMAX, DMIN, DPRODUCT and DCOUNT give 0, DAVERAGE,
DSTDEV, DSTDEVP, DVAR and DVARP give `#DIV/0!`, DGET gives `#NUM!`, and DCOUNTA counts the matching records
(4). An empty field (`DCOUNT(A1:C6,,E1:E2)`) gives DCOUNT and DCOUNTA the count of
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
Ours: field and criteria do not lift (`#N/IMPL!`, `#VALUE!`), and DCOUNT/DCOUNTA with an empty field are
`#VALUE!` (T-1389). A generated sample also showed gaps in header matching and criteria (T-1390).
Status: VERIFIED (excel-graph)
Tasks: T-1086, T-1389, T-1390

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
Ours: GAMMA is `statrs` Lanczos (not exact at integers) and returns `#NUM!` for `GAMMA(171.5)` and
`GAMMA(171.6)`; GAMMALN is fdlibm `lgamma` (63 of the 171 integer cells differ).
Evidence: probes-numeric cases `t1042-gamma-int-1-90`, `t1042-gamma-int-91-171`, `t1042-gamma-half-int`,
`t1042-gamma-unit-interval`, `t1042-gamma-negatives-edges` (first half of each case's cells is GAMMA,
second half GAMMALN, in formula order; GAMMA overflow edge: `t1042-gamma-negatives-edges` Calc!A18:A24;
GAMMALN of x <= 0: Calc!A28:A44 there and A63:A73 of `t1042-gamma-half-int`). Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-0973 (kernel, including the 171 < x < 171.62 overflow edge)

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
  (`T.DIST(2,1E11,FALSE)` = 0.05399096651413289). The exact cut between 1E10 and 1.5E10 is not probed.
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
Evidence: probes-numeric cases `m9-t-dist-cdf`, `m9-t-dist-pdf`, `m9-t-dist-rt`, `m9-t-dist-2t`, `m9-t-inv`,
`m9-t-inv-2t`, `m9-confidence-t`, `m9-t-nonint-df`, `m9-t-edges`. Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-0969 (T.DIST family), T-0967 (T.INV, T.INV.2T, CONFIDENCE.T)

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

## Legacy distribution names sweep (T-1047)
Rule: each legacy name returns exactly the same value or error as its modern equivalent on every paired cell (BETADIST = BETA.DIST cum, BETAINV = BETA.INV, BINOMDIST = BINOM.DIST, CHIDIST = CHISQ.DIST.RT, CHIINV = CHISQ.INV.RT, CHITEST = CHISQ.TEST, CONFIDENCE = CONFIDENCE.NORM, CRITBINOM = BINOM.INV, EXPONDIST = EXPON.DIST, FDIST = F.DIST.RT, FINV = F.INV.RT, FTEST = F.TEST); the aliases need no separate kernel.
Oracle sweep (excel-graph), 12 cases, 33-92 cells each. Aliases were paired with the modern name on the typical-value grid only; the edge, coercion and empty-reference bullets below were probed on the legacy names only. Quirks seen:
- Counts and degrees of freedom truncate: `BINOMDIST(2.9,10,0.5,TRUE)` = `BINOMDIST(2,...)` = 0.0546875, `CHIDIST(3,2.9)` = `CHIDIST(3,2)`, `FDIST(1,2.9,5)` = `FDIST(1,2,5.9)` = `FDIST(1,2,5)`, `CONFIDENCE(0.05,1,100.9)` = size 100, `CRITBINOM(10.9,0.5,0.5)` = 5. `CONFIDENCE(...,0.9)` (size truncates to 0) is #NUM!.
- df above 1E10 is #NUM! for CHIDIST, FDIST (both df) and FINV (`CHIDIST(3,10000000001)`, `m9-legacy-chidist`/`-fdist`/`-finv`); `CHIDIST(3,10000000000)` = 1 and `FDIST(1,10000000000,5)` = 0.5841198129434723 are accepted (the 1E10 boundary was not probed for FINV or FDIST's second df). CHIINV has no such cutoff: `CHIINV(0.5,10000000001)` = 10000000000.333334 and `CHIINV(0.5,10000000000)` = 9999999999.333334 (`m9-legacy-chiinv`), about df - 2/3 at p = 0.5.
- Boundary p: `CHIINV(1,..)` and `FINV(TRUE,2,5)` are 0 (p = 1 allowed), p = 0 is #NUM!; `BETAINV(0,..)`/`BETAINV(1,..)` are both #NUM! (open interval).
- BINOM.INV/CRITBINOM: alpha 0 or 1 is #NUM!, and probability 0 or 1 is #NUM!, even though BINOMDIST accepts them (`BINOMDIST(0,5,0,TRUE)` = 1).
- BETADIST with bounds: x outside [A,B] is #NUM! (`BETADIST(0.5,2,3,2,5)`), A = B is #NUM!, but x = A gives 0 (`BETADIST(2,2,3,2,5)`); x outside [0,1] without bounds is #NUM!.
- Coercion: text numbers (`"0.5"`) and booleans coerce in scalar arguments (`BETADIST(TRUE,2,3)` = 1, `CHIDIST(TRUE,2)` = 0.6065306597126334); an empty reference is 0 (`BETADIST($Z$1,2,3)` = 0, `EXPONDIST($Z$1,1,TRUE)` = 0, `CHIDIST($Z$1,2)` = 1) so a zero df/lambda/p is #NUM!. The `cumulative` flag takes any nonzero number as TRUE (`EXPONDIST(1,1,5)`, `BINOMDIST(2,10,0.5,5)`) and the text "TRUE".
- CHITEST/FTEST: mismatched shapes are #N/A (`CHITEST(B1:D2,B4:D4)`); a zero expected cell is #DIV/0!; a negative expected cell is #NUM!; FTEST with a zero-variance range or fewer than 2 numbers in a range is #DIV/0!; array literals are accepted.
- CHITEST of a 1x1 range is #N/A (`CHITEST(N1:N1,G1:G1)`). A text cell in the actual range drops that pair but the degrees of freedom still come from the range shape: `CHITEST(F1:F3,G1:G3)` with F = {5,0,"x"} = 0.00012340980408667956 (= CHIDIST(18,2)).
Evidence: probes-numeric cases `m9-legacy-betadist`, `-betainv`, `-binomdist`, `-chidist`, `-chiinv`, `-chitest`, `-confidence`, `-critbinom`, `-expondist`, `-fdist`, `-finv`, `-ftest`. Oracle: excel-graph, 2026-10-13.
Status: VERIFIED (excel-graph)
Tasks: T-1047

## Legacy distribution names sweep, second family (T-1048)
Rule: GAMMADIST, GAMMAINV, HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TINV, WEIBULL return exactly the same value or error as their modern equivalent (GAMMA.DIST, GAMMA.INV, HYPGEOM.DIST with cumulative FALSE, LOGINV = LOGNORM.INV, LOGNORM.DIST cum TRUE, NEGBINOM.DIST cum FALSE, NORM.DIST, NORM.INV, NORM.S.DIST cum TRUE, NORM.S.INV, POISSON.DIST, T.INV.2T, WEIBULL.DIST) on every paired cell, so the aliases need no separate kernel. TDIST with tails 1 is the exception: it differs from T.DIST.RT for negative x (T.DIST.2T agrees with TDIST tails 2). TTEST = T.TEST and ZTEST = Z.TEST on every paired cell too.
Oracle sweep (excel-graph), 16 cases, 34-102 cells each; every paired cell (column B) is identical except the TDIST row below. Quirks seen:
- `TDIST(-1,3,1)` and `TDIST(-1,3,2)` are #NUM! (x must be >= 0); `T.DIST.2T(-1,3)` is #NUM! too, but `T.DIST.RT(-1,3)` = 0.8044988905221147. `TDIST(1,0.9,1)` is #NUM! (df truncates to 0); `TDIST(1,1.9,2)` = 0.5000000000000001 (= `T.DIST.2T(1,1.9)`, df truncates to 1); tails truncate (`TDIST(1,3,1.9)` and `TDIST(1,3,TRUE)` = tails 1, `TDIST(1,3,2.1)` = tails 2); tails 0 and 3 are #NUM!, and an omitted tails (`TDIST(1,3,)`) is #NUM!.
- Boundary p differs by function: `GAMMAINV(0,2,1)` = 0 but `GAMMAINV(1,2,1)` is #NUM!; `LOGINV` and `NORMINV` reject both 0 and 1 (#NUM!); `NORMINV(1e-300,0,1)` = -37.0470962993612.
- A boolean p is #NUM! (`GAMMAINV(TRUE,2,1)`, `LOGINV(TRUE,0,1)`, `NORMINV(TRUE,0,1)`) because TRUE coerces to 1, while a boolean x coerces to 1 (`NORMDIST(TRUE,0,1,TRUE)`, `GAMMADIST(TRUE,2,1,TRUE)`) and text numbers coerce everywhere; non-numeric text is #VALUE!.
- The `cumulative` flag of GAMMADIST/NORMDIST takes any nonzero number as TRUE, the text "TRUE", and an omitted (empty) argument as FALSE.
- Zero or negative scale is #NUM! (`NORMDIST(0,0,0,TRUE)`, `GAMMADIST(1,2,0,TRUE)`); `GAMMADIST(0,2,1,..)` = 0 for both cum values; `LOGNORMDIST(0,0,1)` is #NUM!; far tails flush to 0 (`NORMSDIST(-38)` = 0, `NORMDIST(-40,0,1,..)` = 0) while `NORMSDIST(-37)` = 5.7255712225239266E-300.
- NEGBINOMDIST with p = 1 is #NUM! even for zero failures (`NEGBINOMDIST(0,3,1)`); successes = 0 is #NUM!. HYPGEOMDIST with sample_s > number_sample is 0, not #NUM! as documented (`HYPGEOMDIST(5,4,8,20)` = 0, `HYPGEOMDIST(1,0,8,20)` = 0); population_s > number_pop is #NUM! (`HYPGEOMDIST(1,4,25,20)`, `HYPGEOMDIST(1,4,8,5)`), and so is number_pop = 0 (`HYPGEOMDIST(1,4,8,0)`, where population_s and number_sample also exceed it); number_sample > number_pop alone was not probed.
- TTEST: tails 0/3 and type 0/4 are #NUM!, tails accepts a boolean (`TTEST(..,TRUE,2)`) and text numbers, tails/type truncate, an omitted tails or type is #NUM!; unequal counts with type 1 (paired) is #N/A; type 1 on a pair whose differences have zero variance is #DIV/0!; a one-element range is #DIV/0!; an empty range is #DIV/0!.
- ZTEST: sigma 0 or negative is #NUM!; a one-element range is #DIV/0! (with or without sigma); an empty range is #N/A; text x "5" and TRUE coerce, non-numeric text is #VALUE!; an omitted sigma (empty argument) is #NUM!, while an omitted x (`ZTEST(D1:D10,)`) returns a value (1.3161580317707342E-09; that it equals x = 0 was not probed); a range as x lifts and spills (see "Statistical function argument modes").
Evidence: probes-numeric cases `t1048-gammadist`, `-gammainv`, `-hypgeomdist`, `-loginv`, `-lognormdist`, `-negbinomdist`, `-normdist`, `-norminv`, `-normsdist`, `-normsinv`, `-poisson`, `-tdist`, `-tinv`, `-ttest`, `-weibull`, `-ztest`. Oracle: excel-graph, 2026-10-14.
Status: VERIFIED (excel-graph)
Tasks: T-1048
