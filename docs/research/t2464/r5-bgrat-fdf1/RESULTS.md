# T-3567 r5-bgrat-fdf1: F at a df of 1 through bgrat_half (solved), and D at b ≠ 1/2 (still open)

Worker r5-bgrat-fdf1, 2026-11-01 (T-3567; split from T-3468, T-3383, T-2512). Method:
docs/research/excel-kernel-playbook.md. Base kernel: r2-tdist FINAL (r2-bratio `bratio_excel.py` BEST + Excel log1p,
Excel ERFC at b = 1/2, bgrat's ln x cut at y = 0.01), with bgrat at b = 1/2 in its implementable x87 form
(`xlk.bgrat_half_ld`, `bg = "ld"`). Imported, not copied. Scores are bit-exact / within 4 ulp / within half a unit of the
15th significant digit (w15).

Python: `/home/user/rustsheet/bindings/python/.venv/bin/python` (`make py-bindings`, plus `uv pip install mpmath
numpy` into it), run from docs/research/t2464. The D scripts need r3-bgrat's caches first (`r3-bgrat/branches.py`,
then `r3-bgrat/exact.py -a`; both git-ignored).

## 1. Headline

1. **F's incomplete-beta arguments keep r >= 1, and with them F at a df of 1 is bgrat_half.** With the truncated
   dfs m, n and mx = fl(m·x), Excel's F forms
   ```
   r = fl(n/mx) if mx <= n, else fl(mx/n)                      (so r >= 1)
   smaller argument = fl(1/fl(1 + r)),  larger argument = fl(r/fl(1 + r))
   (x, y) = (smaller, larger) if mx <= n, else (larger, smaller)
   ```
   and calls bratio on (m/2, n/2, x, y). F.DIST(., TRUE) is bratio's w and F.DIST.RT / FDIST its w1. The T-2510
   miss ("`FDIST(x,1,9)` 7 ulp, F.DIST sweep cells visibly off") was the argument form, not the kernel: bgrat reads
   only ln y (y > 0.01), and ln y is very sensitive to how y is rounded. F at b = 1/2 takes the same bgrat_half as T.
