# T-2586 r5-f-edges: the edges of Excel's F density

Worker r5-f-edges, round 5, 2026-10-24.

Conventions:
- Counts are bit-exact / within 4 ulp / within half a unit of the 15th significant digit ("w15"), "of n".
- An error value or a 0 counts as a match only when it is the same error or 0.
- "Units" are 2^-53: absolute in the exponent, relative in the density.
- m = d1, n = d2 (both truncated), a = m/2, b = n/2, h = a + b, mx = fl(m·x).
- Side 1 is mx <= n; P = fl(c·LN x) with c = a (side 1) or -b (side 2).
- M-sd is r4-f-small-df's frozen model (`sdmodel`), imported and unchanged.

Run the scripts from this directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
**Model to import.** The model is `edgemodel.value(x, d1, d2, fin="div")` in
`docs/research/t2464/r5-f-edges/edgemodel.py` (M-edge). It returns a float or an error string. `fin="mulr"` gives
M-sdm's last step.

M-edge is M-sd's value chain, unchanged, inside Excel's argument and range rules. Every rule below was measured on
the in-sample probe and then confirmed on the untouched hold-out with no misses:

1. **df truncation.** d1 and d2 are truncated toward zero. Non-integer df give the truncated df's value bit for bit:
   352 of 352 twins in sample and 143 of 143 on the hold-out. Examples: `F.DIST(x,1.5,1)`, `(x,2.5,3.25)`,
   `(x,150.5,99.9)`, `(x,1.75,2.5)` and `(x,30.5,2.999)` all equal their integer twins. `2-2^-52` and `1+2^-52` act
   as 1; `3-2^-51` acts as 2.
2. **df limits.** A truncated df below 1 is `#NUM!`: 0, 0.5, 0.75, 0.99999, 0.9999999, -1 and `1-2^-53`, in either
   position. Truncation happens before the 1E10 cut: 10000000000.25, .5, .75 and .9999 give the 1E10 value.
3. **x < 0** is `#NUM!`. That includes -2^-900 and -2^-1000.
4. **x = 0** depends on trunc(d1) only, whatever d2 is (1, 2.5, 3, 7, 50.5, 1E10):
   - trunc(d1) = 1 is `#NUM!` (d1 = 1, 1.25, 1.5, 1.999999);
   - trunc(d1) = 2 is exactly 1 (2, 2.5, 2.75, 2.99999);
   - trunc(d1) >= 3 is 0 (3, 3.0001, 3.5, 4, 5, 10, 1E10).

   This settles T-2095, including the truncation case it left open.
5. **A subnormal x is `#DIV/0!`.** That covers `F.DIST(EXP(-720),d1,5,FALSE)`, `EXP(-730)` and `EXP(-740)` at
   d1 = 1, 2, 3 and 4, including d1 = 1 and 2. So x is not flushed to 0 first. A division by a subnormal is a
   division by zero here, as in "A subnormal divisor is zero". x = 2^-1022 (the smallest normal) is evaluated.
6. **The d1·x overflow.** The result is `#NUM!` exactly when fl(trunc(d1)·x) overflows a double. It was measured to
   the ulp at m = 2, 3, 7, 10 (in sample) and 5, 6 (hold-out):
   - at m = 2, x = 2^1023 - 2^970 is 0 and 2^1023 is `#NUM!`;
   - at m = 3, fl(DBL_MAX/3) itself is `#NUM!`.

   The product uses the truncated df: `F.DIST(DBL_MAX,1.5,3,FALSE)`, `(…,1.99,3)` and `(…,1.25,3)` are 0, not
   `#NUM!`. With d1 = 1, x = DBL_MAX is 0.
7. **Underflow is the EXP(E)/x structure's.** Here E = ln(F·x) is the exponent and Y = EXP(E) is stored as a double:
   - **F is 0 where Y underflows, even when the density is a normal double.** `F.DIST(1E-300,3,5,FALSE)` is 0
     (exact 2.4E-150), and so are `F.DIST(1E-200,4,5,FALSE)` (exact 5.6E-200) and `F.DIST(1E-200,5,1,FALSE)`.
   - Where Y is subnormal, Excel's F carries Y's lost bits. On the four `thresh` runs, F·x is k·2^-1074 times a
     constant per shape, with k = 2, 8, 39, 183, … At k = 2, F is about 10% off the exact density.
     `F.DIST(2.4266958238320403E-216,3,5,FALSE)` is 4.0719206831704E-108, where the exact value is 3.687E-108.
   - **A subnormal F is 0.** At huge x, F below 2^-1022 is flushed even in the ×2^600 column. 5.9E-308 is kept and
     8.5E-309 is 0 (`hthr`, `huge`). The flush applies to the result only: a subnormal Y inside the chain is kept.

   M-edge models all three exactly: Y = EXP V6 stored, F = fl(Y/x), then F < 2^-1022 becomes 0.

