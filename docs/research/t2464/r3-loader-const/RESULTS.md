# T-2513 r3-loader-const: the shape constant of Excel's Loader beta density (a, b > 1)

Worker r3-loader-const, round 3, 2026-10-24. The goal was to pin the per-shape constant of BETA.DIST(x,a,b,FALSE) for
min(a,b) > 1. r2-density-large identified this density as R's `(a+b-1)·dbinom_raw(a-1, a+b-2, x, fl(1-x))`.

Method:
- Every probe cell sits where both bd0 sides take their log form, so the x-dependent part X is the modelled one.
- Excel's EXP (model V6) is inverted per cell. That gives an interval for the exponent E, and so an interval for the
  constant C = E - X.
- The intervals are intersected per shape, across single-ulp sweeps of a and b, across 414 random shapes, and across
  a half-integer grid.

Units are 2^-53, absolute in E (the same as relative in the density). Counts are bit-exact / within 4 ulp / within 15
significant digits ("w15"). Run the scripts from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 0. Probes, controls, housekeeping
- **Probes.** All are excel-graph, all are literal formulas, and all were filled 2026-10-24. Rows are rebuilt from the
  seeded generator in `r3lib.design()`, so no meta files are kept. Excel's values are read from the eval sidecars.

  | case (probes-numeric) | pdf cells | content |
  |---|---|---|
  | `t2513-r3-loader-const-sweep1` | 11,264 | 22 bases × 32 single-ulp neighbours (a ± 8 ulp, b ± 8 ulp) × 16 x |
  | `t2513-r3-loader-const-diverse1` | 9,936 | 414 shapes (400 random in (1.05, 80), 1-3 decimals, plus r2's named shapes) × 24 x |
  | `t2513-r3-loader-const-grid1` | 7,200 | all 900 half-integer pairs a, b in {1.5, 2, ..., 16} × 8 x (added after sections 1-4) |
  | `t2513-r3-loader-const-hold1` | 3,296 | **hold-out**: 3 new sweep bases (8.3,5.9), (2.7,6.4), (17.1,11.3) × 512, plus 110 new random shapes × 16 |

  - Bases in the sweep probe:
    - k and n-k not half-integers: (5.3,9.7), (3.7,2.9), (20.3,30.1), (12.7,40.3), (2.3,4.6), (4.2,3.3), (6.7,6.1),
      (9.4,15.8), (3.15,7.85), (14.6,9.2), (30.7,25.3), (45.2,60.9), (7.3,11.6);
    - half-integer controls: (2.5,3.5), (4.5,7.5), (7.5,11.5);
    - integer controls: (4,4), (6,9), (12,20);
    - min(a,b) < 2: (1.6,5.3), (7.3,1.45), (1.3,1.8).
  - Each x is a random double with both bd0 sides at v = |k - M|/(k + M) >= 0.205, above Excel's cut (0.1837,
    0.2000001]. x is taken from the higher-density tail, so |E| stays small (sweep median -0.44; at most 13).
  - The cases total 39,156 cells, echo cells included.
- **Hold-out.** The hold-out was created and filled before any tuning. It was read once, in section 6, after M3 and
  M4 were frozen.
- **Parser read-back.**
  - Column B echoes every 8th x, and columns C and D echo each shape once. The grid probe echoes only its shapes.
  - All x echoes match.
  - **592 sweep rows had a shape literal truncated by Excel's 15-digit parser** (for example `=20.29999999999998` was
    read as 20.2999999999999). The cause is that w2-forensics' `make_probes.num` counts digits after deleting any
    "0." substring, so it writes some 16-digit reprs of numbers >= 10 as decimals.
  - `r3lib.load` takes a and b from the echo cells, so every row carries the double Excel actually used. Workers who
    reuse `MP.num` for values >= 10 should know this.
- **Positive controls.**
  - Constant pin on synthetic data with C = Cex + 3.3 units (`a1_pin.py --control 3.3`): 11,253 of 11,264 cells
    consistent (stored E) and 11,042 (extended E). Every recovered interval contains +3.3.
  - The structure test (`a6_struct.py --control`) is consistent on all cells for `g` and `vs`. It over-counts slightly
    (2,606 of 2,576) because adjacent candidate intervals share endpoints.
- **Reuse, by import only:** r2-density-large `dlib` / `loader` / `model` (X part, EXP V6, LN, scorer), w2-forensics'
  encoder, w6's stirlerr table and series constants. GAMMALN values are cached in this directory.
- **Ignored files.** Caches (`*.pkl`) and `__pycache__/` are listed in `.gitignore`.

