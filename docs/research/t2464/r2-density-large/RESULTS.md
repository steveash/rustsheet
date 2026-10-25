# T-2464 r2-density-large: Excel's beta density for a, b > 1

Worker r2-density-large, round 2, 2026-10-23. Method: find the composition of BETA.DIST(x,a,b,FALSE) where w1's M1
does not apply (both shapes > 1, including min(a,b) >= 8), using tie-response and rounding-error regressions on
consecutive-double runs, free-constant structure scans, shape-sweep fingerprints, and one new probe.

Counts are bit-exact / within 4 ulp / within 15 significant digits ("w15"). "Units" are 2^-53: relative for the
density, absolute for its logarithm E. CR is the correctly rounded value. Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 0. Data, controls, housekeeping
- **Data** (`dlib.load()`): 32,264 cells with a > 1 and b > 1.
  - **Round-1 data** (18,440 cells): every pdf cell from w4's `data.tsv` (beta_exact plus the w2 runs and sweeps,
    with 50-digit exact values) and w1's probe1 sheets (2.5,3.5) and (12,20), whose exact values are recomputed
    here at 160 bits.
  - **This worker's probe** (13,824 cells): `t2464-r2-density-large-p1`.
- **New probe `t2464-r2-density-large-p1`** (13,824 cells, excel-graph, filled 2026-10-23 in one 15 s
  `rustsheet-oracle recalc`):
  - Case: `eval/suites/probes-numeric/cases/t2464-r2-density-large-p1.case.json`.
  - Output: `probe_p1.out.json`. Row map: `probe_p1.meta.json`. Builder: `make_probe.py`, which reuses w2's
    exact-double encoder by import.
  - All 13,824 x arguments re-evaluate to the intended double.
  - No oracle sidecar was written into `eval/`.
  - Blocks (all runs of consecutive doubles):
    - `cut±e`: 12 runs of 256 at (2.5,3.5).
    - `s8(a,b)`: a in {7.5, 8, 8.5} × b in {8, 12, 40}, 512 each near the mode.
    - `shp(a,b)`: 16 shapes × 256 near the mode, mirrors included.
    - `hold(2.5,3.5)` and `hold(7.3,11.6)`: 1,024 each.
  - Every choice below was made on round-1 data, apart from the cut, which was read off the `cut` blocks. So `s8`,
    `shp` and `hold` are a hold-out. The row-level hold-out is md5-odd (`dlib.holdout`).
- **Positive controls** (`ctl1.py`, `ctl1.out`):
  - The free-constant scan, run on synthetic data from the final structure with the constant moved by +3.3 units,
    recovers +3.305 and +3.258 units and scores 1,023/1,023 and 1,024/1,024.
  - The wrong structure (exact Loader, cut 0.1) gets 289/519 on the hold-out at x = 0.21. It is indistinguishable
    at x = 0.29, where both sides take the series and nothing is visibly rounded. See section 5.
  - The tie-response estimator gives -0.65, -1.72 and -0.37 on synthetic Loader data (predicted -0.65, -1.68 and
    -0.36). On synthetic log-form data it gives +2.49, +10.60 and +2.50 (predicted b - 1).
- **Reuse.** Code from earlier workers is imported, never copied:
  - w1 `lib` (EXP V6 and LN models, x87 rounding, scorer) and w1 `toms_betaln`;
  - r2-log1p `excel_log1p` and `x87f`;
  - w6 `h2_loader` (the stirlerr table and Arith);
  - w2 `make_probes`.
- **Caches.** GAMMALN values are cached in `gl_cache.pkl`. One write went outside this directory: w1's toms_betaln
  default argument added entries to `w1-density/xcache.pkl`, a git-ignored cache of harness values.

## 1. Headline
**For min(a,b) > 1, Excel's density is a Loader (2000) saddle-point evaluation, R `dbeta`'s
`(a+b-1)·dbinom_raw(a-1, a+b-2, x, y)`. It takes the complement q = fl(1-x) as a separate input, and its bd0 is
built from x87 stored ops on Excel's LN.** This covers min(a,b) >= 8 and 1 < b < 2 as well. There is no a0 = 8
switch and no TOMS brcomp. The x-dependent part is now modelled nearly to the bit wherever bd0 takes its log form.
The per-shape constant, ln(a+b-1) + stirlerr terms - lf/2, carries an unexplained error of -8 to +8 units.

