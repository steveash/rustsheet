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
    `Z.TEST(TRUE,0.5,1)` is #DIV/0! (probes-numeric/t1049-ztest A51; ours #VALUE!, T-2027), while typed text is
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
Ours: matches (T-1290), except that BETA.DIST, F.DIST, F.DIST.RT and FDIST read a TRUE as `#VALUE!`, as their scalar
calls always did (L1, L13: T-1849; WEIBULL's and PHI's are unprobed). Every argument of these functions is `Lift`
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
Ours: matches the lift cells (T-1848). The kernels keep their scalar reads, so a TRUE element is `#VALUE!` in BETA.INV,
F.INV and F.INV.RT, which read numbers without booleans (L2, L3, L19: T-1849). Their quantile comes from statrs 0.18's
`inv_beta_reg`, ported line for line to `statistical/inverse_beta.rs` with a per-call cap of its own: at most 512 Newton
steps and 4,096 step adjustments (64 and 512 for Student's t). Where statrs finishes, the port returns its double bit for bit
(unit test `the_port_is_statrs_where_statrs_finishes`; probes-numeric is unchanged cell for cell), so the values
probes-numeric checks are main's. statrs needs at most 377 steps where it finishes (13 million samples: F at p down to
1E-162, Beta shapes 1E-5 to 1E5, Student t). Below the old thresholds the quantile is still a bisection on the cdf of at
most 1,100 (BETA.INV) or about 1,300 (F.INV) or 264 (T.INV) evaluations. The lifting table charges BETA.INV, BETAINV,
F.INV, F.INV.RT and FINV 2^18 units an element and T.INV, T.INV.2T, TINV and CONFIDENCE.T 2^16: the bound of each
kernel's caps at a full-cost incomplete-beta cdf (140 continued-fraction steps, 3.1 to 3.4 us in release), about 4.0 ms
(F.INV's 1,301-cdf bisection) and 0.82 ms (T.INV's 264-cdf bisection); the cases measured cost far less (167 us for
`BETA.INV(1E-300,0.5,1E6)`, 17.5 us for `T.INV.2T(1E-9,99999)`); lift/kinds/statistical.rs has the measurements.
`F.INV(1E-160,5,5)` and `F.INV(1E-100,5,5)` are 1.3470392369521052E-15 in statrs and here, as on main, where Excel follows
the power law (E4, E5), and `F.INV(1E-50,2000,2)` is 0.008195481623831033 (statrs converges to a wrong root) where Excel
gives 0.004452511009947361 (E7); E8 and E10-E12 miss by 4 to 357 ulp, as on main (all T-1871).
DEVIATION (decision 0014, CLAUDE.md directive 5): where statrs' search panicked or never finished, the capped search is
`#NUM!` and Excel gives a number. With both df below 1E5, statrs panics (a NaN first estimate, an unwrapped incomplete
beta) for every F.INV below p = 1.6E-162, where `p * p` underflows (`F.INV(1E-300,5,5)`, `F.INV(1E-170,5,5)`: E1-E3), and
its step loop never ends once a Newton step is infinite, which a 1-million-sample sweep found for about 7% of F.INV calls with
p below 3.6E-27 (`F.INV(1E-50,6000,4)`, E6) and once for BETA.INV at p = 1-2^-53
(`BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)`, E9). Unit test `quantile_searches_that_hung_or_panicked_are_bounded`;
T-1871 gives Excel's values there.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted BETA.INV, BETAINV, F.INV, F.INV.RT or FINV of more than 512
elements, and a lifted T.INV, T.INV.2T, TINV or CONFIDENCE.T of more than 2,048, is `#ERROR!` before it runs where Excel
computes it (2^27 units, `limits::MAX_LIFTED_CELL_VISITS`, over 2^18 and 2^16 an element;
`ROWS(BETA.INV(SEQUENCE(513)/514,2,3))`, unit test `quantile_kernels_are_charged_their_weight`). At the bound of the caps,
512 F.INV elements take about 2.1 s and 2,048 T.INV elements about 1.7 s, the 2 s budget of the Bessel entries
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
incomplete gamma is statrs 0.18's `gamma_lr` and `gamma_ur` (a series and a continued fraction run to a 1E-15
tolerance) and its quantile statrs' `Gamma::inverse_cdf` (halve and double a bracket, 8 bisections, 4 Newton steps),
ported line for line to `statistical/incomplete_gamma.rs` with a per-call cap of its own: at most 2^19 terms in one
cdf, and in one whole inversion over all its cdfs. Where statrs finishes within the cap the port returns its double bit
for bit (unit test `the_port_is_statrs_where_statrs_finishes`; probes-numeric and ironcalc-calc are unchanged cell for
cell), so the values those suites check are main's. The series needs about 6.8 * sqrt(a) terms at x = a: 481,117 at
the largest chi-squared shape (df 1E10, a = 5E9), so every chi-squared cdf is kept. CHISQ.TEST uses the same capped
cdf (a sheet's cell count keeps its shape under 2^33, where the series can need about 630,000 terms: past the cap, see
the DEVIATION below). The lifting table charges every entry 2^18 units an element: the bound of the cap at the slowest
term, the continued fraction's NaN path (4.1 to 4.7 ns in release; a converging fraction at the cap, which rescales on
about 65% of its terms at a shape of 1E15, costs 2.4 ns, and a series term 2.5 ns), plus the inversion's 2,120 cdfs,
about 2.6 ms (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT` has the measurements; 1.2 ms for `CHISQ.DIST(1E10,1E10,TRUE)`).
Where x / beta underflows to 0 (statrs panicked) the cdf is statrs' value at its smallest x, 0, as Excel's
`GAMMA.DIST(1E-300,2,1E100,TRUE)` is (E7); the Excel evidence is at shape 2 only, and a shape below 1 (where the true
value does not underflow) or near 0 (where the cdf is 1) is unprobed (T-1891). `GAMMA.DIST(1E300,1,1E-10,FALSE)` is 0 here, as on main, where Excel is `#NUM!`
(E18, T-1891); at a statistic of 5E-324, whose half underflows, `CHISQ.DIST(x,2,TRUE)` is 0 and `CHISQ.DIST.RT(x,2)` and
`CHIDIST(x,2)` 1 where main panicked (Excel unprobed). CHISQ.TEST cannot reach that statistic: its sum of
`diff*diff/exp` is never 2^-1074.
The values past the cap were already wrong on main where it finished (statrs loses digits at a huge shape:
 `GAMMA.DIST(1E12,1E12,1,TRUE)` 0.49898078573633475, `POISSON(1E12,1E12,TRUE)` 0.49906665837352593), and so are
the kept ones at df 1E10 (E16, E17: 0.4999982128217492; E20: T-0971).
DEVIATION (decision 0014, CLAUDE.md directive 5): where statrs never finished, panicked, or needs more than 2^19
terms, the capped incomplete gamma is `#NUM!` and Excel gives a number. statrs' series never converges once
`a + 1 == a` (a shape of 2^53 or more: `GAMMA.DIST(1E16,1E16,1,TRUE)`, `POISSON(1E16,1E16,TRUE)`,
`GAMMA.DIST(1E300,1E300,1,TRUE)`, `GAMMA.INV(0.5,1E300,1E-300)`: E1-E3, E5), its continued fraction never ends once its
terms are NaN (`GAMMA.DIST(1.0000001E300,1E300,1,TRUE)`, E4); where x / beta overflows it panicked, and Excel is
`#NUM!` there too (E6). Past the cap: a cdf whose shape is above about 5.9E9 at x just below the shape (the series), or
above about 1E15 at x just above it (the continued fraction, which also grows with sqrt(a): 8,839 terms at 1E9, 90,016
at 1E12, 402,191 at 1E14; `GAMMA.DIST(1000000000000001,1E15,1,TRUE)`), such as `GAMMA.DIST(1E10,1E10,1,TRUE)` (main
0.5000049342568011), `POISSON(1E10,1E10,TRUE)` (main 0.5000143137798188), E9, E11, E12 (main 0.7610722410693369) and
E13 (main 0.020185241475442515); an inversion whose shape is above about 3.5E8, where statrs ends in a number
(`CHISQ.INV(0.26774,1312890000)`, E14: main 1312858246.5965345, close to Excel's, which the cap turns into `#NUM!`;
E15: main 4714346327.662597; at df 1E8 or more CHISQ.INV is `#NUM!` for about 97% of probabilities on main already);
and a CHISQ.TEST whose shape (df/2) passes about 5.9E9 (ranges of more than about 1.2E10 cells) with its statistic
near df (Excel unprobed). Unit tests `incomplete_gammas_that_hung_or_panicked_are_bounded`
and `a_call_past_its_cap_is_nan`; T-1891 gives Excel's values there.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted CHISQ.*, CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV,
GAMMAINV, POISSON or POISSON.DIST of more than 512 elements is `#ERROR!` before it runs where Excel computes it (2^27
units, `limits::MAX_LIFTED_CELL_VISITS`, over 2^18 an element; `ROWS(POISSON(SEQUENCE(513),1,TRUE))`, unit test
`incomplete_gamma_kernels_are_charged_their_weight`). At the bound of the cap, 512 elements take about 1.3 s, within the
2 s budget of the Bessel entries (engineering.md "BESSEL order cap").
Status: VERIFIED (excel-graph)
Tasks: T-1872, T-1891, T-0971, T-0972

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
unchanged here), and checks each element alone. The pmf is statrs' closed form, so HYPGEOMDIST is `Constant`. The cdf
is statrs 0.18's `Hypergeometric::cdf`, which sums the pmf term by term from 0 to x, one `exp` of three `ln_binomial`s
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
sample), such as `HYPGEOM.DIST(1074741824,2147483648,2147483648,4294967296,TRUE)` (1,255,925 terms are not 0; main
1.0000065293398746 after 191 s in release), and any sum of more than 2^18 terms at a population above 2^52, where
nothing is skipped (above 2^53 a count rounds as a double and statrs' terms are not the pmf's:
`HYPGEOM.DIST(524288,1048576,2^59,2^60,TRUE)` was `#NUM!` on main too, its sum infinite). Unit tests
`a_call_past_its_cap_is_nan`, `the_cap_counts_the_terms_summed` and `a_cdf_sum_is_bounded`; T-1896 probes Excel's
values there.
DEVIATION (decision 0014, CLAUDE.md directive 5): a lifted HYPGEOM.DIST of more than 32 elements is `#ERROR!` before
it runs where Excel computes it (2^27 units, `limits::MAX_LIFTED_CELL_VISITS`, over 2^22 an element;
`ROWS(HYPGEOM.DIST(2,3,5,10,SEQUENCE(33)>0))`, unit test `hypgeom_dist_is_charged_its_weight`). At the bound of the
cap, 32 elements take about 2.1 s, about the 2 s budget of the Bessel entries (engineering.md "BESSEL order cap").
HYPGEOMDIST is `Constant` and lifts up to the output cap.
Status: VERIFIED (excel-graph) for the lifting (the Rule's cells); the term cap is a DEVIATION whose Excel values are unprobed (T-1896)
Tasks: T-1873, T-1896, T-0977

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
are 0; a positive density there (moderate x with df1 near 30-140, `F.DIST(20,100,3,FALSE)`) keeps main's `#NUM!`,
unprobed (T-1865). Degrees of freedom large enough to overflow those
powers keep main's `#NUM!` (`F.DIST(3,50,100,FALSE)`; probes-numeric/t1530-f-df-cut `F.DIST(2,5,1E10,FALSE)`, where
Excel gives 0.14167277676179968), because the logarithmic form is not accurate enough there (T-1865); no
probes-numeric cell covers the moderate-x region, and every one is unchanged from main.
Status: VERIFIED (excel-graph)
Tasks: T-1290, T-1865

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
PERCENTRANK (`PERCENTRANK(<#N/A>,100)` is `#N/A`). `QUARTILE(D6,7)` with D6 = " x " is inferred `#NUM!` (inside a BETAINV whose
probability is also invalid, so UNVERIFIED; T-2066 probes it), and `PERCENTILE.EXC(A8:B8,<#VALUE!>)`
with no number in A8:B8 is `#VALUE!` (k's error before the empty-data `#NUM!`). `PERCENTRANK.EXC(B11,1.005)` with B11 =
1.005 is not 0.5 (inferred `#N/A` through `VLOOKUP(..)`; later probed: a single value equal to x is 1, probes/percentrank-single-value B6).
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
Tasks: T-1293

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
Ours: `#N/IMPL!` for every call.
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
Ours: `field` (LiftArrays) and `criteria` (LiftArraysRangeWhole) lift through SPEC-0011's driver, `database`
is read whole, and DCOUNT/DCOUNTA with an empty field count the matching records (T-1389). A generated
sample also showed gaps in header matching and criteria (T-1390).
Lifted calls are bounded (directive 5): each lifted element is charged the database's rows times the
criteria headers and criteria cells it scans (`Model::db_element_cost`), and a call over
`MAX_LIFTED_CELL_VISITS` (2^27) is `#ERROR!` where Excel returns a value, e.g. `DSUM(A:Z, SEQUENCE(1,26), crit)`
over whole columns. A call that does not lift is not charged.
Status: VERIFIED (excel-graph); the lifted-call bound is DEVIATION (decision 0014)
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
Evidence: `docs/research/t1941/` and the `kmyst-gam-*` cases on branch `agent/research/T-1941-crack-gammaln-gamma-ln` (HANDOFF2.md
§2, §3, §12; `gammaln_big_check.c`, `k2_ref.py`, `review3/final/cand9.py`, `w5/p4/ss_fc.txt`, `w6/data/xl_chain_d3ls.txt`);
ironcalc-calc/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx (desktop cache) now passes all 91 cells, and probes/stat-argument-modes
two more.
Implemented in `functions/statistical/gammaln.rs` (`excel_gammaln`), `gamma_kernels.rs` and `x87_register.rs` (K2's register
head). `test/fixtures/gammaln_kernel.fixture` (1,500 random points per region) pins the engine to the research checkers on
every point; `gammaln_matches_the_t1941_runs` reruns the table above from `RUSTSHEET_T1941_ROWS`.
Status: VERIFIED (excel-graph and desktop) for the structure and the bit-exact pieces; the four rationals are best known
models (T-2122)
Tasks: T-1941, T-2128, T-2122

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
Ours: `student_t.rs` `closed_form_tail` (since T-0969 attempt 8); below y = 0.1 the general tail.
Status: VERIFIED (excel-graph) from y = 0.100112 up and below y = 0.099900. The exact cut is UNVERIFIED: whether y = 0.1
itself takes the closed form, and whether Excel tests y or z = 1 - y, need a dense probe near t = sqrt(18).
Tasks: T-0969

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
Status: VERIFIED (excel-graph) for df 1 to 1000 at those t; region y > 0.99 beyond t = 1 UNVERIFIED
Tasks: T-0969, T-2119

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
Rule: BETA.DIST(x,a,b,TRUE) takes a closed form for b = 1, and probably for a = 1:
- I_x(a,1) = exp(a*ln(x)). This matches all 27 b = 1 cells. `pow` does not: BETA.DIST(0.1,5,1,TRUE) is
  1.0000000000000016E-05, not 1E-05.
- I_x(1,b) = 1 - exp(b*ln(1-x)), or equally 1 - pow(1-x,b), with the cancellation either implies:
  22 of 24 a = 1 cells. BETA.DIST(0.1,1,0.0001,TRUE) is 1.0535996061822317E-05, 3.4E-12 relative
  above the true value. The two misses, BETA.DIST(0.9,1,2.5,TRUE) and BETA.DIST(0.9,1,5,TRUE), are
  1 ulp below; no log1p, expm1, pow or correctly rounded form matches them.

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
- **What did not close the residual:** Excel's own GAMMA values (from T-1042), with best 62 of
  136 points, and 360 combinations of fraction, prefactor form and operation order, with best
  16 of 107. Reproduce all of this with docs/research/t1759_beta_direct_branch.py.
Status: VERIFIED (b = 1 closed form); a = 1 PARTIAL (22/24, form not distinguished from pow); the general algorithm is UNVERIFIED
Tasks: T-1744, T-1759, T-0970, T-0969, T-1745

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

### The far-tail density is less accurate than exp(-t), consistent with t = (x/l)^k through EXP(k*LN(x/l)) (lead)
In the tail, Excel's density is off the true value by about one rounded logarithm's error times t.
Match it; do not correct it.
- `WEIBULL.DIST(100,1,1,FALSE)` = 3.7200759760206773E-44, while exp(-100) = 3.720075976020836E-44
  (4E-14 relative; m9-weibull-dist A64).
- `WEIBULL.DIST(1000,1,9,FALSE)` = 6.177532755504216E-50 (A88).
- `WEIBULL.DIST(5,3.7,1,FALSE)` = 9.363099887555361E-166 (A146).

Computing t = exp(k*ln(x/l)), not `powf`, reproduces this. A double-precision re-run covered the
230 in-domain cells (x, k, l > 0 with a literal TRUE/FALSE cumulative; 116 cdf, 114 density) of
m9-weibull-dist, t1048-weibull and t1049-weibulldist. Visibly-off and bit-exact counts are per form:

| form | visibly off | bit-exact |
|---|---|---|
| cdf `-expm1(-t)` | 0 | 94 |
| cdf `-expm1(-(x/l)^k)`, `(x/l)^k` via powf | 1 | 74 |
| cdf, statrs form `-expm1(-x^k*l^-k)` (the engine today) | 1 (m9 A159) | 76 |
| cdf `1-EXP(-t)` (loses the small-x cells) | 24 | |
| density `k/l*exp((k-1)*ln(x/l))*exp(-t)` | 4 (m9 A158, A168; t1048 A8, B8) | |
| density `k/l^k*exp((k-1)*ln x)*exp(-t)` | 2 (t1048 A8, B8) | |
| density `k/l*(x/l)^(k-1)*exp(-(x/l)^k)`, powf | 15 | |
| density, statrs form `k/l*(x/l)^(k-1)*exp(-x^k*l^-k)` (the engine today) | 17 | |

t1048 A8 is `WEIBULL(10,3,8,FALSE)` and B8 its WEIBULL.DIST twin, both 0.08310360884023987. The same t gives
ironcalc-calc WEIBULL_DIST.xlsx E2 exactly (`WEIBULL.DIST(0.8,1,9,TRUE)` = 0.08505277126996899).
The re-runs used libm `exp`/`log`, not Excel's EXP and LN kernels, so the bit-level form is still
open (T-0978).
Evidence: probes-numeric m9-weibull-dist A64, A88, A146, t1048-weibull, t1049-weibulldist (excel-graph);
ironcalc-calc WEIBULL_DIST.xlsx E2 (excel-cached, Microsoft Macintosh Excel).
Status: the values are VERIFIED (excel-graph); the `exp(k*ln(x/l))` form is UNVERIFIED (a fit lead,
gap triage 2026-10-21; T-2239, T-0978).
Tasks: T-2239 (15 digits), T-0978 (last digits)

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
- NEGBINOMDIST with p = 1 is #NUM! even for zero failures (`NEGBINOMDIST(0,3,1)`); successes = 0 is #NUM!. HYPGEOMDIST with sample_s > number_sample is 0, not #NUM! as documented (`HYPGEOMDIST(5,4,8,20)` = 0, `HYPGEOMDIST(1,0,8,20)` = 0); population_s > number_pop is #NUM! (`HYPGEOMDIST(1,4,25,20)`, `HYPGEOMDIST(1,4,8,5)`), and so is number_pop = 0 (`HYPGEOMDIST(1,4,8,0)`, where population_s and number_sample also exceed it); number_sample > number_pop alone is #NUM! for HYPGEOM.DIST (`t1049-hypgeomdist` A40, `(1,25,8,20)`).
- TTEST: tails 0/3 and type 0/4 are #NUM!, tails accepts a boolean (`TTEST(..,TRUE,2)`) and text numbers, tails/type truncate, an omitted tails or type is #NUM!; unequal counts with type 1 (paired) is #N/A; type 1 on a pair whose differences have zero variance is #DIV/0!; a one-element range is #DIV/0!; an empty range is #DIV/0!.
- ZTEST: sigma 0 or negative is #NUM!; a one-element range is #DIV/0! (with or without sigma); an empty range is #N/A; text x "5" and TRUE coerce, non-numeric text is #VALUE!; an omitted sigma (empty argument) is #NUM!, while an omitted x (`ZTEST(D1:D10,)`) returns a value (1.3161580317707342E-09; that it equals x = 0 was not probed); a range as x lifts and spills (see "Statistical function argument modes").
Evidence: probes-numeric cases `t1048-gammadist`, `-gammainv`, `-hypgeomdist`, `-loginv`, `-lognormdist`, `-negbinomdist`, `-normdist`, `-norminv`, `-normsdist`, `-normsinv`, `-poisson`, `-tdist`, `-tinv`, `-ttest`, `-weibull`, `-ztest`. Oracle: excel-graph, 2026-10-14.
Status: VERIFIED (excel-graph)
Tasks: T-1048, T-2035

## F-distribution df limit (T-1530)
Rule: either df above 1E10 (after truncation) makes every F function `#NUM!`: F.DIST (cumulative and pdf), F.DIST.RT,
F.INV, F.INV.RT, FDIST and FINV, in both df positions (probed at 10000000001, 1.5E10 and 1E11). Unlike Student-t, the pdf is cut too.

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

## Student-t quantile at a tiny probability (T-1531)
Rule: T.INV, T.INV.2T, TINV and CONFIDENCE.T never error for a tiny p or alpha: the quantile follows the power-law tail, so it is finite and huge for a small df (`T.INV(1E-300,1)` = -3.183098861837907E299, about 1/(pi p)) and about -79 for df = 99999.

Status: VERIFIED (excel-graph, 2026-10-14; probes-numeric case `t1531-t-tiny-p`, 105 cells, p from 1E-300 to 1E-10, df 1, 1.5, 5, 99999, 100000)

Implemented in: functions/statistical/t_dist.rs `t_tail_quantile`, `t_upper_quantile`

Tasks: T-1531, T-1542

- `T.INV(1E-300,1)` = `T.INV(1E-300,1.5)` = -3.183098861837907E299 (the df = 1.5 value is the df = 1 value: df truncates); `T.INV(1E-300,5)` = -1.568392559099332E60.
- `T.INV.2T(1E-300,1)` = 6.366197723675814E299 (exactly twice the one-sided magnitude); `CONFIDENCE.T(1E-300,1,2)` = 4.50158158078553E299.
- Ours: `statrs`' `StudentsT::inverse_cdf` is wrong for small df well before it fails (`T.INV(1E-10,1)` gives -8.9E7, Excel -3.18E9) and panics from p about 1E-200. For p < 1E-5 we use a closed-form tail asymptotic once x = df/(df+t^2) < 1E-8; otherwise `statrs` for p >= 1E-8 (it is bit-exact with Excel there) and bisection on its cdf below 1E-8 (`statrs`' inverse hangs at p = 1E-100 for df = 1000). Results agree with Excel to about 1E-13 relative for df <= 5, but only 7 of 105 cells are bit-exact, and for df >= 99999 at p <= 1E-200 we are wrong (-37.17 where Excel gives -79.06: for df = 99999 the `statrs` cdf saturates in the bisection branch, for df = 1E5 the normal series is used): T-1542.
- `T.INV.2T` with p in (1,2) is `#NUM!` for us; Excel returns a negative number (`T.INV.2T(1.5,1)` = -1, cells A121-A128), see the entry above.

