# T-2464 w6-literature: source survey for Excel's incomplete beta and beta density

Worker w6-literature, 2026-10-23. Method: survey what Microsoft and outside assessors published about Excel 2010's
BETA.DIST rewrite, and what each open library does (branches, thresholds, series and fractions, prefactor, stopping
rule). Then test the structures that the earlier workers (w1-w5) had not already settled, using Excel's own EXP, LN,
GAMMALN and GAMMA. Coefficients from sources were used only where the hypothesis being tested is "Excel uses this
library's own approximant". Every such test was rejected.

Scores are bit-exact / within 4 ulp / within 15 digits, out of n. "e/o" are the even and odd row halves; nothing
was tuned except where a scan is stated. CR is the correctly rounded value.

## 1. Headline

1. **Microsoft has never named an algorithm.** The only first-party statements are these:
   - "because the BETADIST function was inaccurate, a new algorithm has been implemented to improve the accuracy of
     this function" (Excel 2010 "What's New").
   - `Workbook.AccuracyVersion = 1` brings back the pre-2010 algorithms.

   No whitepaper, blog post or support article that names TOMS 708, Cody, AS 63, Boost, NAG or R turned up. Mélard
   (2014), Keeling & Pavur (2011) and Knüsel (2011) assessed Excel 2010's distributions by accuracy only. The full
   texts could not be reached from this sandbox (DNS/proxy refusals for ulb.be, arxiv.org and uni-muenchen.de), so
   their wording is only known from abstracts and search snippets.
2. **The structure evidence points at Didonato & Morris's TOMS 708 (`bratio`), with Excel's own GAMMALN, LN and
   EXP in the pieces.** w3 and w4 had already shown this from the data. This survey adds:
   - NAG's s14cc is "derived from BRATIO", and Boost's `ibeta_imp` follows the same dispatch.
   - **The density's `ln B` switch is TOMS's 8, not the 10 used by R, NSWC or Apache** (H1, table 3.1). This is a
     discrete, held-out result: at 8 < max < 10, TOMS gets 102/306 against GL-form 3/306 and R `lbeta` 55/306.
3. **Rejected structures** (each beats or ties CR somewhere, but loses to the incumbent model):
   - R `lbeta` (all branches) and NSWC/Apache `logBeta` (cutoff 10) as Excel's ln B (H1).
   - TOMS `gamln`/`gamln1` rationals as Excel's ln Γ(v < 1) (H3): 2,954 against 4,466 on probe1. The density
     inherits Excel's own GAMMALN(v) at v = 0.0001 (-8 units off CR).
   - LN(GAMMA(v)), i.e. R's `lgammafn` for v ≤ 10, as ln Γ(v < 1) (H3): 14 against 134 of 224 at v = 0.25.
   - Boost's prefactor-scaled `ibeta_series` in the bpser branch (H5): 1,538 against w3's 4,442 of 5,183.
   - TOMS's and R's decimal constants as the source of Excel's a, b > 2 density offsets (H4).
4. **A partial lead for the a, b > 2 density: R ≤ 2.7.1 Loader `dbinom_raw` (lf = ln 2π + ln k + ln(n-k) - ln n),
   stored x87, Excel LN/EXP** (H2/H2c/H2d).
   - At (4, 4) this form is bit-exact on 148 of 300 cells with no tuning, against CR's 33. Its errors correlate with
     Excel's at r = 0.85.
   - It also correlates at r = 0.58-0.73 at (12, 20), (100, 120) and (7.3, 11.6).
   - It does not reproduce the +3 to +4 unit offset at (2.5, 3.5), where every Loader term is a table constant.
   - So it is not the structure as written. Per-block "Loader + one constant" fits beat a smooth model with one
     constant only at (4, 4), (100, 120) and (7.3, 11.6) reflected.
