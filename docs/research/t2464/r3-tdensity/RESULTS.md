# T-2513 r3-tdensity: Excel's Student t density T.DIST(t, n, FALSE)

Worker r3-tdensity, round 3, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: relative for the density, absolute for its logarithm E. CR is
the correctly rounded exact density at the double t. Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
**For df >= 3, Excel's T density is Loader's saddle-point `dt` (R 2.x structure) with Excel's x87 pieces. Three
things differ from R:**
- the log branch starts at x^2 > n, not 0.2 n;
- the 1/sqrt(2 pi (1 + x^2/n)) factor is applied as two stored operations;
- the shape constant carries the same per-half-integer stirlerr error that r3-loader-const fitted on BETA.DIST.

df 1 and df 2 are closed forms.

Model **LT1e** (`tdlib.lt1` with `tdlib.lt1e_offsets`):
```
xx = fl(t*t); q = fl(xx/n); w = fl(1 + q)                      x87 ops, each stored to a double
if xx > fl(1.0*n):   u = fl(fl(LN(w)*n)/2)                       (Excel LN; log1p_xl is the same here)
else:                u = fl(-bd0(n/2, fl(fl(n+xx)/2)) + fl(xx/2))
                     bd0 = r3-loader-bd0's Excel bd0: cut 0.2; log form fl(k LN(fl(k/M))) + (M - k);
                     series with v stored and its +delta(2kv) knob (`{"v":"x","ph":1}`)
E = fl(C - u);  ex = EXP_V6(E) stored
T = fl( fl(ex / fl(sqrt(w))) * fl(1/sqrt(2 pi)) )
C = exact Loader constant  -bd0(n/2,(n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2)
    + [e((n+1)/2) - e(n/2) + 2.07] units,   e = r3-loader-const grid_e.tsv (n <= 55; 0 beyond)
df 1: T = fl(1 / fl(pi * fl(1 + fl(t*t))))         df 2: T = EXP(fl(-1.5 * LN(fl(2 + fl(t*t)))))
```

| data | n | **LT1e** | LT1 (in-sample-fitted C for df 3,4,5,7,10,30; exact C elsewhere) | LT1, exact C | CR |
|---|---|---|---|---|---|
| cells.tsv, df >= 3 (never used for a choice) | 18,699 | **9,870 / 18,683 / 18,696** | 11,517 / 18,684 / 18,697 | 5,956 / 18,383 / 18,631 | 3,788 / 18,075 / 18,472 |
| cells.tsv, df 1 and 2 (closed forms) | 1,077 | **1,077 / 1,077 / 1,077** | | | 470 / 1,074 / 1,075 |
| **all 19,776 T density cells** | 19,776 | **10,947 / 19,760 / 19,773** | 12,594 / 19,761 / 19,774 | | 4,258 / 19,149 / 19,547 |
| **hold-out hold2** (untouched until the LT1e run) | 960 | **501 / 959 / 960** | | | 266 / 776 / 839 |
| hold-out hold (read once for frozen LT1, then post hoc for LT1e) | 1,252 | 701 / 1,241 / 1,222 | 703 / 1,204 / 1,228 | | 288 / 1,077 / 1,185 |
| in-sample probes a1 + a2 | 2,406 | 1,251 / 2,404 / 2,402 | 1,412 / 2,405 / 2,406 | | 531 / 2,280 / 2,326 |

- Visible parity: 3 of 19,776 cells are visibly off, against 229 for CR:
  - `T.DIST(±10, 1E6, FALSE)`: LT1e is -46 ulp from Excel and CR -36;
  - `T.DIST(8, 50, FALSE)` (t0969-t-beta-link J152): LT1e is -21 ulp and CR -5. df 50 takes an
    e-table constant near the table's edge, and t = 8 puts |E| near 31.
