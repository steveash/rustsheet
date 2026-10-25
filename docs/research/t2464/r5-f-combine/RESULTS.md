# T-2586 r5-f-combine: M-sd's constant inside r4-f-power's stored sum S2, and K_hi's phase

Worker r5-f-combine, round 5, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: absolute in a logarithm, relative in the density.
a = d1/2, b = d2/2, h = a + b, m = d1, n = d2, mx = fl(m·x), side 1 is mx <= n, r = fl(mx/n) (side 1) or
fl(n/mx) (side 2), s = fl(1 + r), L = Excel's log1p (LN(s) for r > 0.375), P = +fl(a·LN x) | -fl(b·LN x),
H = -h·L(r) (extended), K' = exact K, K_sd = M-sd's constant (r4-f-small-df `sdmodel`). Run every script from this
directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline (negative)
- **The requested combination does not work.** S2 = fl(fl(K_hi + H) + P), E = S2 + (K_sd - K_hi), K_hi = fl(K_sd)
  (`final_score.value(r, "hkp-sd")`) is below M-sd on every hold-out within 15 digits:

  | set | CR | M-sd | M-sdm | **hkp-sd** | hk-sd | dd (r4) | hyb |
  |---|---|---|---|---|---|---|---|
  | **chold (this dir, untouched, 1,536)** | 41/412/604 | 287/1,139/**1,280** | **358**/1,135/1,275 | 282/**1,146**/1,226 | 282/1,146/1,226 | 114/526/700 | 336/1,124/1,223 |
  | myhold (r4-f-small-df, 1,344) | 104/688/882 | 314/919/1,029 | **335**/918/**1,030** | 262/899/954 | 276/**934**/1,003 | 125/768/949 | 328/900/1,001 |
  | hold4 (r4-f-brcomp, 1,536) | 111/646/904 | 287/**1,048**/1,198 | 309/**1,048**/**1,199** | 327/1,014/1,129 | **330**/1,021/1,133 | 94/825/949 | 325/1,025/1,141 |
  | power-hold (r4-f-power, 1,536) | 14/117/294 | 29/355/733 | **32**/352/730 | 20/**383**/648 | 20/381/**740** | 16/276/483 | 20/383/648 |

  In sample it is worse too, except on r4-f-power's own probe at large |c·ln x| (table in section 4.6).
- **K_hi's phase is not the missing piece.** With K_hi free (every double within ±1 grid of K', or a continuous
  phase) and the remainder free per run, HKP11 reproduces Excel on **0 of 330 runs** (all of them; section 4.1), at
  every shape size, both sides, both log1p branches. The per-run best is 52-73% of the cells. The positive control
  (a planted HKP11 chain on the same x runs) is recovered 40/40 on every run.
- **The first-order error structure is right.** The local regression of Excel's admissible exponent on the
  rounding errors of LN(x), of the product fl(c·LN x), of r, of s and of LN(s) gives median coefficients
  **1.00, 1.00, 1.00, 1.00, 1.01** over 160 runs (`s3_ca.out`). What is left is noise:
  - 0.4-1.0 units at small and mid shapes for r ≈ 0.83, against 0.3-0.5 from the interval widths;
  - 1-4 units at r ≈ 0.45-0.5 (s ≈ 1.45-1.5) at small shapes;
  - 5-110 units at large shapes (the stored-term quanta).
- **The noise is there with every product exact.** At (8,8) (a = 4, h = 8) and (2,2) (K' = 0, a = h/2 = 1),
  fl(c·LN x) and h·LN(s) are exact. Excel still scatters by about ±1.5 units around any one constant: 24-34 of 40
  per run at (8,8), 19-24 of 32 at (2,2). It is **not** fixed by:
  - any LN model for x or s (Excel's LN, LN64, CR, exact);
  - any final step (div, mulr, dbl, lnx, lnx64);
  - any one-constant stored chain;
  - any ordering of stored or extended Loader pieces with no free constant, the best of which is 43 of 64 at (2,2).

  So at least one more rounding sits somewhere this round's families do not reach (section 5).

**Model to import: none of this round's beats M-sd.** The best is still r4-f-small-df's
`sdmodel.value(x, d1, d2, "div")` (M-sd) or `sdmodel.value(x, d1, d2, "mulr")` (M-sdm, the most bit-exact cells).
This directory's frozen variants are `final_score.value(row, model)`, with model one of `"hkp-sd"`, `"hk-sd"` or
`"hyb"`; a row is a dict with x, d1, d2 (and cr for "CR").

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph). Both were built by `mkprobe.py` and filled on 2026-10-24 before any
  analysis. x is an exact two-piece formula, and `mkprobe.rows` asserts every echo (0 misreads).

  | case | cells | content |
  |---|---|---|
  | `t2586-r5-f-combine-a` (`ca`) | 12,800 | in-sample: 40 shapes (small (5,5)-(12,12); mid (12,12)-(45,45), (14,20), (22,40); large (56,56)-(400,400), (64,200), (150,250), (300,100), (36,140) and swaps) × 4 runs of 40 consecutive doubles at mx/n ≈ 0.5, 0.85 (side 1) and 1.2, 2.2 (side 2) |
  | `t2586-r5-f-combine-hold` (`chold`) | 3,072 | **hold-out**: 16 other shapes ((5,8) (8,5) (10,10) (13,17) (17,13) (26,26) (35,50) (50,35) (75,75) (96,160) (160,96) (220,220) (130,70) (70,130) (350,350) (11,30)) × the same 4 targets × 24. Filled with `a`; read once, by `final_score.py chold`, after the seven models (hyb included) were frozen |

  Total 15,872 cells. Every cell has r > 0.375.
- **Other data:** r4-f-small-df's `mya`, r4-f-brcomp's `big`, r4-f-power's probe `a` (as `case:t2586-r4-f-power-a`)
  and round 3's `a` were used in sample. `myhold`, `hold4` and power-hold were scored once with the frozen models
  (`final_other.out`), and no choice was made on them.
- **Method.** Each cell gets the interval of extended exponents E that reproduce Excel through the final step
  (`clib.e_interval`, bisection on r3-loader-bd0's EXP V6 to 0.001 units; final steps in `clib.final`). A structure
  is then fitted per run with its free constants by deepest interval overlap (`clib.best_overlap`), so "n of N"
  means n cells are reproduced exactly with one remainder per run.
- **Positive control** (`s1_scan.py ca --plant 3.3 4.7`): Excel's values are replaced by HKP11 with K_hi =
  fl(K' + 3.3 units) and K_tot = K' + 4.7 units. The scan recovers 40 of 40 on every run, with delta_tot 4.6-4.8,
  and the K_hi phase is recovered modulo the grid.
- **Two bugs found and fixed before the counts below.**
  - s3: the product error of P used abs(P), which zeroed half the regressions.
  - s4: E0 - S + Q was converted to a float before subtracting K. At K ≈ 3-300 that loses 16-512 units and fakes
    40/40 fits at coarse binades.
  - The committed `s4_qscan.py` and `s3_reg.py` are the corrected ones. `s4`'s first (wrong) output is not kept.
- **Reuse, by import only:**
  - r4-f-small-df `slib` (loaders, Excel LN, log1p, EXP V6, side/r) and `sdmodel` (K_sd, M-sd);
  - r4-f-brcomp `fblib._load`;
  - r3-fdensity `mkprobe.row_cells` / `case_doc` and `flib` (exact F, stirlerr_exact, tally);
  - r3-loader-bd0 `interval`, `LN64`, `exp_v6`;
  - r4-f-power `fmodel` (dd) and `cands.errs`.
- **Housekeeping.** `.gitignore` covers `*.pkl` and `__pycache__/`; the interval cache is `iv_cache.pkl`. No .py
  file is over 190 lines. `scripts/check_duplication.py`'s scan with this directory's .py files added gives 3,103
  duplicated lines, the tracked-only baseline. The directory is about 220 KB without caches.

## 3. Structure facts measured here
1. **The error coefficients** (`s3_reg.py`, 160 `ca` runs, 94 `mya` runs) are all about 1: xP (c·eLN x), pP (P's
   product rounding), rr (r's rounding), ss (s's rounding) and ls (h·eLN(s)). So M-sd's x-structure holds to first
   order in the log branch, at every size.
2. **E is not a stored double.** 4,313 of 6,400 `ca` intervals contain a double, where 4,321 are expected by chance
   from the interval widths (`s5_storedE_ca.out`). Runs at large |E| contain none.
3. **At large shapes the residual is quantised.** At (80,80) side 1, x ≈ 0.51:
   - Excel's E moves in multiples of 32 units between consecutive x (the grid of P).
   - With H stored as fl(h·L), the needed constant falls in clusters exactly 32 units apart (one ulp of
     fl(K + fl(H)) ≈ 23), plus a ±1.5-unit scatter.
   - With H extended, the clusters sit about 8.4 units apart. Mod 32, H lies on an 8-unit lattice (80·ulp(LN s) =
     40 units).
   - No single stored sum reproduces both the clusters and the scatter (sections 4.3-4.5).
4. **At power-of-two shapes there is unexplained scatter.** Within one s plateau (same H) at (80,80), the needed
   constant drifts by +0.42 units per x step and falls back by about 1.1. This comes from an inline inspection with
   `clib` (fl(Q + H) + P at Q = K' - 55 units, the best continuous phase of 4.5), not kept as a script. That
   is a smooth-in-x term or a fine-grid rounding that no candidate covers.

## 4. Hypotheses and counts (negatives included, decision 0056)
### 4.1 HKP11 with K_hi free (`s1_scan.py`; free K_hi doubles over ±max(grid), free remainder per run)
| set | runs | runs fully fit | cells at the per-run best |
|---|---|---|---|
| ca (log branch) | 160 | 0 | 3,542/6,400 |
| mya (log) | 94 | 0 | 1,721/3,008 |
| big (log) | 29 | 0 | 833/1,392 |
| r3 a (log) | 23 | 0 | 580/1,104 |
| power-a (log) | 7 | 0 | 834/1,568 |
| power-a (series, x ≈ 0.37, 13, 100) | 17 | 0 | 1,919/3,808 |

- On r4-f-power's best runs ((100,50) x ≈ 100, where r4 reported sd 0.7 units), HKP11 reproduces only 163 of 224
  cells with any K_hi.
- fl(K') and fl(K_sd) reach the per-run best on about half the runs (the `tK`/`tSD` columns). That is r4's "phase
  right on about half the runs", but even the best phase leaves 27-48% of the cells.

### 4.2 Other orders and storages with a free constant (`s2_variants.py`, `s5_flex.py`; ca, 6,400 cells)
| structure | cells |
|---|---|
| ext (M-sd, all extended) | 2,580 |
| **hk / HKP10: fl(K_hi + H) + P** | **3,581** |
| hkp / HKP11 | 3,542 |
| hk_hs, hkp_hs (fl(h·L) inside) | 2,581, 2,576 |
| hs (P + fl(h·L)) | 2,497 |
| ph, hsp (fl(P + H)) | 2,062, 1,848 |
| kph (fl(fl(K + P) + H)) | 1,917 |
| Hab (fl(a·L) + fl(b·L)), any order | 2,779-2,784 |
| He, every order with a stored partial sum after P (01 patterns) | 2,729 |

0 runs fully fit with any of them.

### 4.3 Final step, with ext and a free constant (`s2_variants.py ca ext FIN`)
| final step | cells |
|---|---|
| div | 2,580 |
| mulr | 2,589 |
| dbl | 2,486/5,889 (511 unreachable) |
| lnx | 2,531 |
| lnx64 | 2,579 |

None changes the picture.

### 4.4 Free stored-sum magnitude (`s4_qscan.py`, corrected)
- Setup: T = fl(Q + H) with |T| in each binade 2^-3 to 2^9 and Q's phase scanned (32 steps per ulp of T), in the
  forms hk, hkp, pk, phk and phsk.
- At (6,9), (80,80) and (400,400) the best is the smallest binade, which is effectively extended (23-37 of 40).
  Coarse binades are worse.
- At (2,2) every form gives 17-24 of 32.

### 4.5 Brute force on single runs
- **`s6_brute.py`**, on (80,80) s1, (6,9) s2 and (400,400) s2. The setup:
  - one continuous constant Q placed anywhere;
  - summands from {H, fl(h·L), P, -LN x, -LN64 x};
  - every order and flag pattern;
  - plain EXP when ln x is inside the sum.

  The best is QHP10 (fl(Q + H) + P), 82 of 120 (`s6_ca3.out`).
- **`s6` at (2,2)**: the best is 24 + 24 of 64.
- **`s7_pieces.py` at (2,2)**, with no free constant. ln F is the sum of ST, A, B, P and H:
  - ST: stirlerr as the nearest doubles, either one stored difference or three summands;
  - A = (h - 1/2)·L(fl(w/o)), stored or extended;
  - B = -ln √(2π);
  - P and H as above, plus -LN x as an option.

  All orders and flags were tried. The best is 41 of 64 with everything extended, and 43 of 64 with stirlerr split or
  ln x inside. CR gets 30.
- **(2,2), LN variants** (free constant): LN, LN64, CR or exact for x and for s give 23-24 of 32 on side 1 and 19-22
  on side 2.
- **(2,2), value-space splits** (EXP(P)·EXP(H), double or extended EXP, stored H, stored E; no free constant): 15-18
  of 32 per side.

### 4.6 Frozen models in sample (`final_insample.out`)
| set | M-sd | M-sdm | hkp-sd | hk-sd | dd |
|---|---|---|---|---|---|
| ca (6,400) | 1,281/4,472/5,015 | **1,388**/4,472/**5,016** | 1,148/4,112/4,969 | 1,173/4,112/4,969 | 241/2,306/3,281 |
| mya (6,016) | 1,695/4,623/**5,060** | **1,865**/4,624/5,058 | 1,534/4,410/4,944 | 1,754/4,431/4,948 | 718/3,904/4,779 |
| big (2,304) | **156**/1,302/**1,700** | 153/1,304/1,698 | 149/**1,318**/1,678 | 149/1,318/1,678 | 48/557/1,204 |
| power-a (5,376) | 198/1,607/2,546 | 215/1,618/2,550 | 232/**2,050**/**2,822** | 232/1,808/2,575 | **676**/1,477/2,334 |

