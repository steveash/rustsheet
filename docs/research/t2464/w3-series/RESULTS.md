# T-2464 w3-series: Excel's direct-branch power series (bpser) — results

Worker w3-series. Method: the direct branch as prefactor x series, decomposed on w2's consecutive-double runs and
shape sweeps, then scored on every cdf cell. Python: `/home/user/rustsheet/bindings/python/.venv/bin/python`, run from this directory.
Units: "units" = relative error in 2^-53. Scores are bit-exact / within 4 ulp / within 15 digits, "of n".
CR = the correctly rounded value. The hold-out half is `lib.split_half` (an md5 hash of case+cell, odd half).

## 1. Headline

**Excel's incomplete beta follows the TOMS 708 `bratio` dispatch, and its `bpser` is a forward-accumulated Cephes-style
series.** This model (`model.py`, dispatch in `bratio_map.py`) is bit-exact on 10,253 of the 21,834 cdf cells
(beta_exact plus the w2 probes, closed forms a = 1 and b = 1 excluded). CR gets 5,786. Only the two bpser branches
are modelled; every other branch keeps the CR value. In the bpser branches:

| bratio branch (n) | model z53 | model z64 | CR | hold-out z53 / CR |
|---|---|---|---|---|
| w = bpser(a,b,x), min(a,b) <= 1, no swap (4,934) | **4,318**/4,885/4,910 | 2,819/4,910/4,922 | 786/4,515/4,552 | 2,173 / 404 of 2,481 |
| w = bpser, min <= 1, swapped at x > 0.5 (709) | **664**/708/708 | 590/708/708 | 398/708/708 | 329 / 191 of 348 |
| w1 = bpser(b,a,y), w = 0.5 - w1 + 0.5, min <= 1 (1,088) | 1,020/1,088/1,088 | **1,071**/1,088/1,088 | 508/1,088/1,088 | 498 / 255 of 536 |
| w = bpser, a,b > 1, b0 < 40 and b0*x0 <= 0.7 (249) | 124/208/221 | **126**/224/236 | 19/122/196 | 70 / 11 of 142 |
| w = bpser, a,b > 1, swapped at the mean (152) | 143/152/152 | **145**/152/152 | 94/152/152 | 75 / 50 of 79 |

Split by the README's branches (direct means x <= a/(a+b)):

| cells | z53 | z64 | CR |
|---|---|---|---|
| beta_exact, direct cdf (497) | 126/345/412 | **162/385/438** | 54/284/378 |
| beta_exact, reflected cdf (281) | 232/280/280 | **235**/280/280 | 218/280/280 |
| all, direct cdf (16,147) | **5,697**/13,501/15,050 | 4,470/13,542/15,077 | 2,989/13,144/14,766 |
| all, reflected cdf (5,687) | **4,556**/5,686/5,686 | 4,265/5,686/5,686 | 2,797/5,586/5,586 |
| hold-out, all direct (8,094) | 2,842/6,776/7,554 | 2,239/6,791/7,562 | 1,465/6,593/7,406 |

There is also a hold-out that was never used for tuning: T.DIST.RT at 0 < t <= 1 is 0.5 - I_z(0.5, n/2)/2
(statistics.md) with the inner value from this model (`tdist_v2.py`). Over 548 cells the model gets
**389/539/547**, against CR 298/523/542 and 326/540/547 for the previous best (TOMS bpser summing order with the GL
prefactor). For n >= 17 it gets 258/361/361 against CR 201.

The pdf was not attempted: it was outside this assignment and no series is involved.

## 2. The model (min(a,b) <= 1, and a,b > 1 with b0*x0 <= 0.7)

```
dispatch     = R toms708.c bratio: min(a,b) <= 1 -> swap if x > 0.5; bpser / 1 - bpser(b,a,y) by bratio's tests
               (x0 >= 0.29 -> w1 = bpser(b0,a0,y0); ...); a,b > 1 -> swap if lambda = a - (a+b)x < 0 (the mean);
               bpser if b0 < 40 and b0*x0 <= 0.7. y = fl(1 - x). Complement 0.5 - w1 + 0.5, x87 stored.
s            = fl(1/a); c = 1; n = 1, 2, ...: c = c * (((0.5 - b/n) + 0.5) * x); s = s + c/(a+n)
               (every op x87 stored, fl53(fl64(.)); stop when s no longer changes)
alx          = fl53(fl64(a * LN(x)))                Excel's LN, the product stored
L            = fl64(lg(a) + fl64(lg(b) - lg(a+b)))  ln B kept in extended precision
               lg(v) = accurate ln Gamma (64-bit) for v < 1, Excel's GAMMALN for v >= 1;
               if max(a,b) > 8: L = accurate ln B (64-bit). TOMS uses algdiv here, Excel's exact form is unknown.
z53 variant  : I = fl53(fl64(EXP(fl53(alx - L)) * s))    (Excel's EXP)
z64 variant  : I = fl53(fl64(fl53(exp(fl64(alx - L))) * s))   (EXP of the extended argument)
```

