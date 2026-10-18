# T-1778 round 3: ERFC-tail probe data (label: erfc-probes)

Oracle: Microsoft 365 Excel via `rustsheet-oracle fill -b excel-graph --suite probes-numeric` (3 fills: 15 + 1 + 2 + 1
sidecars, 0 failures, never --force; log fill.log). All 19 cases are uncommitted in
`/home/user/rustsheet/eval/suites/probes-numeric/cases/kmyst-erfc-<name>.case.json` (+ `.oracle.json`).
Generators: gen_cases.py (15 cases), gen_zoom.py (jumpzoom), gen_stepmap.py (stepmap1/2), gen_micro.py (stepmicro).
layout.json = per-case columns and run table (run id, first row, n, label, x0). Every result cell is numeric (no errors).

## Run-file formats
* `<case>.txt` (one per case): `col run x value`; col = sheet column (B, C, ...), run = run id below, x = column A,
  value = oracle result (constant columns copied from the case). x and value are float.hex(). Non-numeric would be
  `!type:text` (none occur).
* Combined, round-2 format `run x result` (hex):
  `er.txt` ERFC.PRECISE(x), `ee.txt` EXP(-(x*x)) - runs 0-59 (kmyst-erfc-runs1..4);
  `cr.txt` CHISQ.DIST.RT(2z,1), `cs.txt` ERFC.PRECISE(SQRT(z)), `cx.txt` EXP(-z), `cq.txt` SQRT(z) - runs 100-129
  (kmyst-erfc-chiruns1..3, the x column is z).
* `runs_resid_<case|all>.npy`: columns x, ERFC, Excel EXP, cr exp(-z87), resid (Excel EXP), resid (cr exp), row, tag.
  resid = ERFC/(E*erfcx(x)) - 1 in units of 2^-53 (relative), mpmath 140 bits; z87 = fl53(fl64(x*x)).

## Cases (cells; layout)
| case | cells | content |
|---|---|---|
| split05 | 4,096 | 512 doubles below 0.5, 0.5 (row 513), 511 above; B ERF, C ERFC, D ERFC(-x) |
| split05b | 7,400 | x = 0.4990 + 1E-5 k + 6.18E-8, k=0..1849; same columns |
| decouple | 10,240 | 8 runs x 256 consecutive z at zc 0.3,1,4,9,30,100,300,600; B CHISQ.DIST.RT(2z,1), C ERFC(SQRT z), D SQRT(z), E GAMMA.DIST(z,.5,1,TRUE) |
| expcol | 16,384 | 8 runs x 512 consecutive x at 0.75,1.5,3,5,7.9,12,20,26; B ERFC, C EXP(-(x*x)), D x*x |
| branchscan | 11,520 | 15 runs x 256 (128 below b, b, 127 above), b in 1,1.5,...,20; B ERFC, C EXP |
| driftmap | 24,000 | 8,000 points in (0.5,26.54); B ERFC, C EXP |
| runs1..4 | 4 x 46,080 | (f) 60 runs x 1,024 consecutive x, geometric ladder 0.505..26.5 (ratio 1.0694), full-mantissa x0 = xt(1+6.18E-10); B ERFC, C EXP; run ids 0-59 |
| chiruns1..3 | 3 x 51,200 | (f) 30 runs x 1,024 consecutive z, ladder 0.26..702 (ratio 1.3135); B CHISQ.DIST.RT(2z,1), C ERFC(SQRT z), D EXP(-z), E SQRT(z); run ids 100-129 |
| cutoff | 6,462 | (g) x = 26.50+0.002k (76) and 26.50+1E-4k+6.18E-7 (1,001); B ERFC, C EXP, D CHISQ.DIST.RT(2x^2,1), E NORM.S.DIST(-x*SQRT(2)), F x*x |
| norm | 24,576 | (h) 6 runs x 512 consecutive y (x_erfc = 0.6,1.2,3,6,12,18.8); B NORM.S.DIST(-y), C y*sqrt(.5) and D y/sqrt2 (constants), E ERFC(C)/2, F ERFC(D)/2, G NORM.DIST(-y,0,1,TRUE), H ERFC(A*SQRT(0.5))/2 |
| jumpzoom | 18,000 | follow-up: 10 grids of 600 (step 5E-4) around driftmap change-point candidates |
| stepmap1 / stepmap2 | 51,000 / 52,500 | follow-up: uniform grids 0.5..9 step 5E-4, 9..26.5 step 1E-3 (+6.18E-8); B ERFC, C EXP |
| stepmicro | 12,000 | follow-up: 5 grids of 800 (step 2E-5) across strong steps at 9.0485, 9.6465, 19.3655, 8.0445, 4.115 |
Run x0 values and first rows are in each case's description and in layout.json.

