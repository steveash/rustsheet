# T-2464 w2-forensics: error forensics and new probes for Excel's incomplete beta

Worker: w2-forensics. Method: new oracle probes (consecutive-double runs, shape sweeps, boundary straddles), then
regression of Excel's error on the rounding errors of candidate intermediates, tie-response tests, and exact
simulation of the structures those point at. All numbers are reproducible with the scripts listed at the end. Saved
outputs: `*.out` next to the scripts.

Units: "rel" = (Excel - exact)/exact in units of 2^-53. Scores are bit-exact / within 4 ulp / within 15 digits.

## 1. Probes added (excel-graph, filled 2026-10-23, 41,554 cells)

| case | cells | what |
|---|---|---|
| `t2464-w2f-xruns-1` | 16,384 | cdf and pdf on 8 runs of 1,024 consecutive doubles in x, direct branch: (a, b, x0) = (2.5, 3.5, 0.2113571), (2.5, 3.5, 0.3313571), (7.3, 11.6, 0.2731), (0.5, 6.25, 0.0431), (30.7, 1.7, 0.8117), (4, 4, 0.2317), (1.5, 0.5, 0.3511), (100, 120, 0.4013) |
| `t2464-w2f-xruns-2` | 16,384 | same, 3 reflected runs ((2.5, 3.5, 0.6137), (0.5, 6.25, 0.3113), (7.3, 11.6, 0.6131)) and 5 runs centred (512 below, 512 above) on b*x = 1 (2.5, 3.5), x = 0.95 (40, 1.03), the mean (2.5, 3.5), the mean (0.5, 0.7), and x = 0.5 (0.5, 0.7) |
| `t2464-w2f-absweep` | 8,192 | cdf and pdf at fixed x with 512 consecutive doubles in a (blocks 0-3) or b (blocks 4-7) around (2.5, 3.5, x 0.2113571), (7.3, 11.6, 0.2731), (0.5, 6.25, 0.0431), (30.7, 1.7, 0.8117) |
| `t2464-w2f-unit-shape` | 594 | cdf at a = 1 +- k ulp and b = 1 +- k ulp (k = -16..16), other shape 0.5, 2.5, 5, x 0.1, 0.5, 0.9 (not yet analysed) |

Arguments are exact doubles written as `(H*2^e1+L*2^e2)`; `make_probes.py` checks that the fit lab reader evaluates
every one to the intended double. `../extract.py t2464-w2f w2-forensics/cells_w2f.tsv` (limited to these cases)
writes `cells_w2f.tsv`; `exact_w2f.py` writes `exact_w2f.tsv` (mpmath, 40 digits).

What the blocks show at a glance (`summary.out`):
- Reflected cdf blocks are near correctly rounded: (2.5, 3.5) 70% bit-exact, (7.3, 11.6) 76%, (0.5, 6.25) 45%,
  all within 1 ulp; sd 0.5 units.
- Direct cdf is noisy, and the noise grows with the shapes: sd 1.6-2.4 at (2.5, 3.5), 4.2 at (4, 4), 7.4 at
  (7.3, 11.6), 15 at (100, 120). (0.5, 6.25) direct carries a constant -7.2 offset with sd 1.2.
- The pdf of every block has a per-(a, b) offset with small x-noise: +3.0 to +3.4 at (2.5, 3.5) on both sides of
  the mean, -5.9 at (0.5, 6.25), +17.4 at (40, 1.03), 0 at (1.5, 0.5) and (0.5, 0.7). The offset of (7.3, 11.6) is
  -4.1 on the direct side and +5.1 on the reflected side, so it is not only a Beta-function error.

## 2. Forensics: which intermediates Excel stores (regressions on runs and sweeps)

### 2.1 Direct cdf, a, b > 1: a NEGATIVE response to the rounding of 1 - x (new, decisive against all candidates)
For x in [0.25, 0.5), fl(1 - x) loses x's last bit. Every odd mantissa is a tie, and the ties alternate rounding up
and down with bit 1. Excel's direct cdf error splits cleanly on these three classes: for (7.3, 11.6) it is +4.4
(exact 1 - x), -1.6 (1 - x rounded up) and +10.1 (rounded down). The tie response is
c = d(rel err)/d(rel error of fl(1-x)), measured from the two tie classes (`tie_response.py`, `tie_basis.py`):