Best full model **LDe** (`model.py` with `stir=exact,lf=LF0`; `m1_score.py`, `sumtab.out`):

| subset | n | LDe | CR | Loader with w6's stirlerr (LDt) |
|---|---|---|---|---|
| all a, b > 1 | 32,264 | **5,765** / 25,254 / **31,385** | 3,573 / 26,024 / 29,280 | 3,631 / 20,497 / 28,449 |
| round-1 data | 18,440 | **3,071** / 13,227 / **17,561** | 1,127 / 12,956 / 15,456 | 1,558 / 10,101 / 15,649 |
| own probe (hold-out) | 13,824 | **2,694** / 12,027 / 13,824 | 2,446 / 13,068 / 13,824 | 2,073 / 10,396 / 12,800 |
| min(a,b) >= 8 | 6,208 | **1,708** / 4,954 / **5,795** | 1,125 / 4,503 / 5,434 | 1,693 / 4,960 / 5,805 |
| 1 < min < 8 | 26,056 | **4,057** / 20,300 / **25,590** | 2,448 / 21,521 / 23,846 | 1,938 / 15,537 / 22,644 |

- LDe gains 2,192 exact cells and 2,105 w15 cells over CR.
- It loses 770 cells within 4 ulp. This is the constant: (2.5,3.5) sits at -5 units, and shapes like (2.5,2.5) and
  (1.03,40) at -7 to -8.
- With a free constant per shape (section 4), the same structure reaches 80-96% on runs where bd0 takes its log form.

## 2. The structure, with the evidence for each part

### 2.1 Loader with q = fl(1-x), not a log form or brcomp (`d5_yresp.py`, `d5_yresp.out`)
For x in [0.25, 0.5), fl(1-x) rounds on odd mantissas, and the density error responds linearly to that rounding.
The coefficient is in units per unit of relative error of y. Three structures predict different values:
- Loader's bd0(n-k, n·q) predicts **(b-1) - (a+b-2)·y**.
- The log form (b-1)·ln y predicts b - 1.
- brcomp/(x·y) predicts -1.

| block (a,b), x | Excel (se) | Loader | log form | brcomp/(xy) |
|---|---|---|---|---|
| (2.5,3.5) 0.2114 | **-0.66 (0.09)** | -0.65 | +2.50 | -1.00 |
| (2.5,3.5) 0.3314 | **-0.19 (0.03)** | -0.17 | +2.50 | -1.00 |
| (2.5,3.5) 0.2857 | **-0.37 (0.05)** | -0.36 | +2.50 | -1.00 |
| (2.5,3.5) 0.4167 | **+0.16 (0.02)** | +0.17 | +2.50 | -1.00 |
| (7.3,11.6) 0.2731 | **-1.62 (0.13)** | -1.68 | +10.60 | -1.00 |
| (100,120) 0.4013 | **-11.37 (0.69)** | -11.52 | +119 | -1.00 |
| (4,4) 0.2317 | +0.15 (0.22) | -1.61 | +3.00 | -1.00 |

- Six of seven blocks agree within one standard error, including the sign change at x = 0.4167.
- With shapes (a, b) in place of (a-1, b-1), the prediction would be -1.23 at x = 0.21 and 0.00 at x = 0.42. So
  the shapes are a-1 and b-1.
- (4,4) is the exception. It fits n·q formed without q, for example n - n·x (open, section 6).
- (100,120) has min >= 8. It shows the same Loader response, not the TOMS a0 >= 8 Stirling form: that form's
  lambda uses x only, which gives -1 after the /(x·y).

### 2.2 bd0's log form: stored ratio, Excel LN, stored product, then an unrounded add (`d6_bd0reg.py`, `d6_bd0reg.out`)
The density error was regressed on each rounding inside Loader's bd0(k, M) = k·ln(k/M) + M - k. Each rounding is
signed as it enters the exponent, so a coefficient of 1 means "this rounding is in Excel".