## 1. E enters EXP unstored (extended), not as a double (`a1_pin.py`, `a6_struct.py`)
- **No stored E is possible at |E| of about 10.** At (45.2, 60.9), for every one of its 112 cells (E about -8.5 to
  -13.6), no double E gives Excel's value: the outputs of the harness EXP on consecutive doubles skip it. For example
  2.7749122563063622e-05 falls between EXP(-10.492306337620791) = ...632e-05 and the next double down. So r2's stored
  chain E = fl(L5 + fl(lc - lf/2)) cannot be Excel's for |E| >= 8.
- **Free-constant consistency** (largest set of cells per shape sharing one C):

  | structure | sweep | diverse |
  |---|---|---|
  | E = fl53(C + X) stored | 7,724 / 11,264 | 4,379 / 9,936 |
  | **E = C + X extended** | **8,596 / 11,264** | **5,990 / 9,936** |

- **The same test on 150 sweep shapes (2,576 cells):**
  - extended: 2,012;
  - `g`, the lc - lf/2 part stored then L5 added: 1,203 (rejected);
  - `vs`, R 2.x's value-space `p*f` (fl53(EXP(C + X)·fl(a+b-1))): 1,809 (rejected);
  - `lc`, lc = fl(Cs + X) stored with Cs scanned over 81 values: 2,084. This is not adopted: the scan adds freedom, and
    at (45.2, 60.9) a stored lc of magnitude 10 would add ±8 units of noise, where the within-shape sd is 0.6.

## 2. Excel's k, n, n-k are R's stored ones; the single-ulp jumps are n ≠ k + (n-k) (`a2_forms.py`)
- **Argument forms** (within-shape sd of E - X / median |deviation|, sweep):

  | form | sd / median \|dev\| |
  |---|---|
  | **R**: k = a-1, n = fl(a+b)-2, nk = fl(n-k) | **1.33 / 0.45** |
  | B: nk = b-1 | 7.80 / 0.63 |
  | S: n = fl(k + b-1) | 7.87 |
  | Re: n-k unrounded | 7.40 / 0.57 |
  | E: n and nk extended | 11.43 / 0.86 |

  Diverse: R 3.88 / 0.72, B 9.88 / 1.18.
- **The jumps.** Single-ulp neighbours jump by up to ±20 units: (12,20), a = 12 ± k ulp: +0.0, -16.3, -0.1, +15.7,
  ... The needed offset against the exact pdf constant at Excel's pieces regresses on ε = n - k - fl(n-k):
  - sweep: coefficient 1.13, sd 6.87 → 3.69;
  - diverse: coefficient 1.01, sd 9.61 → 4.73.
- **Reading.** The x-part carries -ε, through (M1 - k) + (M2 - nk), and Loader's constant evaluated exactly at Excel's
  own k, n, nk carries +ε. So the "exact Loader at Excel's stored pieces" removes the jumps. Every later residual is
  against that reference.

## 3. The X part (both log forms)
**`a5_xreg.py`: regression of the within-shape residual on the model's own roundings** (coefficient 0 = as
modelled):
- **Modelled roundings.** p.M, p.div, p.ln, p.mul, q.M, q.div, q.ln and q.mul all come out at -0.03 to +0.02 (sweep
  11,264 cells and grid 7,200), with feature sds of 1.7-6.6 units. r2's x87 log form is confirmed at the bit level on
  a new probe.
- **Extra candidates** (coefficient 1 = Excel has them):
  - **bd0 stored to a double**: p 0.81 and q 0.92 (grid); q 1.12 alone (sweep). The bd0 value is small, so the
    feature sd is only about 0.2.
  - X stored: 0.38 to 0.51 (not adopted).
  - R's (t + M) store: 0.00 to 0.02, with feature sd 4.5-11.6. **Rejected again**, now with large shapes.

**`a6_struct.py`: consistency (extended E, free C)**

On 150 sweep shapes, 2,576 cells:

| X variant | consistent |
|---|---|
| **V0** (r2), M = fl(n x) | **2,012** |
| V4, V5/V6 (correctly rounded log, RN53 product) | 2,012 / 2,013 |
| V1 (LN extended), V2 (FYL2X with y = RN64(k ln2)), V3 | 1,397 each |
| V7 (fl(k·fl(LN k - LN M))), V8 | 732 / 706 |
| M extended | 1,095 |
| M1 extended only | 939 |
| M2 = fl(n - M1) | 1,721 |
| M = fl(fl((a+b)x) - 2x) | 1,453 |
| **V0 + bd0 stored** | **2,025** |

