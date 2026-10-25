# T-2464 w5-triangulation: the other incomplete-beta functions as evidence

Worker w5-triangulation, 2026-10-23. Method: work out exactly how F.DIST, F.DIST.RT, FDIST, T.DIST*, TDIST,
BINOM.DIST and NEGBINOM.DIST call Excel's incomplete-beta kernel. That covers which argument and complement each one
forms, which tail it computes and how it rounds the result. Each mapping is checked to the bit where a closed form or a
second oracle cell allows it. The mapped cells are then converted into `ix_evidence.tsv`.

Units: "x/y/z of n" means bit-exact / within 4 ulp / within 15 significant digits. The hold-out half is
`w5lib.holdout` (md5 of case+cell, odd half). Python is `/home/user/rustsheet/bindings/python/.venv/bin/python`, and
scripts run from this directory. EXP and LN are Excel's own, from the harness (`excelmath.py`).

## 1. Headline

1. **One kernel K(a, b, x, y), with TOMS 708 `bratio`'s dispatch, behind every function here.** The caller forms
   both the argument x and its complement y. The kernel computes one tail directly, and the other tail is
   `(0.5 - w) + 0.5`.
   - F.DIST and F.DIST.RT at the same point: 1,578 of 1,578 pairs fit one evaluation. Where the pair shows which
     tail was computed (1,208 pairs), bratio's dispatch predicts it every time. A switch at the mean gets 8 wrong.
   - NEGBINOM.DIST(f,r,p,TRUE) and BINOM.DIST(r-1,f+r,p,TRUE) are the two tails of one I_p(r, f+1): they are exact
     complements on 441 of 460 pairs. The other 19 are BINOM's k = 0 shortcut and closed-form edges. Where the
     computed tail can be decided (348 pairs), bratio's dispatch predicts it every time.
   - So **BINOM.DIST's cdf is the incomplete beta, not a summed pmf.** It is not a running sum of Excel's own pmfs
     (501 of 1,371) or of recursive terms (at most 504 of 1,632).
2. **The kernel's closed forms, now exact, bit-exact on 1,286 of 1,286 cells (633 of 633 on the hold-out) across
   BETA.DIST, F, T, BINOM and NEGBINOM.** They apply after bratio's swap (x > 0.5 → (b, a, y, x)), on the oriented
   (a0, b0, x0, y0):
   ```
   b0 == 1                 : w  = EXP(a0*LN(x0))          (computed), w1 = (0.5 - w) + 0.5
   a0 == 1 and y0 <= 0.9   : w1 = EXP(b0*LN(fl(1 - x0)))   (computed), w  = (0.5 - w1) + 0.5
   a0 == 1 and y0 >  0.9   : no closed form (general kernel; Excel is near the exact value there)
   ```
   - This explains the old open cases: BETA.DIST(0.9,1,2.5|5,TRUE) and "T.DIST.RT at df 2 is a closed form down to
     y = 0.1".
   - It also shows why the F d1 = 2 cells looked like "no closed form": they need the swap and the passed complement.
   - In the a0 = 1 form, the 1 - x0 is recomputed inside the kernel. It is NOT the caller's y.
   - The cut lies between x0 = 0.0998999751 (open) and x0 = 0.09999999999999998 (closed).
3. **Argument rounding per function:**
   - F: x = 1/(1 + d2/(d1·X)), y = 1/(1 + (d1·X)/d2). This gets 197 of 197 b0 = 1 closed cells, against 180 for
     d1X/(d1X + d2). The complement is formed directly, not as 1 - x.
   - T: y = fl(n/fl(n + fl(t·t))). Beyond t = 1 the complement is fl(1 - y); in the centre, z = fl(fl(t·t)/fl(n + t·t)).
   - BINOM and NEGBINOM: x = p, y = fl(1 - p).
4. **Excel's log1p, read off BINOM.DIST(0,n,p).** BINOM.DIST(0,n,p) = EXP(fl(n·l)), bit-exact on 187 of 187 cells
   (11 p values × 17 n values). Here l = Excel's log1p(-p) for p <= 0.375, else LN(1-p).
   - l is correctly rounded at p = 0.1 and 0.2.
   - At p = 0.3 it is **one ulp below** the correctly rounded value: l = -0.3566749439387324. This is a direct
     readout for w1-density's finding 4 (Excel's log1p is not correctly rounded in [0.15, 0.375]).
