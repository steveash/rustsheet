# T-3568 r5-bup: L140 bup+bgrat with n >= 2 bup terms (BETA.DIST, F, BINOM, NEGBINOM)

Worker r5-bup, 2026-11-01 (T-3568; split from T-3468, T-3383, T-2512). Method: docs/research/excel-kernel-playbook.md.
Base: r3-bgrat's n = 1 composition (RN(W + D) on Excel's density in bup's orientation) and r5-bgrat-fdf1's frozen
kernel (`fmodel.CFG`) for the dispatch and as the M0 baseline. Imported, not copied. Scores are bit-exact / within
4 ulp / within half a unit of the 15th significant digit (w15).

Python: `/home/user/rustsheet/bindings/python/.venv/bin/python` (`make py-bindings`, plus `uv pip install mpmath
numpy scipy` into it), run from this directory.

## 1. Headline

1. **The n = 1 composition holds at n >= 2, on Excel's density at the lowest shape.** At the oriented call
   (a0, b0, x0, y0), n = int(b0), c = b0 - n, the cell is RN(W + D) with W = bup's n-term sum times the prefactor
   x87(P·fl(x0·y0)/c), P = Excel's BETA.DIST(y0, c, a0, FALSE). Excel's W error against the 50-digit value tracks
   P's own error: correlation 0.93 to 1.00 per (a0, n) bin, with P errors up to 300 units of 2^-53 at a0 >= 500, and
   sd(eW - eP) 2.9 to 5.6 units at n <= 9 (`resid2.py` → `resid2_tune.out`, `resid2_hold.out`). On these cells
   Excel is far from the true value because its density is (CR 33/195/389 of the 1,650 hold-out cells).
2. **bup's powers of x go through an exponent: term i is fl(R_i · EXP(fl(i·LN(y0))))**, with R_i the product of
   TOMS's stored ratios fl(apb + l)/fl(ap1 + l) kept in doubles and the sum in doubles. TOMS's recurrence
   d = ratio·x·d in any arithmetic (doubles, r2-bratio's w4, x87, exact) leaves the same sd (17.7 to 17.8 units,
   `sforms_tune.out`); the exponent form takes the n <= 9 leftover from sd 4.7 to 1.6 and the n >= 10 one from 19.8 to
   15.0 (tune; hold 17.1 to 9.6), and BETA.DIST rows from 72/379/567 to 179/574/689 of 768 (`powforms2.py`).
   `model.py` freezes it (commit c8cee195a, before the hold-out was scored):
   ```
   ret  = x87(P · fl(x0·y0) / c)                         P = BETA.DIST(y0, c, a0, FALSE)
   t_i  = fl(R_i · EXP(fl(i · LN(y0)))),  R_i = fl(R_{i-1} · fl(fl(apb + l) / fl(ap1 + l))),  l = i - 1
   S    = 1 + t_1 + ... + t_{n-1}  (doubles; TOMS's k and the eps = 1e-15 break)
   cell = RN(fl(ret · S) + D)                            D = BETA.DIST(x0, a0, c, TRUE)
   ```
   This is a structural choice among ranked forms, not a fit: no constant is chosen by scoring, so it carries no
   `FITTED` tag.
3. **Hold-out and fresh sweep** (`score_hold.py` → `score_hold.out`):

   | probe | cells | FROZEN | REC (n = 1 form, exact S) | M0 (r5 kernel) | CR |
   |---|---|---|---|---|---|
   | t3568-r5-bup-tune (in-sample) | 1,650 | 397/1,190/1,424 | 152/784/1,162 | 115/568/887 | 27/177/381 |
   | t3568-r5-bup-hold (filled with tune, scored after the freeze) | 1,650 | **394/1,163/1,383** | 139/742/1,125 | 102/575/892 | 33/195/389 |
   | t3568-r5-bup-fresh (built and filled after the freeze) | 825 | **186/574/696** | 68/382/583 | 56/286/450 | 16/85/197 |

   Per region on the hold-out (FROZEN): BETA.DIST non-integer b 183/578/685 of 788; F (b0 half-integer, c = 1/2)
   32/104/132 of 154; at c = 1 with D = RN(x0^a0) (below): BINOM 63/166/190 of 200, NEGBINOM 78/165/181 of 200,
   integer-b BETA.DIST 41/111/132 of 150, F 29/107/124 of 146.