2. **Hold-out** (probes-numeric t3567-r5-fdf1-hold, 3,578 cells, built and filled after `fmodel.py` was frozen,
   sha256 `89c67ddf…0809b16f05`; `score_hold.py` → `score_hold.out`):

   | region | cells | fmodel (r >= 1) | same kernel, x = fl(mx/fl(n+mx)) | engine today |
   |---|---|---|---|---|
   | bg1: d1 = 1, n 31-400, bgrat | 1,000 | **991/1,000/1,000** | 758/820/820 | 298/572/659 |
   | d1: d1 = 1, n 3-30 | 800 | 614/800/800 | 573/759/776 | 505/766/784 |
   | d2: d2 = 1 | 600 | 443/599/599 | 441/585/585 | 339/522/536 |
   | gen: d1, d2 in [1, 500] | 1,000 | 604/843/886 | 579/802/849 | 577/806/868 |
   | tie: m·x = n | 178 | 135/177/178 | 135/177/178 | 138/177/178 |
   | all | 3,578 | **2,787/3,419/3,463** | 2,486/3,143/3,208 | 1,857/2,843/3,025 |

   The correctly rounded value (mpmath, 50 digits; computed by the excel-fidelity review of PR #2669) is
   1,407/2,484 of 3,578 bit-exact / within 4 ulp: bg1 311/612, d1 298/631, d2 283/416, gen 432/673, tie 83/152.

   Per branch (fmodel): bgrat 1,012/1,023/1,025 of 1,028, bgrat/sw 155/158/158, closed a0 = 1 and b0 = 1 48/48 and
   54/54; what is left is in bpser(a, b > 1), bfrac and L140 bup+bgrat at b ≠ 1/2, each open in its own entry.
3. **The r >= 1 form is F's for every df, not only a df of 1.** On cells.tsv's 3,435 F cdf/tail cells (`c5.py` →
   `c5.out`, same kernel): r >= 1 2,300/3,295; n + mx 2,178/3,237; the density's sides (r <= 1) 1,994/3,113; the
   engine's 1/(1 + .) for both 2,194/3,193. The b0 = 1 and a0 = 1 closed forms (201/201, 162/162 bit-exact) cannot
   tell r >= 1 from the engine's form: they read only the smaller argument. bgrat (71 against 46) and bup+bgrat
   (49 against 33) can, because they read ln of the larger one.
4. **D at b ≠ 1/2 (the direct bgrat increment) is still open.** Along the split probe's two runs of 1,024 consecutive
   doubles, Excel's relative residual against the exact I at its own stored ln x is white noise: sd 5.7 and 8.5
   units of 2^-53, lag autocorrelations within ±0.07, no FFT line above 4x the mean (`dreg.out`). No stored
   intermediate tried (z, LN(z), b·LN(z), a·ln x, (b-1)/2·ln x, ln x, y) cuts the sd below 5.2. Only z's stored rounding
   is visible, with coefficient -6.4 ≈ -z at (30.7, 0.7) (z = 6.3), as r3-bgrat already models. So Excel's D carries
   a noise term of about 6-8 units that no structure seen so far explains (negatives below).

## 2. Data and in-sample steps

- `c0.py` (exact-math, 659 cells of cells.tsv with d1 = 1, d2 >= 3): the exact I at the double x = fl(x/fl(n+x))
  already beats y = fl(n/fl(n+x)) (265 against 162 bit-exact), so F does not hand bratio y = fl(n/(n+mx)).
- `c1.py`/`c2.py` (FINAL kernel): x = fl(mx/fl(n+mx)) with y = fl(n/fl(n+mx)) is 444/642 of 659 (engine 393/629),
  bgrat 32/33 of 38.
- `c3.py` (the 38 bgrat cells, every ln y form): y = fl(r/fl(1+r)) with r = fl(n/x) 33/38/38; RN(n/(n+x)) 31/34;
  1 - RN(x/(n+x)) 32/35; LN(fl(n/s)) 30/33; log1p(-x) 10/21; LN(n) - LN(s) 4/10.
- `c4.py` (all 659, full kernel): r >= 1 469/659/659 (engine 393/629); side 2 everywhere 460/646; side 1 everywhere
  426/635; the density's sides 417/622. FDIST(2,1,9) (gen-m9/c18dbf21f11093f3 A151) is bup+bgrat, 1 ulp off in the
  research kernel (bup's arithmetic, open), bit-exact in the engine today.

## 3. D at b ≠ 1/2: negatives (decision 0056)

All on the 2,908 points of probe t2512-r3-bgrat-split; r3-bgrat's best x87 bgrat (stored ln x and z, grat1's CF in
extended at eps 1e-15) is 206/1,672/2,158 and the stored-z ratio RN(F·Q(b, z_s)/Q(b, z_e)) 246/1,881/2,357.

| tried | score | script |
|---|---|---|
| prefactor E = exp(-(algdiv(b, a) + b ln ν)) rounded to double, through Excel's EXP of a double or of an extended argument | 206/1,688/2,170 (Q exact: 204/1,603/2,089) | `dvar.out` |
| E from TOMS algdiv in doubles + fl(b·LN(ν)) | 210/1,615/2,097 | `dvar.out` |
| E from extended algdiv + b·LN(ν) | 208/1,659/2,141 | `dvar.out` |
| ln x = Excel log1p(-y) / TOMS alnrel(-y) instead of LN(x) | 158/1,287/1,786; 189/1,449/1,974 | `dlnx.out` |
| grat1's CF in doubles (eps 15e-15, 1e-15, 1e-16) | 5/60/85, 200/1,500/2,072, 161/1,415/1,982 | `dcf.out` |
| the same, sd of Excel - model on the two runs | 7.4 and 10.0 (CF in extended: 6.6 and 8.4) | `dcf.out` |
| regression of the run residual on stored-intermediate rounding errors | residual sd 5.2 and 8.3 (from 5.7, 8.5) | `dreg.out` |

The prefactor's form does not matter (its argument is small at b < 1). The CF in doubles adds its own noise rather than
explaining Excel's. Next: a different Q(b, z) algorithm inside Excel's bgrat (Excel's own incomplete gamma, as
GAMMA.DIST computes it, whose upper tail the worksheet cannot show at b < 1), tested by a run probe of
GAMMA.DIST(z, b, 1, TRUE) at z < 1 where P is small and its noise is visible, against the same runs' D.

## 4. Probe

| probe (probes-numeric) | cells | what | script |
|---|---|---|---|
| t3567-r5-fdf1-hold | 3,578 | F.DIST.RT(x, m, n) (A) and F.DIST(x, m, n, TRUE) (B) at 1,789 points, seed 35670, regions bg1/d1/d2/gen/tie | `mkhold.py` |
