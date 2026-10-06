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

## GAMMA is exact at positive integers but not always correctly rounded
Rule: GAMMA(n) for a positive integer n is exactly (n-1)!: `GAMMA(5)` = 24, `GAMMA(6)` = 120,
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
Status: VERIFIED (observation); the formulation is UNVERIFIED (T-0968, oracle sweep T-0966)
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
Status: UNVERIFIED (fits the NORM.S.DIST cells; NORM.DIST/LOGNORM.DIST open, T-0975)
Tasks: T-0975

## HYPGEOM.DIST is not exact for small integer arguments
Rule: HYPGEOM.DIST misses the exact rational result by a few ulp even for tiny populations, so it is
not a direct ratio of exact binomial coefficients: `HYPGEOM.DIST(1,4,8,18,FALSE)` (arguments
truncated from 1.8, 4.2, 8.2, 18.34) = 0.31372549019607815, while 16/51 = 0.3137254901960784 (which
is what C(8,1)*C(10,3)/C(18,4) gives in doubles); `HYPGEOM.DIST(1,4,8,22,TRUE)` = 0.5349282296650714,
while 559/1045 = 0.5349282296650718. Neither a log-gamma nor a log-factorial `exp(ln C + ln C - ln C)`
form reproduces both cells (checked during T-0903), so the formulation is open.
Evidence: `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` Sheet1!F2, F3.
Status: VERIFIED (observation); formulation UNVERIFIED (T-0977, sweep T-0966)
Tasks: T-0977

## BETA.DIST is not exact where the exact value is a double
Rule: `BETA.DIST(2,8,10,TRUE,1,3)` (I_0.5(8,10) = 44923/65536, exactly representable as
0.6854705810546875) returns 0.6854705810546873; the density at the same point,
`BETA.DIST(2,8,10,FALSE,1,3)`, is exact (1.4837646484375). Match Excel's incomplete-beta last digits,
not the exact value.
Evidence: `ironcalc-calc/STATISTICAL/BETA_GAMMA.xlsx` BETA_DIST!G3, G4.
Status: VERIFIED (observation); algorithm UNVERIFIED (T-0970)
Tasks: T-0970