| block | p.div fl(k/M) | p.ln LN | p.mul fl(k·l) | p.add fl(t+M) | q.div | q.ln | q.mul | q.add | sd before → after |
|---|---|---|---|---|---|---|---|---|---|
| (2.5,3.5) 0.6137 | 1.00 | 1.01 | 0.99 | -0.03 | 0.99 | 1.02 | 1.00 | 0.01 | 1.53 → 0.63 |
| (7.3,11.6) 0.6131 | 1.03 | 0.94 | 0.97 | -0.02 | 1.00 | 0.96 | 1.05 | -0.01 | 5.85 → 2.04 |
| (4,4) 0.2317 | 0.99 | 0.99 | 0.98 | -0.02 | 0.95 | 1.01 | 1.02 | 0.02 | 2.84 → 1.03 |
| (2.5,3.5) 0.2114 (q side small) | 1.04 | 1.02 | 1.02 | -0.05 | -0.05 | -0.13 | 0.18 | -0.04 | 1.13 → 0.80 |
| (7.3,11.6) 0.2731 (both sides small) | -0.02 | 0.31 | -0.03 | -0.01 | 0.01 | 0.04 | -0.12 | 0.02 | 2.21 → 2.18 |

Reading:
- The log form is t = fl(k · LN(fl(k/M))): the ratio is x/np, not np/x, and Excel's own LN is used.
- The bd0 value is then t + (M - k), with **no visible rounding**. R's order, fl(fl(t + M) - k), is rejected:
  p.add comes out at 0.
- M is fl(n·x) and fl(n·fl(1-x)).

### 2.3 The series/log cut is |x - np| < 0.2·(x + np), not R's 0.1 (`d6_cut.out`, probe `cut` blocks)
Each `cut` run puts the p side at e = (np - k)/k = e0.

| e0 | -0.40 | -0.38 | -0.36 | -0.34 | -0.30 | +0.30 | +0.34 | +0.38 | +0.40 | +0.45 | +0.50+ | +0.55 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| v = \|d\|/(k+np) | .250 | .235 | .220 | .205 | .176 | .130 | .145 | .160 | .167 | .184 | .2000001 | .216 |
| p.div / p.ln / p.mul | 1.1/0.9/1.1 | 0.9/0.9/1.1 | 0.9/1.0/0.9 | 1.1/1.0/1.1 | ~0 | ~0 | ~0 | ~0 | ~0 | ~0 | 1.1/~/1.4 | 1.0/0.6/0.9 |

- The log form appears exactly when v > 0.2. The bracket is (0.1837, 0.2000001].
- An |e| <= 0.375 rule, the brcomp/alnrel cut, is rejected: e = +0.38, +0.40 and +0.45 take the small-ratio
  branch.
- Below the cut no rounding is visible. The branch is consistent with R's Taylor series in v, and with any
  accurate method. Its arithmetic is not identified (section 5).

### 2.4 Shape constant: Excel LN(a+b-1) and LF0's LN(k), LN(n-k), LN(n) (`c4_sweepreg.py`, `c4_sweepreg.out`)
On w2's sweeps (x fixed, a or b stepped by single ulps, at (7.3, 11.6)), the needed exponent offset was regressed
on the rounding errors of LN(a+b-1) and of -LN(k)/2, -LN(n-k)/2 and +LN(n)/2. Here n = fl(a+b) - 2.
- The coefficients come out at 0.95 to 1.66, and mostly about 1. This is R ≤ 2.7.1's
  lf = ln 2π + ln k + ln(n-k) - ln n, built from Excel's LN, plus LN(a+b-1).
- The residual stays at sd 1.9, so another rounding in the constant is not captured.
- At (2.5, 3.5) the same pieces have too little variance to resolve.

### 2.5 stirlerr is not Excel-GAMMALN-based (`c5_stirvar.py`, `c3_sweep.py`)
On the sweeps, every variant that builds stirlerr from Excel's GAMMALN fails:
- the GL(v+1) - (v+.5)·LN v + v - ln√2π formula, x87 stored or extended;
- the same with LN(GAMMA) or GL(v) + LN v;
- R/w6's table plus formula.

Each has rms(need - candidate) of 36-63 units at (7.3,11.6), against need sd 2.5. **Exact stirlerr** has rms 3.0
with rho +0.4 to +0.85. So Excel's stirlerr is accurate, a table, a series or a better formula, and not the
GAMMALN difference. At (2.5,3.5) none explains the 3.9-unit sweep noise.

