# T-3570 r5-bpser-lg: lnΓ(a0 < 1) in bpser's L, and the per-shape ceiling

Round 5, 2026-11-01. This directory takes the two open bpser items of T-3468 (split from T-3383 and T-2512):
which lnΓ(a0) Excel's bpser puts in L when a0 < 1, and the 55-84% per-shape ceiling of r2-bpser-small. The method is
in docs/research/excel-kernel-playbook.md.

- Model under test: r2-bpser-small section 2. In the 1 < b0 < 8 sub-branch it is
  I = fl(EXP(fl64(alx - L)) * s), with alx = fl(a0 LN x), s = w3's forward series, and
  L = fl(g + fl(GL(b0) - GL(fl(a0 + b0)))). Here g is the lnΓ(a0) piece under study.
- Every scored cell is an unswapped w_bpser call, bpser(a0, b0, x), with a0 < 1 < b0 < 8.
- GL(b0) and GL(a0 + b0) are Excel's own GAMMALN values, read from the probe's own columns, so g is the only free
  piece in L.
- Scores count bit-exact cells, or a0 points whose best g set contains a candidate.
- The hold-out is the md5-odd half of a0 (`pin.hold`). No numeric fit was made: every candidate is a structural form
  with no tuned constant. Both halves are reported, and they agree.
- Run every script with `/home/user/rustsheet/bindings/python/.venv/bin/python` from this directory (`make py-bindings`
  builds it; add `mpmath` and `numpy`). They import r2-bpser-small's `lib` (and through it r2-bratio's `bratio_excel`).
  Nothing is copied from those modules.

## 1. Headline

1. **No single lnΓ(a0) form is bpser's g at every a0. This is a negative, with counts.**
   - Candidates: Excel's worksheet GAMMALN(a0) ("gl"), TOMS's fl(gamln1(a0) - LN(a0)) ("g1ln53"), and five others.
   - gl is right at a0 near 0.15 and 0.3: it is in the best g set on 143 of 144 a0, and g1ln53 never beats it there.
   - gl also leads from 0.7 up, 24 to 4 at 0.7 and 6 to 3 at 0.85.
   - Just above 0.5, gl is wrong and g1ln53 is right more often: 19 a0 against 7 (`side.out`).
   - Between 0.4 and 0.6 neither form wins.
   - Over all a0 where the two give different L, gl wins on 52 and g1ln53 on 36 (tune). On the hold-out half it is
     51 to 19 (`lgwin.out`).
   - So r2's "GL53" stays the best single form, and the shape dependence r2-bpser-small reported is confirmed at
     single-ulp resolution. The cause is still unknown.
2. **Confirmed (already in statistics.md, T-1941): below 0.7, worksheet GAMMALN(v) is exactly
   fl(GAMMALN(fl(1 + v)) - LN(v)).**
   - It held on 360 of 360 a0 in 0.15-0.6.
   - Above the 0.7 cut it holds only by chance: 45 of 72 at the 0.7 center, 12 of 72 at 0.85 (`lgbest.out`).
   - So "gl" in bpser means Excel's [1, 2) kernel evaluated at fl(1 + a0), then LN(a0) subtracted.
3. **The ceiling is not specific to b < 1.** Here b0 > 1, and the best g per a0 still makes only 3,355 of 4,032
   cells exact (83%). Only 171 of 504 a0 are fully explained (`ceil.out`).
   - The ceiling falls as a0 falls: 66% of cells at a0 = 0.15, 82-89% from 0.3 up.
   - The misses are ±1-2 ulp. They move with x at a fixed a0, and they are not near-ties of the final product
     (`grid.py`; the rounding phase is spread over ±0.5).
   - So the missing piece varies with x and a: it sits in alx, s or EXP, not in L.
4. **No series or composition variant tried raises that ceiling. All are negatives** (tune half,
   a0 = 0.15 and 0.3, 432 cells, ceiling over g):

   | variant | cells at the ceiling |
   |---|---|
   | r2's model (c, t, s stored) | 299 |
   | plain SSE doubles | 304 |
   | s in an x87 register | 244 |
   | c in a register | 299 |
   | all in registers | 227 |
   | t unstored | 298 |
   | s unstored into the product | 220 |
   | EXP(z) with z stored | 230 |
   | LN(s) folded into the EXP argument (3 forms) | 209-226 |
   | fl(fl(E/a) fl(a s)) | 258 |
   | fl(fl(E fl(a s))/a) | 241 |
   | x^a times a free per-a0 constant K (3 x^a forms × 3 product orders) | 230-264 |

   Sources: `sforms_tune_lo.out`, `zforms2_tune_lo.out`, `kforms_tune_lo.out`. SSE's +5 is within noise: it gains
   at 0.15 and ties at 0.3.

## 2. Evidence, in order

1. **Probe design** (`mkprobe.py`). For each b0 in (1.61, 3.37, 5.83) and each center a0 in (0.15, 0.3, 0.4, 0.5,
   0.6, 0.7, 0.85), a0 steps through 24 consecutive doubles (center - 12 ulps to center + 11 ulps).
   - Each a0 gets 8 short-decimal x, the largest from a fixed list where the model dispatches to bpser.
   - Each a0 also gets the columns GAMMALN(a0) and GAMMALN(fl(a0 + b0)), and each b0 group gets GAMMALN(b0) once.
   - That is 504 a0 points and 5,043 cells.
