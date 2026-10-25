# T-2586 r4-f-small-df: the F density at small df, and Excel's shape constant

Worker r4-f-small-df, round 4, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: relative in the density, absolute in its logarithm. a = d1/2,
b = d2/2, h = a + b, m = d1, n = d2, side 1 is fl(m·x) <= n, r = fl(mx/n) (side 1) or fl(n/mx) (side 2), L = Excel's
log1p (14-term series to 0.375, LN(fl(1+r)) above). M-xd is r4-f-brcomp's best model, E = K + fl(c·LN x) - h·L(r)
(extended) with c = a | -b, F = fl(EXP(E)/x), K exact. Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
- **Found: Excel's shape constant.** It is the exact K plus a per-(shape, side) offset that follows a Stirling/Loader
  decomposition of K:

  K_side = (h - 1/2)·ln(h/o) + 1/2·ln(w/2π) + stirlerr(h) - stirlerr(a) - stirlerr(b), with (w, o) = (a, b) on side 1
  and (b, a) on side 2.

  Excel's offset from the exact K, in units, is

  delta = e(h) - e(a) - e(b) + C0 + [(h - 1/2)·L(fl(w/o)) - (h - 1/2)·ln(h/o)].

  - e(v) is r3-loader-const's per-half-integer stirlerr error table (`grid_e.tsv`), the same table that fits
    BETA.DIST's Loader constant. e(0.5) and e(1) fit as +0.22 and +0.06, so effectively 0.
  - The bracket is the rounding of (h - 1/2)·ln(h/o) when Excel forms it as (h - 1/2)·log1p(fl(w/o)) with its own
    log1p, the product kept extended. Above w/o = 0.375 that is LN(fl(1 + w/o)), and its rounding is the side
    asymmetry.
  - C0 = +5.33 is one global constant.
  - Fit: 56 (shape, side) groups (probe `mya`, max(d) <= 10, plus r3-tdensity's d1 = 1 runs). RMS 0.52 units, 55 of 56
    within 1 unit. The null (offset 0) has RMS 3.91, and every standard lnB/ratio construction has RMS 3.4-4.4
    (`s17_kfit.out`, `s19_sidefit.out`). A positive control recovers a planted side form at RMS 0.25, and wrong forms
    score >= 0.64 (`s19_control.out`).
- **Model M-sd** (`sdmodel.py`) is M-xd with that constant. It was frozen before the hold-out was read. On the
  untouched hold-out `myhold` (14 new small-df shapes):

  | model | myhold (1,344) | of which both df <= 10 (192) |
  |---|---|---|
  | CR | 104/688/882 | 5/110/158 |
  | M-xd | 146/828/1,006 | 0/113/167 |
  | **M-sd** | **314/919/1,029** | **57/180/192** |

  Everything that is still visibly off sits where |c·ln x| is large, which is the power-term problem
  (r4-f-power's lane): for example (1,25) side 2, (25,1) side 1, (200,2), (1,200).
- **The constant also explains the large shapes, out of sample.** The table and C0 were fitted at max(d) <= 30. Yet
  M-sd predicts r4-f-brcomp's measured large-shape offsets (its section 5) to about 3 units (`s24_bigconst.out`):

  | shape | measured | M-sd |
  |---|---|---|
  | (300,300) | -64 | -61 |
  | (160,160) | -35 | -32 |
  | (60,60) | -14 | -11 |
  | (40,100), side 2 minus side 1 | +73 | +61 |
  | (100,200), side 2 minus side 1 | +127 | +118 |

  At m = n the offset is (h - 1/2)·(LN(2) - ln 2), the double ln 2's error times h. Scores rise on every set (section
  5). r4-f-brcomp's `big` goes from 28/533/1,106 to 156/1,302/1,700 of 2,304, and its hold-out `hold4` from
  121/781/945 to 287/1,048/1,198 of 1,536.
- **The d1 = 1 "+3 to +5 unit offset"** (round 3) is this constant. With a = 1/2, e(1/2) = 0, so delta = e(h) - e(b)
  + C0 + side term. That is r3-tdensity's T-density constant pattern, e(h) - e(n/2) + 2.07, shifted by
  C0 - 2.07 = +3.3. The measured F-minus-T offsets per df (df 3, 4, 5, 6, 7, 8, 10, 12, 20, 30) are +1.7 to +4.1, mean +2.8.
- **Rejected: closed forms at df 2.**
  - Excel does not use (1 + 2x/d2)^-(d2/2+1) at d1 = 2, or x^-2·(1 + 2/(d1 x))^-(d1/2+1) at d2 = 2. It does not
    use 1/(1+x)^2 at (2,2) either.
  - The in-sheet POWER columns equal F.DIST on 164 and 185 of 1,792 rows, against 385 for CR (`s5_power.out`).
  - At (2,2), about 45 arithmetic and log/exp closed forms reach at most 25-33 of 64 per side. That includes an all-double
    CRT-style family (`s7_22.out`, `s15_dbl22.out`, `s16_jac22.out`).
  - d1 = 2 and d2 = 2 go through the general chain, with the same constant model.
- **Not reached: bit-exactness.** With the constant fixed, about 40% of small-df cells are bit-exact and about 99%
  are w15. A residual noise of about 0.5-0.9 units (one output ulp on about half the cells) remains, even at (2,2),
  where K = 0, c = ±1 and fl(c·LN x) = ±LN x is exact.
  - No final-step or grouping variant tested removes it (section 4.3).
  - The best variant, M-sdm (fl(EXP(E)·fl(1/x))), gains bit-exact cells in sample and on all hold-outs: myhold
    335/918/1,030.

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph). Both were built by `mkprobe.py` and filled on 2026-10-24 before any
  analysis. x is an exact-piece formula (w2-forensics' encoder), so no 15-digit literal truncation. Every x matches
  its echo in column A: `mkprobe.rows` asserts it, 0 misreads.

  | case | cells | content |
  |---|---|---|
  | `t2586-r4-f-small-df-a` | 15,872 | in-sample `mya`: d1 in {1,2,3,4} × d2 in {1,2,3,5,10,50,100} and the swaps (47 shapes) × 4 runs of 32 consecutive doubles at mx/n ≈ 0.12, 0.62, 1.61, 8.3 (side 1 series, side 1 log, side 2 log, side 2 series). The 14 shapes with a df of 2 also get C = POWER(1+d1·x/d2,-h) and D = POWER(1+d2/(d1·x),-h) |
  | `t2586-r4-f-small-df-hold` | 2,688 | **hold-out** `myhold`: (1,6) (1,25) (1,200) (2,6) (2,15) (2,200) (6,2) (15,2) (200,2) (25,1) (8,1) (3,7) (7,3) (4,30) × 4 runs of 24. Filled with `a`, read once by `final_score.py` after M-sd and M-sdm were frozen; `s21`/`s22` read it after that |

  Total 18,560 cells.
- **Other data, all scored with the frozen models:**
  - r3's `a`, `sw`, `hold`, `holdsw` and r4-f-brcomp's `big`, `hold4`, through `fblib.load`;
  - cells.tsv's F pdf cells (`cells`, which includes the t2220 sweeps);
  - r3-tdensity's d1 = 1 column G (`tdens`);
  - any probe case, parsed by verify-r3-fdensity's own `vlib.case_rows` (`case:<id>`).

  Only `mya` and `tdens` were used for any choice (the C0/e(.5)/e(1) fit and the side-form choice). `sw` was looked
  at (`s2_const.out`) but not fitted.
- **Positive controls.**
  - `s4_kscan.py --control`: a planted per-group delta is recovered to 0.05 units on 3,955 of 3,968 cells
    (`s4_control.out`). The scan's vectorised EXP misses about 0.3%.
  - `s19_sidefit.py --plant`: see section 1.
- **Reuse, by import only:**
  - r4-f-brcomp `fblib` (loaders, kernels) and `ratlib` (M-xd, `L_of`, `_lnB`);
  - r3-fdensity `flib` (exact F, tally, Excel LN, sidecar readers);
  - r3-loader-bd0 `r3lib` (EXP V6 on an extended argument, LN64, x87 store);
  - r3-tdensity `tdlib` (Excel log1p, w15);
  - r2-bratio `bratio_excel` (TOMS betaln with Excel's functions);
  - verify-r3-fdensity `vlib` (case parser);
  - r3-loader-const `grid_e.tsv` (read).
- **Housekeeping.**
  - `.gitignore` covers `*.pkl` and `__pycache__/`. My GAMMALN/GAMMA cache is `xl_cache.pkl`. Note: `flib.with_exact`
    still writes r3-fdensity's git-ignored `cache.pkl`, as it does for r4-f-brcomp.
  - No .py file is over 240 lines. The directory is about 420 KB without caches.
  - 0 duplicated 10-line windows against every tracked .py and all t2464 subdirectories (checked with
    `check_duplication`'s normaliser).

## 3. What Excel does at small df (structure)
Local-rounding regressions (`s13_local.py`, `s13_local5_P.out`). The target is the centre of Excel's admissible E
interval under F = EXP(E), regressed on the rounding errors of candidate pieces, per run, over 92 runs with max(d) <= 5:
- LN(x)'s error gets coefficient about c: +0.5 at a = 1/2, +1 at a = 1, +2 at a = 2, -1 to -2.5 on side 2. So the x
  power is c·LN x with c = a | -b, and the 1/x does not cancel that LN x error, even at (2,2).
- fl(c·LN x)'s product rounding gets about +1 (half-integer c). fl((c-1)·LN x) and fl(c·LN r) get about 0.
- L = LN(fl(1+r))'s error gets about -h on runs with r > 0.375 (two-regressor fit, `s13_local5_L.out`).

So M-xd's x-structure holds at small df. What remains after it is noise of about 0.5-0.9 units, larger than the 0.36
that interval quantisation explains.

## 4. Hypotheses and counts (negatives included, decision 0056)
### 4.1 Closed forms at df 2 (rejected)
- **POWER columns** (`s5_power.out`, 1,792 rows): F == C on 164, F == D on 0, F == D/x² on 185, F == D/x/x on 176.
  CR == F on 385.
- **(2,2) arithmetic** (`s7_22.out`, `s16_jac22.out`): 1/((1+x)(1+x)), (1/(1+x))², EXP(-2·LN(1+x)), EXP(-2L) with
  /x/x, /fl(x·x) or LN x forms, and y·q/x Jacobians in the div, w5 and 1 - y forms. Best per side 25 and 33 of 64,
  against CR 26 and 24.
- **All-double family at (2,2)** (`s15_dbl22.out`, 72 combinations of LN/log1p/EXP as Excel's or correctly rounded,
  E stored or extended, /x or not): best 33 of 64.

### 4.2 The power-term form at small df, free constant per (shape, side) (`s6_formscan.py`, 2,944 cells, max(d) <= 5)
Best count over delta, summed:

| form | xd | xn64 (fl(c LN x) - ln x unstored) | xnex | xn | exact P | x (r3) | r | rdiv | mx | mx1 |
|---|---|---|---|---|---|---|---|---|---|---|
| sum | 1,423 | 1,515 | 1,515 | 1,400 | 1,263 | 1,186 | 1,087 | 1,081 | 1,054 | 1,016 |

No form reaches the whole group, so even a perfect constant leaves about 50% bit-exact.

### 4.3 Final step and grouping (exact or free constant)
- **`s9_fin.py`** (exact K, max(d) <= 5, 2,944 cells): div 461, divx 452, mulr 549, mulrx 494, CR 450.
- **`s14_split.py`** (free constant, max(d) <= 3, 1,152 cells; split EXPs, value-space constants):

  | variant | count |
  |---|---|
  | m-xd | 613 |
  | xn64 | 623 |
  | s3-s8 (two EXP factors) | 524-576 |
  | c1-c4 (constant in value space) | 454-527 |
  | E stored (m-xd-st, xn64-st, xn-st, m-xd-st2) | 384-562 |
  | h·L stored | 602 |

- **`s11_einv.py`**: under fl(EXP_double(E)/x), 2 of 32 cells of (2,2) at x ≈ 0.12 are unreachable for any E. So
  "EXP rounded to double, then divided by x" is not literally Excel's last step. With F = EXP(E) and the ln x folded
  in, every cell is reachable.
- **With M-sd's constant** (`s20_variants.out`, max(d) <= 10):

  | final step | mya (3,968) | tdens (1,784) |
  |---|---|---|
  | div | 1,488/3,803/3,934 | 736/1,763/1,777 |
  | xn64 | 1,614/3,808/3,937 | 800/1,761/1,775 |
  | **mulr** | **1,654**/3,804/3,934 | **851**/1,762/1,776 |
  | divx | 1,615/3,808/3,937 | 800/1,761/1,775 |

### 4.4 The constant
- **Offsets** (`s4_kscan.out`, `s4_kscan_tdens.out`): the free delta per (shape, side), M-xd base, in units.
  - Both df odd: (1,1) +4.9/+4.6, (1,3) +4.9/+5.3, (3,3) +7.0/+7.3, (3,5) +8.0/+6.7.
  - One odd: (2,3) +0.5/+2.35, (3,2) +2.75/+0.1, (3,4) +3.3/-1.35, (4,3) -0.3/+3.45.
  - Both even: (2,2) -0.2/+0.2, (2,4) +1.45/+3.5, (4,2) +3.8/+0.5.
  - The offsets are swap-symmetric, (m,n) side 1 ↔ (n,m) side 2.
- **Rejected as the constant** (`s17_kfit.out`, 47 groups): 7 lnB forms × 8 ratio-term forms.
  - lnB forms: exact; GAMMALN in three orders; TOMS betaln with Excel's or TOMS's own pieces; LN of a GAMMA ratio.
  - Ratio-term forms: exact; fl(c·LN(fl(m/n))); extended; c(LN m - LN n); fl(a LN m) + fl(b LN n) - fl(h LN n | m),
    stored or extended.
  - Best RMS 3.39 (TOMS betaln), against the null's 3.91.
- **Accepted** (`s18_etab.out`, `s19_sidefit.out`):
  - e(h) - e(a) - e(b) alone correlates +0.79 with the offsets (RMS 1.55).
  - Side term candidates (RMS):

    | side term | RMS |
    |---|---|
    | fl(c·LN(fl(m/n))) | 2.03 |
    | extended | 1.98 |
    | fl((h-.5)·LN(fl(h/o))) | 2.58 |
    | extended | 2.40 |
    | fl((h-.5)·L(fl(w/o))) | 0.76 |
    | **(h-.5)·L(fl(w/o)) extended** | **0.52** |
    | + fl(.5·LN w) | 2.57 |

- **Ablation** (`s23_ablate.out`, w15):

  | constant | mya | sw | big |
  |---|---|---|---|
  | full | 5,060 | 2,400 | 1,700 |
  | no e(v) | 5,013 | 2,393 | 1,705 |
  | no side term | 4,983 | 2,353 | 1,081 |
  | C0 only | 4,905 | 2,340 | 1,087 |

  At large shapes the side term carries everything (above v = 28 e(v) is only the linear gauge). At small shapes
  both parts count.

## 5. Scores of the frozen models (`final_insample.out`, `final_hold.out`, `final_cases.out`)
| set | CR | M-xd | **M-sd** | M-sdm |
|---|---|---|---|---|
| mya (6,016) | 624/3,575/4,519 | 761/3,930/4,767 | 1,695/4,623/**5,060** | 1,865/4,624/5,058 |
| tdens (2,406) | 140/1,994/2,222 | 155/2,129/2,303 | 1,040/2,363/**2,398** | 1,166/2,360/2,397 |
| a (3,168) | 337/1,958/2,638 | 399/2,292/2,786 | 822/2,638/**2,941** | 897/2,640/2,941 |
| sw (2,400) | 585/2,167/2,227 | 411/2,202/2,294 | 1,073/2,384/**2,400** | 1,116/2,383/2,400 |
| cells (1,599) | 152/1,077/1,308 | 192/1,208/1,382 | 474/1,363/**1,462** | 523/1,363/1,461 |
| big (2,304) | 45/344/857 | 28/533/1,106 | 156/1,302/**1,700** | 153/1,304/1,698 |
| **myhold (1,344)** | 104/688/882 | 146/828/1,006 | 314/919/**1,029** | 335/918/1,030 |
| hold (1,440, r3's hold-out) | 114/733/1,060 | 163/854/1,149 | 337/1,119/**1,255** | 374/1,123/1,253 |
| holdsw (400, r3's) | 37/311/395 | 58/372/400 | 145/400/**400** | 192/400/400 |
| hold4 (1,536, r4-f-brcomp's) | 111/646/904 | 121/781/945 | 287/1,048/**1,198** | 309/1,048/1,199 |
| verifier hold (960) | 47/337/568 | 80/528/708 | 159/651/**784** | 192/650/781 |
| t2220-f-dist-sweep-pdf (1,000) | 111/703/845 | 145/777/883 | 337/890/**936** | 375/890/936 |
| t2220 d1-1 (300) | 14/179/228 | 18/209/242 | 70/238/**261** | 73/236/260 |
| t2220 d2-1 (270) | 22/170/208 | 25/196/232 | 52/208/**237** | 61/210/237 |

- One class regresses against M-xd: the verifier hold-out's min(d) <= 2 class, 133 against 140 of 160. All 7 lost
  cells are (1,30) side 2 at x = 59-153, where |c·ln x| ≈ 61-75 and both models are poor (13 and 20 of 40). That is
  power-term territory.
- **±1-ulp check** (`s22_ulp.out`): of M-sd's 1,030 misses on myhold, 356 are 1 ulp off, 249 are 2-4 and 425 are
  more than 4 (the power-term cells). Matches at x ± 1 ulp (135) are what a 1-ulp-noisy model gives by chance; the
  echoes show no parser misreads.

## 6. Open (most promising first)
1. **The power term at large |c·ln x|** (r4-f-power's lane). It is the remaining visible-parity blocker at small df.
   Examples: (1,25) side 2, (25,1) side 1, (8,1) side 1 at x ≈ 0.015, (2,200).
2. **The about 0.7-unit residual noise** behind the about 40% bit-exact rate. It survives every final-step,
   grouping and storage variant tried (section 4.3).
   - The constant's decomposition suggests that Excel evaluates ln F term by term as (h-1/2)·log1p(w/o) +
     1/2·ln(w/2π) + stirlerr pieces + the x terms. The order and storage of that sum is the next thing to enumerate,
     especially at large shapes, where r4-f-brcomp saw stored-T quanta.
   - C0 = 5.33 is probably 1/2·ln(w/2π)'s implementation (for example, a stored ln √(2π) or ln 2π constant) plus
     the table's gauge γ. It was not separated.
3. **e(v) above v = 28.** The linear gauge extrapolation works on sw, big and hold4, but it is not measured there.
   Non-half-integer df (truncation) was not probed.

## 7. Scripts
| script | what |
|---|---|
| `slib.py` | loaders (`mya`/`myhold`, fblib sets, `tdens`, `case:<id>`), Excel pieces, chain forms (`chain`), M-xd with a given K (`mxd_K`), vectorised EXP V6 for scans, harness cache |
| `mkprobe.py` | the two probes (`--write`) and `rows()` |
| `sdmodel.py` | **frozen M-sd / M-sdm** (`value(x, d1, d2, fin)`, `delta(m, n, side)`) |
| `s0_survey.py`, `s1_offsets.py`, `s3_ktable.py` | per-run D, offsets against M-xd, offsets with K pieces |
| `s2_const.py` | x = 1 sweep offsets against GAMMALN/TOMS/GAMMA lnB errors |
| `s4_kscan.py` | free-constant scan per (shape, side) (`--control`, `--set`) |
| `s5_power.py` | F.DIST against the POWER columns |
| `s6_formscan.py`, `s9_fin.py`, `s14_split.py`, `s20_variants.py` | power-term forms, final steps, split chains |
| `s7_22.py`, `s15_dbl22.py`, `s16_jac22.py` | (2,2) closed forms and families |
| `s8_look.py`, `s10_reg.py`, `s11_einv.py`, `s12_resid.py`, `s13_local.py` | per-cell views, E inversion, regressions |
| `s17_kfit.py`, `s18_etab.py`, `s19_sidefit.py` | constant constructions, the e(v) table, the side term (`--plant` control) |
| `s21_byshape.py`, `s22_ulp.py`, `s23_ablate.py` | per-run scores, ±1-ulp check, ablations |
| `final_score.py` | frozen-model scores by class |

`s24_bigconst.out` comes from an inline call of `sdmodel.delta` on r4-f-brcomp's eight large shapes.