- hkp-sd wins only on power-a, the large shapes with |c·ln x| up to 230. There it has 2,400 of 4,480 against M-sd's
  2,020 at max(d) > 60.
- `hyb` (hkp-sd for max(d) > 60, else M-sdm) was frozen on that evidence. It **fails on the hold-out**: 405 of 672
  against M-sd's 461 for max(d) > 60 on chold, and the same loss on myhold and power-hold. The S2 rounding helps
  only far from the mode, and not in the log branch.

## 5. Open (most promising first)
1. **The ±1.5-unit scatter at exact-product shapes** ((2,2), (8,8)) is the cleanest target. It survives:
   - every LN model;
   - every final step;
   - every single stored sum;
   - every Loader-piece order with nearest-double stirlerr.

   Candidates not tried here:
   - Excel's internal exp is not EXP V6 on an extended argument, for example an exp2 split with its own stored
     t = E·log2 e. This would shift every interval; test it on (2,2) side 1, where E = LN x - 2·LN(fl(1+x)) has no
     other rounding.
   - The density is assembled in value space from more than one EXP or POWER: x^a and (1+r)^-h as x87 pow
     (FYL2X/F2XM1) with their own roundings.
   - Stirlerr values other than R's table.
2. **The within-plateau drift** of +0.42 units per x step at (80,80) points at a term that depends smoothly on x and
   that the model misses or mis-scales, for example ln x entering twice with different roundings (fl(a·LN x) and an
   exact or LN64 ln x), or an mx-based term. A per-run regression on (x - x0) and on the frac of LN(x)/ulp would
   test it.
