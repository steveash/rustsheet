# T-2586 r4-f-brcomp: is Excel's F density TOMS 708 brcomp / x?

Worker r4-f-brcomp, round 4, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: relative in the density, absolute in its logarithm. a = d1/2,
b = d2/2, h = a + b, m = d1, n = d2, mx = fl(m·x), r = the smaller of fl(mx/n) and fl(n/mx), s = fl(1 + r). "Side 1"
is mx <= n, "side 2" is mx > n. Run scripts from this directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
- **Rejected: F.DIST(x,d1,d2,FALSE) is not TOMS 708 brcomp(a, b, y, q)/x.** This holds in every branch: a0 >= 8
  Stirling (lambda, rlog1, bcorr), a0 < 8 log forms (gam1, gamln1, algdiv, betaln), every swap rule and every way of
  forming (y, q). The evidence:
  - It is worse than the correctly rounded value (CR) everywhere. On the untouched hold-out `hold4` it gets
    69/501/822 of 1,536, against CR's 111/646/904.
  - Its noise does not track Excel's. The within-run correlation is |r| <= 0.5, mostly < 0.25 (`s2_corr_a.out`).
  - Structurally it switches in the wrong place.
    - Excel's density responds to the rounding of s with slope about -h whenever r > 0.375. This holds at every
      shape, Stirling region included: on hold4, 15 of 15 runs with r > 0.375 have slope < -0.6h.
    - brcomp shows that slope in 3 of 15 (`s4_slopes_hold4.out`).
    - brcomp's alnrel switch is on the beta variable (x <= 0.375, i.e. r <= 0.6), and its Stirling branch sees only
      lambda.
  - Its stored terms are too small. a·u and b·v are below 32 at (300,300). Excel's ln F moves in quanta of 256-512
    units there (section 4), so it holds terms of magnitude 128-512.
- **Found: how Excel's x-dependence is stored** (step analysis on consecutive doubles, `s7`, `s9`, `s10`):
  - ln F = T + P - ln x, where P = fl(a·LN(x)) on side 1 and fl(-b·LN(x)) on side 2.
  - The 1/x is accurate: an unstored -LN x, or a value-space division.
  - T is a stored double holding the constant and -h·L(r).
  - After P is removed, Excel's step-to-step changes are exactly 0 (±2 units of EXP noise) or whole quanta of ulp(T).
    For r > 0.375, T = fl(C - h·LN(fl(1+r))) is consistent with one constant C per run on 48 of 48 points in most
    informative runs.
- **Best model, frozen before the hold-out was read** (M-xd, `ratlib` pw=xd, hl=ext):
  - Form: E = K + fl(c·LN x) - h·L(r) in extended, F = fl(EXP(E)/x).
  - c = a (side 1) or -b (side 2); L = Excel's log1p (series to 0.375, LN(fl(1+r)) above); K exact.
  - Hold-out `hold4` (16 new shapes): **121/781/945 of 1,536**, against CR 111/646/904 and round 3's ratio model
    118/709/912.
  - Round 3's hold-outs: `hold` 163/854/1,149 of 1,440 (CR 114/733/1,060, r3 161/776/1,091); `holdsw` 58/372/400
    (r3 71/373/400).
  - Gains over CR and r3 hold on every set except `sw`/`holdsw`. Those are x = 1, 2.75 and 1.6 sweeps, where r3's
    form is equal or better.
  - It is not bit-exact. The large-shape class is still poor: 7/140/237 of 768 on hold4.
- **New: the shape constant is NOT accurate at large shapes.**
  - Round 3's fact 1 (accurate to ±1 unit) was measured with min(d) <= 7.
  - Near the mode, M-xd's residual against Excel is flat within a run (sd 0.7-2 units on the x ≈ 0.95 and 1.05 runs)
    but sits on a per-shape offset. Excel minus exact K, in units: (16,40) -9, (30,30) -7, (40,100) -23, (60,60) -14,
    (100,30) +5, (100,200) +5, (160,160) -35, (300,300) -64.
  - When m != n, the offset also differs by side, by about one ulp of the c'·ln(m/n) term: (16,40) +23, (40,100) +73,
    (100,200) +127, (100,30) -62 (side 2 minus side 1).
  - Neither TOMS betaln (Excel LN/GAMMALN or TOMS gamln) nor any of six stored forms of c'·ln(m/n) reproduces the
    offsets (`s12_const_big.out`, section 5). This constant error is a main remaining source of visible misses at
    large shapes.