## Inverse CDF functions at a tiny or near-1 probability (T-1543)
Rule: NORM.INV, NORM.S.INV, LOGNORM.INV, CHISQ.INV, CHISQ.INV.RT, GAMMA.INV and BETA.INV return a value (or `#NUM!`) for p from 1E-300 to 1-1E-16; none panics or hangs in Excel.

Status: VERIFIED (excel-graph, 2026-10-15; probes-numeric case `t1543-inverse-cdf-tiny-p`, 102 cells, p in 1E-300, 1E-200, 1E-100, 1E-20, 1-1E-16, 1-1E-10)

Implemented in: functions/statistical/beta.rs `beta_quantile`, functions/statistical/special.rs `bisect_unit_interval` (BETA.INV only; the rest need no guard)

Tasks: T-1543

- `1-1E-16` is a different double from 1 (`NORM.INV(1-1E-16,0,1)` = 8.209536151601386), but `1-1E-17` is 1 and gives `#NUM!`.
- Excel's CHISQ.INV and GAMMA.INV are exact in the lower tail (`CHISQ.INV(1E-100,1)` = 1.5707963267948966E-200, `GAMMA.INV(1E-300,1,1)` = 1E-300, `CHISQ.INV(1E-300,1)` = 0) and CHISQ.INV.RT is `#NUM!` for p below about 1E-15 with df = 1 but finite for df = 50 (`CHISQ.INV.RT(1E-300,50)` = 1592.68...). Ours give `#NUM!` for CHISQ.INV(p<=1E-200,1), GAMMA.INV(p<=1E-20,1,1) and CHISQ.INV.RT(p<=1E-20,df) (Excel: values), and wrong digits elsewhere in the tails (`CHISQ.INV(1E-300,50)` 0.0033 vs 2.04E-11; `CHISQ.INV(1-1E-16,50)` 179.80 vs 180.68; `GAMMA.INV(1E-300,100,1)` 1.0165 vs 0.0380): see the follow-up task.
- Excel's BETA.INV is erratic where the shape is extreme: `BETA.INV(1E-300,0.5,100)` = `BETA.INV(1E-200,0.5,100)` = 2.7050408442680296E-151 (the true quantile underflows to 0, ours 0); `BETA.INV(p,0.01,0.01)` is `#NUM!` for every p; `BETA.INV(1E-200,1000,1)` = 0.2064 and `BETA.INV(1E-20,1000,1)` = `#NUM!`, neither near the true quantile 0.9549925860214368 (ours). These are not implemented.
- Ours: `statrs`' `Beta::inverse_cdf` panics from p about 1E-200 and does not finish for `BETA.INV(1E-100,1000,1)` or for any shape of 1E8 or more, at any p. `BETA.INV` uses it for shapes up to 1E5 and p of 1E-15 or more (bit-exact with Excel in ironcalc-calc BETADIST_BETAINV.xlsx) and bisects on the cdf otherwise. A sweep of NORM.INV, NORM.S.INV, LOGNORM.INV, CHISQ.INV, CHISQ.INV.RT, GAMMA.INV and CONFIDENCE.NORM over the same p found no panic or slow case.

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
Rule (all from excel-graph sweeps `probes-numeric/t1049-*`; none implemented by this task):
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
  infinite), the df=2 value is 1 (F) and 0.5 (CHISQ), higher df give 0; `WEIBULL.DIST(0,0.5,3,FALSE)`
  is 0, not `#NUM!`, and so is the shape-1 density at 0, which is mathematically 1/lambda:
  `WEIBULL.DIST(0,1,3,FALSE)` = 0 (`weibulldist` A27; `WEIBULL(0,1,1,FALSE)` = 0 too, probes-numeric/t1048-weibull A16,
  gap triage 2026-10-19). The density at x = 0 is 0 for every probed shape. Implementing these edges is T-2035. `F.DIST.RT(0,...)` and `CHISQ.DIST.RT(0,...)` are 1.
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
Ours: GROWTH spills collapse to the first cell, the LINEST stats block is short or `#N/A`, many last-digit differences (T-1741, T-1742, T-1743).
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
Evidence: probes/regression-trend (excel-graph, 2026-10-16, T-1620).
Ours: differs for the empty arguments, array constants, zero-variance x and the text/error inputs. A single point with a constant term matches (T-1402: TREND is that value whatever new_x is, GROWTH its ln/exp round trip); a single point without one (`TREND(5,2,4,FALSE)`) is a fit through the origin, UNVERIFIED. The error-before-text and size-before-text order also applies to TREND, LINEST and LOGEST here, inferred from GROWTH/TREND cells and unprobed for them; a range that holds text and then an error cell gives that error, where probes/regression-trend says `#VALUE!` (T-1726). LINEST and LOGEST of one point are accepted by Excel (linest-modes K141, logest-modes K162); ours still rejects them (T-1741, T-1742).
Status: VERIFIED (web)
Tasks: T-1726 (overlaps T-1402)

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
PERCENTRANK* reads a typed boolean as a one-element array and typed text as a number (a single cell holding either is unprobed, UNVERIFIED, and unchanged). DEVSQ is one-pass (devsq.rs, T-1749).
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
probed, the others assumed alike).
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

