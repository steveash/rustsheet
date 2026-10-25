# T-2513 r3-m1-residual: closing the misses of Excel's beta density for min(a,b) <= 1

Worker r3-m1-residual, round 3, 2026-10-24. Counts are bit-exact / within 4 ulp / within 15 significant digits
(`m1lib.score`), per subset, in-sample and hold-out separately. Base = w1's M1 as integrated in r2-bratio
(`bratio_excel.Kernel.density`) with r2-log1p's Excel log1p patched in for x <= 0.375 (`m1lib.base_density`).

## 1. Result: model M2

```
tx = fl(fl(a-1) * LN(x));  ty = fl(fl(b-1) * ly)       ly = Excel log1p(-x) for x <= 0.375, else LN(fl(1-x))  (as M1)
lnB:  min(a,b) = 1           -> -LN(max)                                (new: also 1 <= max <= 2 and max > 8)
      min < 1, max <= 8      -> fl(GL(min) + fl(GL(max) - GL(a+b)))   GL = Excel's GAMMALN at the argument (as M1)
      min < 1, max > 8       -> fl(GL(min) + algdiv(min, max))         algdiv: OPEN (TOMS in doubles is the frozen choice)
hi = fl(fl(tx + ty) - lnB)                 every op a double op
lo = (tx + ty - lnB) - hi                  exactly (the two additions' rounding errors)
density = fl( EXP(hi) * fl(1 + lo) )       EXP = Excel's EXP (model V6) of the double hi; fl(1 + lo) equals EXP(fl(lo))
```
This holds for every b, b = 1 included: M1's two exponent rules (an extended E = tx + (ty - lnB) fed to EXP, and a
stored E at b = 1) are both replaced by the one split above. `final_score.py` is the frozen model.

Read as code: the exponent is summed in doubles, the rounding error of the sum is carried as a double-double tail, and
the result is EXP(sum) times EXP(tail) (or times 1 + tail), two doubles multiplied and rounded. Whether Excel's source
writes EXP(hi)·EXP(lo) or EXP(hi)·(1 + lo) cannot be told apart here (the two give the same double on every row).

### Scores (`final_insample.out`, `final_hold.out`, `final_log1p.out`)