| block | a | b | x | Excel c | Cephes incbet | best combination | LibreOffice | NR betai | TOMS bratio (R) |
|---|---|---|---|---|---|---|---|---|---|
| xruns-1:0 | 2.5 | 3.5 | 0.211 | **-3.72 (0.17)** | +0.14 | +3.55 | +3.61 | +4.15 | +1.24 |
| xruns-1:1 | 2.5 | 3.5 | 0.331 | **-1.36 (0.09)** | +3.12 | +3.13 | +3.49 | +4.14 | +1.04 |
| xruns-2:3 | 2.5 | 3.5 | 0.286 | **-1.91 (0.11)** | +2.92 | +4.17 | +3.43 | +5.09 | +1.98 |
| xruns-1:2 | 7.3 | 11.6 | 0.273 | **-8.51 (0.39)** | +11.62 | +11.51 | +11.03 | +11.57 | +7.95 |
| xruns-1:5 | 4 | 4 | 0.232 | **-7.17 (0.27)** | -0.93 | +4.34 | +4.28 | +4.29 | +1.06 |
| xruns-1:7 | 100 | 120 | 0.401 | **-10.58 (0.73)** | +123.3 | +127.7 | +119.1 | +120.5 | -0.55 |
| xruns-1:3 | 0.5 | 6.25 | 0.043 | -0.20 (0.14) | +0.11 | +4.99 | -0.13 | +6.73 | +0.06 |
| xruns-1:6 | 1.5 | 0.5 | 0.351 | +0.15 (0.09) | +0.01 | +0.95 | -0.04 | +0.62 | -0.03 |
| xruns-2:6 | 0.5 | 0.7 | 0.417 | +0.02 (0.11) | -0.00 | -0.69 | -0.47 | +0.81 | -0.00 |

- Positive control: the same estimator applied to each candidate's own output recovers that candidate's known
  structure. Prefactors using fl(1-x)^b give c of about +b (Cephes +123 at b = 120), and power-series paths that
  never use 1 - x give 0.
- Every published candidate has c >= -1 where Excel is at -1.4 to -10.6. So for a, b > 1, Excel's direct cdf uses
  y = fl(1 - x) in a place where a larger y gives a SMALLER result, and in no place with the +b weight of an
  x^a y^b prefactor. For min(a, b) < 1 the response is 0, which means a path that does not use 1 - x (power series)
  and argues for a split at min(a, b) = 1, not at b*x = 1.
- The magnitudes are not explained by simple mixtures of b, -(1-x) pdf/I (all of x taken as 1 - y) and
  -a(1-x)/x (`tie_basis.out`). Excel's c is about -(1-x) pdf/I plus +2 to +4 for the small blocks, but -10.6
  against -36.8 at (100, 120).
- The direct cdf is not a 1 - w1 complement in doubles: Excel's values are not quantized at 2^-53 (`grid_test.py`:
  the fractions of multiples of 2^-53, 2^-54 and 2^-55 match the exact values' 0.25/0.50/1.00 in every block).
- Regressors (`runs_regress2.out`, a, b > 1 direct cdf): the coefficient on Excel's LN(x) rounding is +0.2 to +4.9,
  not a (2.5 to 100). The fl(a*LN x) and fl(b*LN(1-x)) product roundings are under 0.8. So x^a is not formed from a
  stored LN(x) times a in this region.

### 2.2 Direct cdf, min(a, b) < 1 (0.5, 6.25): stored a*LN(x), and Excel's GAMMALN for b and a + b
- x run (xruns-1:3): the LN(x) error coefficient is +0.51 (0.03), = a.
- a sweep (absweep:2): the rounding of fl(a*LN x) has coefficient +1.02 (0.19), so the product is stored.
- b sweep (absweep:6): Excel GAMMALN(b) error -0.99 (0.02), GAMMALN(a+b) error +0.99 (0.02).
- a sweep: the error of fl(a+b) has coefficient +1.83 (0.03), = psi(a+b) = 1.83, so GAMMALN is evaluated at the
  stored sum. But **Excel GAMMALN(a)'s own error is NOT inherited** at a about 0.5: +0.46 (0.20) cdf, -0.16 (0.18)
  pdf, against -1. So ln Gamma(a) for a < 1 is computed some other way (TOMS gamln for a <= 0.8 is gamln1(a) - ln a,
  a candidate). This refines the statistics.md "Student t prefactor" form, which only varied b.