## F.TEST / FTEST is not correctly rounded (T-1927)
Excel's F.TEST is not the correctly rounded value of `2*min(lower, upper)` tail: with the f
our two-pass variance gives, the exact p of `F.TEST({1,2,3,4},{5,6,7,2.4})` is 0.5023992107800743
but Excel returns 0.5023992107800741, and `F.TEST(C1:C6,B1:B8)` of probes/stat-argument-modes
(df 1,7) is 0.91959790471759887 exactly against Excel's 0.9195979047175995, while
`F.TEST(A1:A8,B1:B8)` (df 7,7) and `FTEST(B1:C5,B1:B5)` (df 9,4) are Excel's correctly rounded
values. An exact incomplete beta therefore fixes some cells and regresses others; matching all
needs Excel's own algorithm (see the F.DIST/BETA.DIST fit notes).
F.TEST is not built on Excel's own F.DIST either (probes/ftest-decomposition, excel-graph,
2026-10-22, T-1927): `2*MIN(F.DIST(f,..,TRUE),F.DIST.RT(f,..))` with f = `VAR.S(x)/VAR.S(y)` is
0.7947858780819459 and 0.5236816407506065 where F.TEST is 0.7947858780819458 and
0.5236816407506063 (A8/A7, B8/B7), and F.TEST is exactly symmetric in its arguments (A7 = E7,
C7 = F7). Over the 33 distinct F.TEST inputs on file (probes stat-argument-modes,
ftest-decomposition; probes-numeric m9-legacy-ftest, t1049-ftest) Excel is within -6..+4 ulps of
the exact two-tailed p, on both sides; neither the exact p (12 of 33 bit-exact) nor the A&S
26.6.4-26.6.8 finite series in doubles (11 of 33), with a two-pass, textbook or corrected
variance, or with f taken as larger over smaller, reproduces it. Open: T-2359.

## Paired functions: a size mismatch is `#N/A` before an error cell (T-1927)
COVARIANCE.S, COVARIANCE.P, COVAR and a paired T.TEST (type 1) of arguments with different cell
counts are `#N/A` even when a cell holds an error: `COVARIANCE.S(H1:H3,I1:I4)` with H2 `=1/0` is
`#N/A`, as is the argument order swapped; with equal sizes the error wins
(`COVARIANCE.S(H1:H3,I1:I3)` `#DIV/0!`). CORREL, PEARSON and RSQ already check sizes first
(`#N/A`). The tests that allow unequal sizes return the error: `T.TEST(H1:H3,I1:I4,2,2)` and
`F.TEST(H1:H3,I1:I4)` are `#DIV/0!`. Evidence: probes/ftest-decomposition G1:G13 (excel-graph,
2026-10-22).
Engine: `Model::paired_samples` (`PairedSamplesError::sizes_differ`) in
crates/rustsheet-engine/src/functions/statistical/paired.rs.
Status: implemented (T-1927)