- Grid (7,200 cells): V0 5,427; V0 + bd0 stored **5,550**.
- At min(k, n-k) >= 6 (grid, 2,888 cells): V0 2,144, V5 2,143, V1/V3 1,420.
- M2 = fl(n - M1), r2's guess for (4,4), loses on the (4,4) family as well: 221 against 360 of 512. The (4,4) base
  itself is at the noise floor (within-shape sd 0.46).

**Open: large k.**
- On the grid, about half the shapes with min(k, n-k) >= 6 have a within-shape sd above 1, for example (13,13) and
  (14,15).
- The misfits are about ±8 or +16 units: half and one ulp of t when |t| is in [8,16), where t = k·LN(k/M) is about
  10.7. Nothing tried reproduces them (V1-V8, the M forms, the bd0 store).

## 4. The constant: stirlerr is accurate; the LN pieces are Excel's; a residual of about 2-4 units is unexplained
Reference: the exact Loader constant at Excel's pieces. Fitted: the needed offset per shape, as the median of E - X.

**stirlerr** (`a3_const.py`, 650 sweep shapes; group "formula" = 550 shapes with a non-half-integer piece <= 15):

| stirlerr variant | formula-group rms |
|---|---|
| exact | 3.36 |
| exact, LF0 extended | **3.16** |
| exact, LF1 | 3.46 |
| table + Excel-GAMMALN formula, x87 stored | 26.1 |
| same, extended | 23.2 |
| GAMMALN(v) + LN(v) | 19.7 |
| correctly rounded lgamma, x87 | 17.9 |
| correctly rounded lgamma, extended | 15.0 |
| correctly rounded lgamma, LN extended | 7.96 |
| lgamma held to 64 bits, extended ops and LN | 3.32 (= exact) |

- **Every formula with a double-rounded lgamma or LN is rejected.** Excel's stirlerr is accurate to well below a unit.
  This confirms r2's 2.5 with a wider set of variants.
- `a7_creg.py` (1,064 shapes): the GAMMALN errors at a, b, a+b, k+1, nk+1, n+1 and the R-formula pieces all get
  coefficients of 0.00 ± 0.07.

**Series cuts** (`a8_series.py`, 311 clean shapes with a piece > 15):
- R's term counts (5 terms to 35, 4 to 80, 3 to 500) give rms 3.72.
- A 4-term/3-term cut at 50 gives 3.08. The cut is sharp (48: 4.27, 52: 3.34), but it rests on a few shapes. This is a
  lead, not adopted.

**Table** (`a10_grid.py`, 529 grid shapes with within-shape sd <= 1):
- With the LN roundings fixed at coefficient 1 and the R table, the residual is mean +0.37 and sd 1.70 units.
- Fitting a free correction e(v) per half-integer (identified up to αv + γ) lowers it to 1.02 in-sample and 1.22 on a
  held-out half of shapes.
- So R's table entries are right to about a unit, and no entry is grossly off.
- The residual trends with n:

  | n | residual mean | sd |
  |---|---|---|
  | < 8 | +1.69 | 1.74 |
  | 8-15 | +0.40 | 1.39 |
  | 15-22 | -0.08 | 1.46 |
  | 22-31 | -0.42 | 1.86 |

  This trend is unexplained.

**LN roundings and composition** (`a7_creg.py`, `a9_compose.py`):
- Regression coefficients: L5 = LN(n+1) 0.92 (0.82 on the grid); LN(n)/2 1.18; LN(nk)/2 0.99; LN(k)/2 -0.42 to +0.89.
  These are noisy, and their sign and size favour Excel's stored LN.
- 896 compositions were enumerated: lf orders A-D, LF1 and log-of-product G/H, each op stored or extended, LN or
  ln64, and chains ext/K/S/KC.
- They all land within 0.1 of each other: best rms 3.57 on 537 clean sweep and diverse shapes (exact Loader 3.95) and
  1.81 on the grid (2.28). Stored Excel LN beats ln64 everywhere. **No composition explains the residual.**

**Needed offset on the exact table shapes** (residual against the exact Loader constant; the r2 value from `c2_const`
in brackets):

| shape | residual (r2) |
|---|---|
| (2.5,3.5) | +2.60 (+2.9 to +3.4) |
| (3.5,2.5) | +2.92 (+3.46) |
| (2.5,2.5) | +6.21 (+6.3) |
| (1.5,1.5) | +3.73 (+4.1) |
| (4,4) | -2.37 (-2.4) |
| (6,9) | -0.58 |
| (7.5,11.5) | -1.76 |
| (12,20) | -0.75 (-1.1) |
| (8,12) | -2.23 (-3.3) |
| (11.6,7.3) | -2.53 (-2.2) |
| (100,120) | +7.25 (r2 -0.6, from its near-mode series cells) |