- Bit-exactness is still about 55-62% in all three branches. Section 5 has what is open.
- **The constant is a cross-validation, not a fit.** The per-df offsets LT1 needs are predicted by
  e(h) - e(n/2) + c0, using r3-loader-const's per-half-integer stirlerr corrections e(v). Those were fitted on
  BETA.DIST cells only. The single gauge constant c0 = +2.07 is fitted on the 6 in-sample dfs.
  - Fit: rms 0.43 units over 10 dfs (`s20_lt1e.py`; dfs 6, 8, 12 and 20 come from the hold-out's even half),
    against a spread of 1.25 in the offsets themselves.
  - The T density fixes that table's free gauge α = 2 c0 = +4.13 units per unit v, which BETA.DIST cannot
    identify (k + (n-k) = n cancels αv there; h - n/2 = 1/2 does not).

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph, all filled 2026-10-24; built by `mkprobe.py`; row maps in `<name>.meta.json`).
  - **Format.** Every t is an exact-piece formula (w2-forensics' `enc`, never a 15-digit literal). Column A's value
    is read back: all 4,366 t values match the intended doubles, with 0 parser misreads.
  - **Inner columns** sit beside each T.DIST cell. Column B is `T.DIST(A,n,FALSE)`, followed by:
    - `A*A`;
    - `POWER(1+A*A/n,-h)`, `POWER(n+A*A,-h)` and `POWER(n/(n+A*A),h)`;
    - `BETA.DIST(n/(n+A*A),n/2,0.5,FALSE)` and `BETA.DIST(A*A/(n+A*A),0.5,n/2,FALSE)`;
    - `F.DIST(A*A,1,n,FALSE)`.

  | case | cells | content |
  |---|---|---|
  | `t2513-r3-tdensity-a1` | 9,504 | in-sample: df 3, 4, 5 × runs of 96 consecutive doubles at t ≈ 0.3125, 1.7, 4.3 + an exact run t = 1.5 + k 2^-20 (64; t*t, n + t*t exact) |
  | `t2513-r3-tdensity-a2` | 12,150 | in-sample: df 7, 10, 30, the same blocks, + df steps 3..100 at t = 0.5, 0.9, 2 |
  | `t2513-r3-tdensity-hold` | 11,268 | hold-out, built and filled with a1/a2 before any analysis: df 6, 8, 12, 20, 3, 4 × runs of 48 at t ≈ 0.43, 1.23, 2.9, 7.7 + df 101..299 at t = 1.37 |
  | `t2513-r3-tdensity-hold2` | 1,920 | second hold-out (A, B only), built after `hold` had been read and before LT1e was scored: df 9, 11, 14, 16, 24, 28, 41, 53 × runs of 40 at t ≈ 0.77, 2.2, 5.9 |

  Total 34,842 cells.
- **Hold-out discipline.**
  - LT1, its knobs and its offsets were frozen on a1/a2. The first hold-out was then scored once (`final_hold.out`):
    - frozen offsets: 703/1,204/1,228;
    - a free-C structure test (C fitted on the hold-out's even half, scored on the odd half): 367/597/593 of 597,
      against CR 129/505/555.
  - LT1e came after that. On `hold` its counts are post hoc. `hold2` is its clean test.
- **Positive control** (`s17_final.py --control`). Synthetic Excel values from LT1 with every constant moved by
  +3.3 units: the fit recovers +3.28 to +3.32 and scores 352/352 on every df.
- **Reuse, by import only.**
  - r3-loader-bd0 `r3lib`: x87 helpers, Excel LN, EXP V6 on an extended argument, bd0 with its knobs, interval
    overlap.
  - r2-tdist `tlib`: the cells.tsv loader.
  - r2-log1p `excel_log1p`, w2-forensics `make_probes.enc`, w6 `h2_loader` (stirlerr table and series constants).
  - r3-loader-const `grid_e.tsv`, read in place.
  - `scripts/check_duplication.py` with this directory added reports no new duplicated window.
- **Ignored files.** Caches (`exact_cache.pkl`, `gl_cache.pkl`) and `__pycache__/` are in `.gitignore`.

## 3. Evidence for the structure

### 3.1 Not one product of an Excel inner column (`s2_ratio.py`, `s4_jac.py`, `s7_brc.py`)
- **One stored product K * X with a per-df K** (interval overlap, 355 cells per df). The best is POWER(1+t*t/n,-h)
  at 115/355 (df 3). The spread of T/X is 2.2-9.1 units.
  - `|t| F.DIST(t*t,1,n,FALSE)`: at most 99/355, sd 3-7 units.
  - POWER(n+t*t,-h): at most 111.
  - POWER(y,h): at most 133.
- **The beta densities times the Jacobian, f_B |t| y^2/n** (4 x87 orders): at most 109/355.
  - Through BETA.DIST(y,n/2,1/2): sd 5-28 units, because that cell's complement is fl(1-y).
  - Through BETA.DIST(z,1/2,n/2): sd 2-6.5 units.
- **TOMS brcomp(n/2, 1/2, y, z)/|t|** with the M1 density at y (complement z, 2 log switches × E stored or
  extended × 4 orders): at most 302/2,112.

### 3.2 Not EXP of a double (`s10_eset.py`)
At t ≈ 4.3, 38-45 of every 48 Excel values lie outside the image of Excel's EXP over doubles. Near the mode
(t ≈ 0.31) 0-19 of 48 do. So a value-space factor follows the EXP.

### 3.3 Which roundings Excel has (`s3_reg.py`, `s8_yz.py`, `s9_pred.py`, `s11_fsel.py`, `s13_resid.py`)
Forward selection of rounding features on each consecutive-double block. Coefficient 1 means Excel has that rounding
with that sensitivity.
- **Large t, t ≈ 4.3, x^2 > n.** The selected roundings are:
  - fl((n/2) LN(w)): 0.85 to 1.35;
  - LN(w), at its weight n/(n+1);
  - t*t: 1.0;
  - q = t*t/n: 0.8 to 1.0;
  - fl(sqrt(w)): about 1.

  Four features bring sd 1.8-5.3 down to 0.7-0.8. That is Loader's log branch u = (n/2) ln(1+q) with a separate
  1/sqrt(1+q).

  With LT1, the slope of Excel's error on the model's error is 0.90-1.00 (df 3-10), and the residual sd is 0.69-1.01.
- **Middle t, t ≈ 1.7 and t = 1.5 at df 3, 4, 5, x^2 < n.** The selected roundings are:
  - y = fl(n/fl(n+t*t)), with weight n/2;
  - LN(y);
  - (n/2) LN(y);
  - sqrt(w).

  This is bd0(n/2, (n+x^2)/2) in its log form, whose ratio k/M is y. So Excel keeps the bd0 path well past R's
  x^2 > 0.2 n. With LT1, the slope is 0.85-0.92.
- **The cut.** The bd0 path wins at q = 0.578, 0.616, 0.72 and 0.96, and the log branch wins at q = 1.85, 2.6, 3.7,
  4.6 and 6.2. q > 1 (x^2 > n) scores 1,232/2,112, against 1,011 for R's 0.2 n and 1,158 for no log branch at all
  (`s12_logbr.py --all`, den sw3).

### 3.4 Free-constant scan of the variants (`s12_logbr.py`, `s14_perblock.py`; one C per df shared by all blocks, 2,112 cells)

| variant | exact |
|---|---|
| **bcut 1, den sw3, bd0 {v:x, ph:1} (LT1)** | **1,257** |
| bcut 1, den sw3, bd0 R-default (v extended) | 1,232 |
| bcut 1, den sw3, all series ops stored | 1,239 |
| bcut 1, den sw2 (fl(fl(ex c)/fl(sqrt w))) | 1,213 |
| bcut 1, den old (fl(ex / fl(sqrt(fl(2 pi w))))) | 1,051 |
| bcut 1, old with 2 pi w, sqrt, division extended (5 variants) | 990-1,085 |
| bcut 1, den sw3, E extended | 890 |
| bcut 1, R 3.x denominator (EXP(t-u) M_1_SQRT_2PI EXP(-l_x2n)) | 949; with R 3.x's q > 0.2 cut 812 |
| no log branch (bd0 path everywhere), sw3 | 1,158 |
| R 2.x cut x^2 > 0.2 n, sw3 | 1,011; den old 415/736 on the series blocks |
| bd0 cut 0.1 (R) instead of 0.2, no log branch | 1,100 |

- **Per block** (`s14_perblock.py`), fitting C per block instead of per df gives the same counts. So the constant
  is shared by all t of a df, and the misses are structural.
- **t ≈ 0.31 blocks.** The 32 denominator × E × ex combinations (`s12_logbr.py --blk=t0.3125`) all score 297-444
  of 576. No combination is best on every df: sw3 gets 93/96 at df 3 and sw2 gets 94/96 at df 10.

### 3.5 Generic structures (`s6_struct.py`; one free C per df; 2,112 cells)
E = C - h L with L one of:
- LN(w) (`L1`);
- Excel log1p(q) (`Lp`);
- LN(n + t*t) (`Lu`, the df-2 form);
- -LN(y) (`Ly`).

Each L was tried with E stored (`Es`), extended (`Ee`) or a stored product (`Ep`), and with the chains K·EXP
(`K`/`Ke`), `Q`/`Qe` (fl(EXP(fl(C - fl(n/2 L)))/sqrt w)) and `Hs`:

| structure | exact |
|---|---|
| L1_Es | 410 |
| L1_Ee | 552 |
| L1_Ep | 494 |
| Lp_Es | 590 |
| Lp_Ee | 754 |
| Lu_Es | 276 |
| Lu_Ee | 374 |
| Ly_Es | 386 |
| Ly_Ee | 497 |
| L1_K | 481 |
| L1_Ke | 552 |
| L1_Q | 458 |
| L1_Qe | 562 |
| L1_Hs | 397 |
| Lu_K | 325 |
| Lu_Ke | 374 |
| Ly_K | 456 |
| Lp_K | 709 |

All are rejected against LT1's 1,257. Lu_Es is the df-2 closed form: it does not extend to df >= 3.

## 4. The constant
- **The offsets LT1 needs** (units of E, from the exact Loader constant):

  | df | offset | source |
  |---|---|---|
  | 3 | +2.48 | in-sample (`lt1_offsets.json`) |
  | 4 | -1.61 | in-sample |
  | 5 | +1.70 | in-sample |
  | 7 | -0.01 | in-sample |
  | 10 | +1.53 | in-sample |
  | 30 | -0.76 | in-sample |
  | 6 | -0.33 | hold-out even half |
  | 8 | +0.84 | hold-out even half |
  | 12 | -0.40 | hold-out even half |
  | 20 | -0.11 | hold-out even half |

  These offsets are why 1-ulp-level differences matter at large |E|: in `hold`'s df-8 t ≈ 7.7 block, LT1 with the
  exact constant leaves 14 of 48 cells visibly off (15 ulp). With the fitted constant all are w15.
- **Rejected as the constant.**
  - `s15_t0.py`, `s16_k0.py`: T.DIST(0,n,FALSE) (m9-t-dist-pdf: Excel is +3 ulp from CR at df 3, -1 at df 4).
    Every parameter-free chain misses it at df 3. That includes Loader with R's stirlerr table and bd0 (3 series
    variants × 3 sum orders × 3 output chains), and 30 GAMMALN/GAMMA/EXP/SQRT compositions of Γ(h)/(Γ(n/2) sqrt(nπ)).
    The best composition is 1/6.
  - GAMMALN errors (an inline check against Excel GAMMALN from the harness): the stored GL(h) - GL(n/2) error is
    -29 units at df 20 and -19 at df 30, against needed offsets of -0.11 and -0.76.
  - bd0(n/2,(n+1)/2) arithmetics (`s19_const.py`, 768 r3lib knob combinations): best rms 1.14, against the
    offsets' own rms of 1.25. These knobs cannot make errors of 2 units at |bd0| < 0.07.
- **Accepted:** stirlerr's own per-half-integer error. r3-loader-const's e(v) (grid_e.tsv) gives
  e(h) - e(n/2) + 2.07, with residuals:

  | df | residual |
  |---|---|
  | 3 | +0.41 |
  | 4 | -0.36 |
  | 5 | +0.19 |
  | 6 | +0.18 |
  | 7 | +0.64 |
  | 8 | -0.52 |
  | 10 | -0.31 |
  | 12 | -0.12 |
  | 20 | +0.54 |
  | 30 | -0.64 |

  The opposite sign gives rms 2.32. So BETA.DIST and T.DIST share one stirlerr whose errors are real. The T density
  pins the gauge: e_true(v) = e_grid(v) + 4.13 v + γ (γ is still free).

## 5. What is open (why bit-exactness is about 55%)
1. **The bd0 series.** Most cells take the series (x^2 < n and |k - M| < 0.2 (k + M), i.e. x^2 < n/2). It shows
   the same unidentified arithmetic r3-loader-bd0 found for BETA.DIST: slopes 0.5-0.7 on its blocks, and
   `hold`'s df-20 t ≈ 2.9 block (v ≈ 0.17) has 10 w15 misses.

   The T density is a clean probe of that series. It has a one-parameter family, k = n/2 and M = (n + x^2)/2, with
   d = -x^2/2 exact.
2. **The denominator.** sw3 (division first) against sw2 (the 1/sqrt(2 pi) product first) is not settled per df.
   Near the mode no variant scores more than 444/576.
3. **The stirlerr table.** e(v) is a fit, so its own noise (about 0.4 units) limits the constant. For df > 55 the
   exact constant is used.
4. **Very large df.** `T.DIST(±10, 1E6, FALSE)` (series at k = 5E5) is -46 ulp.

## 6. Negatives (decision 0056; scripts named)
- **Excel inner columns.** No single product K·POWER(...), K·F.DIST·|t|, Jacobian-times-BETA.DIST composition or
  M1 brcomp matches (section 3.1).
- **Generic exp-of-log forms.** 18 structures, each at most 754/2,112 (section 3.5).
- **Loader variants.** About 70 scored (section 3.4, `s12_logbr.py` invocations).
  - R 2.x as written (cut 0.2 n, EXP/sqrt(2 pi w)), in-sample: 415/736 on the series blocks.
  - R 2.x on 2,220 old-data cells (`s1_loader.py`, exact-arithmetic knobs): 995/2,220.
  - The R 3.x form: 812-949.
- **Log-branch variants** (1,376 cells with x^2 > 0.2 n; s12 without `--all`):

  | variant | exact |
  |---|---|
  | -LN(fl(n/fl(n+xx))) | 466 |
  | the same with n + xx extended | 539 |
  | the y/w switch at y = 0.5 (`auto`) | 589 |
  | `autoe` | 613 |
  | `autoz` (-log1p(-z)) | 633 |
  | -LN(fl(1-z)) | 377 |
  | LN(u) - LN(n) | 303 |
  | LN(w) with the R 2.x cut (`{}`) | 485 |
  | LN(w) with sw3 | 554 |

  With the cut at x^2 > n, plain LN(w) is right (log-branch blocks score 50-78 of 96).
- **Constant chains.** Section 4.
- **df 1.** 1/π/(1+t*t) scores 21/27. `1/(π (1+t*t))` and `1/(π + π t*t)` both score 27/27, so they are not yet
  separated. CR scores 18/27.

## 7. Scripts and outputs

| file | what |
|---|---|
| `tdlib.py` | data loaders (cells.tsv via r2-tdist, probes via meta + sidecar), exact references, Loader dt with knobs (`loader_u`, `loader_dt`, `log_branch_l`), generic structures, M1 brcomp, free-constant fit (`fit_free_C`), the frozen `LT1` / `lt1`, `lt1e_offsets` |
| `mkprobe.py` | the four probes and their meta files |
| `s1_loader.py` | R 2.x dt, free C, old data |
| `s2_ratio.py`, `s4_jac.py`, `s7_brc.py` | inner-column compositions |
| `s3_reg.py`, `s5_blocks.py`, `s8_yz.py`, `s9_pred.py`, `s11_fsel.py`, `s13_resid.py` | rounding-feature regressions |
| `s6_struct.py`, `s12_logbr.py`, `s14_perblock.py` | free-constant structure scans |
| `s10_eset.py` | EXP-image test |
| `s15_t0.py`, `s16_k0.py`, `s19_const.py` | the constant at t = 0, bd0 knobs against the offsets |
| `s17_final.py` → `final_insample.out`, `final_hold.out`, `lt1_offsets.json` | frozen LT1: fit on a1/a2, score in-sample, old, hold (`--hold`), control (`--control`) |
| `s18_miss.py` | visibly-off cells and read-back check |
| `s20_lt1e.py` → `lt1e.out` | LT1e (stirlerr-table constant): all sets, including hold2 |
| `s21_small.py` → `small_df.out` | df 1 and df 2 closed forms |

## 8. Suggested follow-ups (for the coordinator)
- **statistics.md.**
  - Replace the T-density half of "Student t density noise is not input rounding (T-0969)" with LT1e.
  - Add to the stirlerr part of "Beta density for min(a,b) > 1 is Loader's saddle point" that the T density
    confirms e(v) and fixes α = +4.13 units/unit v.
  - New entry: df 1 = 1/(π(1+t·t)) in stored x87 ops (27/27; probes m9-t-*).
- **Engine task** (T-0969 unblock). Port LT1e: Loader dt with the x^2 > n cut, the sw3 denominator, and the bd0 /
  stirlerr from T-2507's Loader structure. Visible parity on the T density goes from 229 visibly-off cells (CR) to 3.
- **Research.** Use T.DIST runs to pin the bd0 series arithmetic (section 5.1), jointly with r3-loader-bd0.
