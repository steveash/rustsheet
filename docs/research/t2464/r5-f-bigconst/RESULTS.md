# T-2586 r5-f-bigconst: the F density's shape constant at large shapes (min(d) >= 16)

Worker r5-f-bigconst, round 5, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: absolute in a logarithm, relative in the density. a = d1/2,
b = d2/2, h = a + b, w = min(a, b), o = max(a, b), side 1 is fl(d1·x) <= d2, r = fl(mx/n) | fl(n/mx), s = fl(1 + r),
L = Excel's log1p. "need" = the constant offset (units, against the exact K) that reproduces Excel under M-sd's
x-structure E = K + fl(c·LN x) - h·L(r), F = fl(EXP(E)/x). side_err = M-sd's bracket
(h - 1/2)·L(fl(w/o)) - (h - 1/2)·ln(h/o) for the (w, o) of that side. Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
- **Excel's F(1; d1, d2) is the exact density to about one ulp at large shapes.** On the 361 x = 1 cells (all
  pairs of 19 df from 16 to 1000), CR is bit-exact on 152, and Excel is within -3..+2 ulp of CR on all 361. Its
  relative error has mean -0.54 and sd 1.38 units (`b6_ba.out`, `b7_ba.out`). On the hold-out's 144 x = 1 cells,
  "exact E through EXP V6" hits 54 and M-sd only 37 (`b6_bhold.out`).
  - So round 4's "-64 units at (300,300), -35 at (160,160)" is not an error in Excel's value. It is the constant
    that M-xd needs to cancel its own H = -h·L(r): that H carries Excel's LN(fl(1 + r)) error times h, for example
    h·(LN(2) - ln 2) at d1 = d2.
  - M-sd's bracket is that cancellation: at x = 1, r = fl(w/o), so K + H keeps only -1/2 of L's error. The bracket's
    coefficient is confirmed at large shapes. Regressing need on h·eL and h·er (Excel's log1p error and the rounding
    of w/o) over 380 (shape, side) groups gives coefficients 1.00 and 0.99, plus about -0.47·eL, so (h - 1/2)·eL
    (`b5_cols.py`).
- **M-sd's e-table part is wrong at large shapes.** Above v = 28, M-sd extrapolates its e-table linearly, so
  e(h) - e(a) - e(b) + C0 is the constant +1.54 units when a, b and h are all above 28. Where the table is read, it
  ranges from -0.7 to +2.7 on this grid. What the data needs is:

  | set | groups | need - side_err: mean | sd | rms vs M-sd | rms vs M-bc |
  |---|---|---|---|---|---|
  | in-sample (`ba`) | 380 | -0.42 | 1.27 | 2.21 | 1.27 |
  | hold-out (`bhold`) | 156 | +0.09 | 1.28 | 1.88 | 1.38 |

  The e-table values themselves do not help where they exist: the correlation is 0.14 on the 30 groups with h <= 28.
- **Model M-bc** (`bcmodel.py`, frozen before the hold-out was read): delta = side_err + C_BIG (C_BIG = -0.42) when
  min(d1, d2) >= 16, M-sd's delta otherwise. Everything else is M-sd. Hold-out `bhold`, read once
  (`final_hold.out`):

  | model | bhold all (2,592) | x ≈ 1 runs (1,404) | runs at x ≈ d2/d1 (1,188) |
  |---|---|---|---|
  | CR | 121/576/1,193 | 114/476/932 | 7/100/261 |
  | M-sd | 389/1,773/1,985 | 325/1,404/1,404 | 64/369/581 |
  | M-sdm | 428/1,775/1,979 | 354/1,404/1,404 | 74/371/575 |
  | **M-bc** | **607**/**1,787**/**1,991** | 533/1,402/1,404 | 74/385/587 |
  | M-bcm | **643**/1,786/1,990 | 559/1,403/1,404 | 84/383/586 |

  On other workers' sets, scored with the frozen models (`final_other.out`), M-bc's large-shape gains are in
  bit-exact cells (with within-15-digit gains of 12 to 38 cells):
  - r4-f-brcomp `big`: 156 → 364 bit-exact, 1,700 → 1,722 w15.
  - r5-f-combine `chold` big class: 79 → 164, 723 → 735.
  - `hold4` big class: 72 → 93.
  - r5-f-combine `ca` big class: 310 → 294 bit-exact (-16), but 2,295 → 2,333 w15.
  - Unchanged on the small-shape sets (`myhold`, `cells`, r3 `hold`).