## 3. Key evidence, in the order found

1. **The prefactor exponent is a*LN(x) - lnB, with Excel's LN, and no y.** On the (0.5, 6.25) x-run
   (`regress1/2/3.py`), Excel's error regresses on a*e(LN x) with coefficient +1.01. It regresses on EXP(alx - L)'s
   rounding with +0.70 (+1.0 on the b- and a-sweeps), on EXP(alx) with -0.03, and on EXP(alx + b log1p(-x) - L) with
   -0.02. A 2F1(a+b, 1; a+1; x) form with the x^a y^b prefactor scores 596 against 658 (`hser.py`). The
   y-dependence is 0, as w2's tie test found.
2. **The plateau.** With P = EXP(z)/a, every series variant and every rounding mode of an exact S, bpser or Cephes
   pseries lands on 639-668 of 1,024 (`sscan.py`, `zoo*.py`, `extz*.py`, `zfine*.py`). Interval inversion
   (`interval.py`) shows that no smooth S times that P covers more than 770 of 1,024. The spectrum of Excel's error
   lacks the 88-cycle line that the P*fl(S) final rounding must produce, and has a 112-cycle line. `qlines2.py`
   matches that line to the stored first partial sum fl(1 + a*w1). So the series is accumulated term by term into a
   stored double.
3. **Forward accumulation breaks the plateau.** S = 1 + sum a*c_n/(a+n), summed forwards in double: 950/1,024 on the
   x-run (`accum.py`, `accum2.py`), 508/512 on the b-sweep and 436/513 on (0.5, 0.7).