So the method reproduces r2's numbers where both measure log-form cells.

**Largest clean residuals** (within-shape sd < 1.2): (15.4, 61.69) -13.7, (13.4, 59.9) -12.7, (45.2, 60.9) -10.5 on
all 21 neighbours, (68.46, 28.54) -10.7, (12.7, 40.3) +5.5 on all neighbours, and small min(k, n-k) (< 3, sd 3.8-4.9).

## 5. Full models (`m1_model.py`)
All the models below share these pieces:
- k, n, nk as in R's stored form;
- X from r2 (series branch as r2);
- E kept extended into EXP V6, unless the chain says otherwise.

Their constants:
- **M3**: L5 = LN(n+1) (Excel), Cs = exact stirlerr, lf = LF0 with Excel LN pieces summed in extended.
- **M4**: M3 with each bd0 stored to a double.
- **M4s**: M4 with lf stored once.
- **M4x**: M4 with r2's x87-stored lf.
- **Mexact**: exact constant.
- **LDe-r2**: r2's stored chain, re-implemented. It reproduces r2's 5,765 / 25,254 / 31,385.

| model | sweep (11,264) | diverse (9,936) | grid (7,200) |
|---|---|---|---|
| M3 | 2,625 / 10,038 / 10,746 | 1,315 / 7,901 / 9,142 | 1,925 / 7,068 / 7,144 |
| **M4** | 2,621 / 10,038 / 10,746 | 1,333 / **8,006 / 9,222** | 1,930 / 7,072 / 7,146 |
| M4s | **2,831** / 10,025 / **10,774** | **1,339** / 7,987 / 9,218 | 1,808 / **7,075 / 7,152** |
| M4x | 2,122 / 9,958 / 10,673 | 1,153 / 7,765 / 9,112 | 1,490 / 6,947 / 7,120 |
| Mexact | 1,970 / 10,014 / 10,728 | 1,184 / 7,834 / 9,105 | 1,541 / 6,958 / 7,123 |
| LDe-r2 | 2,209 / 9,509 / 10,412 | 1,469 / 7,115 / 8,710 | 1,727 / 6,580 / 7,043 |