## 2. Probes, controls, hygiene
- **Probes** (probes-numeric, excel-graph). Both were built by `mkprobe.py` and filled on 2026-10-24 before any
  analysis. Rows are rebuilt from the generator, and x is checked against its echo in column A (0 misreads):

  | case | cells | content |
  |---|---|---|
  | `t2586-r4-f-brcomp-hold` | 3,072 | **hold-out** `hold4`: 16 shapes not in any r3 probe: (1,30) (2,7) (2,80) (4,9) (9,4) (11,11) (16,16) (18,64) (24,24) (32,90) (45,12) (64,64) (70,150) (120,40) (150,150) (200,200). 3 runs of 32 consecutive doubles each at x ≈ 0.37, 1.9, 13. Read once by `final_score.py` with the frozen models; `s4_slopes.py`/`ulp1.py` read it after that |
  | `t2586-r4-f-brcomp-big` | 4,608 | in-sample: 8 large shapes (16,40) (30,30) (40,100) (60,60) (100,30) (100,200) (160,160) (300,300). 6 runs of 48 each at x ≈ 0.21, 0.6, 0.95, 1.05, 2.4, 6 |

  The x values are exact doubles from w2-forensics' encoder, so there is no 15-digit literal truncation. The df are
  integer literals.
- **Also used:** round 3's probes `a`, `sw` (in-sample) and `hold`, `holdsw` (round 3's hold-outs, which I used for
  no choice), plus cells.tsv's F pdf cells. I loaded them through r3's `mkprobe.rows` and `flib.cells_rows`.
- **Positive controls:**
  - `s5_enum.py --control` and `s8_enum.py --control` recover a planted chain at residual sd 0.0 on 1,008 and 2,304 of
    2,304 cells (`s5_control.out`, `s8_control.out`).
  - `s10_tfit.py --control` recovers T = fl(C - h·xl) on 1,440 of 1,440 points. A wrong L form scores 931-1,349,
    which is the baseline for "wrong" (`s10_control.out`).
  - The reconstruction M-x equals round 3's ratio-xl-e cell for cell on every set (`final_*.out`). This checks the
    import of r3's pieces.
