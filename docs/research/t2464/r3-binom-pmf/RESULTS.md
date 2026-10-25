# T-2513 r3-binom-pmf: Excel's BINOM.DIST(k,n,p,FALSE) and NEGBINOM.DIST(f,r,p,FALSE)

Worker r3-binom-pmf, round 3, 2026-10-24. Goal: use the binomial pmf, where k and n are integers, to pin Excel's
Loader pieces (stirlerr, bd0, lf, the exponent chain) for both the pmf and the beta density.

Counts are bit-exact / within 4 ulp / within 15 significant digits ("w15"), as `blib.tally` computes them. "Units"
are 2^-53, absolute in the exponent E (the pmf is EXP(E)). Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`. Outputs sit next to the scripts as `*.out`.

## 0. Data, probes, controls, housekeeping
- **Old data:** the pmf cells of `../cells.tsv`, 2,211 after filtering: BINOM.DIST/BINOMDIST 1,644 and
  NEGBINOM.DIST/NEGBINOMDIST 567. They come from t2145-binomial-sweep, t2145-negbinom-sweep and the legacy probes.
  p is in {0.1, 0.125, 0.2, 0.25, 0.3, 0.5, 0.7, 0.9, 0.99, 0.999} and n <= 100 (one 1000).
- **New probes** (excel-graph, filled 2026-10-24, all three filled before any tuning). The builder is
  `pmfprobe.py`, which rebuilds the row map, so there are no meta files. p arguments are w2's exact-double
  literals, and every 16th row echoes p: 0 misreads in all three probes (`blib.echo_check`).
  - `t2513-r3-binom-pmf-a` (10,890 cells, in-sample).
    - 14 (k,n) shapes: (3,8), (5,16), (10,32), (40,128) and (100,256), where n·p is exact, plus (3,10), (7,20),
      (13,30), (30,100), (37,77), (150,1000), (1,10), (2,50) and (45,50).
    - Runs of 96 consecutive doubles p, placed at bd0 ratio v = -0.35, -0.12, +0.12, +0.35 on the smaller side.
    - Column A is BINOM.DIST(k,n,p,FALSE). Column B is BETA.DIST(p,k+1,n-k+1,FALSE) at the same p.
  - `t2513-r3-binom-pmf-n` (4,544 cells, in-sample).
    - Column A: NEGBINOM.DIST(n-k,k,p,FALSE) on the first 48 rows of every `a` run.
    - Column B: large-n BINOM runs at bd0 of about 3 and 20 nats, 64 doubles each. The shapes are (3000,1e4),
      (3e4,1e5), (3e5,1e6), (5,1e5), (1,1000), (999,1000), (500,1000) and (4e6,1e7).
  - `t2513-r3-binom-pmf-hold` (10,584 cells, **hold-out**).
    - Nine other shapes: (4,12), (6,40), (17,64), (25,60), (60,200), (400,1500), (1,30), (29,30), (11,23).
    - v = -0.25, -0.07, +0.06, +0.25, 96 each, with columns BINOM, BETA and NEGBINOM.
    - Scored once, after the models below were frozen (`c1_hold.out`).
- **Positive control** (`c2_control.py`, `c2_control.out`). Each frozen model was planted as synthetic Excel values
  on 1,320 probe-a rows.
  - The scorer recovers each one 1,320/1,320.
  - Other candidates match 204-1,141 of them.
  - So a model difference of the size seen on real data can be resolved.
- **Reuse** (imports, no copies):
  - r3-loader-bd0 `r3lib`: numpy x87 ops, Excel LN memo, EXP V6 on extended input, Loader bd0 with its knobs,
    interval fits.
  - r2-log1p `excel_log1p`;
  - excelmath (GAMMALN);
  - w2 `make_probes` (encoder, loaded by path).
- **Housekeeping.** Caches (`eset.pkl`, `gl.pkl`) are git-ignored here. About 1 MB of scripts and outputs. No
  sidecar copies.

## 1. Headline
1. **k = 0 and k = n are Loader's own closed forms, not R's.** BINOM.DIST(0,n,p,FALSE) = EXP(fl(n·log1p(-p))),
   with r2-log1p's series: 173/173 on cells.tsv (CR 54). BINOM.DIST(n,n,p,FALSE) = EXP(fl(n·LN(p))): **172/172**
   (CR 70), including p = 0.99 and 0.999.
   - R's `dbinom_raw` would use -bd0(n,np) - nq for q < 0.1, and -bd0(n,nq) - np for p < 0.1.
   - Loader's (2000) paper code is `if (x==0) return exp(n*log(1-p)); if (x==n) return exp(n*log(p));`, which
     matches.
   - This rule is new. `a1_survey.out`.
2. **BINOM.DIST's pmf and BETA.DIST(p,k+1,n-k+1,FALSE) are one kernel** (`b2_share.py`, `b2_share_a.out`).
   - On all 56 probe-a runs, the difference of their exponent errors has sd 0.38-1.13 units. That is the
     output-rounding floor.
   - Each one alone has sd up to 195 units (for example (150,1000) at v = -0.35: 195.0 and 195.0, difference 0.40).
   - Every p-dependent rounding is shared. Only a per-shape constant differs, from LN(n+1) and the final rounding.
   - **So the binomial is a valid window on the beta density's Loader pieces at integer shapes.**
3. **bd0's log form is confirmed at integer shapes** (`b3_reg.py`, `b3_reg_a.out`).
   - The ratio fl(k/M), Excel's LN and the product fl(k·l) each enter the exponent with coefficient 0.95-1.10 on
     every log-form side.
   - M - k enters as stored, which only shows where the ratio is beyond 2.
   - M's own rounding (M = fl(n·p) and fl(n·fl(1-p))) enters with coefficient about 1.
   - On log-log runs the residual is at the floor where n·p is exact or small:
     - (10,32): 0.45;
     - (40,128): 0.47;
     - (100,256): 0.41, from sd 142;
     - (3,10): 0.30.
   - It stays large at (13,30): 11.4, (30,100): 15.4, (7,20): 4.0 and (150,1000): 115. That is an unmodelled
     stored quantity (section 3).
4. **NEGBINOM.DIST is not R's `dnbinom`.** It is the kernel shifted by one: p · dbinom_raw(r-1, f+r-1, p, q).
   - **Rejected:** NEGBINOM.DIST(n-k,k,p) shares no rounding with BINOM.DIST(k,n,p) at the same p
     (`b21_neg.out`). The sd of the difference is the quadrature sum, for example (30,100): 44.2 and 49.0 give 59.2.
     fl(fl(k/n)·BINOM) is exact at 264/2,640.
   - **Accepted:** regressed on the bd0 roundings of dbinom_raw(k-1, n-1, p, q), the coefficients are about 1 and
     the sd drops: (30,100) 44.2 → 0.62, (3,8) 4.49 → 0.52, (3,10) 6.06 → 0.71, (7,20) 8.91 → 0.87
     (`b3_reg_neg_shift.out`).
   - With the unshifted kernel, nothing drops (`b3_reg_neg.out`).
5. **The interior pmf is not solved to the bit.** Its exponent is computed in log space with a stored lc (R's
   `lc = ...; return exp(lc - 0.5*lf)`, the argument extended). It is not Loader's non-log
   exp(lc)·sqrt(n/(2πk(n-k))) (section 3). lf/2 and the stored chain are only partly identified. The best frozen
   models improve visible parity over CR by 6-9 points, in and out of sample:

| data | n | CR | L-ext/LF0 | **L-lcxx/LF0** | **L-lcx/LF1** |
|---|---|---|---|---|---|
| cells.tsv BINOM interior | 1,300 | 178/754/980 | 188/925/1,075 | 286/1,098/**1,175** | 389/1,070/1,150 |
| cells.tsv NEGBINOM (shifted kernel) | 567 | 123/407/462 † | 155/487/519 | 147/522/**533** | 150/505/524 |
| probe a, BINOM + BETA | 10,560 | 1,612/6,534/8,002 | 1,300/7,598/8,890 | 2,298/8,461/**9,495** | 2,427/8,332/9,236 |
| probe n, NEG + large-n BINOM | 4,432 | 549/2,382/2,759 † | 886/2,707/2,986 | 902/2,929/**3,153** | 699/2,824/3,131 |
| **hold-out**, BINOM + BETA + NEG | 10,368 | 2,021/6,951/8,031 † | 1,422/8,062/8,765 | 1,601/8,171/8,840 | 2,006/8,234/**8,917** |
| hold-out BINOM only | 3,456 | 762/2,345/2,790 | 516/2,762/3,014 | 638/2,759/2,992 | 794/2,763/**3,129** |

† NEGBINOM's "CR" column is fl(p · CR binomial(r-1, f+r-1)), not the correctly rounded NEGBINOM value.

The models (`blib.MODELS`, `blib.model_value`, `c1_score.py`) are:
- C = stirlerr(n) - stirlerr(k) - stirlerr(n-k), from R's table for n <= 15 and R's series above, extended;
- b1, b2 = r3-loader-bd0's bd0 (cut 0.2, series with its ph knob, log form stored with M - k stored);
- **L-lcxx**: lc = fl(fl(C - b1) - b2), then E = lc - LF0/2, extended;
- **L-lcx**: lc = fl(C - b1 - b2), then E = lc - LF1/2;
- L-ext: E = C - b1 - b2 - h, all extended;
- BETA: EXP(LN(n+1) + E), extended;
- NEG: fl(p · BINOM(r-1, f+r-1));
- k = 0 and k = n: item 1.

Both were frozen on cells.tsv, probe a and probe n before the hold-out was opened. On w15, L-lcxx/LF0 was best in
sample and L-lcx/LF1 best out of sample. Neither reaches the bit: the best in-sample exact rate is 23%. The
x-dependent part is right where n·p is exact, so the gap is the constant and chain of section 3. On the hold-out,
CR beats every model on bit-exact counts (2,021 against 1,422-2,006), while the models win on w15.

Large n (probe n column B, 1,792 cells, n from 1,000 to 10^7) is still poor for every model: 417-434 exact and
846-870 w15, against CR's 233/814. The series side of bd0 and stirlerr's large-n series dominate there (open).

## 2. What the exponent's structure is not (negatives, same data)
- **Not EXP of a stored double E.**
  - Nearly no probe-a cell with |E| >= 4 is EXP_V6 of any double. For example, all 96 cells of (3,8) at
    v = -0.35 have an empty admissible E set.
  - Chain "C" (E = fl(C + X)) with a free constant per block gets 0/96 on 9 of the 13 log-log runs. With the other
    stored-E chains R, S, K, Rg and SC, it is 107-121 of 623 (`b1_fit.py`, section 1 commands, and the ad hoc chain
    runs summarized here).
- **Not Loader's paper form exp(lc)·sqrt(n/(2πk(n-k))) with lc stored** (`b19_cfit.py`, `b19_cfit_mul.out`).
  - With one free constant per block and F = sqrt(fl(n/fl(fl(2π·k)·(n-k)))), extended sqrt, it gets 0/96 on every
    run with |lc| >= 16. Examples: (10,32), (100,256), (30,100), (40,128) at v = -0.35 and (150,1000) at ±0.12.
  - It reaches 92/96 at (3,10) only, where the log form also fits.
  - A model-free check, b10 (`b10_fres.out`), lets D float per cell and fits one real factor per block. It hits
    96/96 at large periods, but the factor residues match no F formula (`b10_fres.out` against the candidate
    columns).
  - The additive structure is the one consistent with the regression floor at (100,256), which leaves no room for
    a stored lc there.
- **Not R's n·q = n - n·p** (`b5_scan.py`). Free-constant all-cell counts on the 13 v = -0.35 runs:
  - nq = fl(n·fl(1-p)): Ce 747 and R:exe 632 of 1,248;
  - fl(n - fl(n·p)): 669 and 559;
  - n - n·p extended: 731 and 603.
- **Not other ratio formations** (`blib.bd0_v` knob `mr`, `b5_scan.py`). fl(k/fl(np)) wins at 339 of 623 odd
  cells (chain Ce, log-log runs). The alternatives:
  - fl(k/np) with np extended: 302;
  - mixed: 316-318;
  - fl(fl(k/n)/p): 283;
  - fl(fl(k/p)/n): 282.
- **The log form's pieces stay stored** (`b5_scan.py`, totals over the 13 runs, chain Ce/H:exe):
  - stored ratio, LN and product: 747/803;
  - LN kept extended: 472/526;
  - ratio and LN extended: 406/459;
  - product extended: 235/567.
- **Summation trees.** Over all 120 trees of E = C - h - b1 - b2 with store flags and one free constant per block,
  the best is 1,665/3,552 (47%, `b6_trees.py`, 37 probe-a runs). No storage pattern of these four terms fits every
  run.
- **Rejected direct forms** (`a3_forms.py`). rms of D - dX on 1,300 cells.tsv interior cells:
  - EXP(GAMMALN(n+1) - GAMMALN(k+1) - GAMMALN(n-k+1) + k·LN p + (n-k)·log1p(-p)), any store pattern: 156-231 units;
  - the exact-C direct form: 80-86;
  - Loader with exact pieces: 35. Its median |residual| is 2.5-2.9, against 8-24 for the direct forms.

## 3. What is open, measured
1. **lf/2 and the stored chain.** Two runs isolate the chain.
   - At (3,8) v = -0.35 the residual after the x-part model is fully explained by the rounding of one stored sum
     in the binade [4,8): rho 0.99, slope 1.00 (`b15_store_E.out`). The same holds at (5,16) (rho 0.97).
   - With E = C - fl(h' + b1 + b2), C pinned at R's table constant (fitted C - C_R = -0.38 units) and h' free
     (`b13_ch.py`), (3,8) reaches 96/96, but only for h' = h_exact - 1.85 ± 0.75 units (mod 8).
   - No lf formula tried gives that:
     - LF0 +2.15;
     - LF1 +0.15;
     - 0.5·LN(f) for R 2.0's f = 2π·x·(n-x)/n +0.15;
     - Loader's n/(2πx(n-x)) +0.15;
     - reorderings +0.15 (`blib.h_variants`, `b20_hplat.out`).
   - With C fixed to R's constant and h' free over a full period, most log-log runs reach 0/96 (`b20_hplat.out`).
     So C is not R's table/series value to 0.4 units on most shapes: the per-shape constant of r3-loader-const, seen
     at integer shapes.
2. **The unexplained noise at (13,30), (30,100) and (150,1000)** (sd 11-115 units). It has the size of a stored
   value at E's binade (32-128 units of ulp). It is not explained by:
   - the rounding of fl(n·p), fl(n·q), fl(1-p), fl(n - np), the ratios, fl(k - np) or products of p and q
     (`b14_mf.py`, matched filter, |rho| <= 0.21);
   - a stored lc or E with a free constant (`b15_store_lc.out`: rho 0.00-0.39).
   - Where n·p is exact, (100,256), (40,128) and (10,32), no such noise appears.
3. **Series side and large n:** as r3-loader-bd0 found, the series branch's arithmetic is not identified.
   - Series runs (v = ±0.12) reach only 4-94% in-run with free constants (`b1_fit_a.out`).
   - Large-n BINOM stays at about 48% w15.

## 4. For the coordinator, and shared with r3-loader-const and r3-loader-bd0
- New rules for statistics.md:
  - (a) BINOM.DIST(n,n,p,FALSE) = EXP(fl(n·LN(p))), 172/172.
  - (b) BINOM.DIST's pmf shares every p-dependent rounding with BETA.DIST(p,k+1,n-k+1,FALSE). Both call one
    dbinom_raw.
  - (c) NEGBINOM.DIST(f,r,p,FALSE) is p · dbinom_raw(r-1, f+r-1, p, q), not R's (r/(f+r))·dbinom_raw(r, f+r, p, q).
  - (d) The interior pmf's exponent is extended, and EXP is applied to it once. Neither stored E nor Loader's non-log
    product.
  - (e) bd0's log form at integer shapes is r3-loader-bd0's: stored ratio, LN, product, and M - k stored beyond
    ratio 2.
- Engine (T-2507): the k = 0 and k = n closed forms and the NEGBINOM shift are cheap and exact.
  - Use L-lcx/LF1 or L-lcxx/LF0 for the interior pmf only if the visible gain (hold-out w15 8,917 or 8,840 against
    CR's 8,031 of 10,368) outweighs the exact-count loss. On the hold-out, CR is better at the bit (2,021 against
    1,601-2,006).
  - Keep CR for n > 1,000 until the series is solved.
- For r3-loader-const: integer (k, n) runs where n·p is exact are the cleanest constant probes. (100,256),
  (40,128) and (10,32) at v = -0.35 have the x part at the 0.41-0.47 floor. (3,8) pins the chain as
  C - fl(h' + X) with C at R's table constant within 0.4 units.

## 5. Scripts
| script | what |
|---|---|
| `blib.py` | shared lib: data, exact values, scorer, stirlerr, x part (`bd0_v`, `xpart_v`), chains (trees), fits, frozen models |
| `pmfprobe.py` | the three probes (`--write` writes the cases) |
| `a1_survey.py`, `a2_need.py`, `a3_forms.py`, `a4_resid.py` | cells.tsv survey, closed forms, direct-form rejection |
| `b1_fit.py`, `b4_miss.py`, `b5_scan.py`, `b6_trees.py`, `b13_ch.py`, `b17_hscan.py`, `b20_hplat.py` | free-constant structure fits |
| `b2_share.py`, `b21_neg.py`, `b3_reg.py`, `b7_corr.py`, `b14_mf.py`, `b15_store.py` | chain-free diagnostics: shared roundings, regressions, matched filters |
| `b8_factor.py`, `b9_fscan.py`, `b10_fres.py`, `b11_hfree.py`, `b12_lattice.py`, `b16_full.py`, `b18_fscan2.py`, `b19_cfit.py` | final-structure tests (stored E, multiplicative factor, lattices) |
| `c1_score.py`, `c2_control.py` | frozen-model scores (in-sample, hold-out) and the positive control |
