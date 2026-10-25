# T-2464 r2-log1p: Excel's log1p (ln(1-x) for x <= 0.375)

Worker r2-log1p, round 2, 2026-10-23. Method: isolate the ln(1-x) that BETA.DIST's density and BINOM.DIST(0,n,p) use,
pin it to the bit per x by interval inversion of closed-form cells, then fit implementations with exact x87 stored
ops. Counts are bit-exact / within 4 ulp / within 15 significant digits unless a table says otherwise.

## 1. Result

**Excel's ln(1+a) for -0.375 <= a < 0 is the atanh series in TOMS 708 alnrel's variable, summed forward.** Every op
is an x87 op stored to double, fl53(fl64(.)) (`excel_log1p.py`):

```
r  = a / (a + 2)          the quotient is double-rounded (x87): fl53(fl64(a / fl(a+2)))
t2 = r * r
p  = r;  s = 0
for k = 1..14:  p = p * t2;  s = s + p * c_k      c_k = RN(1/(2k+1)), a double; terms added largest first
l  = 2 * (r + s)
```

For a < -0.375 (x > 0.375) it is Excel's LN(fl(1+a)). The switch is TOMS brcomp's: x <= 0.375 takes the series.
At x = 0.375 the two forms give the same double, so `<` and `<=` cannot be told apart. The next double above 0.375
already takes LN.

| data (pinned l, section 2) | n | model | CR log1p (LN(fl(1-x)) above 0.375) |
|---|---|---|---|
| all pinned rows (runs, grid, hold, tiny) | 16,027 | **16,000** / 16,027 / 16,027 | 10,971 / 16,027 / 16,027 |
| even rows / odd rows | 8,014 / 8,013 | 7,997 / 8,003 | |
| `hold` case (own grid, not used for any choice) | 4,882 | 4,875 | 3,393 |
| x < 1e-4 (runs, hold, tiny) | 747 | 747 | 549 |
| x in [1e-4, 0.05) | 2,632 | 2,632 | 1,713 |
| x in [0.05, 0.15) | 4,177 | 4,176 | 2,905 |
| x in [0.15, 0.25) | 3,986 | 3,976 | 2,707 |
| x in [0.25, 0.375] | 4,085 | 4,069 | 2,697 |
| x in (0.375, 0.6), LN(fl(1-x)) | 400 | 400 | 400 |
| **out of sample: every BINOM.DIST(0,n,p) cell in ../cells.tsv** (general n, = EXP(fl(n*l)), per w5) | 345 | **345 / 345 / 345** | 311 / 331 / 339 |
| the same, p <= 0.375 | 170 | **170** | 136 / 156 / 164 |

- Within 4 ulp the model fits every row; in fact every miss is exactly 1 ulp off. The 27 misses are all at -r > 0.05.
  Most of them are exact ties of r + s that Excel rounds the other way (section 4). The maximum error is 1.0 ulp.
- The cells.tsv rows were never used for fitting. They have arbitrary n (not powers of two) and include w5's p = 0.3
  readout ("one ulp below CR").

**Downstream: w1's density model M1 with this log1p** (`dens_m1.py`; M1 is otherwise unchanged):

| subset | n | M1 with CR log1p | M1 with Excel log1p (even / odd) |
|---|---|---|---|
| probe1, modelled shapes, x <= 0.375 | 5,400 | 4,084 / 5,305 / 5,365 | **4,402 / 5,399 / 5,399** (2,192/2,704, 2,210/2,696) |
| probe1, modelled shapes | 12,800 | 10,057 / 12,705 / 12,765 | **10,375 / 12,799 / 12,799** |
| old pdf cells, modelled shapes | 1,107 | 562 / 1,094 / 1,100 | 560 / 1,096 / 1,100 |

On probe1 the modelled density is now within 15 digits on 12,799 of 12,800 cells. The old cells do not move: their
misses come from other parts of the composition (lnB, the exponent sum).

## 2. Probes and pinning

New oracle cases (excel-graph, filled with `rustsheet-oracle fill --case ...`; the sidecars are next to them). Each
case is in `eval/suites/probes-numeric/cases/`:

| case | rows | content |
|---|---|---|
| `t2464-r2-log1p-pilot` | 195 | p = 0.1, 0.2, 0.3; runs of 64 consecutive doubles at 0.3, around 0.375 and at 0.0123456789 |
| `t2464-r2-log1p-runs` | 5,328 | 16 runs of 256 consecutive doubles from irregular starts 0.0031415 + 0.0230137 i; 400 doubles centred on 0.375; 160 around each of 2^-2, 2^-3, 2^-4, 2^-5; 64 each at 1e-3, 1.37e-5, 3.1e-8 |
| `t2464-r2-log1p-grid` | 5,450 | p = 0.000137 + 0.0000713113 k (k < 5,250), plus 200 points in (0.375, 0.5) |
| `t2464-r2-log1p-hold` | 5,400 | p = 0.00041 + 0.0000698713 k (k < 4,400), plus 1,000 log-spaced p in [1e-12, 0.025] |
| `t2464-r2-log1p-hix` | 5,264 | x in (0.625, 1): BETA.DIST(x,1025,1,FALSE) and BETA.DIST(x,1025,1.5,FALSE) |

Columns: A = p as an exact-double formula `(hi*2^e1+lo*2^e2)` (all echoed back bit-exact), B = BINOM.DIST(0,n,A,FALSE)
and C = BETA.DIST(A,1,n+1,FALSE). Here n = 2^k with n·|l| in [256, 512), so n·l is exact. The script is
`make_probes.py <name>`, and `make_probe_hi.py` builds hix.

**Pinning (`pin.py`).** B = EXP(n·l) (w5: BINOM.DIST(0,n,p) = EXP(fl(n·l)), 187/187). For each p, the doubles l
within ±6 ulps of CR log1p(-p) were put through Excel's EXP (harness, model V6). With n = 2^k, adjacent l are about
2^8 output ulps apart, so the inversion is unique.
- runs: 5,264 pinned, plus 64 #NUM!. BINOM.DIST returns #NUM! for n >= 2^31, so p < about 5e-7 needs another column.
- grid: 5,450 pinned.
- hold: 4,882 pinned, plus 518 #NUM!.
- No row gave two candidates, and no finite row gave none.

**Density cross-check (`dens_check.py`).** For each n, ln(C) - n·l must be the same constant on every row (to
1e-16) when l is right, and a wrong l moves it by n·ulp(l) >= 7e-15. All 15,596 rows sit on their group's constant,
so **BETA.DIST's density uses the same ln(1-x) as BINOM.DIST**.
- The constant is LN(b) (lnB = -ln b for a = 1).
- EXP(fl64(n·l + LN(n+1))) reproduces 11,730 of the 15,596 C cells to the bit (`dens_model.py`). A stored exponent
  reproduces 35.
- `dens_model.py` uses that model to pin 431 of the 582 #NUM! rows (`tiny.pins.tsv`), for p down to 1e-12.

**Positive control (`control.py`).** BINOM cells were synthesised from a planted kernel on the 9,880 grid and hold
points in [1e-4, 0.375] and run through the same inversion and scorer:
- The forward sum, planted, is recovered 9,880/9,880. The Horner variant, also planted, is recovered 9,880/9,880.
- The two kernels disagree on 55 rows (0.56%). So the data separates structures at the 0.2% level, and the
  pipeline adds no misses of its own.
- `x87f.py` is an integer-exact fl53(fl64(a op b)). It agrees with `excelmath.x87` on 12,000 random ops.

## 3. How the function was identified (in order; all scripts re-run from this directory)

1. **No published log1p fits** (`score_impls.py`, `score_impls.out`). The table gives exact / within 1 ulp / n per
   p band; rows are the pinned runs, grid and hold cases.

   | candidate | [0.001,0.0625) | [0.125,0.25) | [0.3,0.375) | (0.375,0.6) |
   |---|---|---|---|---|
   | correctly rounded | 1,948/2,996 | 3,422/4,982 | 1,339/2,125 | 225/400 |
   | fdlibm s_log1p | 1,943 | 3,449 | 1,338 | 210 |
   | Cephes log1p | 1,941 | 3,428 | 667 | 400 |
   | TOMS alnrel (3,3) rational, double or x87 | 1,877 | 2,675 | 387 | 29 |
   | x87 FYL2XP1 | 1,947 | 3,421 | 1,339 | 225 |
   | Kahan's two forms | 1,718 / 1,874 | 2,937 / 3,126 | 1,162 / 1,165 | |
   | log(1+t) and LN(fl(1+t)) | 108 | 1,162 | 667 | **400** |
   | atanh series summed from r (w1) | 1,921 | 2,409 | 886 | |

   - The TOMS alnrel rational is 2.2e-16 relative off the true value at t = 0.23. That is 4,000 units of 2^-64, far
     outside Excel's error band.
   - The atanh series is the only candidate that fits the smallest p: 59/59 below 1e-6.