## 3. LDe, the full model (`model.py`)
```
k = a-1, n = fl(a+b)-2, nk = n-k                                (x87 stored)
p side: M = fl(n*x);  q side: M = fl(n*fl(1-x))                 (fl(1-x) is the caller's y)
bd0(k, M): d = k-M, s = fl(k+M);
    |d| <  0.2*s : Loader's series in v = d/s, extended            (accurate; not resolved)
    |d| >= 0.2*s : t = fl(k * LN(fl(k/M)));  bd0 = t + (M - k), extended
X  = -bd0(k, Mp) - bd0(nk, Mq)                                   (extended)
Cs = stirlerr(n) - stirlerr(k) - stirlerr(nk)                     (exact values)
lc = fl(Cs + X);  lf = fl(fl(fl(LN2PI + LN(k)) + LN(nk)) - LN(n))
E  = fl(LN(fl(a+b)-1) + fl(lc - lf/2));  density = EXP_V6(E)
```

### 3.1 Region map (exact counts, `m1_full_exactstir.out`)
- **Exact rate at or above 75%** (the constant happens to be right):
  - (7.3,11.6) reflected run: 972/1,024 (CR 57);
  - s8(8,12): 404/512 (CR 0);
  - shp(3,2): 179/256.
- **About 30-55%:**
  - (4,4): 544/1,024 (CR 140);
  - (7.3,11.6) direct: 314/1,023 (CR 60);
  - hold(7.3,11.6): 308/1,024 (CR 0);
  - s8(8.5,12): 199, s8(8,40): 196, s8(8.5,40): 163, s8(8,8): 147 of 512;
  - (4.5,4.5): 147/256 (CR 27);
  - (2,2): 145/256;
  - (12,20) over all x: 345/1,600 (CR 111).
- **Zero exact but 100% w15:** every (2.5,3.5) block. The model sits 4.5-5.0 units low with sd 0.6-1.2, and the
  hold-out block matches. The same holds for (1.5,1.5), (6,1.5), (1.9,3.3) and (3.5,2.5).
- **Poor:**
  - (100,120): 46/1,024, sd 11 units;
  - (30.7,1.7): 0/1,023, sd 6.6 units;
  - (40,1.03): 13/1,024, though 1,008 are w15 against CR's 0 (Excel is 17 units off CR there).

## 4. Structure ceiling with a free shape constant (`s2_scan.py`, `run_s2_all.sh`, `s2_all_*.out`)
E = fl(C + X), with C fitted on the even half and scored on the odd half.
- "x87" is the section 3 X: x87 stored ops, Excel LN, extended add, cut 0.2.
- "exact Loader" is mpmath with R's cut 0.1.

| block | x87: even / hold-out | exact Loader: hold-out | CR exact (whole block) |
|---|---|---|---|
| (2.5,3.5) x 0.21 (p log) | 343/504 / **330/519** | 241/519 | 20/1,023 |
| (2.5,3.5) x 0.33 (both series) | 433/503 / **438/521** | 435/521 | 0/1,024 |
| (2.5,3.5) x 0.61 (both log) | 405/509 / **418/515** | 198/515 | 82/1,024 |
| (2.5,3.5) x 0.29 (both series) | 307/502 / 326/522 | 326/522 | 0/1,024 |
| (2.5,3.5) x 0.42 (both series) | 451/533 / 415/491 | 418/491 | 0/1,024 |
| (7.3,11.6) x 0.61 (both log) | 478/506 / **494/518** | 35/518 | 57/1,024 |
| (7.3,11.6) x 0.27 (both series) | 200/471 / 194/552 | 162/552 | 60/1,023 |
| (4,4) x 0.23 (both log) | 370/505 / **378/519** | 75/519 | 140/1,024 |
| (30.7,1.7) x 0.81 | 83/536 / 69/487 | 53/487 | 51/1,023 |
| (100,120) x 0.40 (series) | 28/531 / 19/493 | 12/493 | 28/1,024 |
| (40,1.03) x 0.95 | 143/512 / 126/512 | 99/512 | 0/1,024 |
| probe1 (2.5,3.5), all x | 423/806 / **423/794** | 284/794 | 133/1,600 |
| probe1 (12,20), all x | 235/781 / **215/819** | 75/819 | 111/1,600 |