5. **`ix_evidence.tsv`: 11,257 rows, 7,294 distinct (x, a, b) points.**
   - 6,081 rows are a directly computed kernel tail, 2,515 are derived complements, 1,162 are closed forms, 714 are
     intervals, 508 are uncertain, 172 do not go through the kernel and 105 are saturated.
   - In the direct, derived and closed rows there are 6,273 distinct points, mostly at shapes BETA.DIST data does
     not reach: a = 1/2 against b = n/2 up to 500, F half-integers, and integer BINOM shapes.

## 2. The mappings (verified)

### 2.1 Which tail the kernel computes (`f_pairs.py`, `f_pairs2.py`, `binom_negbinom.py`, `pairs_closed.py`)
`dispatch.py` is bratio's branch logic (R `toms708.c`) reduced to (branch, swap, computed tail). It is the plain TOMS
logic and was not tuned. Its tests: min(a,b) <= 1 swaps at x > 0.5, then fpser/apser, bpser, w1 = bpser(b0,a0,y0) at
x0 >= 0.29, bpser at x0 < 0.1 && (x0·b0)^a0 <= 0.7, bgrat at b0 > 15, otherwise bup + bgrat. For a, b > 1 it swaps at
lambda < 0, takes bpser at b0·x0 <= 0.7, L140 at b0 < 40, otherwise bfrac/basym.

| test | n | result |
|---|---|---|
| F.DIST + F.DIST.RT at one point: q = (0.5-p)+0.5 or p = (0.5-q)+0.5 | 1,578 | 1,578 (370 satisfy both) |
| computed tail = bratio's, F pairs where it is decidable | 1,208 | **1,208** (mean switch: 1,200) |
| the 8 pairs the mean switch gets wrong, e.g. (X,d1,d2) = (1,1,100) bgrat, (1.5,5,1) swapped bup+bgrat, (0.621,1,3) | 8 | 8 by bratio |
| NEGBINOM(f,r,p) vs BINOM(r-1,f+r,p), exact complements | 460 | 441 |
| computed tail = bratio's, BINOM/NEGBINOM pairs where it is decidable | 348 | **348** |
| computed tail at closed-form shapes (closed rule and bratio agree) | 89 | 89 |

