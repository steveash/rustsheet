# T-2464 r2-bpser-small: bpser / apser / fpser for min(a, b) <= 1, out of sample

Worker r2-bpser-small, round 2, 2026-10-23. Method: port TOMS 708 bpser with all its prefactor branches, plus apser and
fpser, with Excel's EXP/LN/GAMMALN and x87 stored arithmetic. Compare it with w3's form I = fl(EXP(z) * s),
z = fl(a LN x) - L, on a new probe of 2,000 distinct shapes. Then find the L (ln B) that Excel uses.

- Scores are bit-exact / within 4 ulp / within 15 significant digits (w15).
- "Hold-out" is the md5 half of the SHAPE (a, b) (`lib.shape_hold`), not of the cell, so a fit cannot hide in a few
  shapes. Old sets use the same shape-level split.
- Run with `/home/user/rustsheet/bindings/python/.venv/bin/python` from this directory.
- All scripts import r2-bratio's `bratio_excel` (the TOMS port: Excel EXP/LN/GAMMALN, dispatch, bup, bgrat) and
  `score_lib` (data and scorer). Nothing is copied from them.
- The bpser sub-branch is TOMS's, set by bpser's own (a0, b0) = (min, max): `b0<=1`, `1<b0<8`, `b0>=8`.

## 1. Headline

1. **TOMS's own bpser prefactors are rejected out of sample, in every sub-branch.** This covers the b0 <= 1
   gam1 product, the 1 < b0 < 8 m-loop with gamln1, and b0 >= 8 algdiv, all ported x87 stored with Excel EXP/LN.
   - On the 2,000-shape probe the TOMS port is bit-exact on 1,901 / 4,602 (1 < b0 < 8), 879 / 3,083 (b0 <= 1) and
     949 / 2,868 (b0 >= 8). w3's form gets 2,910 / 1,645 / 1,415.
   - TOMS's own (1 + a s) series is worse still: 1,445 / 879 (`score_new.out`).
   - Product forms I = fl(fl(X K) s), with X = EXP(fl(a LN x)) and any shape constant K, are below the EXP(z) form on
     all 14 runs (`kscan_runs.out`).
   - So Excel's bpser is not TOMS's code. It is one exponent form for every sub-branch.