3. **Large shapes, far from the mode.** S2 = fl(S1 + P) is real there: hkp-sd gains about 380 w15 cells on power-a.
   But it is not bit-exact. The structure is conditional, perhaps a different code path for |ln x| large or for
   x > some cut, which needs a probe sweeping x from 1 to 100 at one large shape.

## 6. Scripts
| script | what |
|---|---|
| `clib.py` | loaders (`ca`/`chold` + slib's), pieces P/H, final steps, cached interval inversion `e_interval`, runs, overlap |
| `mkprobe.py` | the two probes (`--write`) and `rows(name)` |
| `s1_scan.py` | HKP11 with K_hi scanned per run (`--plant` control, `--branch`) |
| `s2_variants.py` | chain variants and final steps with a free constant |
| `s3_reg.py` | per-run error-coefficient regression |
| `s4_qscan.py` | free stored-sum magnitude scan |
| `s5_flex.py` | order × storage × H-form search with a K_hi j-scan |
| `s6_brute.py` | continuous-phase brute force on single runs |
| `s7_pieces.py` | Loader-piece orders with no free constant |
| `final_score.py` | frozen models (`value(row, model)`) and scores |

Outputs: `s1_*.out`, `s2_ca.out`, `s3_ca.out`, `s3_mya.out`, `s5_ca.out`, `s5_storedE_ca.out`, `s6_ca3.out`,
`final_insample.out`, `final_hold.out` (the one hold-out read), `final_other.out`.