2. **The output is a function of the stored r = fl(a/(a+2)).** The candidates `ideal2atanh_*` (in `cands.py`) take
   RN(2·atanh(r)) with r stored in four ways:

   | r stored as | score |
   |---|---|
   | double | 99.3% (2,087/2,125 in [0.3,0.375)) |
   | extended (fl64) | the CR level |
   | double sum, extended quotient | 1,559/2,125 |

   - `resid.py` and `epsband.py` invert each row to the band of correction error it allows. The residual is per-row
     noise of about ±50·2^-64 relative, not an approximation-error curve.
   - So the correction is computed in double, and its own roundings set the misses.
3. **The quotient is x87 double-rounded.** Two rows (p = 0.1412237000000012 and 0.2264790662) needed a correction
   error of 8 or more ulps of S. Both are rows where fl53(fl64(a/s)) differs from the IEEE quotient by 1 ulp (`kset.py`).
   - With the x87 quotient, every row admits some S (`kset.out`).
   - The x87 final add fixes the 2 near-tie misses below r = 0.02, which the double add misses
     (`approx_scan.py`: 2,387/2,387 against 2,385/2,387).
4. **Outer form `2r + c` against `2r·W`** (`struct_scan.py`, all scores out of 15,197):
   - Any "2r + correction" form with a correctly rounded correction scores about 15,121.
   - Every "2r·W" form, with W stored, scores 9,989.
5. **The tail is a forward power sum, not Horner** (`gen_scan.py`, `tail_scan*.py`, `sum_scan*.py`, `coef_scan.py`).

   Rows are p in [1e-4, 0.375] (14,881); "-r > 0.1" is the 6,914-row subset used by the later scans.

   | tail form | all 14,881 rows | -r > 0.1 (even + odd) |
   |---|---|---|
   | **Σ RN(1/(2k+1))·p_k, forward** | **14,854** | **6,888** |
   | Σ p_k/(2k+1) (divide) | 14,841 | |
   | Horner S = 1/3 + t2/5 + ..., any of 4 placements | 14,812 | 6,853-6,855 |
   | sum smallest first | 14,803 | 6,855 |
   | T1 + (rest) / (T1+T2) + rest / pairwise | | 6,854-6,857 |
   | r·Σ c_k t2^k or (r·t2)·Σ | | 6,836-6,839 |
   | coefficients held as extended reciprocals | | 6,879 |
   | 2·W, W = 1 + ... (Horner incl. 1) | 9,764 | |
   | fdlibm Lg1..Lg7 polynomial as 2r + r·R | 13,291 of 15,197 (IEEE r, `approx_scan.py`) | |

   - Truncation: N = 10 and N = 11 lose rows at the top of the range. N >= 12 is identical; 14 is used.
   - A convergence stop (|term| <= eps·|s|, eps 1e-17 to 2.2e-16) does not help.
   - Excel's own c_1 is RN(1/3) exactly (`coef_scan.py 1`, scanned ±6 ulp). The score peaks at 0 on both halves:
     even 3,441, with 3,420 at +1 and 3,416 at -1; odd 3,447, with 3,429 and 3,427. c_2 and c_3 are insensitive at
     the ulp level.
6. **Storage pattern** (`ext_scan.py`, `ext_scan2.py`, `ext_scan3.py`: every subset of t2, p, term, s and r+s held
   in extended, in both sum orders). Fully stored is best or within 5 rows of the best (the noise level). Every subset that keeps s
   extended loses 25-35 rows. So all ops are stored.

## 4. What is left (27 misses, 0.17%)