| set | n | M1 (base) | **M2, algdiv TOMS-d** | M2, algdiv accurate |
|---|---|---|---|---|
| old pdf cells, min <= 1 (of the 2,035; the other 928 have min > 1, not this worker's) | 1,107 | 566 / 1,095 / 1,100 | **815 / 1,101 / 1,103** | 859 / 1,101 / 1,103 |
| old: a < 1, max <= 8 | 400 | 274 | **400** | 400 |
| old: b < 1, max <= 8 | 64 | 37 | **64** | 64 |
| old: a = 1 / b = 1 (all max) | 15 + 9 + 22 + 9 | 11 + 6 + 17 + 3 | **15 + 9 + 22 + 9** | same |
| old: min < 1, max > 8 (algdiv) | 562 + 26 | 209 + 9 | 281 + 15 | 323 + 17 |
| w1 probe1 (modelled shapes) | 12,800 | 10,375 / 12,799 / 12,799 | **12,796 / 12,799 / 12,799** | 11,879 / 12,799 / 12,799 |
| probe1 b = 1 ((0.5,1), (1.5,1)) | 3,200 | 3,184 | **3,200** | 3,200 |
| probe1 a = 1 ((1,7), (1,20)) | 3,200 | 2,564 | **3,199** | 3,199 |
| probe1 a < 1, max <= 8 ((0.5,7), (0.25,3.3)) | 3,200 | 2,254 | **3,198** | 3,198 |
| probe1 b < 1 ((3,0.5)) | 1,600 | 1,287 | **1,600** | 1,600 |
| probe1 (0.5,20) (algdiv) | 1,600 | 1,086 | **1,599** | 682 |
| r2-log1p BETA.DIST(p,1,2^k+1) rows | 15,596 | 11,713 / 15,569 / 15,569 | **15,566 / 15,569 / 15,569** | same |
| probe a (in-sample, this worker) | 4,096 | 2,427 / 3,937 / 3,964 | **3,173 / 3,917 / 3,964** | 3,302 / 3,956 / 3,970 |
| **hold (untouched until frozen)** | 4,096 | 2,318 / 3,981 / 4,060 | **3,106 / 3,991 / 4,062** | 3,469 / 4,051 / 4,079 |

Per block (in-sample a / hold-out, M2 TOMS-d; M1 in brackets):
- `gl` (a < 1, 1 < b < 8; 24 shapes each, different a and b): **960 / 960** and **960 / 960** (703, 704).
- `w4` ((0.6, 7.3), (0.5, 6.25) and neighbours): **160 / 160** and **159 / 160** (121, 110).
- `a1big` (a = 1, b from 8.25 to 1.1e9): **959 / 960** and **957 / 960** (770, 704).
- `blt1` (b < 1, a from 1.1 to 900): 346 / 480 and 256 / 480 (248, 180); the misses are the algdiv shapes (a > 8).
- `albig` (a < 1, b > 8): 364 / 960 and 371 / 960 (276, 287). This is the open algdiv.
- `runs` (consecutive-double runs): 384 / 576 and 403 / 576 (309, 333); the misses are the algdiv shapes in them.

**Ceiling: with a free lnB per shape, M2's exponent path is 4,094 / 4,096 (in-sample, 88 of 90 shapes at 100%) and
4,020 / 4,096 on the hold-out** (`lscan_*_m2.out`; of the 8 hold-out shapes below 100%, (0.38, 9.6) and (15.5, 0.35)
have lnB = 0.0023 and -0.017, where the needed shift is beyond the scan's ±12 ulps, and three a = 1 shapes miss 1-2
rows each). So **everything except algdiv is closed**: the remaining misses of M2 are the lnB of min < 1 < 8 < max.

### Hold-out discipline
- Probe `hold` (shapes and x different from `a`, except the two w4 shapes (0.6, 7.3) and (0.5, 6.25) at new x) was
  built and filled together with `a`, after M1's exponent residual had been found on probe1 and r2-log1p's rows but
  before any tuning on `a`. Its densities were not read until M2 was frozen; then it was scored once, for the two
  pre-registered algdiv choices (`final_hold.out`). The `gl` probe reads only GAMMALN at both probes' arguments.
- The frozen algdiv choice is TOMS-d, chosen on the in-sample total (16,784 against 16,040 for the accurate value,
  driven by probe1's (0.5, 20)). On the hold-out the accurate value does better (3,469 against 3,106): it is reported,
  not swapped in.

## 2. How M2 was found (exponent)

1. **lnB is not the a = 1 residual.** On r2-log1p's 15,596 rows BETA.DIST(p,1,2^k+1), tx = 0 and ty = 2^k·l is exact
   (l pinned per row from BINOM.DIST(0,2^k,p)), so E = ty - lnB is exact. A double lnB scan peaks at j = 0 with zero
   hits at j = ±1 on all 22 b (`lscan_log1p.out`); a sub-ulp (extended) lnB scan peaks at 0 ± 1/32 ulp
   (`fscan_log1p.out`). M1 still gets only 75%.
2. **The needed exponent offset is run-constant** (`need.py`, consecutive-double runs): about ±(1-4)·2^-55, i.e.
   Excel ≈ exp(E)·(1 + ε) with |ε| up to 2^-53, constant over a run and varying across runs.
3. **Rejected explanations of that noise**, every one at or below M1's 75% on the exact-E rows:
   - EXP internals on an extended argument (`expvar.out`, 1,950 rows): V6 1,456; f stored 1,447; u stored 1,453;
     1 + u stored 1,281; reciprocal stored or a signed F2XM1 1,456; Cody-Waite with a double r 1,399; e^int(E) ·
     e^frac(E) from a double table 1,292. The correctly rounded exp(E) gets 1,451, so no accurate exp of the exact E
     fits either.
   - A table-driven exp indexed by frac(t) at N = 32 ... 8192 entries: no more consistent bins than a shuffled null
     (`tbin.out`). Bins by floor/nint of E or t, E·8, E·64, t·16, t·256 or by b are also at null level (`kbin.out`).
   - E rounded to k = 54 ... 65 bits: at most the 59-bit value (E is exact in 59 bits).
   - The density's ln(1-x) held extended (the series' final r + s unstored): 14 of 3,899 (the l in the density is
     BINOM's stored l).
   - Product forms with a per-b constant C scanned over ±6 ulps (`cprod.out`): EXP(ty)·C 2,109, extended EXP(ty)·C
     2,252, EXP(ty)/C 2,374, extended /C 2,602 of 3,899, against M1's 2,933. A factor of fl(1-x)/(1-x) (the caller's y)
     does not fit either (10,426 rows in the interval against 11,730 for none).
4. **The split.** EXP(fl(E))·(1 + (E - fl(E))) fits all 1,950 exact-E rows. On probe1 the hi of the double-op sum
   fl(fl(tx + ty) - lnB) beats fl(E), fl(tx + fl(ty - lnB)) and fl(fl(tx - lnB) + ty) (`hivar.out`: 12,796 against
   12,004, 12,202, 11,965), and the final step must round the correction factor to a double: fl(EXP(hi)·fl(1+lo))
   12,780 (b = 1 stored) or 12,796 (b = 1 split too), against fl(EXP(hi)·(1 + lo)) unrounded 11,437, and against
   R64 / P + fl(P·lo) / P·R64(1 + lo) forms at 11,437-11,438 (`m1score.out` modes dd, ddf, m2).
5. **Variants of lo** (`m1score.out`): only the rounding error of fl(s - lnB) (dd_e2) 8,867 on probe1; only that of
   s = fl(tx + ty) (dd_e1) 6,467; adding the two products' rounding errors to lo (`dd2.py`) 7,783-11,172. So the
   products tx, ty are stored doubles whose errors are not carried, and both additions' errors are.
6. **Pieces stay as M1 had them** (`txty.out`, scored under the unrounded-factor form): LN(x) stored, ly stored,
   products stored (11,437 on probe1) against an extended ln x (11,217), an extended LN(fl(1-x)) (10,242), extended
   products (7,786).

## 3. lnB

### min(a, b) = 1: -LN(max)
TOMS betaln has no such case, but every a = 1 or b = 1 shape fits lnB = -LN(max) (Excel's LN) at its pinned value:
32 of 32 shapes (probe a, probe1, old), including (1, 9), where TOMS algdiv in doubles is 1 ulp low (need +1;
-LN(9) is +1), and (1, 2) / (2, 1), where the GL form (GL(3) = ln 2 + 1 ulp) misses and TOMS's gsumln fits but
breaks (1.5, 1) (623 of 1,600 against 1,600). M1's "a = 1, 2 < b < 8: -LN(b)" is the special case of this.
Hold-out a1big: 957 of 960.

### min < 1, max <= 8: GL form with Excel's GAMMALN (item 4 of the assignment: lnGamma(a0 < 1))
`lgam_glbranch.out`, rows of the GL branch only, lnGamma(min) candidates in the GL form:
| set | n | GL (Excel GAMMALN) | TOMS gamln | fl(gamln1 - LN) | correctly rounded |
|---|---|---|---|---|---|
| probe a | 1,648 | **1,648** | 1,245 | 1,284 | 1,412 |
| old | 478 | **478** | 325 | 325 | 445 |
| probe1 | 6,400 | **6,398** | 3,537 | 3,537 | 6,398 |

Excel's own GAMMALN values matter: the engine model differs from Excel's on 14 of 47 arguments below 1
(probe `t2513-r3-m1-residual-gl`, e.g. 0.71: -2 ulps, 0.77: +2, 0.99: +2) and this probe's values are used.
**(0.6, 7.3) and (0.5, 6.25)** (item 3): with exact two-piece literals (no 15-digit cut) both are 40 of 40 in-sample and
the hold-out's w4 block (with (0.6, 5.8) and (0.5, 7.7)) 159 of 160. The density's lnB is the GL form there; the
GL53/glalgdiv losses of verify-r2-bpser-small are in bpser's L (the cdf), not in the density.

### min < 1 < 8 < max: algdiv (item 1) - OPEN, sharply pinned
Under M2 every probe a algdiv shape has a unique lnB that fits all its rows (`lnbshape_a.out`: 22 of 22 shapes at
40/40, 48/48 or 96/96). Needed lnB, in ulps from TOMS algdiv in doubles with Excel's GAMMALN and LN:
(0.05, 12.5) -1, (0.15, 20.2) +1, (0.2, 99) -2, (0.25, 150) -8, (0.3, 8.5) +1, (0.3, 9.9) +6, (0.35, 9.9) +2,
(0.4, 2e4) +1, (0.5, 9.3) +1, (0.5, 20) 0, (0.55, 45.5) -1, (0.6, 1000.5) -1, (0.7, 11) -1, (0.75, 333.3) +1,
(0.8, 8.25) -2, (0.9, 14.7) -2, (0.9, 150) -1, (0.95, 33) +1, (9.5, 0.5) +1, (20.5, 0.3) +6, (61, 0.8) -1,
(88.8, 0.95) -1; 0 at (0.45, 60.6), (0.65, 3000), (0.99, 25), (12, 0.15), (500, 0.7) and probe1's (0.5, 20).
In units of ulp(algdiv), Excel's algdiv is within about ±1.7 ulp of the exact ln(Γ(b)/Γ(a+b)) on every shape
(`algneed.py`; the large shifts in lnB units are shapes where GL(min) + algdiv cancels).

Candidates scored on these pins (`lnbcands.out`; min = 1 shapes take -LN(max) in every row of the table; probe a has 52
max > 8 shapes (2,448 rows, per-shape best 2,446), the old cells 287 two-row shapes (best 606), probe1 (0.5, 20) and
(1, 20) (best 3,198)):
| candidate | probe a rows / shapes | old rows / shapes | probe1 rows |
|---|---|---|---|
| TOMS algdiv in doubles, Excel LN and GAMMALN (frozen) | 1,525 / 31 | 314 / 128 | **3,198** |
| x87 stored ops throughout | 1,525 / 31 | 314 / 128 | 3,198 |
| accurate algdiv, fl(GL + alg) | 1,654 / 35 | **358 / 148** | 2,281 |
| TOMS fully extended (x87), TOMS alnrel | 1,614 / 35 | 303 / 120 | 1,663 |
| extended, correctly rounded alnrel | 1,636 / 34 | 294 / 115 | 1,663 |
| extended, r2-log1p's series as alnrel(a/b) | **1,677 / 35** | 314 / 126 | 2,281 |
| extended, LN(fl(1 + a/b)) | 1,247 / 27 | 65 / 24 | 1,599 |
| extended, log b extended | 1,607 / 34 | 351 / 144 | 2,281 |
| extended, GL + alg extended | 1,614 / 35 | 303 / 120 | 1,663 |
| doubles, series alnrel | 1,524 / 31 | 292 / 116 | 3,198 |
| doubles, correctly rounded alnrel | 1,403 / 29 | 301 / 119 | 1,663 |
| semi-exact: exact arithmetic with LN(b), a/b, d, alnrel, u, v, w or the final value rounded (1-3 of them) | at most 35 shapes | | 3,198 only with the final value rounded |

lnGamma(min) in this branch: GL beats TOMS gamln, fl(gamln1 - LN) and the correctly rounded value in every
combination (`lgam_a.out`, `lgam_old.out`: all rows of min < 1 shapes, GL 2,118 / 774 against at most 1,854 / 725).
No candidate reproduces both the probe a pins and (0.5, 20). `algsearch.py` (x87 with every subset of d, w, u, v, r, s
stored, × 4 alnrel × 2 log b × 3 final orders × 2 sums = 3,072 variants, on the 309 min < 1 shapes of probe a, probe1
and the old cells): best 2,490 rows (d and v stored, series alnrel, log b extended) against TOMS in doubles 2,365 and
a per-shape best of 3,578 (`algsearch.out`; its planted-variant control is `algsearch_ctl.out`). Negative.

## 4. Probes (eval/suites/probes-numeric/cases, excel-graph, filled 2026-10-24)
- `t2513-r3-m1-residual-a` (16,384 cells) and `-hold` (16,384 cells): A = x, B = a, C = b as exact two-piece literals,
  D = BETA.DIST(A,B,C,FALSE); all 4,096 + 4,096 echoes equal the intended doubles (`control.out`). Blocks in
  `mkprobe.py`; metadata `probe_a.meta.json`, `probe_hold.meta.json`.
- `t2513-r3-m1-residual-gl` (1,138 cells): Excel's GAMMALN at the 569 shape arguments of both probes, the old cells and
  probe1 (`mkprobe_gl.py`, `probe_gl.args.json`). Total 33,906 cells.

## 5. Controls
- `control.out`: lnB shifts of -4..+4 ulps planted per shape in M2-synthetic data are recovered at 100% on 89 of 90
  shapes; M2 reproduces only 3,084 of 4,096 M1-synthetic values (so 100% per-shape fits are not automatic); every
  probe echo is exact.
- `algsearch_ctl.out`: a planted x87 variant (w, u, v stored, log b extended) is ranked first by the same search (tied
  with 11 variants that differ only in the final order or sum storage, which are invisible there).
- `bratio_excel` default betaln = `algdiv_v.algdiv(ar="d")` on every shape (`lnbfit.out` header).

## 6. Open, and next steps
1. **algdiv for min < 1 < 8 < max.** The pins above (52 shapes in `lnbfit_tab.pkl`, rebuilt by `lnbfit.py`; old cells
   add about 280 two-row shapes) are the target. Excel's value is near the exact one but not equal to it or to TOMS in
   doubles. Next: probe ln(Γ(b)/Γ(a+b)) directly as GAMMALN(b) - GAMMALN(a+b) is not possible (cancellation), so
   pin more shapes on b sweeps at fixed a (one a, 40 b values, 40 x each) to see whether the error is a smooth
   function of b (an approximation) or rounding noise (a storage pattern); also try a Stirling difference with
   Excel's GAMMALN-internal Stirling series (T-1941's GAMMALN model) instead of TOMS's del(b).
2. **Port M2 to the engine** (T-2503): the exponent split, -LN(max) at min = 1, Excel GAMMALN; the density is then
   bit-exact wherever lnB is right (all shapes with max <= 8 or min = 1 here, in and out of sample).
3. **The cdf prefactor**: bpser's z = fl(a LN x) - L and bup/bfrac's density·x·y should be rescored with the M2 split
   (EXP(hi)·fl(1 + lo)) in place of EXP of an extended argument (r2-bratio `bpser_w3`, `w3_z = "z64"`).
4. Out of scope here: min(a, b) > 1 (Loader) and a >= 8 with b > 1.

## Scripts
| script | what |
|---|---|
| `m1lib.py` | shared: base model, pieces, exponent modes (`combine`: m1, ext, dd*, ddf, dde, m2), loaders, Excel GAMMALN from probe gl, scorer |
| `final_score.py` | **frozen model M2** (`lnb_m2`) and the score tables (`final_*.out`) |
| `m1score.py` | exponent modes per region (`m1score.out`) |
| `lscan.py`, `fscan.py` | per-shape double / sub-ulp lnB scans |
| `need.py`, `tbin.py`, `kbin.py` | needed exponent offsets per row (`need_*.tsv`, git-ignored) and the table / bin tests |
| `expvar.py`, `cprod.py`, `dd.py`, `dd2.py`, `hivar.py`, `txty.py` | exponent negatives and the split's variants |
| `algdiv_v.py`, `lnbfit.py`, `lnbcands.py`, `lnbshape.py`, `algneed.py`, `algsearch.py`, `lgam.py` | lnB: algdiv variants, pins, lnGamma(min) |
| `mkprobe.py`, `mkprobe_gl.py` | the three probes |
| `control.py` | positive controls |
Run from this directory with `PYTHONPATH=. /home/user/rustsheet/bindings/python/.venv/bin/python <script>`.