- **Reuse, by import only:**
  - r2-bratio `bratio_excel.Kernel` (TOMS brcomp/brcmp1, bcorr, rlog1, alnrel, betaln, Excel EXP/LN/GAMMALN), subclassed
    in `fblib.FKernel` for the knobs;
  - r3-fdensity `flib` (loaders, exact F, scorer, Excel LN) and `mkprobe` (row builder and readers, loaded by
    importlib);
  - `c1_model` (round 3's ratio model);
  - r3-tdensity `tdlib.log1p_xl` (Excel log1p);
  - r3-loader-bd0 `exp_v6`.
- **Housekeeping:** `.gitignore` covers `*.pkl` and `__pycache__/`. No .py file is over 170 lines, and the directory
  is 256 KB.

## 3. Hypothesis A: brcomp(a, b, y, q)/x (rejected)
Variants of `fblib.model` on probe a (3,168 cells; CR 337/1,958/2,638), `s0_first.out`:

| variant | counts |
|---|---|
| TOMS brcomp, w5 (y, q), bratio swap, Excel GAMMALN | 235/1,668/2,515 |
| + Excel log1p as alnrel | 226/1,672/2,536 |
| no swap | 236/1,671/2,512 |
| always swap | 235/1,673/2,502 |
| div (y, q) | 234/1,650/2,513 |
| TOMS gamln (final_score `brcomp`) | 236/1,605/2,492 |

On the other sets with TOMS gamln:

| set | brcomp | CR |
|---|---|---|
| sw | 225/1,572/1,782 | 585/2,167/2,227 |
| cells | 127/868/1,183 | 152/1,077/1,308 |
| big | 30/257/763 | 45/344/857 |
| **hold4** | **69/501/822** | 111/646/904 |
| hold | 95/634/943 | 114/733/1,060 |
| holdsw | 24/199/336 | 37/311/395 |

- **Free-offset check** (`s1_runs_a.out`): brcomp's within-run sd against Excel is larger than CR's in 65 of 66 runs.
  For example, (100,10,x1.9): Excel's own noise is 3.4 units, brcomp's is 11.8. The a0 = 5 log branch stores
  50·ln y, and Excel does not.
- **Argument forms:** (y, q) as r/(1+r) and 1/(1+r) ('rs', 'rsm') give the -h slope below r = 0.375, where Excel shows
  none, and miss it above 0.375 in the Stirling region (`s4_slopes_a.out`).

## 4. The x-dependence (what Excel stores)
- **Step analysis** (`s7_steps_big.out`). At (300,300), x ≈ 6, Excel's ln F changes by -2, 0, -513, +254, -257, ...
  units between consecutive doubles, while the exact ln F changes by -143 per step. So Excel's value is a sum of
  coarse stored doubles plus a small smooth part:
  - quanta of 512 and 256 at (300,300);
  - quanta of 256 and 128 at (100,200, x ≈ 6);
  - quanta of 256 at (300,300, x ≈ 0.21).
- **Removing the candidate power term** (`s9_resid_steps.out`). Subtracting P = fl(c·LN x) with c = a or -b, and the
  exact -ln x, leaves residual steps of exactly 0 (±2) or whole quanta of a single term:
  - +256/+512 at (300,300,x6);
  - +128 at (100,200,x6);
  - -256 at (300,300,x0.21).

  The other power forms leave mixed residuals: fl((a-1)·LN x) and fl(-(b+1)·LN x) (round 3's form), and fl(c·LN r)
  - ln x. At m = n, P with r and P with x coincide.
  - The size of that remaining term matches |K_side - h·L|. For example, K = -lnB(150,150) ≈ 213.5 minus h·L ≈ 46
    gives 167 at (300,300,x6), whose ulp is 256 ✓.
- **T consistency** (`s10_tfit.py`, runs with ulp(T) >= 16, 1,440 points; `s10_big_r*.out`):

  | L form | consistent points |
  |---|---|
  | h·xl (Excel log1p, extended product) | 1,219 |
  | fl(h·xl) | 1,204 |
  | TOMS alnrel | 1,183 |
  | fl(a·xl) + fl(b·xl) | 1,162 |
  | h·LN(fl(1+r)) always | 1,034 |
  | h·(-LN(fl(1/s))) | 889 |
  | exact log1p | 792 |

  - With r formed as fl(fl(m/n)·x), as one rounding, or as an inverse: 1,142-1,166 for the best form, so r = fl(mx/n) |
    fl(n/mx) stands.
  - With r through z = r/(1+r) and log1p(-z): 734-765 (rejected).
  - Runs with r > 0.375 mostly reach 48/48 with h·LN(fl(1+r)).
  - **Runs with r <= 0.375 reach only 23-44 of 48 under every L form.** At (300,300,x6), T jumps twice by 2 quanta (512)
    in one step. A single smooth T with ulp 256 rising 57 units per step cannot do that (`s11_jumps.out`).
  - So below the switch, the -h·L part lives in at least two stored terms, for example a-part and b-part pieces with
    their own constants. Or it is not Excel's 14-term atanh series of r. Not identified.
- **Chain enumeration with exact K and a free per-run offset** (`s5_enum_*.out`, `s8_enum_big.out`). The median
  within-run sd on big is:

  | chain | median sd | mean sd |
  |---|---|---|
  | CR | 31.3 | 56.4 |
  | r3 form (fl((a-1)LN x) etc.) | 8.5 | 32.4 |
  | xd (P = fl(c LN x), /x) | 5.4 | 18.3 |
  | xn (P = fl(c LN x), -LN x) | 5.6 | 18.2 |
  | T grouped as fl(K - hL) + P with exact K ('kl') | 6.7 | 21.6 |

  'kl' is worse than xd, because K's true stored value (section 5) sets where T rounds.

## 5. The constant (open)
- `s12_const_big.out`: the per-run median of (Excel - M-xd), in units, is flat in x within a shape and side. The
  values are listed in section 1.
- Candidates, as -lnB error in units:

  | shape | TOMS betaln (Excel LN, either gamln) |
  |---|---|
  | (16,40) | -13.8 |
  | (30,30) | +17.0 |
  | (40,100) | +11.6 |
  | (60,60) | -17.0 |
  | (100,30) | +4.2 |
  | (100,200) | +30.5 |
  | (160,160) | -68.7 |
  | (300,300) | -46.1 |

  Only (100,30) and roughly (60,60) agree.
- The side difference was tested against six stored or unstored forms of c'·ln(m/n): fl(c' LN(fl(m/n))),
  c'·LN(m/n) extended, -fl(c' LN(fl(n/m))), fl(c'(LN m - LN n)), fl(c' LN m) - fl(c' LN n), and c'(LN m - LN n)
  extended. None gives the observed +23 / +73 / -62 / +127 (inline check in this session; the `ratlib` knob
  `kside=st` made it worse: `s12_const_big_kst.out`).
- Next: probe x = 1 and x = 1 ± a few ulp at min(d) >= 16 on a dense shape grid, and fit the constant against Loader's
  stirlerr form (r3-loader-const) and a Stirling/bcorr form with stored pieces.

## 6. Scores of the frozen models (`final_insample.out`, `final_hold.out`)
| set | CR | brcomp | r3 ratio-xl-e (= M-x) | **M-xd** |
|---|---|---|---|---|
| a (3,168) | 337/1,958/2,638 | 236/1,605/2,492 | 398/2,185/2,693 | 399/**2,292/2,786** |
| sw (2,400) | 585/2,167/2,227 | 225/1,572/1,782 | 421/**2,224/2,310** | 411/2,202/2,294 |
| cells (1,599) | 152/1,077/1,308 | 127/868/1,183 | 172/1,052/1,297 | **192/1,208/1,382** |
| big (2,304) | 45/344/857 | 30/257/763 | 27/510/1,091 | 28/**533/1,106** |
| **hold4 (1,536)** | 111/646/904 | 69/501/822 | 118/709/912 | **121/781/945** |
| hold (1,440, r3's) | 114/733/1,060 | 95/634/943 | 161/776/1,091 | **163/854/1,149** |
| holdsw (400, r3's) | 37/311/395 | 24/199/336 | **71/373/400** | 58/372/400 |

By class on hold4:

| class | M-xd | CR | r3 |
|---|---|---|---|
| max(d) >= 50 | 7/140/237 of 768 | 26/143/229 | 15/139/230 |
| min(d) <= 2 | 37/165/169 of 192 | 29/151/164 | |
| other | 77/476/539 of 576 | 56/352/511 | |

On cells, min(d) <= 2, M-xd gets 91/607/678 of 743 against CR's 77/536/637.

±1-ulp check (`ulp1.out`): of M-xd's misses, 697 of 2,769 (a), 59 of 2,276 (big), 203 of 1,415 (hold4) and 249 of
1,277 (hold) are one ulp off. Most large-shape misses are many ulps off (the constant, and the r <= 0.375 T term).

## 7. Open, most promising first
1. **Excel's constant at large shapes** (section 5). Its error is up to 60-130 units and differs by side. This is now
   the main visible-parity blocker for the 15-digit target near the mode.
2. **The -h·L term for r <= 0.375.** It is consistent with neither one stored fl(C - h·series(r)) nor any tested L.
   The double-quantum jumps point to two stored pieces. Next: fit T as fl(C1 - a·L) + fl(C2 - b·L) (two free
   constants) by a 2-D interval search, on runs with m != n.
3. Whether the final 1/x is a value division or -LN x in extended. xd and xn tie within 1 unit (`s8_enum_big.out`);
   single-ulp runs at small shapes can separate them.

## 8. Scripts
| script | what |
|---|---|
| `fblib.py` | brcomp/x model (knobs args/swap/alnrel/rlog1/fin/gamln), loaders for every set, class scorer |
| `ratlib.py` | log-form chain family (pw/hl/sum/grp/est/kside knobs); M-xd is pw=xd, hl=ext |
| `mkprobe.py` | the two probes (`--write`) and `rows()` |
| `s0_first.py` | brcomp variants on probe a |
| `s1_runs.py`, `s2_corr.py` | per-run offset/sd against Excel; correlation of model and Excel noise |
| `s3_lattice.py` | sorted D per run with term sizes |
| `s4_slopes.py` | slopes on s1/s2 rounding, Excel against brcomp variants |
| `s5_enum.py`, `s8_enum.py` | chain enumeration by within-run sd (`--control`) |
| `s6_perrun.py` | per-run offset/sd of one chain |
| `s7_steps.py`, `s9_resid_steps.py`, `s11_jumps.py` | step quanta, steps minus a power term, jump attribution |
| `s10_tfit.py` | T = fl(C - hL) interval consistency (`--control`, `--r`) |
| `s12_const.py` | per-shape constant offsets against TOMS betaln (`--kside`) |
| `final_score.py`, `ulp1.py` | frozen-model scores; ±1-ulp misses |