`miss_fwd.py` and `tail_cr.py` list them:
- 16 of the 27 are exact ties of r + s (the model's exact sum lies on a midpoint), which Excel resolves toward the
  true value.
- The other 11 sit 0.004-0.031 ulp past a midpoint.
- In each case Excel's s differs from the model's s by 1 ulp of s. The direction is not "more accurate": the needed
  s is closer to RN(atanh(r) - r) in 15 cases and not closer in 12. A correctly rounded tail scores 14,813, against
  14,854 for the model.
- So the remaining difference is in the low bits of s: the term order, how p_k is formed, or a coefficient's low
  bits beyond c_1. It is not in the structure.
- All 27 are within 1 ulp, so this gap is invisible at 15 digits.

## 5. Other findings and negatives
- **For x > 0.625 the density's ln x is LN(x), not alnrel(-y)** (`hix_check.py`). TOMS brcomp would take alnrel(-y)
  when y <= 0.375. On BETA.DIST(x,1025,1,FALSE):
  - LN(x) puts all 5,264 rows on one offset. The series at -fl(1-x) puts 4,041 of 5,232 there.
  - Excel's LN equals the correctly rounded ln x on every one of these x, so the two cannot be separated here.
  - This is the b = 1 path, which may be a closed form. Column C (b = 1.5, a >= 8, unsolved composition) is
    inconclusive: 2,937 against 2,756.
- **Excel's series inside TOMS algdiv makes the density worse** (`algdiv_try.py`). The test patched
  w1's toms_betaln alnrel, used with a positive argument a/b, into M1. Density cells with max > 8 and min <= 1:
  - probe1 drops from 2,314 to 1,086 of 3,200.
  - The old cells go from 224 to 213 of 606.
  - So either algdiv does not use this function, or its positive-argument branch differs. w1 and w6 reported
    algdiv as unresolved.
- BINOM.DIST(0,n,p) returns #NUM! for n = 2^31 and above (n = 2^30 still evaluates).
- The density's exponent at a = 1 is extended. EXP(fl64(n·l + LN(b))) gives 11,730/15,596 bit-exact, and a stored
  exponent gives 35. The remaining 25% is the density composition (lnB for b > 8 is TOMS algdiv(1,b), not exactly
  LN(b)), which this worker did not pursue.

## 6. Next hypotheses
1. **Close the last 0.17%.** Collect more near-tie rows: dense runs at p in [0.2, 0.375], where about 0.4% of rows
   miss. Then test the low-bit variants with the kset method (`kset.py`), which gives each row's admitted s offsets.
   Test them as a set, not one row at a time.
2. **The positive-argument alnrel.** algdiv and bcorr call alnrel(a/b) with a > 0. Probe it through cells whose
   lnB comes from algdiv, at shapes where a/b sweeps (0, 0.375], with a = 1 and b > 8. Then compare this series with
   TOMS's rational and other positive-a forms.
3. **Put this log1p into the cdf prefactor.** brcomp and bpser use the same ln(1-x) for x <= 0.375. Rescore w3's
   bpser model and w4's L140 with `excel_log1p.ln1m` in place of CR log1p.
4. **Engine:** `excel_log1p.log1p_series` is 14 stored multiply-adds. It is the drop-in for every `log1p(-x)` in the
   beta code at x <= 0.375 (statistics.md quirk entry to be written by the coordinator).

## Scripts
| script | what |
|---|---|
| `r2lib.py` | shared helpers: exact-double literals, case writer, oracle reader, cached Excel EXP/LN, BINOM inversion, x87 r |
| `x87f.py` | integer-exact x87 stored op fl53(fl64(a op b)) |
| `excel_log1p.py` | **the model** (`log1p_series`, `ln1m`) |
| `make_probes.py`, `make_probe_hi.py` | probe builders |
| `pin.py` | pins l per row → `*.pins.tsv`; `dens_model.py` pins the tiny-p rows → `tiny.pins.tsv` |
| `dens_check.py`, `dens_model.py` | density cross-check and density-only pinning |
| `cands.py`, `score_impls.py` | published log1p implementations (imports w1's `log1p_impls.py`) |
| `resid.py`, `epsband.py`, `sband.py`, `kset.py` | interval inversion of the correction (`resid`/`epsband` use the IEEE quotient, the first pass) |
| `struct_scan.py`, `approx_scan.py`, `gen_scan.py`, `tail_scan.py`, `tail_scan2.py`, `sum_scan.py`, `sum_scan2.py`, `coef_scan.py`, `coef_ext.py`, `ext_scan*.py` | structure scans |
| `miss_fwd.py`, `tail_cr.py`, `small_p.py` | miss analysis |
| `control.py` | positive control |
| `final_score.py` | the tables in section 1 |
| `dens_m1.py`, `algdiv_try.py`, `hix_check.py` | downstream density tests |

Each `*.out` file is the output of the script with the same name. Run them with
`/home/user/rustsheet/bindings/python/.venv/bin/python <script>` from this directory.