### 2.3 Density: two regimes
- min(a, b) < 1, or a or b <= 2: the power terms come from Excel's LN.
  - The LN(x) error coefficient is -0.52 (0.02) at a = 0.5 and +0.49 (0.03) at a = 1.5, = a - 1. The pdf takes
    (a-1)*LN(x).
  - (0.5, 6.25) b sweep: GAMMALN(b) -1.00 (0.01), GAMMALN(a+b) +0.99 (0.01), the same B as the cdf.
- a, b > 2 (2.5, 3.5), (7.3, 11.6), (100, 120): no dependence on LN(x) rounding (coefficient -0.04 (0.06) at
  (2.5, 3.5), +0.9 (1.4) where a - 1 = 99) or on the product roundings. The x-noise is small (sd 0.56 at
  (2.5, 3.5)), with a per-(a, b) offset. That is the signature of an accurate power evaluation (extended precision,
  or a saddle-point/bd0 form) times a constant.
- R's `dbeta` has exactly this split: a <= 2 or b <= 2 takes the log form, a, b > 2 takes Loader's
  `log(a+b-1) + dbinom_raw(a-1, a+b-2, x, 1-x)`.
  - R 3.6 dbeta (C, SSE) is bit-exact on 962 and 973 of the two (0.5, 0.7) blocks. The correctly rounded density
    gets 565 and 646.
  - At (4, 4), the Loader branch, R gets 273 (SSE) and 381 (x87 build) of 1,024 against 140 for correctly rounded,
    and the error correlation is r = 0.73 to 0.79.
  - At (2.5, 3.5) and (7.3, 11.6) it does not match (r 0.1 to 0.6, few exact).
  - So the a, b > 2 branch is Loader-like at best (`pdf_cands.out`). The x87 build is better at (4, 4) and much
    worse at (0.5, 0.7).

### 2.4 The density log form, bit-exact at (0.5, 0.7)
**pdf = EXP(((a-1)*LN(x) + (b-1)*LN(fl(1-x))) - lnB)**, with:
- every multiply and add stored x87 (fl53(fl64));
- Excel's own LN and EXP;
- lnB = LN(GAMMA(p)*(GAMMA(q)/GAMMA(p+q))) with p = min and q = max, using Excel's GAMMA. That is R's lbeta form.
  GL(b) + (GL(a) - GL(a+b)) and GL(a) + GL(b) - GL(a+b) give the same lnB double here.

It is **bit-exact on 2,048 of 2,048** cells in xruns-2:6 and xruns-2:7. Those runs are centred on the mean
(0.41667) and on x = 0.5, so both straddle the mean switch. With log1p(-x) in place of LN(fl(1-x)) the score drops
to 962 + 973, and with extended (unstored) sums to 921 + 937.
- Hold-out (`holdout_small.out`), the same rule on pdf cells not used to find it:
  - max(a, b) < 1 elsewhere (22 cells, a = 0.0001 or (0.5, 0.5), (0.5, 0.25), (0.5, 0.75)): 4/22 bit-exact with the
    GAMMA-ratio lnB, 11/22 with GL b+(a-s);
  - min < 1 and max <= 2 (1,056 cells): 207 bit-exact with the GAMMA ratio, 827 with GL b+(a-s), 1,056/1,056 within
    4 ulp and 15 digits.

  So the x-dependent assembly holds near (0.5, 0.7) and the lnB choice is shape-dependent and still open.