2. **Positive control** (`pin.py --control`, `pin_control.out`). Data planted from the model, with g alternating
   between gl and g1ln53 by row, is recovered on 504 of 504 a0: the planted g is always in the pinned set. On the real
   data only 170 of 504 a0 admit any g that makes all 8 cells exact (`pin.out`), so the analysis uses each a0's
   ceiling (`ceil.py`, `lgbest.py`) instead of requiring all 8.
3. **Per-a0 winner** (`lgwin.out`; on the a0 where gl and g1ln53 give different L, the form with more exact cells
   wins):

   | center | tune: differ / gl wins / g1ln53 wins | hold: differ / gl wins / g1ln53 wins |
   |---|---|---|
   | 0.15 | 3 / 3 / 0 | 0 / 0 / 0 |
   | 0.3 | 3 / 3 / 0 | 15 / 15 / 0 |
   | 0.4 | 15 / 7 / 8 | 15 / 8 / 4 |
   | 0.5 | 18 / 5 / 12 | 15 / 5 / 10 |
   | 0.6 | 36 / 20 / 10 | 11 / 7 / 4 |
   | 0.7 | 15 / 11 / 3 | 17 / 13 / 1 |
   | 0.85 | 7 / 3 / 3 | 4 / 3 / 0 |

4. **Side of 0.5** (`side.out`). GAMMALN switches kernels at x = 1.5, so a0 = 0.5 is a boundary for gl.
   - Below 0.5 (K3 side) gl and g1ln53 mostly agree: they differ on only 3 of 36 a0.
   - Above 0.5 they differ on 27 of 33 a0. There g1ln53 alone is in the best set on 19 and gl alone on 7.
   - At 0.4, below the center, the split is 11 to 8. At 0.6, below the center, it is 12 to 10.
5. **Other lnΓ(a0) candidates rejected** (`lgbest.out`; "+" counts a0 where the candidate is in the best set and gl
   is not, "−" the reverse; tune / hold):

   | candidate | + / − (tune) | + / − (hold) |
   |---|---|---|
   | correctly rounded lnΓ(a0) | 13 / 43 | 7 / 36 |
   | fl(CR lnΓ(1 + a0) - LN a0) | 11 / 33 | 7 / 28 |
   | fl53(fl64(gamln1 - LN)) | same as g1ln53 | same as g1ln53 |
   | Excel's LN(GAMMA(a0)) | 25 / 108 | 9 / 99 |
   | fl(LN(GAMMA(fl(1 + a0))) - LN a0) | 24 / 91 | 7 / 80 |
   | LN(fl(GAMMA(fl(1 + a0)) / a0)) | 25 / 96 | 12 / 108 |

   - The GAMMA and LN(GAMMA) values are Excel's own, from probe `-aux`.
   - **Raw-t kernel** (`rawt.out`). Excel's [1, 2) kernel with its multiplier t = a0 itself instead of
     fl(fl(1 + a0) - 1) loses. R is recovered from Excel's GAMMALN(fl(1 + a0)) by inverting the final product. Where
     raw-t and gl differ, raw-t is in the best set on 11 a0 and gl on 34.
   - **K3 kept past x = 1.5** (`k3ext.out`). Engine port; the control is K3 == gl on 228 of 258 a0 ≤ 0.5. It loses:
     at 0.6, 25 against gl's 36; at 0.7, 0 against 25.

## 3. Not settled (for a later round)

- **lnΓ(a0) between 0.4 and 0.7**, mainly just above 0.5. The next probe should step a0 through (0.5, 0.56], where
  g1ln53 wins most clearly. Hold b0 at one value and use 16 x per a0, so that each a0's g is pinned despite the
  ceiling.
- **The ceiling** (headline, item 3). It is now known to hold for a0 < 1 < b0, so the cause is not bpser's b < 1
  first factor (1 - b) x, which r2-bpser-small named as a candidate. It worsens as a0 falls.
  - The remaining candidates are Excel's LN at these x and the EXP argument's extended bits.
  - LN: compare Excel's LN(x) column with the engine's FYL2X model at exactly these 15 x.
  - EXP: is z formed with alx unstored? r2 rejected that on one run only, at 171 of 512.
  - A probe with LN(x), EXP(alx - L) side columns would separate them.

## 4. Probes added (filled from excel-graph on 2026-11-01 with `rustsheet-oracle fill`)

| case | cells | what | script |
|---|---|---|---|
| `t3570-r5-bpser-lg` | 5,043 | 504 a0 (7 centers × 3 b0 × 24 consecutive doubles) × 8 x of BETA.DIST cdf, plus GAMMALN(a0), GAMMALN(fl(a0 + b0)) and GAMMALN(b0) | `mkprobe.py` |
| `t3570-r5-bpser-lg-aux` | 2,520 | at the same a0: GAMMALN(fl(1 + a0)), GAMMA(a0), LN(GAMMA(a0)), GAMMA(fl(1 + a0)), LN(GAMMA(fl(1 + a0))) | `mkaux.py` (built after `lgwin.py`; it measures building blocks, nothing is fitted to it) |

`t3570-r5-bpser-lg.meta.json` holds [row, a0, b0, [x...]] per a0.