### 2.2 Closed forms (`closed_final.py`, `closed_variants.py`, `closed_kernel.py`, `closed_b1.py`, `a1_threshold.py`)
Every variant was scored on the 1,286 cells where the final rule applies. "direct" and "reflected" are the README
branches (direct: x <= a/(a+b) on the kernel's x).

| variant | direct (886) | reflected (400) | hold-out (633) | all (1,286) |
|---|---|---|---|---|
| **oriented, LN(fl(1-x0)), (0.5-w1)+0.5, cut y0 <= 0.9** | **886/886/886** | **400/400/400** | **633/633/633** | **1,286/1,286/1,286** |
| same, cut x0 >= 0.1 (misses the 28 cells at x0 = fl(1-0.9)) | 873/873/873 | 385/385/385 | 620 | 1,258 |
| same, complement 1 - w1 | 881/886/886 | 397/400/400 | 630 | 1,278 |
| oriented, LN(y0) (the caller's complement) | 873/885/885 | 385/397/398 | 618 | 1,258 |
| unoriented (closed form on the caller's x, as BETA.DIST was thought to do) | 878/885/885 | 312/352/354 | 591 | 1,190 |

Per source (all / hold-out): BETA.DIST 133/133, F 367/367, T.DIST.RT df 2 508/508, BINOM 112/112 and NEGBINOM 138/138.
These are the `CUT=y0<=0.9` default of `closed_final.py`; the output is `closed_final.out`.

Below the cut (a0 = 1, x0 < 0.1) the closed form is not used. On the x0·b0 <= 0.7 cells the exact value scores
400/666/666 of 666, against 110/351/503 for the closed form. On the x0·b0 > 0.7 cells it is 30/51/53 of 57 against
23/42/51.

Positive and negative controls:
- The cut is pinned from both sides by T df 2. t = 4.24 (x0 = 0.100112) is closed; t = 4.245 (x0 = 0.0998999) is
  not (+1 ulp), and t = 4.25 is -6 ulp.
- The rule's alternatives fail on the same cells, as the table shows, so the rule is not an artefact of easy cells.

### 2.3 Argument forms
- **F** (`f_argform_closed.py`, `f_args*.py`, `f_w3model*.py`):
  - On the b0 = 1 closed cells, which depend on x0 alone, the forms score:

    | form | b0 = 1 closed cells |
    |---|---|
    | 1/(1 + d2/(d1X)) and 1/(1 + d1X/d2) | **197/197** |
    | d1X/(d1X + d2) | 180 |
    | 80-bit extended | 183 |
    | X/(X + d2/d1) | 188 |
    | 1 - (the other) | 75 |

    1/(1 + (d2/d1)/X) also scores 197. It differs from the chosen form on only 2 of the 566 closed cells and on 287
    of all 5,065 F cells, so the two are not separated.
  - With w3-series' bpser model, z64 variant, on the bpser cells:

    | form | no swap (344) | swapped (443) |
    |---|---|---|
    | 1/(1+q) | 179/341/342 | 197/436/436 |
    | d1X/(d1X+d2) | 160/327/332 | 191/431/432 |
    | 1/(1+r/X) | 174/332/333 | 196/436/436 |

  - The small-side argument is formed directly. On the w > 0.5 upper tails, the exact tail at v = d2/s has median
    |err| 2 ulp. At 1 - w the median is 12 ulp and the p90 is 9E4 ulp (`f_args.out`).
- **T** (`t_argform_closed.py`, `t_link.py`, `t_link2.py`):
  - At df 2, y = fl(2/fl(2 + t·t)) with complement fl(1 - y) is 474/474 (oriented a0 = 1) and 34/34 (oriented
    b0 = 1). With complement z = fl(t·t/fl(2 + t·t)) the b0 = 1 cells score 13/34.
  - At other df, beyond t = 1, T's complement is fl(1 - y) too. T.DIST.RT equals BETA.DIST(y,n/2,1/2)/2 in every
    branch that reads the complement: bgrat 19/19, swapped bpser-type 13/13.
  - The centre (t <= 1, df ≠ 2) is 0.5 - BETA.DIST(z, 1/2, n/2)/2 with z formed from t (19 of 23, statistics.md).
- **BINOM/NEGBINOM**: x = p, y = fl(1-p). BINOM is 1 - I_p(k+1, n-k). The a/b orientation is not separable, since
  bratio's dispatch is symmetric.

### 2.4 Wrapper identities (`t_pairs.py`)
| identity | n | bit-exact |
|---|---|---|
| T.DIST.2T(t) = 2·T.DIST.RT(t) | 123 | 123 |
| TDIST(t,n,1) = T.DIST.RT, TDIST(t,n,2) = 2·T.DIST.RT | 17 | 17 |
| T.DIST(-t,TRUE) = T.DIST.RT(t) | 30 | 30 |
| T.DIST(t>0,TRUE) = 1 - T.DIST.RT(t) | 113 | 113 |
| BINOM.DIST(k=n,FALSE) = EXP(n·LN(p)) | 172 | 172 |
| BINOM.DIST(0,n,p,·) = EXP(fl(n·l)), l = Excel log1p(-p) (p <= 0.375) / LN(1-p) | 187 (pinned l) | 187 |
| NEGBINOM.DIST(0,r,p,·) = EXP(r·LN(p)) | 101 | 101 |

The BINOM k = 0 cdf equals its pmf and uses log1p(-p), not the kernel's b0 = 1 form, so it is BINOM's own shortcut
(`not-kernel` in the evidence file). The general BINOM pmf is not modelled.

## 3. What the triangulation says about the kernel (for the other workers)
1. **T.DIST.RT and BETA.DIST agree to the bit except in bup + bgrat.** At the same (y, n/2, 1/2) the two are
   identical in bpser (84/84), w1 = bpser (41/41) and bgrat (36/36), but only 20/57 in bup + bgrat. Both read the
   same complement fl(1 - y), so the difference is in how the caller combines the tails in that branch.
   - On the 29 bup + bgrat cells with t > 1, none of E/2 (10), 0.5 - H/2 (7), 0.5 - I/2 (5), ((0.5-I)+0.5)/2 (5) or
     (1-I)/2 (5) explains them. Open.
   - Corollary: the 1-4 ulp T misses of statistics.md ("T.DIST.RT is BETA.DIST's incomplete beta") are all bup + bgrat.
2. **The general kernel at a = 1 or b = 1 is visible only through these functions.** BETA.DIST takes the closed
   forms there, except at a0 = 1 with y0 > 0.9. F at d1 = 2 or d2 = 2 with v < 0.1, T df 2 at y < 0.1 and BINOM at
   p < 0.1 give the general kernel at a or b = 1 exactly:
   - 360 T df-2 cells at y < 0.1, with exact 194/360/360, so Excel is near CR there;
   - 198 F direct or derived rows with a or b = 1 and `closed` = 0.
3. **Kernel noise per source** (direct rows, `check_evidence.out`): median |err| is 2 ulp for F, T and NEGBINOM, but
   6 ulp for BINOM's integer shapes (p99 202). The BINOM rows (a = k+1, b = n-k up to 100) are the strongest data
   for the a, b > 1 branches (L140, bfrac).
4. The w3-series bpser model scores lower on F's bpser cells (z64: 179/344 and 197/443) than on BETA.DIST. The
   misses concentrate at a = b = 1/2 (8/57) and at a = 1/2 with b = 50 (0/40) (`f_w3byshape.py`). These are the
   shapes to test the next bpser and ln B candidates on.

## 4. Negatives (all scored, scripts named)
- The closed forms applied to F's unoriented argument, as for BETA.DIST (`f_closed.py`): best 78/102/107 of 125
  (a = 1 cdf), 118/144/149 of 154 (a = 1 rt), 79/112/114 of 123 (b = 1 cdf). Superseded by the oriented rule.
- BINOM cdf as a running double sum of Excel's own pmfs: 501 of 1,371, and 549 with extended accumulation
  (`binom_sum.py`).
- BINOM cdf and pmf as sums of recursive terms from EXP(n·l), in five orders (`binom_rec.py`): cdf at most 504 of
  1,632, pmf at most 315 of 1,631.
- BINOM.DIST(0,n,p) as exact q^n: 36/106/134 of 171. As EXP(n·LN(1-p)): 128/159/168. With libm pow: 36
  (`binom_closed.py`).
- BINOM.DIST(n-1,n,p,TRUE) as 1 - EXP(n·LN p): 125; as exact: 136; as (0.5-P)+0.5: 139/170. The remaining misses
  are resolved by the oriented rule and bratio's general branches.
- F argument forms u/(1+u), q/(1+q) (u = d1X/d2): 179/416/425 of 443 on the swapped bpser cells and worse on the
  closed cells. The complement-based forms 1 - w and 1 - v are far off.
- T complement z beyond t = 1 (instead of fl(1 - y)): rejected by bgrat 19/19 and bpser-type 13/13 cells matching
  BETA.DIST(y)/2, and by df 2's b0 = 1 cells (13/34 against 34/34).
- mog (`/home/user/fundamental-research-labs/mog`, `compute-functions/src/statistical/distributions/{f_dist,
  binomial, negative_binomial}.rs`): all statrs. No hints.

## 5. `ix_evidence.tsv`
Built by `build_evidence.py`; checked by `check_evidence.py` (`check_evidence.out`). Columns:
- `source_fn`, `case`, `cell`.
- `x`, `a`, `b`: the kernel's first argument and shapes, as the caller passes them, not oriented.
- `side`: lower means `excel_ix` is Excel's I_x(a,b); upper means it is Excel's 1 - I_x(a,b).
- `excel_ix`: the implied value of that tail. It is exact where the wrapper is exact (F, BINOM and NEGBINOM as is,
  T = 2L). For `interval` rows it is a midpoint, with `ix_lo` and `ix_hi`.
- `confidence`:
  - `direct`: the kernel computes this tail directly (section 2.1).
  - `derived`: (0.5 - w) + 0.5 of the computed tail.
  - `closed`: a closed form (section 2.2).
  - `interval`: T centre form or T.DIST(t > 0); I_z = 1 - 2L with ±ulp(L) bounds.
  - `uncertain`: T at y > 0.99 beyond t = 1, or df 2 at y < 0.1, whose complement is unverified.
  - `not-kernel`: BINOM k = 0.
  - `saturated`: an Excel value of 0 or 1.
- Extra columns:
  - `y`: the complement passed to the kernel.
  - `excel`: the cell's own value.
  - `composition`: the formula used.
  - `branch`: bratio's branch, or `closed:a0=1` / `closed:b0=1`.
  - `computed_tail`.
  - `closed`.
  - `exact`: the 40-digit tail, computed without cancellation. The oriented small argument is taken as given and
    the other side is its exact complement; empty for max(a,b) > 2000.
  - `err_ulp`, `ix_lo`, `ix_hi`.

The reference-argument BETA.DIST cells of t0969-t-beta-link (552 cells, columns E, H, I) are included as
`BETA.DIST(ref)`; cells.tsv lacks them. The density functions (T.DIST FALSE: 19,793 cells; F.DIST FALSE; the BINOM
and NEGBINOM pmfs) are not I_x evidence and are left out. statistics.md already records that the t density is not
BETA.DIST's density (at most 20 of 184).

Rows by source and confidence (`build_evidence.out`):

| source | direct | derived | closed | interval | uncertain | other |
|---|---|---|---|---|---|---|
| F.DIST | 620 | 764 | 161 | | | 56 saturated |
| F.DIST.RT | 865 | 604 | 177 | | | 42 saturated |
| FDIST | 71 | 23 | 19 | | | 7 saturated |
| T.DIST.RT | 3,441 | 4 | 508 | 573 | 484 | |
| T.DIST (TRUE) | 15 | | 2 | 98 | 10 | |
| T.DIST.2T, TDIST | 71 | | 7 | 43 | 14 | |
| BINOM.DIST, BINOMDIST | 598 | 577 | 128 | | | 172 not-kernel |
| NEGBINOM.DIST | 200 | 209 | 142 | | | |
| BETA.DIST(ref) | 200 | 334 | 18 | | | |

## 6. Next hypotheses (most promising first)
1. **Use BINOM and NEGBINOM direct rows for the a, b > 1 branches.** That is 798 direct rows with integer shapes
   (L140 bup + bpser, bfrac), where Excel's noise is largest (median 6 ulp). The integer b gives L140's n = b - 1
   with b0' = 1, a bup with b0 = 1, which is cleaner than BETA.DIST's fractional b.
2. **T's bup + bgrat combination.** Probe T.DIST.RT next to BETA.DIST(y), BETA.DIST(1-y,1/2,n/2) and
   BETA.DIST(z,1/2,n/2) on a run of consecutive t at df 7 to 30, t in [1.2, 3]. Identify what T does differently
   (37 of 57 cells differ by 1-4 ulp).
3. **Pin the a0 = 1 cut** (y0 <= 0.9 or x0 >= fl(1-0.9)) with a ±k-ulp run around x0 = 0.1 in BETA.DIST(x, 1, b) and
   BETA.DIST(x, b, 1) at x = 0.9 ± k ulp. Also probe b0 = 1 at tiny x0 (no lower cut is seen there).
4. **Pin Excel's log1p on (0, 0.375]** with BINOM.DIST(0, n, p) for many p, at 3 to 4 values of n per p. Each p gives
   one log1p(-p) double exactly (`binom_log1p.py` method). This feeds w1's density (ln y) and brcomp.
5. **F 1/(1 + d2/(d1X)) against 1/(1 + (d2/d1)/X)**: a probe at d1 = 3, 5, 7 and d2 = 2 (b0 = 1 closed), where the
   two differ.

## Scripts and outputs
| file | what |
|---|---|
| `w5lib.py` | cells, exact I (cached), Excel EXP/LN (cached), scoring, hold-out |
| `dispatch.py` | bratio dispatch: branch, swap, computed tail |
| `f_pairs.py`, `f_pairs2.py` (`.out`) | F one-evaluation and the computed-tail test |
| `binom_negbinom.py` | BINOM vs NEGBINOM complements and the computed tail |
| `pairs_closed.py` | computed tail at closed shapes |
| `closed_final.py`, `closed_variants.py` (`.out`) | the closed-form rule and its variants, hold-out |
| `closed_kernel.py`, `closed_b1.py`, `closed_miss.py`, `a1_threshold.py`, `f_a1.py`, `f_a0miss.py` | how the rule was found |
| `f_closed.py`, `f_closed_miss.py` | the unoriented closed forms (negative) |
| `f_args.py` to `f_args4.py`, `f_argform_closed.py`, `f_w3model*.py`, `f_w3byshape.py` | F argument forms |
| `t_link.py`, `t_link2.py`, `t_pairs.py`, `t_argform_closed.py` | T mappings |
| `binom_closed.py`, `binom_miss.py`, `binom_log1p.py`, `binom_sum.py`, `binom_rec.py` | BINOM forms, the log1p readout, the sum negatives |
| `build_evidence.py`, `check_evidence.py` | `ix_evidence.tsv` and its checks |
| `w3copy/` | a vendored copy of w3-series' model (lib/model/series) so that caches stay here |

Caches (`*.pkl`) are git-ignored and rebuild automatically.