- **Not reached: bit-exactness or visible parity at large shapes.** Two problems remain:
  - **About 1.3 units of noise in the constant.** It is reproducible: swap pairs agree bit for bit. Neither a
    stored-piece family nor Loader value-space forms nor any other candidate in section 4 explains it.
  - **The runs at x ≈ d2/d1, where |P| is large.** There the stored-sum quanta dominate: the needed constant
    jumps by about ±32 units between consecutive x at (64,128) side 2, and the per-run rms is 116 units
    (`b2_cells.py`, `b3_reg.py`). That is r4-f-power's and r5-f-combine's open structure, not the constant.
  - **The side class w = max is not identified.** It is d1 < d2 on side 2, or d1 > d2 on side 1, so it needs
    x > d2/d1 (or x < d2/d1). There |P| >= b·ln(d2/d1) can never be small. M-bc uses the same rule there (M-sd's
    bracket for that side + C_BIG). That is a choice, not a measurement.

**Model to import:** `docs/research/t2464/r5-f-bigconst/bcmodel.py`:
- `bcmodel.value(x, d1, d2, "div")` (M-bc) or `bcmodel.value(x, d1, d2, "mulr")` (M-bcm, the most bit-exact cells);
- the constant alone: `bcmodel.delta(d1, d2, side)` in units over the exact K (K: r4-f-small-df `slib.K_exact`);
- `bcmodel.side_err(d1, d2, side)` is M-sd's bracket alone.

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph). Both were built by `mkprobe.py` and filled on 2026-10-24 before any
  analysis. x is the exact formula of w2-forensics' encoder, and `mkprobe.rows` asserts every echo (0 misreads).

  | case | cells | content |
  |---|---|---|
  | `t2586-r5-f-bigconst-a` (`ba`) | 12,996 | in-sample: every pair of {16, 17, 20, 24, 31, 40, 50, 63, 64, 80, 100, 127, 128, 160, 200, 256, 300, 400, 1000} (361 shapes) × 9 consecutive doubles around x = 1 (`x1`) + 9 around fl(d2/d1) (`sw`, both sides); d1 = d2 gets 18 doubles around 1 instead |
  | `t2586-r5-f-bigconst-hold` (`bhold`) | 5,184 | **hold-out**: every pair of {18, 26, 33, 36, 48, 57, 70, 96, 140, 220, 330, 600} (144 shapes), same layout. Filled with `a`. Read once by `final_score.py bhold` after `bcmodel.py` was frozen, then by the post-freeze diagnostics `b4_table.py bhold` and `b6_x1.py bhold` (no choice made on them) |

  Total 18,180 cells.
- **Method.** clib's interval inversion (r5-f-combine) gives each cell's admissible extended E through
  fl(EXP_V6(E)/x). At x = 1 that is EXP alone, since P = 0 and there is no 1/x. Then:
  - `b1_need.py`: the deepest overlap of the needed constants per (shape, side, run);
  - `b4_table.py`: the mean of the interval centres over the x ≈ 1 run, written to `need_ba.tsv` and
    `need_bhold.tsv`.
- **Positive control** (`b1_need.py ba --plant 2.5`, `b1_control.out`): Excel's values are replaced by M-sd's chain
  with delta + 2.5 units. Every one of the 1,064 runs is fully reproduced, and the delta is recovered with rms 0.16
  units.
- **Reuse, by import only:**
  - r5-f-combine `clib` (e_interval, pieces, best_overlap, K_sd, final);
  - r4-f-small-df `slib` and `sdmodel`;
  - r3-fdensity `flib` (exact F, stirlerr_exact, tally, R64) and its `mkprobe.row_cells` / `case_doc`;
  - w2-forensics `make_probes` (enc, run).
- **Housekeeping.**
  - Scripts are prefixed `b` because r5-f-combine's `s*.py` names shadow on sys.path.
  - `.gitignore` covers `*.pkl` and `__pycache__/`. clib writes its git-ignored `iv_cache.pkl` in r5-f-combine.
  - No .py file is over 120 lines. The directory is about 300 KB.
  - 0 duplicated 10-line windows against every tracked .py and all t2464 subdirectories (check_duplication's
    normaliser).

