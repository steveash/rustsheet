# T-2513 r3-loader-bd0: the rest of Excel's Loader density for a, b > 1 (bd0 branches, integer shapes, large shapes)

Worker r3-loader-bd0, round 3, 2026-10-24. The base is r2-density-large's structure, `(a+b-1)·dbinom_raw(a-1, a+b-2,
x, fl(1-x))` with x87 stored ops on Excel's LN. This round re-implements it with numpy x87 arithmetic (`r3lib.py`) and
fits a free constant per block by exact interval inversion. Every count is bit-exact / within 4 ulp / within 15
significant digits ("w15"). "Units" are 2^-53, absolute in the exponent E. Python:
`/home/user/rustsheet/bindings/python/.venv/bin/python`, run from this directory.

## 0. Data, method, controls
- **Probes** (excel-graph, filled 2026-10-24, built by `mkprobe.py`, cases and sidecars in
  `eval/suites/probes-numeric/cases/`):
  - `t2513-r3-loader-bd0-a` (12,288 cells, in-sample): 8 shapes × 6 runs of 256 consecutive doubles. The runs are
    placed at p-side ratio v = (k - nx)/(k + nx) = ±0.10, ±0.19 (series, just inside the 0.2 cut) and ±0.30 (log
    form). Four shapes have n = a+b-2 a power of two, so n·x is exact and only the series' own arithmetic rounds:
    (20.5,45.5), (40.25,89.75), (100.5,157.5), (5.5,12.5). The other four do not: (20.3,30.1), (5.3,9.7),
    (40.7,55.3), (100,120).
  - `t2513-r3-loader-bd0-b` (4,992 cells, in-sample): 10 integer and half-integer shapes ((3,3), (4,4), (5,5),
    (4,6), (6,4), (3,5), (2,5), (4.5,4.5), (10,10), (6,9)) at v = ±0.15, ±0.30, 128 cells each.
  - `t2513-r3-loader-bd0-hold` (11,776 cells, **hold-out**): 8 other shapes ((12.7,33.3), (60.5,67.5),
    (24.5,41.5), (7.5,9.5), (150,90), (3,7), (6,6), (9.25,4.75)) at v = -0.25, -0.17, -0.07, +0.06, +0.15,
    +0.24. It was built and filled together with a and b, before any tuning, and scored once, after the model below
    was frozen (`final_hold.out`).
  - The x arguments are exact-piece formulas (w2's encoder). Every hold-out miss in the log-form blocks was
    rechecked at x ± 1 ulp: 0 of 114 matched (`ulpcheck.py`).
  - Round-1 data: r2's `dlib.load()` rows, of which the w2 `xruns` blocks are used for structure.
- **Scoring** (`r3lib.fit_block`, `final_score.py`). Each cell's admissible constants form an interval: the C values
  for which the model's chain gives exactly Excel's double, found by bisection through the exact EXP V6 model
  (`r3lib.exp_v6`, numpy x87 plus one correctly rounded F2XM1, equal to w1's mpmath model on 4,000 inputs). C is
  the deepest overlap on the even (md5) half, and the odd half is scored. The shape constant is the r3-loader-const
  worker's question, so every count here is a structure count with one free constant per block. An exact-endpoint
  version of the overlap was dropped: it scored 100% at rounding ties, a false positive (section 6).
- **Chain-free diagnostics** (`rmsfit.py`, `mfilter.py`, `sparse.py`, `phase.py`, `cmean.py`). D = ln(Excel) - exact
  ln pdf and dX = model X - exact X (exact Loader with exact n·x). The residual r = D - dX is regressed on the rounding
  errors of candidate intermediates. A slope of 1 on dX means the modelled roundings are Excel's.
- **Positive controls** (`control.py`, `control.out`): synthetic Excel values from the final model with C moved by
  +3.3 units. The fit recovers +3.29 to +3.31 and scores 122-127 of 122-127 on the odd half. The r2 knobs score
  49-87 of 127 on the same synthetic series data. For chain K at (40.25,89.75) the fit lands on an alias, -4.0,
  because lc's grid is coarse there, but it still scores 122/122.
- Housekeeping: caches (`eset_cache.pkl`) are git-ignored. Shared code is imported: r2-density-large `dlib` (data,
  Excel LN), w1 `lib` (EXP V6) and w2 `make_probes` (encoder).

## 1. Headline
1. **Series branch (small ratio).**
   - Excel's x part carries **+1 × the rounding error of a stored fl(2k·v)**, with v = fl(d/(k+M)) stored. The
     sawtooth has full amplitude, ulp/2 (`phase_3017.out`), and the weights are +0.87 to +1.24 on 12 series blocks
     (`sparse*.out`).
   - Every natural formula that stores 2k·v gives the *opposite* sign. Control: bd0 = fl(2kv) - d doubles the error
     instead of cancelling it.
   - With this term added (phenomenologically, knob `ph`) a large unmodelled noise remains, 17-40 units at
     (100.5,157.5) even with n·x exact. So the series is not R's or Loader's loop in any storage pattern tried. Its
     arithmetic is **not identified**.
2. **Log form at ratios beyond 2** (|v| > 1/3): M - k is **stored** (knob `ld`). Inside Sterbenz's range it is exact
   anyway. On the hold-out log-form blocks this gives 553 → 656 of 770 exact. The ratio, LN and the product t stay
   stored; each extended variant scores 0-71 against 113-121 (section 4).
3. **Exponent chain.** Where both sides take the log form and |E| is large, E is **not stored**:
   - The fully extended chain E = C + X (`Ce`) is bit-exact on the probe's log-form blocks: (40.25,89.75) 121/122,
     (5.3,9.7) 118/119, (5.5,12.5) 113/120.
   - Up to 46% of Excel's outputs (xruns (100,120)) and 808 of 1,600 (p1 (12,20)) are not EXP of any double E.
   - On small shapes with |E| < 1 the stored chain K (E = fl(L5 - h) + fl(C + X), stored) still wins: 3,505 against
     3,060 of 5,649 on xruns.
   - So the chain differs by shape or magnitude. This is open and belongs to r3-loader-const.
4. **(4,4) is not an exception.** The x87 model reproduces Excel's q-tie coefficient (+0.14 against Excel's +0.15,
   `tie44.out`) and sits at the noise floor (rms 0.53, floor 0.69). r2's prediction of -1.61 was the series-form
   linearisation, which does not apply when the q side takes the log form. Integer shapes take no table path: probe b
   is consistent with the general model (section 3).
5. **The cut is 0.2, confirmed from the q side.** Cut 0.25 drops (5.5,12.5) at v_q = 0.2028 from 113 to 13 of 120,
   and (40.25,89.75) at v_q = 0.2344 from 121 to 1. This puts the cut at or below 0.2028, consistent with r2's bracket (0.1837, 0.2000001].
6. **Large shapes.** n·x rounding enters exactly as modelled (slope of D on dX 0.96-1.09 on all 48 probe-a blocks).
   What is left at (100,120) and (100.5,157.5) is the series noise of item 1 (sd 17-41 units) and, on log-form
   blocks with |E| > 15, the chain.

## 2. Scores (free constant per block, odd halves; `final_score.py`)
Models:
- **r2-x87**: r2's knobs, cut 0.2, series extended, v extended, M - k extended.
- **r3**: v stored, the +δ(2kv) term (`ph=1`) and M - k stored (`ld=x`).

Chains:
- **C**: E = fl(C + X).
- **Ce**: E = C + X, extended.
- **K**: lc = fl(C + X), E = fl(fl(L5 - h) + lc).

| set | n | CR | r2-x87 (C) | r2-x87 (Ce) | **r3 (Ce)** | r3 (K) |
|---|---|---|---|---|---|---|
| **hold-out, all** | 5,831 | 667/3,863/4,743 | 1,777/4,248/4,914 | 2,040/4,594/5,170 | **2,234/4,828/5,313** | 1,880/4,218/5,009 |
| hold-out ll (both log) | 770 | 59/356/466 | 253/396/472 | 553/715/715 | **656/770/770** | 236/262/529 |
| hold-out ls | 998 | 116/665/833 | 269/697/860 | 264/741/893 | **283/836/902** | 271/781/877 |
| hold-out sl | 750 | 88/453/504 | 225/507/572 | 185/536/575 | 210/538/576 | **262**/503/535 |
| hold-out ss (both series) | 3,313 | 404/2,389/2,940 | 1,030/2,648/3,010 | 1,038/2,602/2,987 | 1,085/**2,684**/3,065 | **1,111**/2,672/**3,068** |
| in-sample probe a | 6,053 | 586/2,710/3,983 | 1,109/3,269/4,311 | 1,361/3,571/4,541 | **1,510/3,802/4,738** | 1,406/3,473/4,340 |
| in-sample probe b (integer) | 2,465 | 617/2,239/2,434 | 1,172/2,311/2,465 | 1,138/2,411/2,465 | 1,223/**2,427**/2,465 | **1,293**/2,323/2,465 |
| round-1 xruns | 5,649 | 216/4,095/4,767 | 3,230/5,189/5,544 | 2,990/5,141/5,503 | 3,060/**5,224/5,589** | **3,505**/5,208/5,584 |

Hold-out ablation of r3 (Ce) (`final_hold_ablate.out`):
- without `ph`: 2,110/4,665/5,219;
- without `ld`: 2,132/4,775/5,260;
- with v extended: 2,052/4,625/5,192.

Each piece adds 100-180 exact cells and 50-90 w15 cells out of sample.

Chain totals with the r3 knobs (`chains.out`, even/odd halves; old xruns, then probe a):

| chain | xruns odd half | probe a odd half |
|---|---|---|
| C | 3,261 | 1,225 |
| Ce | 3,060 | 1,509 |
| K | 3,505 | 1,406 |
| Ke | 3,333 | 1,354 |
| R | 3,184 | 1,250 |
| Rg | 3,466 | 1,406 |
| Rge | 3,294 | 1,354 |

No single chain is best on both sets.

## 3. Evidence per question

### 3.1 The small-ratio branch
- **The term.** The residual r is regressed on candidate roundings (`mfilter_old.out`, `sparse.py`). Out of about 40
  candidates the top feature is the rounding error of fl(2k·v_s), with v_s = fl(d/fl(k+M)) stored. Its |ρ| is 0.33 to
  0.50 per side, against a chance level of 0.03, and its weight is about 1 ulp of 2kv, on both sides:

  | block | p weight | q weight | sd before → after (units) |
  |---|---|---|---|
  | (30.7,1.7) x 0.81 | 1.07 | (log) | 4.91 → 4.25 |
  | (100,120) x 0.40 | 1.03 | 1.15 | 11.1 → 8.5 |
  | (7.3,11.6) x 0.27 | 1.03 | 0.9 | 1.81 → 1.63 |
  | (40,1.03) x 0.95 | 1.24 | (log) | 2.09 → 1.96 |
  | (100.5,157.5) v+0.10 (n·x exact) | 0.99 | 0.96 | 21.0 → 16.8 |

  With v unstored (2k·d/sm) the correlation disappears, so v is stored. Binned by the phase of 2kv on its ulp grid,
  the residual traces -ulp·frac exactly (0 → -3.95 → +4.38 → 0 for ulp = 8 units, `phase_3017.out`).
- **The sign is the puzzle.** bd0 ≈ 2kv - d. Any formula that stores 2kv and combines it linearly puts +δ(2kv) into
  bd0, and so -δ into X. Excel has +δ in X. Control: bd0 = fl(2kv) - d (`s0=ejd`) leaves twice the residual,
  amplitude 7.9 against 4.4 (`rmsfit` and `phase.py` with `s0=ejd`). The phenomenological knob `ph` subtracts
  fl(2kv) - 2kv from bd0.
- **Rejected forms of the series** (chain-free sd of D - dX, xruns (30.7,1.7) / (100,120); the r3 base is 4.91 /
  11.10; `rms_old.out` and the runs in this session's scripts):
  - each R variable (v, s, ej, term, v²) stored instead of extended: equal or worse (4.90 / 11.14);
  - the coefficient RN(1/(2j+1)) as a multiply (`coef=mul`): identical;
  - Loader 2000's s = (x-np)²/(x+np), d² stored or extended: 4.91 / 11.25;
  - s = fl(2kv) - d (`ejd`): 5.85 / 14.10;
  - ej = 2·M·v (`ejx=M`): 21.0 / 31.5;
  - bd0(M, k) with the arguments swapped: 164 / 124 (trends);
  - k·log1p(d/M) with Excel's log1p, series for a < 0 and LN(fl(1+a)) above (`lp1`): 15.4 / 53.6;
  - the same with the series for both signs (`lp2`): 5.59 / 15.2;
  - k·log1p(fl(k/M) - 1) (`lp3`): 15.2 / 65.5;
  - 2k·atanh(v) by Excel's 14-term series, then - d (`atv`, `atv2`): 4.74 / 10.20, slope 0.65 / 0.75;
  - n = a+b-2 kept extended (`n=e`): 143 against 202 exact on (7.3,11.6) x 0.27, and 160 against 478 on (7.3,11.6)
    x 0.61;
  - nk = b - 1 instead of n - k (`n=b`): 225 against 478 on (7.3,11.6) x 0.61;
  - n·x extended (`np=e`): 2.09 / 13.2;
  - nq = n - fl(n·x) (`nq=sub`): 2.27 / 16.96;
  - exact q = 1 - x (`q=ex`): 2.12 / 15.16.
- **What remains.** With `ph`, the power-of-two blocks of probe a (n·x exact, d exact) still have sd 3-41 units
  against a floor of about 1 (`rms_a.out`). Their spectra put 15-26% of the power in the lowest bins
  (`cmean`/FFT), so the noise is structured but unidentified.
- **The branch value is offset as well** (`cmean_a.out`). One shape should give one mean residual on every block. It
  does within the series blocks: (40.25,89.75) at -3.7 to -4.8 on all four ss blocks. The log-form block of the same
  shape sits at -14.66, with sd 0.49. The gap ss - ll is +10.5 units at (40.25,89.75), +6.7 at (20.5,45.5), -20 at
  (100.5,157.5) and about 1 at (5.x, …). The two branches therefore differ in value by a shape-dependent amount, not
  only in rounding. A single constant per shape cannot fit both.

### 3.2 The log form
- On all six log-form blocks with ratio inside [1/2, 2] the r2 structure with chain Ce is bit-exact or nearly so.
- Beyond a ratio of 2 (v_q = 0.38-0.57) Sterbenz no longer covers M - k, and storing it helps:
  (100,120) 57 → 71 of 132, (20.3,30.1) 42 → 58 of 112, (40.7,55.3) 44 → 63 of 133. fl(t + fl(M-k)) scores the same.
- Kept extended instead, each piece breaks blocks that are otherwise bit-exact:
  - the product t: 1-28 of 120;
  - LN, as FYL2X unstored: 35-71;
  - the ratio k/M: 18-35.
- Two blocks stay poor: (20.5,45.5) v-0.30 at 56/113 with sd 8 and no correlated feature, and (100.5,157.5) v-0.30 at
  65/138. Both have |E| > 6. They point to the chain (section 1, item 3) or a rounding of the shape constant that
  depends on x.

### 3.3 Integer and half-integer shapes, (4,4)
- The tie estimator applied to the model's own error reproduces Excel on all six xruns blocks (`tie44.out`):
  - (4,4): +0.14 against +0.15;
  - (7.3,11.6): -1.71 against -1.62;
  - (100,120): -11.36 against -11.37.
- At (4,4) the x87 model's rms is 0.53, at the floor. The alternatives are worse: n - fl(n·x) gives 1.77 and exact q
  gives 1.29.
- Probe b, 10 integer shapes: r3 (Ce) is w15 on 2,465 of 2,465, exact on 1,223 (CR 617). It shows no table or
  shortcut path.

## 4. Negatives in one place (decision 0056)
- **Series:** every alternative in section 3.1, plus fixed term counts (`nt`, no change at these v) and comparing the
  stored or the own-precision partial sum (`cmp`, no change).
- **Log form:** t, LN or the ratio kept extended; R's fl(fl(t+M)-k) (`ld=R`, r2 had already rejected it).
- **Cut:** 0.25 and 0.3, both rejected (section 1, item 5); 0.1, 0.15 and 0, all worse on (7.3,11.6) x 0.27 (98 or
  64 against 202).
- **Chain:** value-space forms out = fl(EXP(·)·(a+b-1)) in five variants score 2,011-2,296 against 4,122 on the xruns
  odd half. Stored E is impossible on large-|E| blocks (unreachable outputs).
- **Exact-endpoint interval overlap:** 100% hits that sit exactly on rounding ties, a method artefact. The bisection
  version is used instead.

## 5. Next hypotheses
1. **The +δ(2kv) series.** Try split or compensated arithmetic: a double-double 2kv with ej_hi and ej_lo, R ≥ 4.1's
   ebd0 (yh, yl), log1pmx-based bd0 and Kahan-summed loops. Test each with the same phase plot. Probe at n = 2^m
   with k + M exactly representable (sm exact), so that v's only rounding is the quotient's.
2. **The offset between branches.** Measure ll - ss mean offsets on many shapes, jointly with r3-loader-const. The
   series path may build its constant differently (for example, the lc composition or a stirlerr variant applied
   only when both sides are series).
3. **The chain.** Use reachability (the share of Excel outputs that are not EXP of any double) as a cheap per-block
   test of stored against extended E, across |E| and shapes.

## 6. Scripts
| script | what |
|---|---|
| `r3lib.py` | numpy x87 Loader x part with knobs; EXP V6; chains; interval fits |
| `mkprobe.py` | builds the three probe cases and their block metas |
| `scan.py` | free-constant fits per block, knob specs, chain choice |
| `final_score.py` | frozen-model scoring (exact/w4/w15 by branch, CR baseline) |
| `rmsfit.py` | chain-free sd of D - dX and the slope on dX |
| `mfilter.py` | matched filter |
| `mreg.py` | multiple regression |
| `sparse.py` | greedy raw-weight regression |
| `phase.py` | residual binned by the phase of a quantity |
| `cmean.py` | per-block mean residual (value offsets between branches) |
| `tie44.py` | r2's q-tie estimator on Excel and on the model |
| `resid.py` | per-cell admissible-constant intervals along a run |
| `control.py` | positive controls |
| `ulpcheck.py` | x ± 1 ulp misread check |

Outputs:
- `final_insample.out`, `final_hold.out`, `final_hold_ablate.out`, `chains.out`;
- `rms_old.out`, `rms_a.out`, `mfilter_old.out`, `sparse_a.out`, `phase_3017.out`;
- `cmean_a.out`, `tie44.out`, `control.out`.