4. **The callers use the same composition at their oriented call.** BINOM.DIST(k, N, p, TRUE) is the swapped call
   I_{1-p}(N - k, k + 1) and NEGBINOM.DIST(f, r, p, TRUE) is I_p(r, f + 1). Both have an integer b0, so c = 1. F at
   an odd d2 has c = 1/2, and at an even d2 c = 1.
   - At c = 1 the D column, BETA.DIST(x0, a0, 1, TRUE), is not L140's bgrat increment. x0 > 0.9 swaps it to a0 = 1
     with y0 > 0.9, where Excel returns (0.5 - w) + 0.5 (statistics.md "Incomplete beta at a shape of 1 outside the
     closed forms", T-3007). That return is 0 on 123 hold-out cells at c = 1, all with x0 > 0.9, whose true value
     is up to 5e-17 (`fsplit.py` → `fsplit.out`; tune 116).
   - So D = RN(x0^a0) is the better increment there: hold 215/556/639 against 179/481/566 for the D column on the same
     708 c = 1 cells.
   - For F the companions BETA.DIST(x0, ...) and BETA.DIST(y0, ...) re-derive their other argument as 1 - (their x).
     Where F's y0 is not fl(1 - x0) (261 of 300 hold rows), P is Excel's density at a neighbouring point, and the F
     cells score 44/157/200 of 261. The 39 rows with y0 = fl(1 - x0) score 7/26/29 (`fsplit.out`). An engine port
     forms P at F's own (x0, y0) and does not have this problem.
5. **What is left: a noise that grows with the exponent's binade.** After the frozen form, the n >= 10 BETA.DIST
   residual has median |e| 2.5 units and sd 15 (tune). Grouped by the binade of the largest exponent argument
   (n - 1)·|ln y0|, the median |e| is 0.5 to 1.1 below 16, 1.9 in [16, 32), 5.3 in [32, 64) and 8.5 in [64, 128).
   So one more rounding of an exponent-sized quantity is still unmodelled (section 3). It reaches the 15th digit on
   about 15% of n >= 10 BETA.DIST cells.

## 2. bup's arithmetic in the min <= 1 bup+bgrat branch (T at n <= 30): not this form

`tbup.py` → `tbup.out`, r2-tdist's probe p1 part A, with Excel's density and the solved bgrat_half. This is a check
on cells the model was not chosen on. On 1,344 T cells the exponent form scores 411/1,342/1,344 and TOMS's recurrence
in doubles 412/1,339/1,343. The exact sum on Excel's density stays best at 598/1,344/1,344, as r2-tdist found.
That branch has b = 1/2 <= 1, so TOMS's k = 0 and the eps break applies from the first term, and bup's x there is the
T y, close to 1. The n >= 2 L140 form does not carry over, and the branch's bit-level arithmetic stays open. It is
within 15 digits on 1,344 of 1,344 cells already.

## 3. Negatives (decision 0056)

| tried | on | score | script |
|---|---|---|---|
| bup's sum in doubles, TOMS order / grouped d·(ratio·x) / w4 / x87 / x87 stored per step | tune, BETA.DIST bgrat | sd 17.7-17.8 units, all alike; cells 144-170 bit-exact of 784 | `score.py`, `sforms.py` |
| apb = c + a0 exact instead of stored | same | sd 19.7 (stored 17.8) | `sforms.py` |
| no eps break | tune | identical to the break | `score.py` |
| W as a loop of one-term bups, each on Excel's density at its own shape c + j (BETA.DIST(y0, c+j, a0, FALSE)) | terms-tune 300 points | 5/30/80 (REC 27/152/221); W's error is 270-290 units from the upper densities' | `terms.py` → `terms_tune.out`, `terms_hold.out` |
| regression of the leftover on the stored-input rounding errors (apb, ap1, a unit x error) and n | tune | sd 19.89 → 19.83 at n >= 10 | `sreg.py` → `sreg_tune.out` |
| the leftover as the exponent rounding of the density (by \|ln P\| or \|ln W\|) | tune | n <= 9 sd 4.6 even at \|ln P\| >= 256 | `expbins.py` → `expbins_tune.out` |
| x^i with the product i·LN(y0) kept extended, EXPX(fl64(i·LN y0)) | tune | n <= 9 57/154/167 (double product 88/170/171) | `powforms2.py` |
| one exponent per term, EXP(fl(i·LN y0) + LN R_i) (double / extended) | tune | 16/107/141 / 16/117/148 at n <= 9 | inline, recorded here |
| R_i through EXP of a running LN sum; EXP(lnR)·EXP(i ln y) | tune | 12/85/124, 17/86/126 at n <= 9 | inline |
| x^i as y0^(c+i)/y0^c or y0^(c+i-1)/y0^(c-1) through EXP | tune | 27/146/161, 37/145/164 at n <= 9 | inline |
| the exponent form in the T bup+bgrat branch | 1,344 T cells | 411 (rec 412, exact 598) | `tbup.py` |

## 4. Probes (probes-numeric)

| probe | cells | what | script |
|---|---|---|---|
| t3568-r5-bup-tune | 4,950 | 1,650 points (seed 35681): A the caller's cell, B = D, C = P; regions beta 800, bint 150, f 300, binom 200, negb 200 | `mkprobe.py` |
| t3568-r5-bup-hold | 4,950 | the same at seed 35682; built and filled with tune, scored only after `model.py` was committed | `mkprobe.py` |
| t3568-r5-bup-fresh | 2,475 | half size at seed 35689, built and filled after the freeze | `mkprobe.py --fresh` |
| t3568-r5-bup-terms-tune, -terms-hold | 5,898, 5,967 | BETA.DIST(y0, c + j, a0, FALSE) for every j < n at 300 BETA.DIST points of each half (the loop hypothesis) | `mkterms.py` |

Every point takes L140 bup+bgrat with n >= 2 in `fmodel.CFG`. The meta files record the oriented call and the D
point's own branch. `sha256.txt` holds the sidecars' hashes as filled. mkterms.py read the hold probe's P column
only to drop P = 0 rows before the hold-out was scored.

## 5. Not attempted (handed on)

- The a0 < 1e-15 band at 0.04 <= y < 0.25 (Excel 1e-14 to 3.8e-13 relative off the true value, bgrat's
  incomplete-gamma ratio): not in this round.
- The exponent-binade leftover at n >= 10 (finding 5). Next: a run probe along consecutive doubles of y0 at a fixed
  (a0, b0) with n near 30, to see which stored quantity makes the sawtooth (playbook section 2). Excel's own POWER
  at (y0, i) is a candidate for x^i as well.
- The pins are excel-graph values. Residuals of a few ulps are inside the documented web/desktop floor
  (docs/research/oracle-calibration.md).
