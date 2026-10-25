# T-2464 w4-fractions: the direct branch as a continued fraction times a prefactor

Worker w4-fractions, 2026-10-23. Assignment: test the direct (unreflected) cdf as a continued fraction (CF) times a
prefactor. Candidates were Cephes incbcf/incbd, NR betacf, A&S 26.5.8 forward and backward, TOMS 708 bfrac and
LibreOffice's fraction, each in SSE double, stored x87 and x87 extended, with every stopping rule.

Units: "rel" is relative error in units of 2^-53. Scores are bit-exact / within 4 ulp / within 15 digits (w15).

**Headline.** Excel's direct branch for a, b > 1 is **TOMS 708 `bratio`'s branch logic, with Excel's own density
in place of `brcomp`/`brcmp1`**. Every x^a y^b / B(a,b) prefactor is computed as `BETA.DIST(.,FALSE)·x·y`. A
continued fraction (TOMS `bfrac`) appears only where `bratio` itself calls it, at min(a,b) >= 40. For b < 40 and
b·x > 0.7, the cdf is TOMS's L140 path: `bup(b0', a, y, x, n) + bpser(a, b0', x)`.

On the 5,631 run cells of w2's L140 blocks this is bit-exact on 3,063 cells, against 1,312 for the correctly rounded
value. Swapping in Excel's own density and bpser cells (new probe) gives 3,571 of 4,095 (87%). On held-out old
cells it gets 35/78/77 of 78, against CR's 10/48/58. Every published CF combined with any prefactor stays at or
below the correctly rounded value.

## 0. Data and housekeeping
- `data.tsv` (`prep.py`, then `reexact.py`): 23,034 (x, a, b) points. Each has Excel's cdf and pdf where present
  (`../beta_exact.tsv` and w2-forensics' `exact_w2f.tsv`), its branch, and exact cdf and pdf at mpmath 50 digits.
- **Note for the coordinator:** the "30-digit" exact values in `w2-forensics/exact_w2f.tsv` are only accurate to
  double precision. They differ from 50-digit values by up to 0.96 units (median 0.36; `reexact.out`). This is
  harmless for w2's tie classes, but wrong for any sub-ulp residual work. Use `data.tsv` here.
- **New probe** `probe/w4p1` (excel-graph, 2026-10-23, 15,354 formulas, one 15-second fill). It covers every
  direct cell of w2's xruns-1:0, 1:1, 1:2, 1:5 and 2:3. For each it has `BETA.DIST(1-x, b0', a, FALSE)` (the
  density L140's `bup` would need), `BETA.DIST(x, a, b0', TRUE)` (bpser's value) and `1-x`.
  - b0' = b - int(b) in double (1 when b is an integer).
  - Builder: `probe/make_probe.py`. Raw output: `probe/w4p1.out.json`.
  - The only write outside this directory was the oracle's git-ignored cache. The case is not in `eval/suites`.
- External code: R's `toms708.c` and R 3.6 nmath sources are fetched, not committed (`ext/.gitignore`). Fetch
  commands are in `../w2-forensics/ext/README`. `ext/toms708_logv.c` is a patched copy with the x+y=1 check removed
  and a switchable log choice.

## 1. Structural test: the tie response c = d ln F / d ln y (exact arithmetic)
w2 measured how Excel's direct-cdf error responds to the rounding of fl(1-x). Here c is computed for each structure
in mpmath, with y an independent input: `ideal_c.py`, `ideal_c2.py`, `ideal_c3.py`, `ideal_as63.py`, `toms_c.py`
and `toms_c2.py`. `classes.py` confirms that the response is linear in the rounding of fl(1-x). In each run, the
mean by the low 2-3 mantissa bits of x follows the 1-x tie pattern; even classes are flat.

| block (a, b, x) | Excel c | Py·incbcf (Cephes/NR/LO/A&S) | Py·incbd/y | no-y prefactor·incbd | Py·bfrac(λx) | bfrac(λy) | 1-I_y(b,a) | R TOMS bratio | **TOMS L140 with pdf·y·x (Ha)** |
|---|---|---|---|---|---|---|---|---|---|
| 2.5, 3.5, 0.211 | -3.72 (0.17) | +3.50 | +2.31 | -1.19 | +3.50 | -2.05 | -7.71 | +1.15 | **-3.68** |
| 2.5, 3.5, 0.331 | -1.36 (0.09) | +3.50 | +2.16 | -1.34 | +3.51 | -2.35 | -3.56 | +1.04 | **-1.31** |
| 2.5, 3.5, 0.286 | -1.91 (0.11) | +3.50 | +2.22 | -1.28 | +3.51 | -2.22 | -4.72 | +1.08 | **-1.97** |
| 7.3, 11.6, 0.273 | -8.51 (0.39) | +11.60 | +9.97 | -1.63 | +11.61 | -3.78 | -11.15 | +7.68 | **-8.49** |
| 4, 4, 0.232 | -7.17 (0.27) | +4.00 | +2.82 | -1.18 | +4.00 | -1.90 | -11.05 | +1.26 | **-7.22** |
| 100, 120, 0.401 | -10.58 (0.73) | +120 | +116 | -3.65 | +120 | -7.96 | -36.8 | 0.00 | bfrac path: c_pdf + 1 = **-10.5** |

How to read the table:
- Py means a prefactor that uses y = fl(1-x).
- No 0/1 combination of the CF basis terms fits (`ideal_c3.out`, best sum of squares 35).
- AS 63's series behaves like incbd (`ideal_as63.out`).
- R's bratio with four log-selection variants in brcomp/brcmp1 scores +1.15 to -7.22 (`toms_c2.out`).

**Ha** (`ideal_bup.py`) is TOMS L140, `w = bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0)`, with:
- bup's `brcmp1(b0', a0, y0, x0)` equal to Excel's density of Beta(b0', a0) at y0, times y0, times x0;
- that density in w1's M1 log form, (b0'-1)·LN(y0) + (a0-1)·LN(fl(1-y0)), since y0 > 0.375;
- x0 taken from x.

Ha matches all five blocks within one standard error. Hb (b0'·LN y0 + a0·LN(1-y0), raw brcmp1) gives -7.22,
-3.19, -4.31, -11.15 and -10.36, so it is rejected.

At (100, 120), the only run with min(a,b) >= 40, TOMS takes bfrac. The pdf's own c is (b-1) - (a+b-2)y = -11.5
(section 5), and brcomp = pdf·x·y adds +1. That predicts -10.5, against -10.58 measured.

## 2. The (100, 120) block: where a continued fraction is used
- **The cdf shares the pdf** (`share_test.out`). Here the errors of Excel's cdf and pdf at the same x correlate at
  +0.96, and cdf - pdf has sd 4.45 against 15.2 for the cdf alone. Every other block correlates between -0.31 and
  +0.23.
  - Assembling cdf = pdf_e·x·y·CF/a from Excel's own pdf cells lifts the (100,120) block from 34 (CR) to 169/1,024
    (`t1_pdfshare.out`).
  - Over all 15,657 direct cells, no CF × pdf assembly beats CR: the best is 2,931 against 2,920. This covers 16 CF
    variants × 3 precisions × 5 orders × {Excel pdf, CR pdf}.
- **The fraction depends on λ = fl(a - fl((a+b)·x))** (`t4_lambda.out`).
  - Excel's implied fraction error q = cdf_e/(pdf_e·x·fl(y)/a)/CF_exact regresses on that λ's rounding with slope
    -0.725 (se 0.04).
  - The exact sensitivity of TOMS bfrac to ln λ there is -0.71 (`t10_lamall.out`). A double-precision bfrac
    simulation gives -0.718. This is the positive control.
  - Other λ forms ((a+b)y - b, a·y - b·x, extended) have slopes of +0.01 to +0.33.
  - Cephes, NR, A&S and LibreOffice fractions have no λ, and their noise correlates with q at -0.03 to +0.10
    (`t2_cfnoise.out`).
- **The fraction's own arithmetic was not found.** After the λ term the residual is white, sd 3.78, with no
  low-bit class structure (`t5_resid.out`).
  - None of these correlate with it: 64 masks of double/extended storage in bfrac, in R's C order or the Fortran
    order (`c/bfmix.cpp`, `t6_mix.out`); eps 1.5e-14, 1e-15, 2.2e-16, 1e-14, 3e-15; SSE, stored x87 and extended
    (`c/bfrac.cpp`, `t3_100_120.out`).
  - Nor do the roundings of LN x, LN y, a·LN x, b·LN y, x·y or y+1 (`t7_regress.py`).
  - Best exact: 185/1,024 (extended bfrac, double λ, fl(pdf_e·fl(x·y))·r), against CR's 34.
- Elsewhere the λ regression is confounded by the 1-x tie classes wherever (a+b)x shares the dyadic low bits
  (2.5/3.5, 4/4). At (30.7, 1.7) the slope is -0.004 (0.06) against bfrac's -0.80, so bfrac is not used there.
  That agrees with TOMS, which takes bpser/bup for b < 40.

## 3. Region map of the direct branch, a, b > 1 (TOMS 708 bratio paths)
`tomsx.toms_path` is TOMS's own selection: λ < 0 swaps; b0 < 40 takes bpser if b·x <= 0.7, otherwise L140 (bup,
plus bpser if x <= 0.7, plus bgrat beyond 0.7); b0 >= 40 takes bfrac or basym.

| path | old direct cells (beta_exact) | w2 run cells | what Excel does (this work) | best exact here |
|---|---|---|---|---|
| bpser (b < 40, b·x <= 0.7) | 150 | none | TOMS bpser shape; prefactor not pinned | 22/150 (CR 6), `t29_bpser.out` |
| L140, x <= 0.7 | 78 | 5,631 | **bup with an Excel-pdf brcmp1, plus bpser** | old 35/78 (CR 10); runs 3,063/5,631 (CR 1,312) |
| L140 with bgrat (x > 0.7) | 25 | none | not tested | |
| bfrac (b0 >= 40) | 43 | 1,024 | **bfrac with λ = fl(a - fl((a+b)x)), brc = pdf·x·y** | runs 185/1,024 (CR 34); old 6/43 (CR 2) with the CR pdf |
| basym | 0 | none | | |

For min(a,b) <= 1 the tie response is 0, consistent with TOMS's bpser/bup/fpser/apser paths that never touch
fl(1-x). w2 also found a switch at x = 0.5, TOMS's min <= 1 rule. Neither was modelled here.

## 4. L140 in detail: scores
Model "TOMS-X L140" (`tomsx.py`, `l140_best`):
- y0 = fl(1-x); n = int(b); b0' = b - n (if b0' = 0, then n -= 1 and b0' = 1).
- P = Excel's density of Beta(b0', a) at y0. It is modelled by w1's M1, with lnB =
  fl(GL(min) + fl(GL(max) - GL(a+b))), Excel's GAMMALN and LN, and EXP model V6.
- bup = fl(fl(P·fl(y0·x)) / b0') · W, with TOMS's forward sum `d *= (apb+l)/(ap1+l)*y0; w += d`.
  - apb + l, ap1 + l, their quotient, d and w are stored as doubles.
  - The product (quotient·y0) stays in an x87 register.
- Q = TOMS bpser(a, b0', x), with prefactor EXP(fl(a·LN x) - lnB)/a.
- cdf = fl(bup + Q).

| subset | n | TOMS-X exact (even/odd) | w4 | w15 | CR exact | CR w4 | CR w15 |
|---|---|---|---|---|---|---|---|
| xruns-1:0 (2.5, 3.5) | 1,023 | 615 (298/317) | 1,023 | 1,023 | 260 | 1,021 | 1,023 |
| xruns-1:1 (2.5, 3.5) | 1,024 | 519 (253/266) | 1,024 | 1,024 | 293 | 1,024 | 1,024 |
| xruns-2:3 (2.5, 3.5) | 1,024 | 725 (356/369) | 1,024 | 1,024 | 360 | 1,024 | 1,024 |
| xruns-2:5 (2.5, 3.5, direct half) | 513 | 324 (166/158) | 513 | 513 | 218 | 513 | 513 |
| xruns-1:5 (4, 4) | 1,024 | 580 (286/294) | 1,024 | 1,024 | 96 | 779 | 979 |
| xruns-1:2 (7.3, 11.6) | 1,023 | 300 (158/142) | 1,021 | 1,023 | 85 | 605 | 1,023 |
| **all L140 run cells** | **5,631** | **3,063** | **5,629** | **5,631** | 1,312 | 4,966 | 5,586 |
| **hold-out: old beta_exact L140 cells** (never tuned on) | **78** | **35** | **78** | **77** | 10 | 48 | 58 |

On the hold-out, CR is up to 150 ulp off where TOMS-X is exact, for example BETA.DIST(0.1, 100, 10, TRUE).
`t25_old.out` lists every cell.

With Excel's own pieces (probe w4p1, P and Q read from the sheet) the model scores (`t23_W2.out`, `t24_joint.out`):
- (2.5, 3.5): 898, 925 and 905 of about 1,024; (4, 4): 843 of 1,024. That is 3,571 of 4,095 (87.2%), even and odd
  1,788 each.
- The W rounding placement was picked here. Fully stored W gives 3,474; extended W 3,161; Horner 2,777; reverse
  summation 3,312.
- The product order P·(y0·x) beats (P·y0)·x, which scores 2,886 to 3,402. Putting /b0' first or last makes no
  difference when b0' is 0.5 or 1.
- The internal P is the rounded double, not the extended EXP result (`t21_pint.py`). On cells where M1 reproduces
  the worksheet density, the double P gives 2,199 of 2,624 and the extended P 1,874.
- The remaining misses are ±1 ulp. A one-ulp nudge of W fixes most of them, and of P about half
  (`t19_miss.out`). The sum or product placement is not fully pinned.
- **(7.3, 11.6) is the exception: the worksheet density is not bup's density.** With Excel's own P cell the model
  scores 0/1,023 (all 9 to 18 ulp low). The modelled M1 P (GAMMALN-form lnB) gives 300.
  - The worksheet `BETA.DIST(y0, 0.6, 7.3, FALSE)` runs -14.1 rel (sd 6) from exact, while the cdf needs about
    +10 (`t16_s2.py`).
  - A single scale factor peaks at only 271/1,023 (`t17_scale.out`).
  - So for (b0', a) = (0.6, 7.3) the worksheet density and bratio's internal brcmp1 take different lnB or
    exponent paths. This is open.

## 5. Side finding for the density workers: Excel's pdf for a, b > 2 is Loader's saddle point
At (100, 120) (`t8_pdfreg.out`), Excel's pdf error regresses on:
- the absolute rounding of fl((a+b-2)·x), with coefficient +0.133;
- the absolute rounding of fl((a+b-2)·y), with coefficient -0.082;
- the tie of fl(1-x), with coefficient -11.57.

The predictions are dbd0(k, M)/dM = -(1 - k/M) = +0.131, -0.088 and -((a+b-2)y - (b-1)) = -11.5. This is R's
`dbinom_raw(a-1, a+b-2, x, y)`, with np and nq rounded doubles.

The tie prediction (b-1) - (a+b-2)y also holds for the pdf c of w2's other blocks: -0.65 / -0.67 measured, -0.17 /
-0.19, -0.36 / -0.37, -1.69 / -1.62 and +0.17 / +0.14. (4, 4) is the exception: predicted -1.61, measured +0.12.

R 3.6's series bd0 correlates +0.64 with Excel's pdf error (sd 10.3 against 14.7). The direct x·log(x/np) form and
the log1p form are far noisier (sd 86 and 56) (`t9_pdfloader.out`). So it is Loader, but not R's exact arithmetic.

## 6. Negatives (all on the same scorer)
- Generic CF × prefactor on all 15,657 direct cells that have both cells (`t1_pdfshare.out`): every combination
  is at the CR level or below, best 2,931 against CR's 2,920. The search covered:
  - NR betacf with eps 3e-16, 2.2e-16, 1e-15, 1e-14 and a fixed 300 terms;
  - Cephes incbcf with thresh 3·MACHEP and 1·MACHEP, and incbd;
  - LibreOffice's fraction;
  - TOMS bfrac with λ_x and λ_y, eps 1.5e-14 and 2.2e-16;
  - A&S backward with 30, 60, 120 and 240 levels;
  - each in SSE, stored x87 and extended, with 5 assembly orders and Excel's or the CR density.
- Tie response: every published CF × prefactor structure has the wrong c for a, b > 1 and b < 40 (section 1).
- At (100, 120), every non-λ fraction has noise uncorrelated with Excel's (|corr| <= 0.10), and none of the 128
  bfrac storage/order masks explains the residual (section 2).
- bfrac's λ from y, (a+b)y - b, is rejected at (100, 120): corr -0.02, c -7.96 against 0.
- Complement forms 1 - I_y(b,a) (any CF) are rejected by c (-7.7 to -2,584 against -3.7 to -10.6).
- L140 alternatives: Hb (raw brcmp1 exponents a, b), R's TOMS logs, extended P, extended or Horner W, and the
  `(P·y0)·x` order all score lower or fail c (section 4).
- bpser region: an Excel-style exp-log prefactor gets 16 to 22 of 150, against CR's 6, so it is not pinned.

## 7. Most promising next hypotheses
1. **Write all of Excel's beta cdf as a TOMS 708 bratio port, with brcomp and brcmp1 equal to `pdf(x)·x·y`**, where
   pdf is the worksheet density function.
   - Next: bgrat (L140 with x > 0.7), basym, fpser/apser, and the min(a,b) <= 1 paths (bpser/bup with swaps at
     x > 0.5).
   - The reflected branch follows as bratio's swap with w = 0.5 - w1 + 0.5. Test it against w2's reflected runs:
     R's bratio already led there, 2,220 against CR's 1,951.
2. **bup's density at (b0', a) = (0.6, 7.3)**. Find why the internal brcmp1 is about 24 units above the
   worksheet density. Probe the worksheet `BETA.DIST(y0, b0', a, FALSE)` for a in 6.5 to 8 and b0' in (0, 1), and
   compare it with the cdf's implied P. TOMS's brcmp1 1 < b0 < 8 branch (gam1, the product c, esum) is the first
   candidate.
3. **The last 13% of L140 misses**, which are ±1 ulp. Pin the W and product rounding with a probe at fixed x over
   consecutive b (b0' varies, n fixed), and at b0' = 0.5 with n = 2..10.
4. **The bfrac residual (sd 3.8) at large shapes.** Run more min(a,b) >= 40 runs, irregular and at several λ.
   Regress the residual on the per-iteration quantities of bfrac (w = n·x·(b-n) and alpha/beta roundings) with a
   phase-free spectral test. Also try TOMS's eps = 15·max(eps, 1e-15) with the original Fortran BFRAC compiled for
   x87.
5. **The density for a, b > 2 is Loader's dbinom_raw with fl(n·x), fl(n·y)** (section 5). Peel stirlerr and bd0
   with Excel's LN, EXP and GAMMALN. That pdf is the cdf's prefactor in every a, b > 2 path, so solving it lifts
   the cdf directly.

## Scripts (run with /home/user/rustsheet/bindings/python/.venv/bin/python from this directory)
| script | purpose | output |
|---|---|---|
| `prep.py`, `reexact.py` | build data.tsv, recompute exact values at 50 digits | `reexact.out` |
| `common4.py` | loaders, x87 via numpy longdouble, CF runner, scorer | |
| `c/cf.hpp`, `c/cfmain.cpp` (`cf0/1/2.bin`) | 16 CF variants × SSE / stored x87 / extended | |
| `c/bfrac.cpp` (`bf0..4.bin`), `c/bfmix.cpp` | TOMS bfrac orders × λ forms × eps × precision; mixed-precision masks | |
| `ideal_c*.py`, `ideal_as63.py`, `ideal_bup.py`, `toms_c*.py` | tie-response structure tests | `*.out` |
| `classes.py` | Excel error by low mantissa bits of x | `classes.out` |
| `share_test.py`, `t1_pdfshare.py`, `t2_cfnoise.py`, `t3_bfrac.py`, `t4_lambda.py`, `t5_resid.py`, `t6_mix.py`, `t7_regress.py`, `t10_lamall.py`, `t26_bfrac_old.py` | pdf sharing, fraction noise, λ | `*.out` |
| `tomsx.py` | the TOMS-X model (L140, path map) | |
| `t11`–`t13`, `t14_pieces.py`, `t15`–`t24`, `t27_final.py` | L140 scans and scores | `*.out` |
| `t25_old.py` | hold-out on beta_exact | `t25_old.out` |
| `t8_pdfreg.py`, `t9_pdfloader.py`, `ext/r36/drvpdf.c` | Loader pdf tests | `*.out` |
| `t29_bpser.py` | bpser region side check | `t29_bpser.out` |
| `probe/make_probe.py` | the w4p1 oracle case | `probe/w4p1.out.json` |
