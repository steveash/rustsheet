# T-2586 r4-f-power: the stored power term of Excel's F density F.DIST(x, d1, d2, FALSE)

Worker r4-f-power, round 4, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: absolute in a logarithm, relative in the density.
a = d1/2, b = d2/2, h = a + b, m = d1, n = d2, mx = fl(m·x), L = Excel's LN(x) (a stored double). Run every script
from this directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
**The large-shape noise is two stored doubles, not a value-space power or an x87 pow.**
1. **The x-power is the stored product P = fl(c·LN(x)) of Excel's LN**, with c = a when mx <= n and c = b when
   mx > n. It enters lnF as +P or -P. The remaining x^-1 is applied apart from it (section 4.2).
   - Regression coefficient on P's own rounding is +0.9 to +1.2 on every large-shape run, on r3's probe a and on
     this directory's probe a (`s2_r3a.out`, `s2_a.out`).
   - The product uses the stored LN(x). LN64 (unstored) or exact ln x in the product doubles the residual
     (median 22.8 against 46.5 units, `m2_a.out`). The correctly rounded double ln x is the same as LN(x) here.
   - 13 other splits of -(b+1)·ln x into stored products all fit worse (`m3_a.out`, `s18_a.out`): fl((a-1)L),
     fl((b+1)L), fl(aL) - fl(hL), (a-1)L - hL, and h·LN(mx) forms.
2. **Then one stored sum: S2 = fl(fl(K_hi + H) + P)**, where:
   - H = -h·L(r), with r the smaller ratio and L Excel's log1p (r3's switch);
   - K_hi is a double close to the shape constant K';
   - the first sum S1 = fl(K_hi + H) is a stored double too.

   This is what the "lattice" is. S2 has magnitude 30-260, so its grid is 64-512 units. At x ≈ 100, its rounding
   depends on x only through the stored L (one value per run of about 6 consecutive x), as `s14_a.out` shows.
   - With K_in = K' + delta scanned per run, the structure reproduces Excel to sd 0.8 units at (100,100) and 0.7
     at (100,50), x ≈ 100. Unmodelled, D's sd there is 172 and 80 (`m9_a_HKP11.out`).
   - On the other runs it leaves 2-6 units at x ≈ 0.37 and 1.9. At x ≈ 13 and 100 it leaves isolated L-plateaus off
     by exactly ±1 ulp of S2 (±128 or ±256). All of them sit where S1 + P is an exact tie or exactly representable
     (`s24_off.py`).
   - The other orders (K+P first, P+H first, all six with stored or extended partial sums) and an unstored product
     fit worse (sd 47-109 at x ≈ 100, `m7_a_x100.out`, `m11_fine.py pext`).