**Round-2 data** (r2-density-large's 32,264 cells with a, b > 1):

| subset | n | CR | LDe-r2 | **M3** | M4 | Mexact |
|---|---|---|---|---|---|---|
| all | 32,264 | 3,573 / 26,024 / 29,280 | 5,765 / 25,254 / 31,385 | **6,312** / 25,807 / 31,461 | 6,266 / 25,708 / 31,433 | 3,991 / **28,961 / 31,543** |
| round-1 runs | 18,440 | 1,127 / 12,956 / 15,456 | **3,071** / 13,227 / 17,561 | 2,448 / 13,447 / 17,637 | 2,403 / 13,342 / 17,609 | 1,565 / 15,751 / 17,719 |
| r2 probe p1 | 13,824 | 2,446 / 13,068 / 13,824 | 2,694 / 12,027 / 13,824 | **3,864** / 12,360 / 13,824 | 3,863 / 12,366 / 13,824 | 2,426 / 13,210 / 13,824 |
| min(a,b) >= 8 | 6,241 | 1,127 / 4,515 / 5,448 | 1,711 / 4,968 / 5,813 | **2,687** / 5,043 / 5,844 | 2,664 / 4,923 / 5,809 | 1,517 / 4,983 / 5,848 |
| 1 < min < 8 | 26,023 | 2,446 / 21,509 / 23,832 | **4,054** / 20,286 / 25,572 | 3,625 / 20,764 / 25,617 | 3,602 / 20,785 / 25,624 | 2,474 / 23,978 / 25,695 |

- On the round-1 runs, LDe's stored chain gets more bit-exact cells, even though a stored E is impossible at
  |E| >= 8. At |E| < 1 the EXP image cannot tell the chains apart, and LDe's constant happens to fit blocks such as
  (7.3, 11.6) at x 0.61.
- The constant, not the structure, limits every model to about 20-30% bit-exact.

## 6. Hold-out (`t2513-r3-loader-const-hold1`, 3,296 cells, read once after freezing M3/M4)

| model | all | sweeps (1,536) | shapes (1,760) |
|---|---|---|---|
| CR | 251 / 1,738 / 2,435 | 135 / 958 / 1,206 | 116 / 780 / 1,229 |
| LDe-r2 | 685 / 2,675 / 3,029 | 394 / 1,467 / 1,474 | 291 / 1,208 / 1,555 |
| Mexact | 658 / 2,987 / 3,161 | 445 / 1,511 / 1,511 | 213 / 1,476 / 1,650 |
| M3 | 952 / 2,990 / **3,176** | 658 / 1,522 / 1,522 | 294 / 1,468 / **1,654** |
| **M4** | **960 / 2,996** / 3,172 | 658 / 1,522 / 1,522 | **302 / 1,474** / 1,650 |

- On the hold-out, M3 and M4 reach 96% w15, against 92% for LDe and 74% for CR. They are 29% bit-exact, against 21%
  for LDe and 8% for CR.
- On the hold-out sweeps, (8.3,5.9), (2.7,6.4) and (17.1,11.3), they reach 99.1% w15 (1,522 of 1,536).

## 7. What is identified, and what is open
**Identified** (sections 1-3, hold-out confirmed):
- The structure:
  - E = L5 + Cs - lf/2 + X, all extended into EXP V6 (no stored E);
  - Excel's stored k = a-1, n = fl(a+b)-2, nk = fl(n-k), whose ε = n - k - nk causes the single-ulp jumps;
  - r2's x87 log-form bd0 at the bit level;
  - bd0 probably returned as a double.
- Accurate stirlerr. Every GAMMALN or double-rounded formula is rejected.
- L5 = Excel's LN(n+1), and LF0 with Excel's LN pieces.

**Open:**
- **The residual of the constant:** sd 1.7 on half-integer shapes (an n trend from +1.7 to -0.4), and 3-5 elsewhere.
- **Large shapes:** -10 to -14 units at n of 70-105 with pieces in 40-70. The 4-term series cut at 50 is a partial
  lead.
- **Small pieces:** shapes with min(k, n-k) < 3.
- **The large-k X misfit**, at t in [8,16).

## 8. Next hypotheses (most promising first)
1. **Large n.**
   - Single-ulp sweeps with every piece in (35, 120), at log-form x. They decide the series term counts and cuts
     (lead: 4-term cut at 50), and whether stirlerr(v > 15) is the series at all.
   - Grid shapes with k >= 6, to find what flips t's rounding at |t| in [8,16). Candidates: an ln of k/M computed as
     log1p of (k-M)/M near the cut, or a different log for ratios > 2.
2. **Pieces below 3.** Probe the stirlerr of small non-half-integer v directly. Keep n fixed and step a up and b down
   by the same ulp, so k and n-k move and n does not. Compare accurate methods, such as recursion from v + 1 or v + 2,
   and lgamma at 64 bits.
3. **The n trend on the grid.** Test ln 2π as a long-double constant, LN(n+1) against log1p(n), and lf/2 against
   0.5·lf with lf stored. The trend is about 2 units across n = 1..30.
4. **MSVC /fp:precise semantics.** Function returns are rounded to double, which fits bd0 stored. Test whether the
   stirlerr returns and the lf variable are rounded too: M4s, with lf stored once, is the best in-sample on two of three
   probes.

## 9. Scripts
| script | what |
|---|---|
| `r3lib.py` | Probe design and case writer (`python r3lib.py [--write]`), data loader with echo checks, EXP V6 inversion (stored and extended E), X variants (`xpart_v` forms, `xpart_t` t/M/bd0-store variants), exact constant, GAMMALN cache |
| `a1_pin.py` | Per-shape constant pin, stored or extended E, positive control (`pin_*.tsv`) |
| `a2_forms.py` | Argument forms: within-shape spread and per-shape offsets (`off_<probe>_R.tsv`) |
| `a3_const.py` | stirlerr / lf candidates against the needed offsets (`--formula` for the formula grid) |
| `a5_xreg.py` | X-part rounding regression, with the bd0-store, X-store and R-order candidates |
| `a6_struct.py` | Structure consistency: ext / g / g2 / lc / vs, t and M variants, `--bd0st`, `--base`, `--min-k`, `--control` |
| `a7_creg.py` | Matched regression of the constant residual on LN, k·ln(k/n), GAMMALN and formula roundings |
| `a8_series.py` | Series term-count cut scan |
| `a9_compose.py` | Enumeration of 896 lf / L5 / chain compositions |
| `a10_grid.py` | Grid fit of per-half-integer stirlerr corrections and the n trend (`grid_e.tsv`) |
| `m1_model.py` | Full models M3/M4/M4s/M4x/Mexact/LDe-r2 and CR; datasets sweep, diverse, grid, r2, hold |