## Analysis scripts (first pass; logs alongside)
common.py (loader, mpmath residual, x87mul), an_split.py (split.log), an_decouple.py (decouple.log), an_branch.py
(branchscan.log), an_z87.py (z87.log), an_expuse.py (expuse.log), an_runs.py (per-case residual npy), runs_summary.txt,
an_chpt.py, an_seg.py (PELT; segments_pen30.txt, seg_pen30.log), an_steploc.py (steps_located.txt, steploc.log),
driftmap_bins025.txt, sc_ext.c (round-2 scorer copy with |x| <= 0.6 allowed, used for the x = 0.5 test).

## First-pass findings (details in the final report)
1. Split: x < 0.5 small-x rational (ERFC = fl(1-ERF) at 612/612 points below 0.5, ERFC never odd at 2^-54); x = 0.5+2^-53
   is already on the erfc kernel. x = 0.5 itself matches the round-2 small-x model (that model matches only 216/511 points
   above 0.5, so this single point is weak evidence, likelihood ratio ~2.3 for "x <= 0.5 is small").
2. The internal z is fl53(fl64(x*x)) (x87 double rounding), like the worksheet `*`: at the 576 branchscan points where it
   differs from the single-rounded product, residual mean +0.20 with z87 vs +118 (max 518) with z53. Worksheet x*x equals
   z87 at 4,096/4,096 (expcol).
3. The kernel's exp factor is Excel's own EXP: at 2,477 points where EXP(-(x*x)) != correctly rounded exp(-z87), the
   residual deviates from the local mean by +0.08/-0.03 with Excel's EXP against +1.44/-1.41 with the correctly rounded exp.
   Excel EXP is faithful but not correctly rounded: 2,477 of 118,036 differ (2.1%), all by 1 ulp, 2,221 of them low.
4. Decouple is negative: CHISQ.DIST.RT(2z,1) = ERFC.PRECISE(SQRT(z)) at 32,768/32,768 (2,048 decouple + 30,720 chiruns), and SQRT(z) =
   fl53(fl64(sqrt z)) at 32,768/32,768 (correctly rounded sqrt at only 32,631). The upper-tail path takes the stored root and recomputes z.
5. NORM.S.DIST(-y) = NORM.DIST(-y,0,1,TRUE) = ERFC.PRECISE(fl(y*sqrt(0.5)))/2 at 3,072/3,072; y/sqrt(2) only 590/3,072.
6. Zero cutoff: the result is flushed when below DBL_MIN (last nonzero ERFC 2.2319E-308 at x = 26.5432006, first zero
   at 26.5433006 where true erfc = 2.2201E-308). Same flush in CHISQ.DIST.RT and NORM.S.DIST. Not an x threshold.
7. No branch jump >= 0.3 units at any candidate b in {1,...,20} (128-point means, branchscan.log).
8. The drift is piecewise-flat: plateaus 0.1-1.3 wide, joined by fast transitions (0.01-0.05 wide, not ulp-sharp
   discontinuities per stepmicro), ~45 transitions between 1.84 and 26.5, flat (-0.01 +- 0.03) on 0.5-1.84.
   Deterministic in x (run means vs stepmap segment means: corr 0.956). No lattice in x, x^2, 1/x, 1/x^2, ln x, sqrt x.