3. **The constant behaves as a double-double.** The low part K_lo = K' - K_hi is added outside the rounded sum. With
   K_hi = fl(K'), the residual's mode is within about ±15 units of 0 on all 24 runs (`m12_a.out`). Without K_lo it is
   off by up to half an ulp of K'.
   - The phase of K_hi inside the sum is not fl(K'). The free per-run delta disagrees with fl(K') on 3 of 6 runs at
     x ≈ 100. The lnGamma-piece sum K_hi = fl(fl(Gh - Ga) - Gb) + R fixes (60,200), but not the others.
   - So K_hi is the main open item. It decides the remaining ±1-ulp plateaus.

Frozen model `dd` (`fmodel.py`): F = fl(EXP_V6(fl(fl(fl(K') + H) + P) + K_lo) / x).

## 2. Scores (frozen variants; hold-out read once, after freezing)
| set | CR | m1 (stored P, no stored sum) | **dd** | ddl (-LN x in E) | g3 (K_hi from lnGamma pieces) | dd2 (first-sum error carried) |
|---|---|---|---|---|---|---|
| this probe a (5,376, in-sample) | 62/610/1,336 | 142/1,208/2,200 | **676**/1,477/2,334 | 485/**1,524**/2,379 | 630/1,410/2,177 | 162/1,389/**2,436** |
| r3 probe a (3,168) | 337/1,958/2,638 | 386/**2,278**/2,782 | **474**/2,255/**2,807** | 453/2,260/2,806 | 453/2,290/2,780 | 401/2,252/2,804 |
| r3 sw (2,400) | **585**/2,167/2,227 | 421/2,225/2,311 | 372/2,186/2,286 | 382/2,215/**2,314** | 366/2,158/2,288 | 386/2,210/2,293 |
| cells.tsv F pdf (1,599) | 152/1,077/1,308 | 172/1,121/1,354 | 178/1,133/1,355 | 165/1,076/1,332 | 184/**1,162**/**1,362** | 189/1,140/1,354 |
| **hold (this dir, 1,536, untouched)** | 14/117/294 | 17/138/334 | 16/**276**/**483** | **35**/268/**483** | 16/264/420 | 18/159/368 |
| r3 hold (1,440) | 114/733/1,060 | **167**/848/**1,145** | 137/**922**/1,144 | 137/911/1,142 | 134/920/**1,146** | 144/859/1,143 |
| r3 holdsw (400) | 37/311/395 | 60/371/**400** | 65/368/**400** | **67**/368/**400** | 65/368/**400** | 57/375/**400** |

Files: `fmodel_a.out`, `fmodel_r3insample.out`, `fmodel_cells.out`, `fmodel_hold.out`.

- On the untouched hold-out (8 other large shapes × 4 other x centres, max(d) 120-240), `dd` is within 15 digits
  on 483 of 1,536 cells, against CR's 294 and r3's ratio model (`m1`) 334. Within 4 ulp it is 276 against 117.
- Bit-exactness on the hold-out is not reached (16-35 of 1,536). The K_hi phase is wrong on most shapes, and every
  wrong plateau costs a whole ulp of S2.
- In-sample bit-exact rose 62 → 676 (`dd`). Those cells sit mostly on runs where fl(K') happens to give the right
  phase.
- r3's role here:
  - r3's `hold` was not used for any choice in this directory. r3 itself read it once, and the r3 verifier read it.
  - The cells.tsv cells were not used for any choice either.
  - r3's `a` was used for the first regressions (sections 4.1-4.2).
- At x = 1 (r3 sw), Excel has no first-sum rounding: sd 0.6-0.9 units, against `dd`'s 3.7 at d1 = 7
  (`s26_x1.out`). `dd2` carries the first sum's error and fixes x = 1, but it loses the in-sample bit-exact cells
  (676 → 162). See section 5, item 2.

## 3. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph, both built by `probe4.py` and filled on 2026-10-24 before any analysis of
  them; A = x by w2's exact two-piece encoder, B = F.DIST(A, d1, d2, FALSE); `probe4.rows` asserts the echo equals x
  on every row, so there are 0 misreads):

  | case | cells | content |
  |---|---|---|
  | `t2586-r4-f-power-a` | 10,752 | in-sample: (100,100), (50,100), (100,50), (60,200), (50,50), (20,100) × x ≈ 0.37, 1.9, 13, 100 × 224 consecutive doubles |
  | `t2586-r4-f-power-hold` | 3,072 | **hold-out**: (70,140), (120,80), (90,90), (40,160), (150,60), (80,240), (64,128), (110,55) × x ≈ 0.45, 2.6, 9, 60 × 48 |

  Total 13,824 cells.
- **Hold-out discipline.** The hold-out was filled with `a`. It was read once, by `fmodel.py hold`, after the
  variants list and code were frozen. Any later look at it is not a hold-out.
- **Positive control** (`s27_control.py`, `s27_control.out`).
  - The data are synthetic Excel values from the HKP11 structure with a known K_in on the real x runs.
  - The delta scan recovers the true delta to within the first sum's grid with sd 0.3-0.4 units. The wrong order
    (KPH11) gives sd 62-64.
  - The real-data fits at (100,100) and (100,50), x ≈ 100 (sd 0.8 and 0.7) are as tight as the control.
- **Reuse, by import only:**
  - r3-fdensity `flib` and `mkprobe` (exact F, loaders, scorer, GAMMALN cache);
  - r3-loader-bd0 `r3lib` (x87 helpers, Excel LN, EXP V6 on an extended argument, LN64);
  - r3-tdensity `tdlib` (Excel log1p);
  - excelmath (fl53/fl64).

  The exact values are cached in r3-fdensity's git-ignored `cache.pkl`, and the EXP sets in r3-loader-bd0's
  `eset_cache.pkl`.
- **Duplication.** `scripts/check_duplication.py`, with this directory's .py files added, reports 3,103 duplicated
  lines, the tracked-only baseline. Run helpers are shared through `plib.runs`, `plib.run_rows`,
  `s6_scan.base_fit` and `s6_scan.scan_extra`.

## 4. Hypotheses and counts (negatives included, decision 0056)
### 4.1 A value-space power or Excel's POWER (rejected)
- Value-space or square-and-multiply forms (POWER with an integer exponent) would add noise of a few units that
  depends on all the bits of x. Excel's noise instead depends on x only through the stored LN(x): grouped by L, the
  within-group sd is 3-6 units against 58-131 overall at x ≈ 100 (`s14_a.out`).
- Excel's ^ path, EXP(fl(y·ln x)) with libm ln, gives the same as the stored LN product on these cells (`m2_a.out`
  'cr'). It is the product, not POWER's EXP, that matters, and the products are summed in log space.
- x87 FYL2X-style products in log2 units, c·log2(v) stored: no run picks them (`s6_scan.py`).

### 4.2 Which ln x and which coefficient (`s2_reg.py`, `m2_lnx.py`, `m3_xform.py`, `s18_combo.py`)
- P's own rounding has a coefficient of about +1 on every run.
- Excel's LN error coefficient is about (a-1) on side 1. On side 2 it is between -(b+1) and -b: (100,50) gives
  -23..-25 with b = 25, b + 1 = 26.
- The remaining -ln x is taken in value space (division by x), or as -LN(x) in the exponent. The two cannot be told
  apart on the sd (`m2_a.out` LN/LN = LN/L64). The model keeps the division (`dd`); `ddl` puts -LN(x) in the
  exponent.
- The best stored-product combinations for side 2 (median residual sd over the 6 x ≈ 100 runs, `s18_a.out`):

  | form | median sd |
  |---|---|
  | -fl(bL) - L (kept) | 75 |
  | fl((a-1)L) - fl(aL) - fl((b+1)L) - L | 112 |
  | others | 113-150 |

- Naive no-flip forms with S = fl(1 + mx/n) on side 2 are 3 to 6 times worse (`m4_a.out`).

### 4.3 Extra stored terms and LN errors (rejected; `s1_corr.py`, `s4_resid.py`, `s6_scan.py`, `s8_lnerr.py`, `s9_sum.py`)
- After P, eLN(x) and eL, no extra single regressor removes the large-x residual:
  - products c·LN(v) or c·log2(v), with c in {a, b, h, a±1, b±1, h-1} and v in {x, mx, r, 1+r, n+mx, y, q};
  - LN errors at those v;
  - roundings of mx, r, s, y, q, n+mx;
  - fl(P + H) with P stored or extended.
- h·L stored as a product (rhL) has a coefficient of about 0 (the r3 finding holds).
- The final exponent is not a stored double (`s7_r3a.out`): Excel's value is EXP_V6 of some double on 0-1 of 48
  cells at |lnF| > 32.

### 4.4 Partial-sum orders (`s3_order.py`, `m7_flex.py`, `m8_pieces.py`, `m9_kscan.py`, `m10_sse.py`, `m11_fine.py`)
- **Regressing on the sum roundings with K = fl(K') fails** (`s3_r3a.out`, coefficients about 0). The phase of the
  constant inside the sum is not fl(K'), so the regressors carry the wrong phases. Only the free-K scan (`m9`) shows
  the structure.
- **Free K_in scan (step 2 units, ±260).** HKP11 = fl(fl(H + K) + P) is the best order on every one of the 24 runs
  (`m9_a_HKP11.out`):

  | x centre | residual sd |
  |---|---|
  | 0.37 | 2.2-15.6 |
  | 1.9 | 2.2-5.2 |
  | 13 | 25-89 (the tie plateaus, below) |
  | 100 | 0.7-149 |

  Order and storage (x ≈ 100 runs, each at its best K_in):
  - KPH and PKH with any storage flags: 47-109.
  - HKP with the first sum extended (01): 42-49 at (50,50).
  - HKP with the product kept extended: 59-142.
  - SSE rounding (one correct rounding) instead of x87 changes nothing: 12 off cells at (50,50) both ways
    (`m10_sse.py`).
- The off plateaus: at each one, the exact S1 + P is a tie (frac 0.5) or exactly on S2's grid (`s24_off.py`), and
  Excel lands one ulp of S2 away from ours.
  - Our S1 and Excel's are probably not the same double there. The candidates are the K_hi phase and Excel's
    positive-argument log1p (statistics.md marks it UNVERIFIED for a > 0) inside H.
- The constant split into nearest-double lnGamma pieces summed in with the x terms (`m8_pieces.py`) does not fit
  (114 against 70 at (100,100), x ≈ 100).

### 4.5 Earlier wrong turn, recorded
`s13_plateau.py`, `s15_cross.py` and `s16_two.py` first defined Excel's rounding total as res - rho instead of
res + rho. That made the plateau-level "grid" analysis meaningless until it was corrected. The corrected versions
are the committed ones. `s15`/`s16` (one extra rounding of fl(K1 + c·L) for a free constant K1, or a second product)
found no fit after the correction either. They are superseded by the order scan of 4.4.

## 5. Open (most promising first)
1. **K_hi's phase.** It decides every remaining ±1-ulp plateau.
   - Per run, the best delta (mod the S1 grid) is in `m9_a_HKP11.out`, and fl(K') is right on about half the runs.
   - Next: candidates for K_hi from how Excel would form the constant, for example:
     - TOMS-style lnB through `betaln`/`algdiv` with Excel's own pieces;
     - lnGamma(h) - lnGamma(a) - lnGamma(b) as a double-double;
     - a·ln(m/n) as fl(a·LN(m)) - fl(a·LN(n)).
   - Score them by the off-plateau count on `a` (`m12_khi.py` is the harness), not by sd.
2. **x = 1 against the first-sum rounding.** Excel is accurate at x = 1 (sd under 1 unit), but HKP11 rounds
   S1 = fl(K_hi + H) there.
   - Either the first sum's error is carried (`dd2`), and something else makes the large-x plateaus, or x = 1
     (P = 0) takes another path.
   - A probe at x = 1 ± k ulps, at shapes where fl(K' + H) has a coarse grid, would separate the two.
3. **Positive-argument log1p.** H = -h·L(r) with the series for r <= 0.375 is unverified. At x ≈ 0.37, side 1 has
   r = 0.37 just under the cut, and the residual sd is 2-16 there. A Loader-style test (pin L from runs where
   h·L(r) dominates) would settle it.
4. **Side 1 with large a** ((100,100) and (100,50) at x ≈ 0.37): sd 4-6 with K_in free. The structure may differ
   there (P = +fl(a·L) with |P| about 50).
5. d <= 2, non-integer df and the d1 = 1 offset were not touched here (other r4 workers).

## 6. Suggested statistics.md update (for the lead; not edited here)
Add to "F density is not the beta density" (T-2513):
- The x-power is fl(c·LN(x)) (c = a for mx <= n, c = b otherwise; Excel's LN, the product stored), summed as
  S2 = fl(fl(K_hi + H) + P) with H = -h·log1p_xl(r). The constant's remainder K' - K_hi is outside the rounding,
  and x^-1 is apart.
- This explains the large-shape "lattice": it is one ulp of S2.
- Hold-out t2586-r4-f-power-hold: 16/276/483 of 1,536 (CR 14/117/294; r3's ratio model 17/138/334).
- Open: K_hi's phase; no first-sum rounding at x = 1.

## 7. Scripts
| script | what |
|---|---|
| `plib.py` | loaders (`load`: this dir's probes, `r3:<name>`, `cells`), `runs`, `run_rows`, D in units |
| `probe4.py` | the two probes (`--write`) and `rows(name)` |
| `cands.py` | Excel-stored intermediates (mx, ratios, LN pieces, Excel log1p) and candidate stored terms |
| `s0_look.py`, `s1_corr.py` | D along r3's large runs; single-term correlations |
| `s2_reg.py` | separated-residue regression (P's rounding, eLN(x), eL, h·L product) |
| `s3_order.py` | partial-sum order regression with K = fl(K') (negative) |
| `s4_resid.py`, `s6_scan.py`, `s8_lnerr.py`, `s9_sum.py` | extra-term scans (negatives); `s6_scan` holds `base_fit` and `scan_extra` |
| `s5_show.py`, `s10_seq.py`, `s21_show.py`, `s22_show7.py`, `s23_kshow.py` | residual sequences of one run |
| `s7_lattice.py` | is the exponent a stored double (no) |
| `s11_phase.py`, `s15_cross.py`, `s16_two.py`, `s17_grid.py`, `s19_cscan.py` | hidden-constant and second-term scans (negatives) |
| `s12_toggle.py`, `s13_plateau.py`, `s14_group.py` | plateau analysis (Excel's term is a function of the stored LN(x)) |
| `s18_combo.py` | side-2 stored-product combinations |
| `s24_off.py`, `s25_k0.py`, `s26_x1.py` | off-plateau ties; delta0 per shape; x = 1 accuracy |
| `s27_control.py` | positive control for the K_in scan |
| `m1_fixed.py`, `m2_lnx.py`, `m3_xform.py`, `m4_naive.py` | fixed-coefficient model variants |
| `m7_flex.py`, `m8_pieces.py`, `m9_kscan.py`, `m10_sse.py`, `m11_fine.py`, `m12_khi.py` | sum orders, K_in scans, K_hi candidates |
| `fmodel.py` | the frozen variants and the scorer |

Outputs: `*.out` files of the same names.