Where bd0 takes its log form, the x87 structure is 80-96% bit-exact on the hold-out half. Where both sides take
the small-ratio branch it is 37-85%. The structure is the same on both sides of the mode and of the mean.

## 5. What is still open (measured)
1. **The per-shape constant** (`c2_const.py`, `c2_const.out`). The needed exponent offset is measured against the
   exact constant, then compared with LDe's own constant error.

   | shape | needed | LDe |
   |---|---|---|
   | (2.5,3.5) | +2.9 to +3.4, the same on all 22 blocks | -1.6 |
   | (3.5,2.5) | +3.46 | -1.6 |
   | (4,4) | -2.4 | -3.0 |
   | (7.3,11.6) | -1.7 to -3.1 | -3.3 |
   | (11.6,7.3) | -2.2 | -3.3 |
   | (12,20) | -1.1 | +1.6 |
   | (100,120) | -0.6 | -3.3 |
   | (40,1.03) | -24.4 | -28.8 |
   | (1.03,40) | +8.5 | +1.3 |
   | (2.5,2.5) | +6.3 | -1.4 |
   | (1.5,1.5) | +4.1 | -1.3 |
   | (1.9,3.3) | -5.5 | -2.0 |
   | (8,12) | -3.3 | -3.2 |
   | (8,40) | -0.6 | -0.5 |

   Rejected as the source:
   - -TOMS betaln (Excel GAMMALN and LN) + k ln(k/n) + nk ln(nk/n);
   - -(GL(a) + GL(b) - GL(a+b)) + the same, with exact or x87-stored logs;
   - GAMMALN-based stirlerr (section 2.5).

   The constant is symmetric under (a,b) ↔ (b,a) at (2.5,3.5) and (7.3,11.6), but not at (40,1.03) ↔ (1.03,40).
2. **Noise in the small-ratio branch.** Excel's error has sd 0.8-1.3 units on the `cut±0.30..0.45` and x 0.29 runs.
   The exact series predicts sd at the floor, about 0.4. So that branch has visible roundings that are not
   modelled.
   - The matched filter (`d13_match.py`) repeatedly correlates the residual with the rounding of fl((a+b)·x): |rho|
     0.13-0.36, with the sign changing at the mode.
   - The direct substitutions np = fl(fl((a+b)x) - 2x), and the same for nq (`npmode=sx2/sx2q`), score lower:
     251/522 against 326/522 at x 0.29.
3. **Large shapes.** At (100,120), (30.7,1.7) and (12,20), Excel's error has sd 6-21 units, against 2-11 left by
   the model. Stored intermediates of magnitude 10-100 (series terms, n·x at n = 218) are not resolved.
4. **(4,4).** The q-tie response is 0, not -1.61 (section 2.1).

## 6. Negatives (same scorer)
- **Rejected by the tie response** (section 2.1):
  - brcomp/(x·y) with TOMS logs (prediction -1);
  - the (a-1)·ln x + (b-1)·ln y log form (prediction b - 1);
  - the TOMS a0 >= 8 Stirling brcomp, whose lambda uses x only;
  - Loader on shapes (a, b) instead of (a-1, b-1).
- **bd0 internals:**
  - R's (t + M) - k summation order (p.add coefficient about 0);
  - np/k as the log ratio (p.div coefficient 1 on k/np);
  - R's cut 0.1;
  - an |e| <= 0.375 cut.
- **Exponent chains** (`s3_lc.py`, `s4_chain.py`, `s5_hfine.py`):
  - The value-space forms fl(EXP(lc)·K) and fl(EXP(lc)/K) score 252-277 against 334-349 for the E-space chain at
    x 0.33.
  - The R chain E = fl(LN5 + fl(lc - h)), with h scanned over doubles or ulp/8 steps, does no better than
    E = fl(C + X): 356 against 330 at x 0.21, 448 against 438 at x 0.33, 358 against 326 at x 0.29.
  - Its best h moves between blocks (h0 - 3.1 to -4.0 ulp), so the chain is not resolved.
- **Small-ratio bd0:**
  - Excel's log1p series (r2-log1p) as ln(k/np), entered at a = d/np (`series=lp`, 355/522) or at v
    (`series=lpv`, 371/522), against Loader's series in extended (358/522);
  - Loader's series in x87 stored ops (`sar=x`): equal or lower.