5. **mog** (`/home/user/fundamental-research-labs/mog`) is confirmed to have no Excel-specific hints:
   - BETA.DIST, BETA.INV and the F/T wrappers call `statrs = "0.17"` (`statrs::distribution::Beta`), which is NR's
     Lentz `betacf`;
   - `compute-functions/src/statistical/distributions/{beta.rs,support.rs}` only add argument checks, bisection
     inverses and `statrs::function::beta::ln_beta`.

## 2. Source survey (structure only)

| source | incomplete beta structure | density / ln B | relevance to Excel |
|---|---|---|---|
| **TOMS 708** (Didonato & Morris 1992, `bratio`; R `toms708.c`; Burkardt's f77 copy) | min(a,b) ≤ 1: swap if x > 0.5; fpser (b0 < eps·min(1,a0)), apser, bpser (b0 ≤ 1, or x0 < 0.1 with (x0 b0)^a0 ≤ 0.7, or a0 ≥ min(0.2, b0), or x0^a0 ≤ 0.9), 1 - bpser(b0, a0, y0) at x0 ≥ 0.29 (0.3 when both ≤ 1), else bup(n = 20) + bgrat (b0 > 15: bgrat directly). min > 1: swap if λ = a - (a+b)x < 0; b0 < 40: bpser if b0·x0 ≤ 0.7, else L140 = bup(b0 - int b0, ...) + bpser (x0 ≤ 0.7) or + bgrat; b0 ≥ 40: bfrac (λ-based) or basym (a0, b0 > 100 and λ small). Complement 0.5 - w + 0.5. eps = max(spmpar(1), 1e-15) in the Fortran original. | `brcomp`: ln x / ln y switch at 0.375 (alnrel); a0 ≥ 8 Stirling with `bcorr` and `rlog1`; `betaln`: a0 < 1 → gamln(a0) + (gamln(b0) - gamln(a0+b0)) if b0 < 8, else gamln + algdiv; L40/L60 product reductions for 1 < a0 < 8; Stirling with constant .918938533204673 for a0 ≥ 8 | **Adopted structure** (w1 0.375 switch, w3 dispatch and x ≤ 0.5 switch, w4 L140 and bfrac λ, H1 cutoff 8). Its rationals (gamln1, gam1) and constants are not Excel's (H3, H4). |
| **NAG s14cc / S14CCF** | "derived from BRATIO in DiDonato and Morris (1992)"; power series, the D&M continued fraction, Wise's expansion, asymptotics at a, b ≥ 15 | not stated | Same lineage. A commercial vendor with Excel add-ins. No evidence links it to Microsoft. |
| **Boost.Math** `ibeta_imp` | D&M dispatch, plus binomial_ccdf for integer a, b with b < 40; ibeta_series forward-summed with the prefix multiplied into the terms; ibeta_a_step (bup); beta_small_b_large_a_series (bgrat); ibeta_fraction2 (modified Lentz) | Lanczos power terms (`ibeta_power_terms`) | The prefix scaling is rejected (H5). A Lanczos prefix is ruled out by w3 (Excel's GAMMALN errors are inherited). |
| **R nmath** `pbeta` | TOMS 708 bratio since R 2.3 | `dbeta`: a ≤ 2 or b ≤ 2 → (a-1)ln x + (b-1)log1p(-x) - lbeta (R ≥ 2.9); else ln(a+b-1) + Loader `dbinom_raw(a-1, a+b-2, x, 1-x)`. R 2.0: f·dbinom_raw(...) in 4 shape cases. `lbeta`: p ≥ 10 Stirling + lgammacor; q ≥ 10 lgammafn(p) + corr + p - p ln(p+q) + (q-.5)log1p(-p/(p+q)); else ln(Γ(p)·(Γ(q)/Γ(p+q))) | The density split at 2 matches w2. Loader is a partial lead (H2). `lbeta` is rejected (H1). |
| **Loader (2000)**, "Fast and accurate computation of binomial probabilities" (as in R `dbinom.c`, `stirlerr.c`, `bd0.c`) | | stirlerr: table at half-integers ≤ 15, else lgamma(n+1) - (n+.5)ln n + n - ln√2π; series S0..S4 with cut-offs 15/35/80/500. bd0: Taylor in v = (x-np)/(x+np) when \|x-np\| < 0.1(x+np), else x ln(x/np) + np - x. lf: R ≤ 2.7.1 ln 2π + ln x + ln(n-x) - ln n; R ≥ 2.7.2 ln 2π + ln x + log1p(-x/n); R 2.0 exp(lc)/sqrt(2πx(n-x)/n) | H2: best fit at (4, 4); does not close (2.5, 3.5) |
| **NSWC DBETLN / Apache Commons `LogBeta`** | | a ≥ 10 Stirling; 2 < a < 10 product reductions (b > 1000 special); 1 ≤ a ≤ 2: b < 10 reductions, else logGamma + dlgdiv; a < 1: b ≥ 10 → logGamma + dlgdiv, else ln(Γ(a)Γ(b)/Γ(a+b)) | Rejected by the cutoff (H1) |
| **Cephes** `incbet` (also ALGLIB, old SciPy) | swap if x > a/(a+b); pseries if b·x ≤ 1 and x ≤ 0.95; incbcf or incbd chosen by x(a+b-2) - (a-1); prefactor via pow or exp(a ln x + b ln y + lgam(a+b) - lgam(a) - lgam(b)); MACHEP·3 stopping | lgam | Rejected by w2's tie response and w3's dispatch tests |
| **Numerical Recipes 3** `incbeta` | betacf (modified Lentz) when x < (a+1)/(a+b+2), else 1 - I_{1-x}(b, a); Gauss-Legendre for a, b > 3000; prefactor exp(gammln(a+b) - gammln(a) - gammln(b) + a ln x + b ln(1-x)) | | statrs/mog lineage; rejected (w2, w4) |
| **Gnumeric** `mathfunc.c` | pbeta by Ian Smith (pbeta_smalla for small a), not TOMS 708 | dbeta imported from R | No |
| **LibreOffice** `interpr3.cxx` | GetBetaDist: its own continued fraction (lcl_GetBetaHelperContFrac) with special cases | GetBetaDistPDF with log/pow cases | Rejected by w2 and w4 |
| **Microsoft-side libraries** (Infer.NET, ML.NET, Solver Foundation, ONNX) | No public incomplete-beta code was found that predates Excel 2010 or shares Excel's composition. Not pursued further. | | None |

## 3. Experiments

All scripts run from this directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`. `w1copy/` and
`w3copy/` are copies of w1-density's and w3-series' helpers, with paths patched so that caches are written here.
`w1copy/probe/` is git-ignored; recreate it with `cp -r ../w1-density/probe w1copy/`. Positive controls:
- `h1_lnb.py` and `h3_gamln.py` reproduce w1's M1 exactly: 525/1,052 and 4,466/6,400.
- `h5_boost_series.py` reproduces w3's model exactly: 4,442/5,093/5,131 of 5,183.

### 3.1 H1: which library's ln B in the density (min(a,b) < 1, no unit shape), `h1_lnb.py` → `h1_lnb.out`
Everything else is w1's M1. Rows are beta_exact pdf cells, binned by max(a,b).

| ln B variant | all (1,052) | max<8 (450) | max=8 (14) | 8<max<10 (306) | max=10 (14) | max>10 (268) | probe1 (6,400) |
|---|---|---|---|---|---|---|---|
| **TOMS betaln, algdiv above 8** (w1) | **525**/1044/1048 | 300 (e146 o154) | 9 | 102 (e53 o49) | 11 | 103 | **4,466**/6353/6384 |
| GL form at every max | 314/530/676 | 300 | 9 | 3 | 0 | 2 | 3,451 |
| TOMS formulas, switch at 10 | 426/785/900 | 300 | 9 | 3 | 11 | 103 | 4,466 |
| R lbeta (Excel GAMMA ratio; q ≥ 10 lgammacor, ln Γ(p) = LN(GAMMA)) | 241/809/934 | 70 | 0 | 55 | 4 | 112 | 1,715 |
| R lbeta with GAMMALN(p) | 253/827/946 | 70 | 0 | 55 | 7 | 121 | 1,715 |
| NSWC/Apache (ratio below 10, algdiv above) | 238/825/945 | 66 | 0 | 58 | 11 | 103 | 2,255 |
| exact ln B (control) | 321/856/954 | 93 | 0 | 117 | 4 | 107 | 1,977 |

Findings:
- The switch is at 8 (inclusive to the GL form, as w1 found), not 10.
- The R and NSWC gamma-ratio forms lose badly below 8.
- Above 8 no variant is clean. The exact ln B beats the TOMS algdiv model at 8 < max < 10 (117 against 102), and
  R's lgammacor form edges it above 10 (121 against 103).
- So Excel's algdiv-range ln B is TOMS-shaped but its internals are still open (w1's ±2-unit noise).

### 3.2 H3: ln Γ(v < 1) inside ln B, `h3_gamln.py` → `h3_gamln.out`

| ln Γ(lo) | beta_exact pdf all (1,052) | lo = 0.0001 (45) | lo = 0.5 (587) | lo = 0.75 (196) | probe1 (6,400) |
|---|---|---|---|---|---|
| **Excel GAMMALN** (w1) | 525/1044/1048 | **23** | **266** | 102 | **4,466** |
| GAMMALN(v+1) - LN(v) | **529**/1043/1048 | 23 | 266 | **106** | 4,466 |
| correctly rounded | 503/1005/1026 | 0 | 266 | 103 | 4,466 |
| TOMS gamln (double or stored x87) | 471/1042/1048 | 23 | 208 | 106 | 2,954 |
| TOMS gamln, extended | 446/1004/1026 | 0 | 208 | 104 | 2,954 |
| LN(GAMMA(v)) (R lgammafn) | 382/1004/1026 | 0 | 266 | 102 | 3,471 |

- GAMMALN(0.5) and GAMMALN(0.25) equal the CR values, so only lo = 0.0001, 0.5 vs TOMS, and 0.75 discriminate.
- TOMS's gamln1 puts ln Γ(0.5) one ulp high (+0.63 units) and loses 58 of 266 cells: rejected.
- At 0.0001 the density needs Excel's GAMMALN value, which is 8 units below CR (23 against 0).
- GL1 and GL tie within 4 cells at 0.75 (n = 196), which is not significant.

### 3.3 H2: Loader dbinom_raw for the a, b > 2 density, `h2_loader.py` → `h2_loader.out`
Variants: 3 lf forms (R ≥ 2.7.2 log1p, R ≤ 2.7.1 four-log, R 2.0 non-log), × D stored x87 / E extended / S double,
× lgamma = GAMMALN or LN(GAMMA) in stirlerr, × log1p = CR or LN(1+t). Data: the first 300 cells of each block of
w4's `data.tsv` pdf (w2 runs and sweeps, old beta_exact a, b > 2) plus probe1 (2.5, 3.5) and (12, 20); n = 4,800.
CR gets 367/3949/4450.

| variant | all (4,800) | (4,4) xruns-1:5 | (2.5,3.5) xruns-1:0 | (7.3,11.6) xruns-1:2 | (100,120) xruns-1:7 |
|---|---|---|---|---|---|
| CR | 367/3949/4450 | 33 | 8 | 11 | 11 |
| D LF1 (R ≥ 2.7.2) | 268-313 | 78 | 2 | 0-2 | 4 |
| **D LF0 (R ≤ 2.7.1)** | 330-354 | **148 (e55 o93)** | 0 | 0-2 | 20 |
| D FX (R 2.0) | 356-359 | | 44 | | 25 |
| E FX (extended) | **395**/3829/4404 | | | 95 | |

Every variant is at or below CR overall. Residual structure (`h2b_resid.py` → `h2b_resid.out`; Excel - model, mean and
sd in units of 2^-53, r = correlation of (model - CR) with (Excel - CR)):
- At (4, 4), D-LF0 gives sd 1.69 against CR's 2.93, r = +0.85.
- At (100, 120), D-* gives sd 9.0 against 13.2, r = +0.73.
- At (12, 20), D-* gives r = +0.58 to +0.64.
- At (7.3, 11.6) reflected, D gives r = +0.68, but the mean is -11.5 on both sides of the mean.
- At (2.5, 3.5) Excel is +2.7 to +3.4 above CR with sd 0.5 to 1.6, near the rounding floor. Every Loader variant
  leaves that offset: +3.1 to +4.9. Every Loader term there is a table constant, so the offset is not a stirlerr
  effect.

`h2c_offset.py` → `h2c_offset.out`: one free constant per block on E. It is tuned on the even rows and scored on
the odd rows. The control is the exact E with its own best constant:

| block | control hold-out | D-LF1 | D-LF0 | E-LF1 |
|---|---|---|---|---|
| (2.5,3.5) xruns-1:1 | 67/100 | **81** | 80 | 74 |
| (2.5,3.5) xruns-2:5 | 71 | **88** | 86 | 70 |
| (2.5,3.5) xruns-1:0 | **47** | 37 | 26 | 37 |
| (2.5,3.5) xruns-2:3 | **60** | 48 | 46 | 55 |
| (4,4) xruns-1:5 | 16 | 20 | **36** (c = 0: 100/200) | 18 |
| (100,120) xruns-1:7 | 5 | 21 | **22** | 4 |
| (7.3,11.6) xruns-2:2 | 9 | **20** | **20** | 11 |
| (7.3,11.6) xruns-1:2 | 33 | 14 | 14 | **35** |
| (12,20) probe1 | 2 | **10** | 9 | 5 |

`h2d_lf0.py` → `h2d_lf0.out`: switching ln 2π between the constant and a runtime LN changes nothing. Excel's
LN(2π) is 1 ulp below M_LN_2PI. Neither does an extended lc accumulator: (4, 4) stays at 148/300.

Reading: Loader's saddle-point form with stored ops carries part of Excel's noise at (4, 4), (12, 20), (100, 120)
and (7.3, 11.6). That agrees with w4's finding of the bd0 tie response. The per-shape constant parts, though, are
not R's in any version, and at (2.5, 3.5) the gap is a pure constant.

### 3.4 H4: decimal-constant fingerprints, `h4_const.py` → `h4_const.out`
The density offset predicted if Excel used a library's ln B approximant in exact arithmetic is
-(lnB_lib - lnB)/2^-53:

| (a, b) | TOMS betaln | R lbeta | Excel (Excel - CR) |
|---|---|---|---|
| (2.5, 3.5) | -1.25 | 0.00 | +2.7..+3.4 |
| (4, 4) | 0.00 | 0.00 | -2.40 |
| (7.3, 11.6) | -0.59 | -0.27 | -4.2 direct, +5.4 reflected |
| (12, 20) | -2.36 | -0.41 | -1.0 |
| (100, 120) | -2.35 | -0.33 | -1.1 |

- No match.
- TOMS's 15-digit constants .918938533204673 and .418938533204673 put every Stirling-branch gamln at +2.35 units.
  Excel's density offsets do not show that.

### 3.5 H5: Boost's prefix-scaled series in bpser, `h5_boost_series.py` → `h5_boost_series.out`
The rows are TOMS bratio's `w = bpser(a0, b0, x0)` cells with no swap, from beta_exact plus the w2 probes,
n = 5,183. "Hold-out" is w3's md5 odd half.

| model | all | min ≤ 1 (4,934) | a,b > 1, b·x ≤ 0.7 (249) | beta_exact direct (318) | w2 direct (3,825) | w2 reflected (1,040) | hold-out min ≤ 1 (2,481) |
|---|---|---|---|---|---|---|---|
| CR | 805/4637/4748 | 786/4515/4552 | 19/122/196 | 40/176/241 | 675/3521/3567 | 90/940/940 | 404/2278/2297 |
| **w3 (EXP(z)·s, s from 1/a)** | **4,442**/5093/5131 | **4,318**/4885/4910 | **124**/208/221 | **109**/236/274 | **3,311**/3817/3817 | **1,022**/1040/1040 | **2,173**/2463/2475 |
| Boost, eps stop | 1,538/4323/4124 | 1,467/4124/3903 | 71/199/221 | 66/220/271 | 1,308/3233/2983 | 164/870/870 | 726/2067/1963 |
| Boost, unchanged stop | 1,540/4323/4124 | 1,469 | 71 | 68 | 1,308 | 164 | 727 |

Boost's term scaling is rejected. Excel sums the unscaled series and multiplies by the prefix once, as w3 found.

## 4. Negatives, in one list
- R lbeta, in both of its gamma-ratio orders, as ln B at min < 1 (241-253/1,052).
- NSWC/Apache logBeta (238/1,052).
- The cutoff 10 with TOMS formulas (426/1,052, 3/306 at 8 < max < 10).
- TOMS gamln/gamln1 as ln Γ(v < 1): double, stored and extended (446-471/1,052; probe1 2,954/6,400).
- R lgammafn = LN(GAMMA(v)) for v < 1 (382/1,052).
- Loader dbinom_raw (R ≥ 2.7.2, R ≤ 2.7.1, R 2.0 forms; D, E, S arithmetic; GL or LN(GAMMA) stirlerr; CR or LN
  log1p; ln 2π constant or LN): at most 395/4,800 against CR's 367; none bit-exact on any block.
- Loader plus one constant per block: at or below the smooth-plus-constant control on 4 of 9 blocks.
- TOMS and R decimal constants as the source of the a, b > 2 density offsets.
- Boost prefix-scaled ibeta_series (1,538/5,183 against 4,442).

## 5. Next hypotheses (most promising first)
1. **The (4, 4) Loader hit.** D-LF0 is bit-exact on 148/300 cells, and its even and odd x classes split 55/93.
   That split follows the fl(1-x) tie class, so the y handling is half right.
   - Next: vary how nq = n·y is formed: n·fl(1-x), n - n·x, or fl(n - fl(n·x)).
   - Also vary bd0's series order: TOMS-style or R's, and the v² update.
   - (4, 4) uses only table constants, so it is the cleanest place to pin Loader's arithmetic before the
     non-half-integer stirlerr.
2. **The (2.5, 3.5) density constant (+3 to +4 units).** It is pure, with sd at the rounding floor, so it reads one
   wrong value.
   - Loader's (2.5, 3.5) constants are stirlerr(4), stirlerr(1.5), stirlerr(2.5), ln 5 and ln 2π. Try each as
     computed by Excel itself: GAMMALN-based stirlerr at half-integers, LN(5) at 1 ulp off, a Stirling-series
     stirlerr instead of the table.
   - Find which single value, moved by k ulps, lands the offset. Then check the same rule at (4, 4) (offset -2.4).
3. **The algdiv range of ln B (max ≥ 8, min < 1).** It is TOMS-shaped. R's lgammacor form and the exact value tie it
   or beat it in parts (H1), so Excel's algdiv probably uses its own alnrel/log1p, the same faithful-not-CR log1p
   w1 found below 0.375.
   - Try algdiv with an alnrel built on Excel's log1p, once that log1p is peeled.
4. **Literature still unread** (blocked from this sandbox):
   - Mélard 2014, Comput. Stat. 29:1095;
   - Knüsel 2011 (Excel 2010 report);
   - Keeling & Pavur 2011, Am. Stat. 65;
   - the Brown & Levy 1994 TOMS 708 certification, which lists the post-publication fixes. Fixes that R adopted
     and Excel did not, or the reverse, could explain the bgrat/bfrac residuals.

   A coordinator with web access should check them for any sentence on Excel 2010's method.

## 6. Sources
- Microsoft, "What's New: Changes made to Excel functions":
  https://support.microsoft.com/en-us/excel/what-s-new-changes-made-to-excel-functions
- Microsoft, Workbook.AccuracyVersion (Office 2010):
  https://learn.microsoft.com/en-us/previous-versions/office/developer/office-2010/ff591211(v=office.14)
- Microsoft, WorksheetFunction.BetaDist: https://learn.microsoft.com/office/vba/api/excel.worksheetfunction.betadist
- Mélard, G. (2014), "On the accuracy of statistical procedures in Microsoft Excel 2010", Computational Statistics
  29(5):1095-1128, doi:10.1007/s00180-014-0482-5. Copy: https://guy.melard.web.ulb.be/rech/compsstat37.pdf (not
  reachable from the sandbox); also https://www.researchgate.net/publication/266025083
- Yalta, A. T. (2008), "The accuracy of statistical distributions in Microsoft Excel 2007", CSDA 52(10).
  McCullough & Heiser (2008), CSDA 52(10). Listing: https://ideas.repec.org/a/eee/csdana/v52y2008i10p4602-4606.html
- "Assessing Excel VBA Suitability for Monte Carlo Simulation", https://arxiv.org/pdf/1503.08376. It cites Keeling
  & Pavur and Knüsel on Excel 2010 (snippet only).
- TOMS 708: R `toms708.c` https://raw.githubusercontent.com/wch/r-source/trunk/src/nmath/toms708.c (w2's copy in
  `../w2-forensics/ext/`); https://people.sc.fsu.edu/~jburkardt/f77_src/toms708/toms708.html
- NAG s14ccc: https://support.nag.com/numeric/cl/nagdoc_cl25/html/s/s14ccc.html ("derived from BRATIO")
- R nmath at trunk and tags R-2-0-0, R-2-9-0, R-3-6-3 (`lbeta.c`, `dbeta.c`, `dbinom.c`, `stirlerr.c`, `bd0.c`,
  `lgammacor.c`, `gamma.c`, `lgamma.c`, `pbeta.c`): https://raw.githubusercontent.com/wch/r-source/<tag>/src/nmath/<file>
- Boost.Math `beta.hpp`: https://raw.githubusercontent.com/boostorg/math/develop/include/boost/math/special_functions/beta.hpp
- Cephes `incbet.c`: https://raw.githubusercontent.com/jeremybarnes/cephes/master/cprob/incbet.c
- Apache Commons Numbers `LogBeta.java` (NSWC DBETLN):
  https://raw.githubusercontent.com/apache/commons-numbers/master/commons-numbers-gamma/src/main/java/org/apache/commons/numbers/gamma/LogBeta.java
- Gnumeric `mathfunc.c`: https://raw.githubusercontent.com/GNOME/gnumeric/master/src/mathfunc.c
- LibreOffice `interpr3.cxx`: https://raw.githubusercontent.com/LibreOffice/core/master/sc/source/core/tool/interpr3.cxx
- Loader, C. (2000), "Fast and accurate computation of binomial probabilities" (cited from R's `dbinom.c` header;
  the paper itself was not fetched).
- mog: local clone `/home/user/fundamental-research-labs/mog`, `compute/core/crates/compute-functions/Cargo.toml`
  (`statrs = "0.17"`).

Fetched sources live in `ext/` (git-ignored). The curl commands are the URLs above.

## 7. Files
`h1_lnb.py`, `h2_loader.py`, `h2b_resid.py`, `h2c_offset.py`, `h2d_lf0.py`, `h3_gamln.py`, `h4_const.py`,
`h5_boost_series.py`, each with a matching `.out`. `w1copy/` holds w1's lib, forms, toms, toms_betaln, gen1 and
model_final. `w3copy/` holds w3's lib, model, series and dispatch. `.gitignore` excludes ext/, caches and
w1copy/probe/.