- Per-block lnB scan (`lnb_scan.out`: one free lnB double within +-40 ulp of exact, everything else fixed):
  - (0.5, 0.7): 1,024 + 1,024;
  - (1.5, 0.5): 928/1,024 with log1p(-x), all misses +-1 ulp;
  - (0.5, 6.25): 556 and 465 (the best lnB sits 11 to 14 ulp above exact, which is Excel's GL form, -5.6 units);
  - (30.7, 1.7): 389;
  - (40, 1.03): 250;
  - every a, b > 2 block: at most 294.

  The log form is therefore not the a, b > 2 structure, and is not complete for (0.5, 6.25) either. That block also
  has eGa ~ 0, see 2.2.

### 2.5 Density a, b > 2 and large a: shape-sum sensitivity
- (30.7, b about 1.7) b sweep, pdf: the error of fl(a+b) explains sd 30 -> 5.5, coefficient +1.60 (0.01), where
  psi(a+b) = 3.46. The cdf in the same sweep has coefficient 0.01 (sd 2.1). So the pdf uses the stored a + b at that
  shape with weight 1.6, and the cdf does not use it.
- (7.3, 11.6) and (2.5, 3.5) sweeps: the a+b coefficients are 0.1 to 0.6. Excel GAMMALN errors are not inherited
  (eGa ~ 0 for the pdf, -0.83 for the cdf at a = 7.3).
- A TOMS brcomp-style lambda = a - (a+b)x (or (a+b)y - b) prefactor was tested by regression (`lambda_test.py`).
  The best single fit is absweep:7 pdf, r = 0.97 with the extended-product lambda, but its slope is 2.16 where 1 was
  expected, and other blocks give slopes from -6.7 to +3. Not confirmed.

## 3. Branch map (`boundaries.out`, runs centred on each candidate boundary)
- **x = 0.5 for min(a, b) < 1 is a real switch.** At (0.5, 0.7), the cdf just below 0.5 (but above the mean) has
  mean error -5.9, 0% exact; just above 0.5 it has +0.35, 56% exact. The 64 cells nearest on each side give -5.0
  and +1.1. The pdf is the same on both sides, so the switch belongs to the cdf kernel. This is TOMS bratio's
  min(a, b) <= 1 rule (swap when x > 0.5), not the mean rule.
- The mean also changes the cdf character: at (0.5, 0.7) the error goes from +0.65 to +2.35 and exact from 36% to
  16%; at (2.5, 3.5) from +0.60 to +1.46 and 42% to 23%. So both rules act at a < 1. Statistics.md already has the
  mean switch for T.DIST.
- b*x = 1 (2.5, 3.5): no change in mean, sd or exact rate across the boundary (+0.74/1.98/35% against
  +0.69/1.92/35%). Cephes' power-series rule is not visible.
- x = 0.95 (40, 1.03): no change (+0.24/2.36 against +0.33/2.29).

## 4. Candidate scores on the new probe cdf data (`cand_runs.out`)
Direct blocks (xruns-1:0-7, xruns-2:3, :4; n = 10,240), exact / within 4 ulp:

| candidate | direct (n 10,240) | reflected (n 3,072) | straddles (n 3,072) |
|---|---|---|---|
| correctly rounded | 1,844 / 8,429 | 1,951 / 3,072 | 890 / 2,972 |
| R toms708 bratio (y = fl(1-x)) | 1,083 / 7,550 | **2,220** / 3,072 | 565 / 3,022 |
| Cephes incbet, exp(a ln x), CR Gamma | 701 / 5,661 | 1,802 / 3,072 | 529 / 2,981 |
| T-1744 best combination | 637 / 4,937 | 1,461 / 3,072 | 533 / 2,808 |
| LibreOffice + Excel closed forms | 827 / 5,588 | 1,863 / 3,072 | 410 / 2,641 |
| NR betai | 467 / 3,503 | 573 / 3,072 | 199 / 2,641 |

No candidate beats correctly rounded on the direct branch. TOMS bratio is the best on the reflected runs (2,220
against 1,951).

Density (`pdf_cands.out`, all pdf cells incl. beta_exact, n = 22,515), exact / within 4 ulp / within 15 digits:

| candidate | n 22,515 |
|---|---|
| correctly rounded | 3,102 / 17,155 / 19,755 |
| R 3.6 dbeta (SSE) | 3,739 / 15,240 / 19,044 |
| R 3.6 dbeta (x87) | 1,579 / 14,214 / 17,678 |
| TOMS brcomp/(x y) SSE | 2,104 / 13,053 / 18,748 |

Log-form density (2.4) on min(a, b) <= 2 cells (n = 11,251), global best (GL a+(b-s), log1p, stored): 3,883
bit-exact (`pdf_logform2.py`).

## 5. Negatives (recorded)
- The tie response rejects, for the a, b > 1 direct cdf: Cephes incbet (all variants in the fit lab), the T-1744
  best combination, LibreOffice GetBetaDist, NR betai, and R's toms708 bratio with y = fl(1-x) (SSE build). Each
  has c of the wrong sign or about 0 where Excel has -1.4 to -10.6.
- 1 - w1 complement arithmetic in doubles for the direct cdf: rejected (no 2^-53 quantization).
- A pure lnB offset times the exact density for a, b > 2: rejected (offsets differ by side of the mean at
  (7.3, 11.6): -4.1 and +5.1).
- Excel GAMMALN-form lnB (any grouping) or the Excel GAMMA ratio as the a, b > 2 density offset: predicted -2.4 or
  +0.35 at (2.5, 3.5), observed +3.0 to +3.4. TOMS betaln -2.4. Rejected.
- The density log form with any of 7 lnB forms x 2 ln(1-x) forms x 3 orders: at most 294/1,024 on a, b > 2 blocks.
- TOMS brcomp/(x*y) as the density: below correctly rounded overall.

## 6. Most promising next steps
1. The a, b > 1 direct cdf: find a structure where y = fl(1-x) enters with negative weight. Two concrete tests:
   - a continued fraction in z = x/y with the prefactor from log1p(-x) (Cephes incbd-like). Its predicted c is
     -dlnD/dlnz, computable from the exact function.
   - TOMS bgrat/bup on (b, a, y, x) with the complement taken in x87 extended precision, so no 2^-53 quantization.

   Simulate c for each, and require the table in 2.1 before trying for bit-exactness. A run at x in [0.5, 1) on the
   direct side (a > b, e.g. (30.7, 1.7) already shows sd 4.1 with no ties) separates the 1 - x part from the rest.
2. The density for a, b > 2: Loader's saddle point (R dbinom_raw with stirlerr/bd0) is the only structure with
   r = 0.73-0.79 at (4, 4). Try Excel-built pieces (Excel LN/EXP inside bd0 and stirlerr, Excel GAMMALN for the
   non-half-integer stirlerr), and the older R 2.x dbinom_raw (lf = log(2 pi) + log(x) + log1p(-x/n)).
3. lnB at small shapes: the log-form x-assembly is bit-exact once lnB is right (0.5, 0.7). Map lnB(a, b) per block
   by the lnB scan, and fit which of GL/ratio/TOMS-gamln forms Excel uses where. In particular, ln Gamma(a) for
   a < 1 is not Excel's GAMMALN (2.2).
4. More probes:
   - a/b sweeps at a, b > 2 with x in [0.5, 1) (no 1 - x ties);
   - runs straddling x = 0.5 at several (a < 1, b) pairs, to pin the cdf switch to the ulp;
   - analysis of the filled `t2464-w2f-unit-shape` case (closed-form switch at a, b = 1 +- k ulp).

## Scripts
`make_probes.py` (cases), `exact_w2f.py`, `common.py`, `summary.py`, `modk.py`,
`tie_response.py`, `tie_basis.py`, `runs_regress.py`, `runs_regress2.py`, `sweep_regress.py`, `lambda_test.py`,
`grid_test.py`, `cand_runs.py`, `pdf_regress0.py`, `pdf_offsets.py`, `pdf_cands.py`, `pdf_rlog.py`,
`pdf_logform.py`, `pdf_logform2.py`, `lnb_scan.py`, `holdout_small.py`, `boundaries.py`, `toms.py`, plus `ext/`
(R toms708 and R 3.6 dbeta build shims; fetch commands in `ext/README` and `ext/r36/README`; binaries and fetched
sources git-ignored). Run with `/home/user/rustsheet/bindings/python/.venv/bin/python` from this directory. numpy
and xlsxwriter were installed into that venv.