**Scores of the frozen M-edge on the untouched hold-out** (`final_hold.out`, probe `t2586-r5-f-edges-hold`, 1,396
cells). The hold-out was filled before any analysis and read once.

| section | CR + rules | **M-edge** |
|---|---|---|
| all | 312/827/990 | **459/960/1,094** |
| rules (zero, neg, dflow, dfhigh, ovf, sub `#DIV/0!`; 58 cells) | 50/58/58 | **52/58/58** (every error, 0 and 1 right) |
| non-integer df twins (B = D) | | **143/143** bit-exact |
| frac (non-integer df values, 128) | 34/103/123 | 56/108/127 |
| vrun d1 = 1 (288; (1,9) (1,40) (1,2)) | 41/185/212 | 87/226/250 |
| vrun d1 = 2 (288; (2,9) (2,40) (2,1)) | 65/198/227 | 84/239/251 |
| vrun d2 = 1 (192; (9,1) (40,1)) | 21/93/120 | 44/123/133 |
| vrun d2 = 2 (192; (9,2) (40,2)) | 18/102/154 | 40/109/171 |
| tiny x (48) | 14/17/20 | 19/20/23 |
| thresh, Y subnormal (80) | 0/0/1 | 8/8/9 |
| huge x (36) | 25/25/27 | 25/25/28 |
| hthr, F at the subnormal edge (64) | 44/44/46 | 44/44/44 |
| sub, x ≈ 2^-1022 at d1 = 2 (22) | 0/2/2 | 0/0/0 |

In sample (`final_insample.out`, 1,437 cells): M-edge 737/957/1,035 against CR + rules 579/847/914.
- Rules (zero, neg, dflow, dfhigh, ovf, sub `#DIV/0!`; 111 cells) 103/111/111: every error, 0 and 1 is right.
  The 8 cells that are not bit-exact are numeric dflow and dfhigh values, all within 4 ulp.
- thresh 16/16/25 of 160. tiny 222 of 360. huge 271 of 320. ovf 35/35/35.

On the round-3/4 sets M-edge equals M-sd cell for cell, because the rules do not fire there (`final_other.out`):

| set | d1 = 1 | d1 = 2 | d2 = 1 | d2 = 2 |
|---|---|---|---|---|
| cells | 123/342/366 of 407 | 63/128/129 of 133 | 80/258/288 of 320 | 46/81/81 of 83 |
| myhold | 48/162/184 of 288 | 76/196/218 of 288 | 33/135/146 of 192 | 77/182/220 of 288 |

**Not reached: visible parity at the edges.** Wherever |P| is large, the remaining misses are one hidden rounding,
and this round measures it cleanly (section 3):
- tiny x on side 1;
- huge x on side 2;
- d1 = 1 with large x;
- d2 = 1 with tiny x.

Excel's exponent is E = fl(Q + P) + R, with Q a double that is **not** the nearest double to the constant. P's grid
(up to 1,024 units) rounds Q, so Excel is up to ±512 units off any model that does not know Q. Q's formula is not
found.

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph). Both were built by `edgeprobe.py` and filled on 2026-10-24 before any
  analysis.
  - Column A echoes x. 0 echo mismatches.
  - x is an exact formula that never forms a subnormal intermediate: `(H*2^26+L)*2^E`, or `…*2^-500*2^(E+500)` for
    E < -1000.
  - Cells:
    - B = F.DIST(A,d1,d2,FALSE);
    - C = B·2^600 where the density is below 1E-200, because Graph reports a kept subnormal as 0 (decision 0042);
    - D = the truncated-df twin.

  | case | cells | content |
  |---|---|---|
  | `t2586-r5-f-edges-a` | 3,981 | in sample: zero 44, neg 6, dflow 16, dfhigh 4, frac 336 (7 non-integer shapes × 3 runs of 16 with twins), tiny 360 (36 shapes × x = 10^-k, k = 5…307), thresh 160 (4 underflow runs of 40), sub 28, huge 320 (20 shapes × x = 10^k, k = 10…308), hthr 128 (4 runs into the subnormal F range), ovf 35 |
  | `t2586-r5-f-edges-hold` | 3,166 | **hold-out**: the same sections with other values and shapes, plus vrun 960 (10 shapes with a df of 1 or 2 × 4 runs of 24 consecutive doubles at mx/n ≈ 0.07, 0.55, 1.9, 13). Read once, by `final_score.py probe hold`, after M-edge was frozen. `s16_crrules.py hold` (the CR baseline) ran after that |

  Total 7,147 cells.