2. **The exponent form is exactly z = fl64(alx - fl(g(a0) + fl(G(b0) - GL(a0 + b0)))), where alx = fl(a LN x).**
   - ln B is assembled in doubles, smaller shape first, in TOMS betaln's grouping. Only the final subtraction from
     alx stays in an x87 register (w3's "z64").
   - Then I = fl(EXP(z) * s), with s w3's forward series from fl(1/a).
   - `zforms.py` tried all 120 expression trees and rounding patterns over the pieces (alx, g, G, GL), times 3 or 9
     choices of the lnΓ(v < 1) pieces. This tree wins in all three sub-branches, on tune and hold-out alike
     (`zforms_mid.out`, `zforms_small.out`, `zforms_big.out`).
   - Positive control (`zforms_ctl_mid.out`): planted data are recovered at 6,999 / 6,999 with that tree, and the
     nearest other tree scores 6,928.
3. **Best per sub-branch (new probe, shapes only, never used for tuning).** GL53 means g = G = Excel's GAMMALN. For
   b0 >= 8, "glalgdiv" means L = fl(GL(a0) + algdiv(a0, b0)).

   | sub-branch (n) | r2-bratio BEST (w3 z64) | **GL53[/glalgdiv]** | CR | hold-out exact: BEST / GL53 / CR |
   |---|---|---|---|---|
   | w_bpser b0<=1 (3,083) | 1645/3079/3082 | **2311/3082/3083** | 918/2959/3052 | 882 / **1229** / 509 of 1,670 |
   | w_bpser 1<b0<8 (4,602) | 2910/4602/4602 | **3369/4602/4602** | 1219/3646/4152 | 1427 / **1628** / 605 of 2,250 |
   | w_bpser b0>=8 (2,868) | 1415/2853/2860 | **1489**/2840/**2861** | 781/1751/2074 | 694 / **741** / 399 of 1,375 |
   | w1_bpser 1<b0<8 (822) | 627/822/822 | **715/822/822** | 285/736/794 | 300 / **338** / 133 of 390 |
   | w1_bpser b0>=8 (576) | 400/576/575 | **404**/576/575 | 250/508/547 | 193 / **195** / 121 of 269 |
   | w1_bpser b0<=1 (49) | 22/49/49 | **31/49/49** | 15/44/48 | 10 / **18** / 5 of 22 |
   | fpser cells (184) | 6/32/95 | **132/184/184** | 4/34/81 | 2 / **58** / 2 of 72 |
   | apser cells (192) | 1/22/67 | **11/68/124** | 0/16/65 | 0 / **5** / 0 of 120 |

   Visible parity (w15) holds in every bpser sub-branch on the new shapes: 3,083 / 3,083, 4,602 / 4,602 and
   2,861 / 2,868. Bit-exactness is still only 50-73%.
4. **Excel has no fpser.** For b0 < 1e-15·min(1, a0), TOMS fpser gives 6 / 32 / 95 of 184. Plain bpser gives
   132 / 184 / 184 (`tiny_alt.out`, `score_final_new.out`).
   - The shape-constant relative offsets of about 3e-15 against CR come from ln B, where GAMMALN(tiny) is about 40.
     They are not a literal misparse: probe tiny2 shows all 48 tiny literals parse exactly (`tiny2.out`).
5. **Excel has no apser, or a narrower one.** For a0 < 1e-15 (swapped cells, x > 0.5), bratio's fall-through
   dispatch beats TOMS apser:
   - x0 >= 0.3 goes to w1 = bpser(b0, a0, y0): 14 / 15 at w15, against 6 / 15.
   - The other cells go to bup + bgrat: 84 / 113 at w15 for y < 0.04, against 38 / 113 (`tiny_noapser.out`).
   - In 0.04 <= y < 0.25 Excel is 1e-14 to 3.8e-13 relative above the truth, and every model and CR miss it.
     Those cells have bgrat's z = -nu ln x >= 1.1, where grat1 uses its continued fraction. So this is the bgrat /
     incomplete-gamma problem r2-bratio found, seen here with b tiny.

## 2. The model (min(a, b) <= 1, every bpser call)

```
a0 = min(a,b), b0 = max(a,b) of bpser's own arguments (after bratio's swap)
s   = fl(1/a); c = 1; n = 1, 2, ...: c = fl(c * fl(fl(fl(0.5 - fl(b/n)) + 0.5) * x)); s = fl(s + fl(c / fl(a + n)))
      until s stops changing                                    (w3; order confirmed, factor form invisible)
alx = fl(a * LN(x))                                             (stored; extended product or extended LN: rejected)
L   = fl( GL(a0) + fl( GL(b0) - GL(fl(a0 + b0)) ) )    b0 < 8   (Excel GAMMALN, all stored; T-0969's B(a,b) grouping)
L   = fl( GL(a0) + algdiv(a0, b0) )                    b0 >= 8  (best tried, NOT solved: see section 4)
z   = fl64(alx - L)                                             (extended, EXP of the 64-bit argument)
I   = fl(EXP(z) * s)                                            (EXP's result stored; extended E rejected)
no fpser: b0 tiny goes to this bpser; no apser: a0 tiny falls through bratio (x0 >= 0.3 -> w1 = bpser(b0,a0,y0),
else bup(b0,a0,y0,x0,20) + bgrat)
```

`lib.V(nofpser=True, noapser=True, lg="GL53/glalgdiv")` is this model, and `bscore.py` scores it.

## 3. Evidence, in order

1. **TOMS port** (`lib.Model.prefactor_toms`, `series_toms`; variants in `bscore.VARIANTS`; `score_new.out`,
   `score_nofa_old.out`). x87-stored TOMS bpser is bit-exact as follows (shapes and runs probe, w_bpser):
   - 1 < b0 < 8: 1,901 / 8,186, against 5,788 for w3.
   - b0 <= 1: 1,490 / 5,139, against 2,482.
   - b0 >= 8: 1,137 / 4,404, against 1,501.

   Hybrids on the same cells:
   - TOMS prefactor with w3's series: 2,752 / 1,651 / 1,081.
   - With the x^a and EXP arguments extended: 2,563 / 1,578 / 1,866.

   On b0 >= 8 runs the TOMS-prefactor hybrid wins (618 of 1,536, against 86). That points at the right ln B there,
   not at a TOMS product (section 4).
2. **Probe and the GAMMALN side channel.** The engine's GAMMALN model is exact on only 274 of 771 arguments in [1, 2)
   and 1,738 of 2,585 below 1 (`check_gl.out`). The measured values (probe `-gl`) are therefore used for every shape
   argument. That is why GL-form ln B now wins where r2-bratio (engine model) found it worse.
3. **Constant-L ceilings** (`lscan.py`, `lcover.py` → `lcover_runs.out`). The method inverts each cell's I = fl(E*s)
   into an interval of L and finds the L that most cells of a run admit.
   - With w3's series, five of the 1 < b0 < 8 runs reach 100% at one L: (0.717, 6.31) 512, (0.91, 2.17) 512,
     (1.37, 0.73) 512, (0.53, 1.13) 486 / 486 and (0.43, 3.7) 475 / 512.
   - The b0 <= 1 runs and the a > 1, b < 1 runs top out at 55-84%: (0.37, 0.81) 425, (0.83, 0.29) 348,
     (0.153, 0.571) 361, (0.0731, 0.913) 280, (2.9, 0.61) 389, (5.13, 0.217) 412 / 462.
   - So for those shapes the single-L picture leaves 16-45% unexplained, whatever L is.
4. **Series order** (`lcover_series.out`, `lib.series`). Forward from fl(1/a) is confirmed against forward-from-0 and
   Cephes's tail-first order. Those drop the ceilings to 148-385 of 512, so this is a strong positive.
   - The factor forms (TOMS (0.5 - b/n) + 0.5, 1 - b/n, (n - b)/n, (n - b)x/n, c*x first) are invisible (+-3 cells).
5. **Rejected compositions** (`runmiss_negatives.out`):
   - EXP's result kept in extended for the product: (0.717, 6.31) drops from 512 to 367.
   - alx as an extended product: 171.
   - LN and the product both extended: 168.
6. **Product forms rejected** (`kscan_runs.out`): best K per run, for both X forms and both product orders, is below
   the EXP(z) ceilings on all 14 direct runs.
7. **Tree search** (`zforms.py`; tune = shape half, see section 1, item 2). The winning tree is
   (alx (-g (-G +GL))) with roundings (64, 53, 53) in all three sub-branches.

   | sub-branch | g / G forms tried | best | tune / hold | next best |
   |---|---|---|---|---|
   | 1<b0<8 (6,999 cells, 910 shapes) | gl, g1ln53, g1ln64 | g = gl | 3424 / 1921 | g1ln53: 3380 / 1825 (runs prefer g1ln53: 3185 vs 3120) |
   | b0<=1 (4,260, 526 shapes) | 9 combinations | g = G = gl | 1005 / 1914 | g = gl, G = g1ln53: 901 / 1806 |
   | b0>=8 (3,617, 568 shapes) | gl, g1ln53, g1ln64, with algdiv | GL + algdiv | 880 / 344 | g1ln53 + algdiv: 780 / 314 |

8. **lnΓ(a0 < 1) is still shape-dependent** (`gbins_mid.out`). Neither GL(a0) nor TOMS's fl(gamln1(a0) - LN(a0)) is
   right everywhere.
   - On cells where the two differ, GL wins for a0 < 0.3 (44 vs 5, 57 vs 13) and loses or ties above 0.7.
   - The old w2f (0.5, 6.25) shapes prefer g1ln53: L53 gets 2,902 / 3,262, GL53 gets 1,733.
   - Running gamln1 in x87-stored arithmetic changes nothing: 0 of 2,000 values differ (`lib.X87`).

## 4. Not settled

- **b0 >= 8 ln B.** GL(a0) + TOMS algdiv (stored) is the best form tried. It only reaches 506 / 1,536 on the runs
  and 1,489 / 2,868 on shapes. On the old ix / be b0 >= 8 cells it is worse than the accurate proxy: 240 against
  401 of 604, and 248 against 301 of 386 in w1_bpser.
  - Implied L on the runs (`lcover_runs.out`): (0.29, 13.7) at -1.88 units from the accurate value (GL + algdiv
    gives -0.94); (17.3, 0.47) at +1.65 (TOMS gamln + algdiv extended gives +1.73); (11.9, 0.83) at +6.05 (+5.95).
  - So Excel's algdiv differs from TOMS's by about 1 unit. Its Stirling-difference pieces are the next target.
- **b < 1 series ceiling.** Each shape tops out at 55-84%, and the misses come in sawtooth blocks (`misspos_*.out`).
  `fixwhich.out` shows each miss is fixed by s or E moved one ulp. The runs cannot tell those two apart.
  - The pattern is that of one stored intermediate whose rounding beats with x, not of L.
  - Candidates are the first-term factor (1 - b) x for b < 1, and EXP's own near-ties at E binades -3 to -8.
- **lnΓ(a0 < 1):** see section 3, item 8. A probe of bpser at fixed b0 with a0 stepping by single ulps around 0.3,
  0.5 and 0.7 would separate the forms.
- **apser band:** see section 1, item 5 (bgrat / grat1).
- **Old-set contradictions.** On the old 1 < b0 < 8 cells GL53 loses to r2-bratio BEST: 4,142 against 4,824 of
  8,584. Most of the loss is in w2f (0.5, 6.25), 1,733 against 2,239, and w4p1 (7.3, 0.6), 1,568 against 1,736.
  On the old b0 <= 1 cells GL53 wins: 1,749 against 1,457.

## 5. Probes added (all filled from excel-graph on 2026-10-23 with `rustsheet-oracle recalc`, outputs here)

| case | cells | what |
|---|---|---|
| `t2464-r2-bpser-small-shapes` | 12,000 | 2,000 shapes, a or b in [0.05, 1) on irregular golden steps, the other log-uniform in [0.3, 30]; 6 short-decimal x per shape where BEST fires a bpser call |
| `t2464-r2-bpser-small-runs` | 8,192 | 16 runs of 512 consecutive doubles, at least two per sub-branch (`make_probe.RUNS`) |
| `t2464-r2-bpser-small-tiny` | 384 | fpser (b tiny, x <= 0.5) and apser (b tiny, x > 0.5) shapes, 3-digit tiny literals |
| `t2464-r2-bpser-small-tiny2` | 432 | the tiny rows with exact-piece arguments, plus the parse error of every tiny literal (all 0) |
| `t2464-r2-bpser-small-gl` | 5,843 | =GAMMALN at every shape's a, b and a+b |

Built by `make_probe.py --write` and `make_probe_tiny2.py --write`. Metadata is in `*.meta.json` and Excel's values in
`*.out.json`. No sidecars were written to eval/.

## 6. Scripts

| script | output | what |
|---|---|---|
| `lib.py` | | Model (bx.Kernel + bpser variants `V`, tracing, no-fpser / no-apser), series variants, X87 float, probe loader, measured GL |
| `make_probe.py`, `make_probe_tiny2.py` | | probes |
| `bscore.py` | `score_*.out` | variant table per bratio branch × sub-branch × set, shape hold-out |
| `check_gl.py` | `check_gl.out` | engine GAMMALN vs Excel at the shape arguments |
| `peek_tiny.py`, `tiny2.py`, `tiny_alt.py`, `tiny_apser.py`, `tiny_noapser.py` | `tiny*.out` | fpser / apser |
| `lscan.py`, `lcover.py` | `lscan_shapes.out`, `lcover_runs.out`, `lcover_series.out` | implied L per shape, ceilings, series orders |
| `kscan.py` | `kscan_runs.out` | product-form prefactors |
| `zforms.py` | `zforms_*.out` | exhaustive exponent trees, positive control `plant=` |
| `gbins.py` | `gbins_mid.out` | lnΓ(a0) form per a0 bin |
| `runmiss.py`, `misspos.py`, `fixwhich.py` | `runmiss_*.out`, `misspos_*.out`, `fixwhich.out` | per-run misses |

Caches (`*.pkl`) are git-ignored.