- **stirlerr:**
  - the Excel-GAMMALN formula (x87 stored or extended), LN(GAMMA), GL(v) + LN v and w6's table plus formula
    (`c5_stirvar.out`);
  - LF1 (log1p form) instead of LF0 was not scored on the sweeps. On the fixed-shape constants it is no closer
    (`c2_const.out`, column LDg1 against LDg).
- **Inconclusive by construction** (recorded so nobody repeats them): `d8_ereg.py`, `d11_reg2.py` and
  `d15_chainreg.py` regress on roundings of quantities downstream of the unknown constant, such as E, lc - h and
  C - b. A constant that is off by about 3 units decorrelates those features. Use free-constant scans instead.

## 7. Next hypotheses (most promising first)
1. **Pin the constant with ulp sweeps of a and b at many shapes.** The constant should be ln(a+b-1) + Cs - lf/2
   with Excel LN pieces. Run x fixed at a log-form x and step a and b by single ulps, at shapes with
   non-half-integer k and n-k (for example (5.3, 9.7), (3.7, 2.9), (20.3, 30.1)).
   - The 512-cell fingerprint then decides between stirlerr implementations: a Stirling series below 15, a
     different table, a TOMS bcorr-style del(v), or a gamln1-based value.
   - It also decides between term orders of lc and E.
   - Start from exact stirlerr. The fingerprint already correlates at rho 0.85 at (7.3,11.6) once LN(a+b-1) and
     LF0's LN roundings are included.
2. **The small-ratio branch.** Probe runs at v just below 0.2 for several shapes, with n not a power of 2 so that
   fl(n·x) rounds. Then test series forms: R's ej/(2j+1) against r2-log1p's c_k = RN(1/(2k+1)) multiplies, a
   fixed term count, partial sums stored, and d·v stored. Check whether fl((a+b)·x) enters, as the matched
   filter suggests.
3. **(4,4) and integer shapes.** Test nq = n - fl(n·x) against n·fl(1-x) at several integer shapes. The w6
   D-LF0 148/300 hit at (4,4) fits this.
4. **Large shapes** ((100,120), (30.7,1.7)). The x87 stored series with n·x at n = 218, and stored d, v and s.
5. **Then the cdf.** w4 showed that L140's brcomp is Excel's density × x·y, and bfrac at min >= 40 shares the pdf.
   LDe with the right constant is therefore the prefactor for those branches.

## 8. Scripts
| script | what |
|---|---|
| `dlib.py` | Data loader (a, b > 1, plus own probes), Excel LN/EXP/GAMMALN (by import plus a local cache), rounding modes, tally |
| `loader.py` | Loader bd0 / x-part with pluggable arithmetic, cut, series variants, np/nq forms |
| `model.py` | Full models (LDe / LDt): shape constants and chain |
| `make_probe.py` | Builds the p1 probe case and its meta |
| `d1_stats.py`, `d2_cover.py` | Per-block error stats; constant-offset coverage ceilings |
| `d3_seq.py`, `d7_eset.py`, `d9_resid.py`, `d10_chainres.py` | Run listings: Excel error, admissible E sets, residuals |
| `d4_regress.py`, `d5_yresp.py`, `d14_apbx.py` | Regressions on LN, y-tie and (a+b)x roundings; the tie-response table |
| `d6_bd0reg.py` | Regression on bd0 log-form roundings (`d6_bd0reg.out`, `d6_cut.out`) |
| `d12_spec.py`, `d13_match.py` | Line spectrum; matched filter over about 50 candidate intermediates |
| `d8_ereg.py`, `d11_reg2.py`, `d15_chainreg.py` | Inconclusive chain regressions (see section 6) |
| `s2_scan.py`, `run_s2_all.sh` | Free-constant structure scan with hold-out |
| `s3_lc.py`, `s4_chain.py`, `s5_hfine.py` | Exponent-chain variants |
| `c1_need.py`, `c2_const.py`, `c3_sweep.py`, `c4_sweepreg.py`, `c5_stirvar.py` | Shape-constant analyses |
| `m1_score.py`, `sumtab.py` | Full-model scoring per block and per subset |
| `ctl1.py` | Positive controls |