- **Other data.** cells.tsv's F pdf rows, `mya`, `tdens`, `myhold`, round 3's `hold` and r4-f-brcomp's `hold4`, all
  through r4-f-small-df `slib.load`. No choice was made on them.
- **Positive controls.**
  - `s11_qsearch.py --plant +gh-gb`: a planted Q is recovered on 46 of 46 groups. The next best candidate reaches 31
    (`s11_control.out`).
  - `s12_orders.py --plant L5:300`: the planted ordering (and its ψ-equivalents) reaches 46 of 46. Every other
    decomposition reaches 11-16 (`s12_control.out`).
- **Reuse, by import only.**
  - r4-f-small-df `sdmodel` (M-sd's constant `K_of`, `delta`, `value`) and `slib`: loaders, Excel LN, log1p, EXP V6,
    `side_r`, `chain`, `K_exact`, `GL`.
  - r3-fdensity `flib`: sidecars, w2-forensics' MP, exact F, stirlerr_exact.
  - r3-loader-bd0 `interval` and `exp_v6`.
  - r5-f-combine `final_score.value` (hkp-sd, hk-sd), in `s2_variants.py` only.
- **Housekeeping.**
  - `.gitignore` covers `*.pkl` and `__pycache__/`. The directory writes no cache of its own.
  - No .py file is over 260 lines. The directory is about 120 KB.
  - `scripts/check_duplication.py`'s scan with this directory's .py files added gives 3,103 duplicated lines, the
    tracked-only count, so 0 are added.
  - Module names (`edgeprobe`, `edgemodel`, `elib`, `qbits`) avoid r4-f-small-df's `mkprobe`. That name collision
    silently loaded the wrong probe module in a first run.

## 3. The hidden addend: E = fl(Q + P) + R (new structure evidence)
Where r·h < 1E-18 (tiny x on side 1, huge x on side 2), H = -h·L(r) vanishes. Excel's exponent is then the constant
plus the stored P alone.

Method:
- For each cell, take the centre of the interval of exponents E that reproduce Excel through fl(EXP(E)/x)
  (`elib.e_needed`).
- Form D = E - P - K' in units (`qbits.cell`).
- Group the cells by (shape, side), with q = ulp(P) in units (32-1,024).

Facts (`s8_qdata.out`, `s10_phi*.out`):
1. **Within one grid size q, D is constant to about ±1 unit** across x ranges of up to 10^290. Between grid sizes it
   jumps by exact multiples of the smaller grid. For example, (2,1) side 1 gives D = 32.4 (q 64), -33 (q 128, 256),
   +224 (q 512) and -288 (q 1,024). So E = fl(Q + P) + R:
   - an O(1) double Q is added to P and stored at P's grid;
   - an unrounded remainder R comes after.
2. **P = fl(c·LN x) with c = a | -b is right.**
   - With a free phase for Q, 42 of 46 groups are consistent.
   - P = fl(c·LN(fl(m x))) leaves 17 of 46 groups with no consistent phase (`s10_phi_mx.out`).
   - P = fl((c-1)·LN x), with the 1/x inside the exponent (round 3's form), fits 0 of 44 (`s10_phi_x1.out`).
   - This confirms round 4's "1/x applied apart" independently.
3. **E is not a stored double.** The whole exponent rounded at P's grid fits 0 of 46 groups (`s13_storedE.out`).
4. **The split, measured directly on the underflow runs** (`s17_split.out`). Where Y = EXP(E) is subnormal, Excel's
   F·x is k·2^-1074·(1 + R), with k the same integer as M-sd's quantised Y and R constant within ±1 unit from k = 2 to
   k = 10^12. So:
   - EXP's subnormal store happens on fl(Q + P) alone;
   - R is applied after it, as a factor.

   R per shape is +50.1 (3,5), -455.2 (4,5), -114.9 (10,3) and +108.0 (6,1).
   - The test: Q = K_sd - R, with K_sd M-sd's total constant, predicts the normal-Y offsets. ψ_1024(K_sd - R) is
     413.2, -12.8, 183.5 and 97.0 against the measured 413.5, -13.0, 183.7 and 96.6, all within 0.4 units.
   - So Excel's **total** constant is M-sd's K_sd, confirmed to ±0.5 units far outside the region it was fitted in.
     Excel splits it into a stored double Q that is hundreds of units away from K, and a small remainder R.
5. **What Q is not** (consistent groups out of 46; Q = 0 gives 20):
   - `s9_qcands.out`, `s11_qsearch.out`: fl(K'), fl(K_sd), GAMMALN sums in every order, c·LN(m/n), Loader pieces
     (A, ½·LN(w/2π), stirlerr) and every signed sum of two of 28 Excel pieces score 6-21 (3,081 candidates).
   - `s12_orders.out`: every order of the Loader decompositions, with all their terms present and each post-P term
     rounded to P's grid, scores 16-18 (L3-L6, G4, G2, K1).
   - `s14_logform.out`: the log forms a·LN m + b·LN n - h·LN n - lnB score at most 20 in the x form and 5 in the mx
     form.
   - The GAMMALN or Loader constants built from Excel's pieces sit 2-8 units from K_sd, not the -455 to +115 that
     Q - K_sd needs (an inline check, not kept as a script: QG -8.3, -4.0, -4.7, +3.1 and L3 -5.3, -2.0, -4.7, -4.9 units at the four thresh shapes).

   Q needs an error of hundreds of units relative to K. That points at a constant formed by cancelling terms of
   magnitude 2^8-2^9, or a double-double whose high word is not the rounded sum. It is open.
6. The phase table (`s10_phi.out`, `s15_eps.out`) is the identification target. It gives Q mod q per (shape, side)
   on 42 groups: for example (2,1) s1 Q ≡ K' + [288, 320], (3,2) s1 K' + [-351, -287], (6,2) s1 K' + [-468, -213].
   Swap symmetry holds: (1,2) s1 ↔ (2,1) s2 and (1,5) s1 ↔ (5,1) s2 have the same phase.

## 4. Hypotheses and counts (negatives included)
| hypothesis | where | result |
|---|---|---|
| x = 0 rule by trunc(d1): `#NUM!` / 1 / 0 | zero, 44 + 12 | **44/44, 12/12** |
| df truncated, then < 1 `#NUM!`, > 1E10 `#NUM!` | dflow, dfhigh, twins | **all**; twins 352/352, 143/143 bit-exact |
| x < 0 `#NUM!` | neg | 6/6, 6/6 |
| subnormal x `#DIV/0!` | sub | 6/6, 3/3 |
| `#NUM!` iff fl(trunc(d1)·x) overflows | ovf | **35/35, 19/19** (ulp-exact edge) |
| F = fl(Y/x), Y = stored EXP (underflow, kept subnormal Y), F < 2^-1022 → 0 | thresh, tiny, hthr | every 0 vs number decision right on both probes |
| M-sd value chain in these regions | tiny, huge | in sample 222/360 and 271/320 w15; hold-out 23/48 and 28/36 |
| M-sdm last step (fl(EXP·fl(1/x))) | probe a | 762/953/1,032 against M-edge's 737/957/1,035 (`final_insample.out`) |
| hkp-sd, hk-sd (r5-f-combine) by edge region | cells, mya, tdens | below M-sd everywhere within 15 digits (`s2_variants.py`, run inline) |
| closed forms at d1 = 2 or d2 = 2 | — | not re-tested. r4-f-small-df rejected them, and here (2,n) at tiny x follows the general fl(Q + P) chain (D jumps, s8) |
| E stored | clean region | 0/46 |
| P = fl(c LN(m x)); P = fl((c-1) LN x) | clean region | 17/46 infeasible (x form 4/46); 0/44 feasible |
| Q from 3,081 piece sums; 7 decompositions in every order; log forms | clean region | ≤ 21/46 (null 20) |

Excel's d1 = 1 "+3 to +5 unit offset" (round 3) is M-sd's constant (r4-f-small-df). Here it shows up as part of
K_sd, the total that the split check of section 3.4 confirms.

## 5. Open (most promising first)
1. **Identify Q (the stored part of the constant).** The underflow trick measures R = K_sd - Q directly, to ±1 unit,
   for any shape. It needs a run where EXP(fl(Q + P)) is subnormal: side 1, a >= 1.5, ln F + ln x ≈ -720. The binade
   jumps give Q mod q.
   - A probe of 30-50 shapes × 20 subnormal cells would give R on a grid of (a, b), for example the half-integer
     ladder.
   - That table tests double-double and cancelling-sum constructions by shape. A formula for Q fixes the ±512-unit
     error at every large |P| (tiny/huge x, d1 = 1 large x, d2 = 1 tiny x). It probably also fixes the
     stored-sum noise that r4-f-power and r5-f-combine see at moderate x, where the same Q is rounded at a finer grid.
2. Side 2's R, through huge-x runs whose F is subnormal: blocked, because a subnormal F is flushed, and Y = F·x is
   then not subnormal. Use side-1 swaps: the swap symmetry makes (n,m) s1 stand in for (m,n) s2.
3. For engine integration (T-2095 and the rules), take the rules of section 1 directly. Each is bit-exact on both
   probes.

## 6. Quirks for docs/excel-semantics/statistics.md (not edited here: lane rule)
- F.DIST(x,d1,d2,FALSE):
  - df are truncated, and truncated df < 1 or > 1E10 are `#NUM!`;
  - x < 0 is `#NUM!`;
  - x = 0 gives `#NUM!` / 1 / 0 for trunc(d1) = 1 / 2 / >= 3, any d2;
  - a subnormal x (`EXP(-720)`) is `#DIV/0!`;
  - `#NUM!` exactly when fl(trunc(d1)·x) overflows;
  - F is 0 when F·x underflows below 2^-1075 even if F is a normal double (`F.DIST(1E-300,3,5,FALSE)` = 0);
  - a subnormal F·x keeps its lost bits (`F.DIST(2.4266958238320403E-216,3,5,FALSE)` is 10% off);
  - a subnormal F is 0.

  Evidence: probes-numeric t2586-r5-f-edges-a and -hold (excel-graph, 2026-10-24).
- Structure: in the clean region the exponent is fl(Q + fl(c·LN x)) + R, with R applied after EXP's subnormal store.
  The total constant is r4-f-small-df's K_sd to ±0.5 units, with R = +50.1, -455.2, -114.9 and +108.0 units at
  (3,5), (4,5), (10,3) and (6,1). Evidence: t2586-r5-f-edges-a thresh rows, `s17_split.out`.

## 7. Scripts
| script | what |
|---|---|
| `elib.py` | shared: M-sd import, edge regions, mixed scorer (errors, zeros), `e_needed` (Excel's admissible exponent) |
| `edgeprobe.py` | the two probes (`--write`) and `rows()` |
| `edgemodel.py` | **frozen M-edge** `value(x, d1, d2, fin)` |
| `qbits.py` | clean-region cells, D(q), ψ prediction, spread |
| `s0_survey.py`, `s1_misses.py`, `s2_variants.py` | M-sd / round-5 variants by region on existing sets, misses |
| `s3_look.py`, `s4_probe.py`, `s5_thresh.py`, `s6_vsexact.py` | probe views, section scores, underflow ratios, errors against exact |
| `s7_forms.py`, `s8_qdata.py`, `s10_phi.py`, `s15_eps.py` | needed constants per power form, D(q) data, feasible phases, Q - K' |
| `s9_qcands.py`, `s11_qsearch.py`, `s12_orders.py`, `s14_logform.py`, `s13_storedE.py` | Q candidates, brute force (`--plant`), orders of decompositions, log forms, stored E |
| `s16_crrules.py` | CR + rules baseline |
| `s17_split.py` | the R / ψ(Q) split check |
| `final_score.py` | frozen scores (`probe a`, `probe hold`, `set …`) |