## 3. Structure facts measured here
1. **At x ≈ 1, M-sd's x-structure holds to about 1 unit.**
   - On d1 = d2 side 2 the needed constant is flat to 0.01 units per x step, for example (100,100) side 2
     (`b2_cells.py ba 100 100 x1`).
   - On d1 = d2 side 1 there is a sawtooth of about ±1.3 units. It is tied to the s and mx roundings, at
     (100,100), (200,200) and (300,300).
   - Pooled over 380 runs, the regression on e_mx, e_r and e_s gives coefficients 0.28, -0.03 and -0.18. The rms
     falls only from 0.727 to 0.721 (`b3_reg.py ba x1`). So the sawtooth is shape-specific, not one missing term.
2. **The large-shape constant is M-sd's bracket.** Examples (need / bracket, units; `need_ba.tsv`):
   - (300,300): -64.3 (side 1) and -63.8 (side 2) / -62.6;
   - (160,160): -34.8 and -34.9 / -33.3;
   - (40,100) side 1: -22.7 / -22.5;
   - (1000,128) side 2: -42.5 / -42.0.

   M-sd adds +1.5 to each of these. The largest residuals (need - bracket) are -3 to -4 units, at (64,128),
   (80,160), (128,160) and (160,80).
3. **Away from x = 1 at large |P|, the residual is quantised.** At (64,128) side 2 (x ≈ 2, P ≈ -44.4,
   ulp(P) = 64 units), the needed constant moves by multiples of about 32.4 units between consecutive x. At
   (100,200) side 2 it moves by about 21 and 41. Side 1 at the same x is flat to ±1 at (64,128).

## 4. Hypotheses and counts (negatives included, decision 0056)
### 4.1 x = 1 cells (361 in-sample; `b6_x1.py`, `b6_ba.out`; misses in units)
| candidate for ln F(1) | hits | \|miss\| < 1 | rms miss |
|---|---|---|---|
| exact (through EXP V6) | 152 | 280 | 0.99 |
| -1/2·Excel L(r) + exact rest (`ld_exactln`) | 152 | 281 | 0.98 |
| same - 0.42 units | 178 | 294 | 0.88 |
| M-sd (K_sd + H) | 70 | 191 | 1.78 |
| M-xd (exact K + H), fl(fl(K) + H) + remainder, fl(K + H) | 14 | 38 | 41.5 |
| ½(LN w - LN 2π), ½·LN(fl(w/2π)), ½·LN(fl(w/(2π s))), -½·LN(fl(2π s/w)) | 133-151 | 254-270 | 1.09-1.26 |
| stored exact pieces (A = -½ log1p(w/o), B = ½ ln(w/2π), D = Δstirlerr): fl(E), fl(fl(A+B)+D), fl(A)+fl(B)+fl(D), fl(B), ... | 154-171 | 274-296 | 0.99-1.07 |
| fl(fl(fl(A)+fl(B))+fl(D)) (`st_ABDs`) | 189 | 290 | 1.01 |

On the hold-out's 144 x = 1 cells (`b6_bhold.out`): exact 54, ld_exactln 58, -0.42 64, st_ABDs **47** (its in-sample
lead does not hold), M-sd 37.

### 4.2 Loader / dbinom_raw value-space forms at x = 1 (`b7_loader.py`, `b7_ba.out`)
The identity is f = a·b/(h·x)·dbinom_raw(a; h, p, q), with bd0 = 0 at x = 1, Excel's EXP, LN and log1p, and stored
doubles:

| form | count |
|---|---|
| fl(EXP(lc - lf/2)·fl(ab/h)) | 71/350/361 |
| extended | 69/351/361 |
| EXP(lc - lf/2 + LN(ab/h)) | 101/346/361 |
| (w, o)-symmetric lf | 75/354/361 |
| fl(ab/h)·EXP(lc)/SQRT(2π a q) | 128/361/361 |
| CR | 152/361/361 |

All rejected.