4. **Accumulation starts at 1/a (Cephes terms c_n/(a+n)), and I = EXP(z) * s with no division by a.** This only
   shows where a is not a power of 2: the a-sweep goes 302 -> 444/512, and (1.5, 0.5) goes 283 -> 770/1,024
   (`asweep5.py`, `seriescmp.py`). x87 stored beats SSE double: 444 against 395 on the a-sweep, 506 against 497 on
   (0.5, 0.7). The recurrence form ((n-b)/n, 1-b/n, TOMS's (0.5-b/n)+0.5, and the other orders), the term form and
   the stopping rule are invisible (+-1 cell, `recscan.py`).
   - Positive control (`control2.py`): synthetic data from each known series, on the same points. The wrong series
     scores 165-323 against the right one's 512/1,024. Excel scores 426 (a-sweep) and 770 ((1.5, 0.5)) with
     fwd-from-1/a, against at most 282 and 283 with the others. At a = 0.5 the two forward forms are identical, as
     expected.
5. **ln B.**
   - ln Gamma(v) for v < 1 is NOT Excel's GAMMALN. The accurate value is better: a-sweep 426 against 356, (0.5, 0.7)
     506 against 359.
   - GAMMALN(v+1) - LN(v) fails: the x-run drops to 45.
   - For v >= 1 the GAMMALN errors are inherited: the b-sweep gives 508/512 with Excel GAMMALN (statistics.md's
     GL(b), GL(a+b), as w2 found).
   - The sum is kept in extended precision, and the inner difference must be extended: (0.5, 0.7) gives 506 with it
     and 403 with a stored difference (`zext*.py`).
   - For max(a,b) > 8 an accurate ln B lifts the a < 1, b > 8 beta_exact cells from 0 to 21/24 (`evalv2.py`).
6. **x switch.** For min(a,b) <= 1 the direct formula holds up to x = 0.5, not the mean. On (0.5, 0.7) the cells
   between the mean (0.4167) and 0.5 are 504/511 and 504/513 bit-exact with the direct series. Above 0.5,
   1 - w3(y, b, a) gives 488/511 (`reflected.py`). On (0.5, 6.25) at x = 0.31, which is >= 0.29 and below the mean,
   1 - bpser(b, a, y) gives 825/1,024 against CR 464. Both are exactly bratio's tests, so the whole TOMS dispatch
   was then adopted (`bratio_map.py`).

## 4. What is NOT settled (each affects only the prefactor constant, not the series)

- **z53 or z64.** The w2 runs prefer a stored EXP argument (x-run 950 against 526 for z64 without an offset).
  beta_exact prefers the extended argument: b < 1 and a >= 2 cells go 6 -> 32/65, the a < 1 tiny-x cells lose the
  16-unit z-rounding, and w1_bpser gives 1,071 against 1,020. With z64 the x-runs need L shifted by about +1.1E-16
  ((0.5, 6.25) peak 942 at +1.04E-16; (1.5, 0.5) peak 880 at +1.3E-16; `z64scan.py`, `lg05.py`). That is about
  one ulp of ln Gamma(0.5), but forcing ln Gamma(0.5) = 0.5723649429247002 breaks (0.5, 0.7) (`lg05b.py`).
  - Reading: z64 is the structure, and the remaining misfit is in ln B. One cause is the engine's GAMMALN model,
    whose K2/K3 kernels are not bit-exact (gammaln.rs docs, T-2122). Another is a GAMMALN extended value used before
    its final rounding.
  - At the a-sweep cells 293-302, a+b = 6.750000000000004 carries a -6.96-unit GL(a+b) model error that Excel
    does not have (`amiss.py`).
- **max(a,b) > 8**: the accurate ln B is a proxy for TOMS algdiv. With b < 1 and a >= 10 the model is still 10-30
  units off on some cells.
- **x >= 0.7 with b < 1**: 0/22 bit-exact (`xbins.py`), and b = 0.0001 with a >= 50 is about 50-90 units off.
  bratio sends these to bpser too, so either bpser differs at large n (277-1,486 terms) or ln B is wrong.

## 5. Region map, before the model (`region.py` -> `region.out`; S summed TOMS-style, lnB GL / exact)

| region (direct cdf) | n | CR | best prefactor*S then |
|---|---|---|---|
| a < 1 | 2,742 | 399/2,467/2,490 | 1,003 (Cephes ext, GL lnB) |
| b < 1 | 1,229 | 298/1,148/1,189 | 296 (bpser 'unchanged') |
| a,b > 1, bx <= 0.7 | 262 | 20/129/209 | 70 |
| a,b > 1, bx > 0.7 | 11,915 | 2,273/9,401/10,879 | 1,713 (exact lnB) |

- TOMS's stopping tolerance eps = 1E-15 (`eps_a`) loses against summing to convergence on b < 1: 99 against 291.
- A per-group free constant (`freel.py`, fit on the tune half) shows that before the forward-accumulation finding the
  prefactor structure added little over a free scale: a < 1 hold-out 583 for all-extended, against 527 for CR times a
  free constant.

## 6. Negatives (all scored, details in the scripts and `.out` files)

- Exact S, bpser (1 + a*sum) and the Cephes pseries summing order (sum from n = 2, then + t1, then + 1/a), each in
  dbl, x87 and ext, with stop rules eps/a, relative and unchanged: plateau at 639-668 on the x-run and 184-283 on
  (1.5, 0.5).
- These prefactor variants are rejected:
  - EXP(alx)*EXP(-L)/a, with 540-597;
  - EXP(z + LN S) and EXP(z + LN s), with 502-678;
  - a z of 64 bits rounded to 53 with a fine sub-double offset, with no gain (`zfine*.py`);
  - a PC53 (53-bit precision control) x87 exp, with 552 against 639 (`pc53.py`);
  - an unrounded exp (exp64), with 606-697 under the old S.
- Two-term splits (`s1model.py`):
  - storing the first partial sum and then adding the rest, 643;
  - P*q1 + P*a*R, 595.
- The hypergeometric 2F1(a+b, 1; a+1; x) with an x^a y^b prefactor and log1p(-x) or LN(1-x): 596 and 291.
- GL(v) for ln Gamma(v < 1): the a-sweep gives 356 against 426. GL(v+1) - LN(v) and GL(v+1) - ln64(v): the x-run
  gives 45.
- ln B as LN of GAMMA ratios, in five orders (`lnbforms.py`): worse everywhere except at (0.5, 0.7) and (1.5, 0.5),
  where all forms tie.
- a, b > 1 runs outside bratio's bpser region, (2.5, 3.5) at x 0.21 (b*x = 0.74) and (4, 4) at x 0.23: no series
  form beats CR (`seriescmp.py`). That fits bratio, which sends them to bup + bfrac.

## 6b. a, b > 1 outside bpser (bratio L140: bup + bpser, then bfrac); the series parts tested, not solved

- **Excel-styled L140 (`bup.py`, `bup.out`)**: the w3 bpser plus TOMS bup with an EXP(b' ln y + a LN x - lnB)/b'
  prefactor and x87-accumulated powers of y.
  - 10,370 cells with x0 <= 0.7: 3,448/9,313/10,213, against CR 3,343/9,181/10,305. It is worse than CR on half the
    runs.
  - Its tie response is POSITIVE (+1.0 to +7.6), where Excel has -1.4 to -8.5. My estimator reproduces w2's
    Excel values (-3.70 against w2's -3.72 at (2.5, 3.5)).
- **R toms708 bfrac** (`bfrac_tie.py`): c = -1.0 to +1.9. 1 - bfrac(b, a, y, x) and 1 - bratio(b, a, y, x) are far
  off. All are rejected as the a, b > 1 structure.
- **x recomputed from y** (x' = 1 - fl(1 - x), exact): R bratio(a, b, x', y) gives the right SIGN but about twice the
  size.
  - c = -7.7, -3.6, -11.2, -11.1, -4.7, -36.8, where Excel has -3.7, -1.4, -8.5, -7.2, -1.9, -10.6.
  - In the Excel-styled L140, ln x' in bup's prefactor raises exact counts on some runs: (7.3, 11.6) 111 against CR
    85; (4, 4) 138 against 96 (`bup2.py`).
  - Reading: some, not all, of the x-dependence is evaluated from y. That is a lead for whoever takes the
    a, b > 1 kernel.

## 7. Next hypotheses (most promising first)

1. **Model the remaining bratio branches with the same Excel style.** The largest is bup + bfrac (a, b > 1, b0*x0 >
   0.7: 10,860 direct cells, CR 2,238). For x0 <= 0.7 it is w = bup(b', a, y, x, n) + bpser(a, b', x) with
   b' = frac(b) (bratio L140).
   - The bpser part is this model. bup uses brcomp's x^a y^b prefactor with y = fl(1 - x).
   - That is where w2's negative tie response for a, b > 1 can come from: I = I_x(a, b') + [I_x(a, b'+n) - I_x(a, b')],
     and the bup difference carries y with a weight that can make the total response negative.
   - bfrac (the continued fraction), bgrat and basym follow.
2. **Pin ln B.** Probe GAMMALN-free shapes (a, b < 1, a + b < 1) to read ln Gamma(v < 1) directly. Probe a = 0.5
   with b on a grid where the GAMMALN model is known to be bit-exact (b >= 8 Stirling) to decide z53 or z64 without
   kernel error.
3. **bpser at large n** (x >= 0.7, b < 1; b = 1E-4): check the accumulation against Excel on an x-run near 0.8 with
   b = 0.5 and a = 10. Possible causes: c underflow guards, a different term order, or bratio's apser/fpser cut-offs.
4. **Feed the series into F.DIST, T.DIST and BINOM.** The T.DIST hold-out already gains 63 cells, about 26% over CR.

## Scripts (all in this directory)

- `lib.py`: data (beta_exact + w2 probes), exact values, cached Excel functions, numpy-longdouble x87 ops (checked
  against `excelmath.x87`), scoring.
- `series.py`: the series candidates: bpser_sum, cephes_pseries_sum, accum_sum, accum2.
- `model.py`: the model of section 2.
- `bratio_map.py`: the TOMS dispatch map, with outputs `bratio_map_z53.out` and `bratio_map_z64.out`.
- `branch_scores.py`: the README-branch scores.
- `tdist_v2.py`: the T.DIST hold-out.
- `seriescmp.py`, `control2.py`, `recscan.py`, `accum*.py`, `asweep*.py`, `zext*.py`, `lnbforms.py`, `z64scan.py`,
  `lg05*.py`: the series and prefactor experiments of section 3.
- `regress*.py`, `spec*.py`, `qlines*.py`, `interval.py`, `control1.py`, `binexp.py`, `mix.py`, `expkind.py`: the
  decomposition forensics.
- `dispatch.py`: the bratio dispatch. `bup.py`, `bup2.py` and `bfrac_tie.py` cover a, b > 1 (section 6b).
- `region.py`, `freel.py`, `hybrid.py`, `implied.py`, `byblock.py`, `xbins.py`, `reflected.py`, `listbe.py`,
  `listb1.py`, `amiss.py`, `amax8.py`, `tdist_holdout.py`, `tdist_modes.py`, `noisebudget.py`: the region maps and
  per-cell listings.
- Caches (`*.pkl`, `*.npy`) are git-ignored and are rebuilt automatically.