### 4.3 Extended-precision stirlerr (`b10_stx.py`, `b10_ba.out`)
stirlerr(v > 15) as lgamma(v+1) - (v+½)·ln v + v - ln√(2π), or as lgamma(v) - (v-½)·ln v + ..., with every op
rounded to 64 bits (final value extended or double) scores 160-161 of 361 hits, against 152 for the exact one. This
is a weak test, because there is no planted control.

### 4.4 What the ±1.3-unit residual is not (x = 1 cells, `b9_x1reg.py`; `b11_additive.py`, `b11_ba.out`)
- Regressing Excel's E error (raw rms 1.48) on the roundings of fl(E), A, B, A+B, ln w, ln o, ln h,
  ½·ln(wo/h), ln(wo/(2πh)), on Excel's LN errors at w, o and h, and on F's value rounding: rms 1.36-1.38 for every
  subset. Nothing explains it.
- The residual's mean drifts with w: -0.09 at w < 15, -0.62 at 15-40, -0.94 at 40-100, -1.48 above 100. Its sd is
  1.0-2.0.
- Per-shape-parameter fits on the 190 unordered pairs:

  | fit | params | rms |
  |---|---|---|
  | free u(w) + v(o) | 39 | 0.88 |
  | shared e(w) + e(o) | 20 | 1.08 |

  e(w) + e(o) - e(h) needs 173 parameters for 190 pairs, so it is uninformative.

### 4.5 The switch threshold (exploratory, after the freeze; `b8_thresh.py`, `b8_thresh.out`)
On r5-f-combine's `ca` cells with 6 <= min(d) < 16 (1,920), lowering BIG_MIN to 8 gives 799 bit-exact against 629
at 16, and 10 or 12 give 665. w15 is 1,920 for all of them. This is not frozen, and it was not tested on a hold-out.

### 4.6 Runs at x ≈ d2/d1 (`b1_ba.out`, `sw` rows)
Per-run needed constants (overlap centre minus M-sd) have quartiles -14 / -1.2 / +4.5 (w = min class) and
-16 / -0.4 / +10 (w = max class) over 342 runs each. The best overlap covers 1-5 of 4-5 cells. These runs carry no
usable constant information under M-sd's structure.

## 5. Open (most promising first)
1. **The large-|P| stored-sum structure at large shapes** (32-unit quanta at (64,128) side 2). Without it, the
   w = max side class cannot be pinned, and the x ≈ d2/d1 runs stay at about 50% w15 (`bhold` sw 587 of 1,188).
2. **The ±1.3-unit constant residual and the side-1 sawtooth at x ≈ 1.** The sawtooth is reproducible and tied to
   the e_s / e_mx roundings. Next: per-shape regressions on (100,100), (200,200) and (300,300) side 1 against
   stored-piece candidates for the log1p and mx chain (for example log1p(r) formed from mx and n directly,
   fl(mx + n), q = fl(n/(mx + n))).
3. **C_BIG** is -0.42 in sample and +0.09 on the hold-out. A size-dependent offset (the drift with w in 4.4)
   might fit both, but it needs its own hold-out.

## 6. Scripts
| script | what |
|---|---|
| `blib.py` | imports (clib, slib, sdmodel, r3 probe helpers), `load('ba' / 'bhold' / any clib name)` |
| `mkprobe.py` | the two probes (`--write`) and `rows()` |
| `bcmodel.py` | **frozen M-bc / M-bcm** (`value`, `delta`, `side_err`) |
| `final_score.py` | frozen-model scores by class |
| `b1_need.py` | per-run needed constant by deepest overlap (`--plant` control) |
| `b2_cells.py` | per-cell needed constant with r, s, P |
| `b3_reg.py` | per-cell regression on e_mx, e_r, e_s, e_lx with per-run intercepts |
| `b4_table.py` | per-(shape, side) need table from the x ≈ 1 runs → `need_*.tsv` |
| `b5_cols.py` | need regressed on log1p / LN / product rounding columns |
| `b6_x1.py` | x = 1 candidate constructions (E-space) |
| `b7_loader.py` | Loader value-space forms at x = 1 |
| `b8_thresh.py` | BIG_MIN sweep (exploratory) |
| `b9_x1reg.py` | x = 1 residual regression on stored-double roundings |
| `b10_stx.py` | extended-precision stirlerr variants |
| `b11_additive.py` | per-parameter additive fits of the x = 1 residual |
