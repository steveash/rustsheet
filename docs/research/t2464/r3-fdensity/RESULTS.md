# T-2513 r3-fdensity: Excel's F density F.DIST(x, d1, d2, FALSE)

Worker r3-fdensity, round 3, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53: relative in the density, absolute in its logarithm. "D" is
ln(Excel) - ln(exact F at the cell's double x), in units. CR is the correctly rounded exact density. a = d1/2,
b = d2/2, h = a + b, m = d1, n = d2, mx = fl(m·x). Run every script from this directory with
`/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
**Excel's F density is not its BETA.DIST density times a Jacobian. It is not R's `df` (Loader `dbinom_raw`), and it is
not a GAMMALN formula.** Three structural facts are established, the third on an untouched hold-out:
1. **The shape constant is accurate and symmetric.** At x = 1, D has sd 0.6-0.9 units on all ten df sweeps (d1 or
   d2 in {1, 2, 3, 4, 7}, the other df 1..120). Its correlation with Excel's GAMMALN errors is |r| <= 0.11, although
   those errors have sd 114-132 units. `F.DIST(1,m,n,FALSE)` equals `F.DIST(1,n,m,FALSE)` bit for bit on all 585
   swapped pairs (`b1_sweep.py`, `b1_sweep.out`).
2. **F is not the worksheet beta density at the transformed point.** On probe a, BETA.DIST was evaluated beside each
   F.DIST cell at y = 1/(1+d2/(d1 x)) (w5's F cdf argument) and at y = d1 x/(d1 x+d2) (the engine's argument).
   Within the 66 runs, Excel's F error is uncorrelated with either beta error (|corr| <= 0.49, mostly < 0.3). F's
   noise is also much larger: sd 105 against 57 at (50, 100) and x ≈ 13 (`a8_pair.py`, `a8_pair.out`). The engine
   computes the F density as beta_density × y(1-y)/x. Excel does not.
   - The same holds for d1 = 1 on r3-tdensity's paired columns. F's constant offset is about +3 to +5 units for
     every df 3..100. BETA.DIST(z, 1/2, n/2) carries M1's lnB errors instead (-18 units at n = 15); F does not
     (`a1_tdpair.py`).
3. **The x-dependence goes through ln(1 + r), with r the smaller of mx/n and n/mx and Excel's log1p switch at
   0.375.** D responds to the rounding of the stored s = fl(1 + r) with slope -h:
   - s = fl(1 + mx/n) when mx < n;
   - s = fl(1 + n/mx) when mx > n;
   - only when r > 0.375. Below the cut (the atanh series), the slope is 0.

   On the hold-out probe, 12 of 12 runs with 0.375 < r <= 1 show slopes -0.67h to -2.13h (9 of them -0.94h to
   -1.22h) on the predicted s, and about 0 on the other s. All 24 runs with r <= 0.375 show |slope| <= 0.61h (the largest at (6, 6), r = 0.370, just under the cut)
   (`b6_switch.py`, `b6_insample.out`, `b6_hold.out`). In terms of the beta variable, the switch is at y = 1/2, the
   bratio-style swap.
- **The best model scored ("ratio", `c1_model.py`) is only partly right.**
  - Its form:
    - mx <= n: lnF = K + a ln(m/n) + (a-1) ln x - h·L(fl(mx/n));
    - mx > n: lnF = K - b ln(m/n) - (b+1) ln x - h·L(fl(n/mx));
    - L = Excel's log1p (series to 0.375, LN(fl(1+r)) above);
    - K = exact lnΓ(h) - lnΓ(a) - lnΓ(b).
  - Its gain over CR is small, and largest within 4 ulp. Large-shape noise (sd 20-200 units in runs with max(d) >= 50)
    remains unexplained. Bit-exactness is not reached anywhere.
- Visible parity is not reached. On the hold-out the frozen model is within 15 digits on 1,091 of 1,440 cells
  (CR 1,060).

## 2. Probes, controls, housekeeping
- **Probes** (probes-numeric, excel-graph, all four built by `mkprobe.py` and filled on 2026-10-24 before any analysis;
  rows rebuilt from the generator, no meta files, Excel values from the sidecars):

  | case | cells | content |
  |---|---|---|
  | `t2513-r3-fdensity-a` | 12,672 | in-sample: 22 shapes × 3 runs of 48 consecutive doubles at x ≈ 0.37, 1.9, 13. A = x (exact-piece formula), B = F.DIST, C = BETA.DIST(1/(1+d2/(d1·A)), d1/2, d2/2, FALSE), D = BETA.DIST(d1·A/(d1·A+d2), ...) |
  | `t2513-r3-fdensity-sw` | 2,400 | in-sample: d1 in {1,2,3,4,7} × d2 = 1..120 and the mirror, at x = 1 and 2.75 |
  | `t2513-r3-fdensity-hold` | 2,880 | **hold-out**: 12 other shapes ((1,7), (2,25), (3,12), (6,6), (7,2), (12,40), (25,8), (40,60), (80,30), (3,1), (15,15), (60,200)) × 3 runs of 40, columns A and B |
  | `t2513-r3-fdensity-holdsw` | 400 | **hold-out**: d1 in {5, 9} × d2 = 1..100 and d2 in {6, 9} × d1 = 1..100, at x = 1.6 |

  Total 18,352 cells.
- **Read-back.** Every run x matches its echo in column A: `mkprobe.rows` asserts it, so there are 0 parser misreads.
  The sweep x values are short decimals.
- **Hold-out discipline.** Both hold-out cases were filled with the in-sample ones. They were read once, after the
  ratio model's variants were frozen (`c1_hold.out`, `b6_hold.out`).
- **Positive controls.**
  - `a9_enum.py --control C`: synthetic values from a known log-form chain. The enumeration recovers the chain on
    3,168 of 3,168 cells. A wrong chain is consistent on 538 (17%), which is the baseline for "wrong" in section 4
    (`a9_control.out`).
  - The free-constant fit of R df (`flib.fit_df_block`) reuses r3-tdensity's controlled `interval` and r3-loader-bd0's
    `best_overlap`.
- **Reuse, by import only:**
  - r3-loader-bd0 `r3lib` (x87 helpers, Excel LN, EXP V6 on an extended argument, bd0 with knobs, overlaps);
  - r3-tdensity `tdlib` (interval inversion, Excel log1p);
  - r3-loader-const `r3lib.e_interval_ext` (loaded under its own name by importlib);
  - r2-density-large `dlib` (scorer, log1p);
  - w2-forensics `make_probes` (encoder).

  GAMMALN values come from the harness through `excelmath` and are cached in this directory's `cache.pkl`, which is
  git-ignored with `__pycache__/`.

## 3. Data
| set | cells | CR | note |
|---|---|---|---|
| cells.tsv F.DIST(.,FALSE) | 1,599 (+4 non-positive) | 152/1,077/1,308 | d1 = 1: 21/269/327 of 407, d1 = 2: 35/120/126 of 133, d2 <= 2: 37/286/336 of 403, both > 2: 59/402/519 of 656 (`a0_survey.py`) |
| r3-tdensity a1+a2 column G (d1 = 1) | 2,406 | 140/1,994/2,222 | x = Excel's t·t (column C) |
| probe a | 3,168 | 337/1,958/2,638 | |
| probe sw | 2,400 | 585/2,167/2,227 | |
| hold | 1,440 | 114/733/1,060 | |
| holdsw | 400 | 37/311/395 | |

Excel's noise within the runs grows with the shape:
- d1 = 1 or 2 near the mode: sd 0.7-1.3 units;
- d2 = 10 and x ≈ 13: sd 11;
- (20, 100), x ≈ 13: sd 112;
- (60, 200), x ≈ 13: sd 203 (hold-out).

The noise sits on a lattice:
- (100, 100), x ≈ 0.37: steps of about 62 units;
- (50, 50), x ≈ 0.37: steps of about 31;
- (2, 10), x ≈ 13: steps of about 16.

So a stored term of magnitude [32, 64), [16, 32) and [8, 16) respectively is involved.

## 4. Hypotheses and counts (negatives included, decision 0056)
### 4.1 F = Excel's beta density at a transformed point × Jacobian (rejected)
- Paired columns (`a8_pair.py`): the correlations of eF with eW and eD are -0.49..+0.43 over 66 runs, and F's sd is
  larger than either beta's. On r3-tdensity's d1 = 1 data, F/(H·J) has sd 1.3-14.6 units, and F = fl(H·J_exact) holds
  on 0-29 of 96 cells per run (`a1_tdpair.out`).
- Loader beta density at (y, q) with q formed separately (w5's y, q), times y·q/x in value space, with a free
  constant per run (`b2_bj.py`): consistent on 315 of 2,016 cells (16%, the wrong-structure baseline).

### 4.2 R's df through Loader's dbinom_raw (rejected)
f0 = 1/(n+x·m), q = n·f0, p = x·m·f0, then f = m·q/2 × dbinom_raw((m-2)/2, (m+n-2)/2, p, q) (m >= 2) or
m²q/(2p(m+n)) × dbinom_raw(m/2, (m+n)/2, p, q). x87 stored ops, r3-loader-bd0's bd0, E extended, exact constant
(`flib.df_value`).
- Free constant per run, largest consistent set (`a2_struct.py`):

  | variant | r3-tdensity d1 = 1 | probe a (min(d) > 2 and d1 = 1) |
  |---|---|---|
  | R | 653 of 2,112 | 600 of 2,736 |
  | R, r2 bd0 | | 584 |
  | div p, q | | 620 |
  | w5 p, q | | 599 |
  | q = 1 - p | | 490 |

- Exact constant, whole-model scores (`a3_score.out`): R 293/2,962/3,451 of 4,005 against CR 292/3,071/3,530.
- The R model adds noise that Excel does not have. The within-run sd of (model - Excel) is 2.11 against CR's 1.51
  (median over 24 d1 = 1 runs, `a7_tdens.out`).
- d1 = 2 is dbinom_raw's x = 0 branch, F = q·q^(n/2). On 60 closed-form variants (q forms R/div/w5; n/2·LN q,
  (n/2+1)·LN q, log1p, R's bd0 branch; stores) the best is 32/117/123 of 133 cells, against CR's 35/120/126
  (`a5_d1two.out`).
- Argument-rounding response (`b3_tie.py`): slopes of D on the rounding of R df's p and q. They are not (a-1, b+1) or
  (a, b), and they leave most of the noise.
- The engine's `f_pdf` d <= 2 path (beta_density × y(1-y)/x) is covered by 4.1: it is not Excel's.

### 4.3 Direct log-gamma / log forms (rejected)
- 40 compositions (`a4_gl.py`) combine:
  - lnG = GAMMALN(h) - GAMMALN(a) - GAMMALN(b) in four orders, or exact;
  - the x part A-E: a ln(m/n) + (a-1) ln x - h ln(1+mx/n) | a ln m + b ln n + (a-1) ln x - h ln(n+mx) |
    a ln y + b ln q - ln x | a ln(mx) + b ln n - h ln(mx+n) - ln x | a ln(mx/n) - h ln(1+mx/n) - ln x;
  - stored or extended sums.

  All are worse than CR. The best is C-exact-e at 114/832/1,133 of 1,599 cells.tsv cells, and the median within-shape
  sd is 11-35 units against CR's 8.3. The GAMMALN forms are also refuted by fact 1.
- Free-constant structure enumeration (`a9_enum.py`): every term order, C first or last, sums stored or extended, E
  stored or extended. Best per family of 3,168 cells (`a9_enum.out`):

  | family | best |
  |---|---|
  | A | 564 |
  | A with log1p | 564 |
  | B | 408 |
  | C | 658 |
  | Cw (w5's y, q) | 687 |
  | Cl (TOMS brcomp switch) | 733 |
  | D | 424 |
  | E | 604 |
  | S1 | 538 |
  | J (beta exponents + log Jacobian) | 702 |

  All of these are at the wrong-structure baseline.

### 4.4 Structural regressions (what Excel does)
- Stored-term regression (`a6_reg.py`): stored products a·LN(·), b·LN(·), (a-1)·LN(x), h·LN(n+mx) and so on all get
  coefficients of about 0. So Excel does not store those products.
  - The exception is h·L(r) with r the smaller ratio and L = Excel's log1p. It gets +0.7 to +1.2 on most probe-a runs
    with min(d) > 2.
  - (a-1)·LN(x) gets about 0, even though its magnitude matches the lattice. The x-power term is stored in some other
    form, not identified.
- Local-rounding regression (`b4_local.py`) on every intermediate of the three argument forms: the only consistent
  signal is s1 = fl(1 + n/mx) or s2 = fl(1 + mx/n), with slope about -h. `b6_switch.py` (section 1, fact 3) isolates
  it.
- Semi-exact check (`b5_semi.py`): the exact F times the s-rounding factor on its own does not lower the median run
  sd overall (3.86 → 5.71 with the smaller-r rule, 7.68 / 6.66 for always s1 / s2). This is because the rule must
  also switch to the series below 0.375 (c1's L), and because the large-shape noise dominates those runs.

### 4.5 The ratio model (`c1_model.py`, frozen before the hold-out was read)
The ratio model with variants:
- L: xl = Excel log1p, ln = always LN(fl(1+r)), m = exact log1p of the stored r;
- ln x term: x = stored product of Excel LN, m = exact;
- h·L: stored or extended.

| variant | probe a (3,168) | sw (2,400) | cells.tsv (1,599) | **hold (1,440)** | **holdsw (400)** |
|---|---|---|---|---|---|
| CR | 337/1,958/2,638 | 585/2,167/2,227 | 152/1,077/1,308 | 114/733/1,060 | 37/311/395 |
| ratio-xl | 382/2,081/2,661 | 396/2,192/2,293 | 156/1,055/1,290 | 145/781/1,084 | 62/366/399 |
| **ratio-xl-e** (h·L unstored) | 398/**2,185**/2,693 | 421/**2,224**/**2,310** | 172/1,052/1,297 | **161**/776/**1,091** | **71/373/400** |
| ratio-xl-lxm (exact ln x) | 388/2,178/**2,719** | 385/2,195/2,260 | **175/1,099/1,323** | 134/775/1,075 | 65/374/400 |
| ratio-m | 321/1,962/2,649 | 483/2,170/2,234 | 149/1,070/1,305 | 110/751/1,067 | 48/326/391 |
| ratio-ln (no series) | 252/1,648/2,325 | 124/757/1,026 | 110/809/1,134 | 97/709/990 | 30/185/265 |

- On the hold-out, the log1p switch (xl against ln) is confirmed: 1,091 against 990 w15.
- The gain over CR is real but small: +31 w15 and +43 within 4 ulp on `hold`, +5 and +62 on `holdsw`.
- By class on `hold`:

  | class | ratio-xl-e | CR |
  |---|---|---|
  | max(d) >= 50 | 117 of 360 | 100 |
  | min(d) <= 2 | 435 of 480 | 436 |
  | other | 539 of 600 | 524 |

  So the d <= 2 cells and the large shapes are unsolved.
- `rpow` variants, with the x power as fl(c·LN(r)) on the ratio (c = a-1 or b+1): 321-325/1,893-1,958/2,582-2,610 on
  probe a. That is worse than CR, so they are rejected.

## 5. Open (most promising first)
1. **The large-shape noise.** It is sd 20-200 units in runs with max(d) >= 50, and it sits on a lattice that matches a
   stored [32, 64) term at (100, 100) and x ≈ 0.37. The term (a-1)·ln x has the right magnitude, but no stored-product
   form of it fits.
   - Next: run b4-style local regressions on candidate forms of the power term:
     - POWER(x, a-1) or x^(a-1) in value space;
     - x87 FYL2X/F2XM1 pow with the product stored in log2 units;
     - (a-1)·log1p(x-1);
     - the power taken on y, q or r instead of x.
   - A probe with exact x = 2^k·(1 + j·2^-30) runs would make LN(x) exact-ish and isolate the power term.
2. **The +3 to +5 unit offset at d1 = 1** across df 3..100 (r3-tdensity data and sw). It looks like one fixed
   constant, for example sqrt(pi) or a stored 1/B(1/2, b) piece. Fit it against the constants of M1/TOMS (gam1,
   algdiv, bcorr).
3. **d <= 2 cells.** The ratio model gives no gain there (min(d) <= 2: 1,079-1,100 of 1,152 on probe a, CR 1,099).
4. **df truncation.** This is not tested here: cells.tsv has no non-integer df pdf cells, and every probe uses integer
   df. A small probe at d = 2.9, 3.0001 and 5.5 is needed, compared with statistics.md's FDIST truncation entry.
5. **The constant's form.** K is accurate (fact 1). TOMS brcomp's Stirling/bcorr constant and Loader's stirlerr are both
   candidates. Probe x = 1 at larger shapes (min(d) >= 16) to separate them.

## 6. Scripts
| script | what |
|---|---|
| `flib.py` | data loaders (cells.tsv, r3-tdensity's column G, sidecars), exact F and beta, scorer, R df model (`df_value`, `fit_df_block`), (y, q) forms and Jacobians, GAMMALN cache |
| `mkprobe.py` | the four probes (`--write`) and `rows(name)` |
| `a0_survey.py` | CR by class |
| `a1_tdpair.py` | d1 = 1 F against BETA.DIST(z) on r3-tdensity data |
| `a2_struct.py` | R df free-constant structure test (variants) |
| `a3_score.py` | R df whole-model scores |
| `a4_gl.py` | 40 GAMMALN/log compositions |
| `a5_d1two.py` | d1 = 2 closed-form variants |
| `a6_reg.py` | stored-term regression |
| `a7_sd.py` | within-run sd of R df variants against CR |
| `a8_pair.py` | F against BETA.DIST at both transformed points (probe a) |
| `a9_enum.py` | log-form structure enumeration with free C (`--control`) |
| `b1_sweep.py` | x = 1 / 2.75 df sweeps against GAMMALN errors |
| `b2_bj.py` | Loader beta at (y, q) × Jacobian, free C |
| `b3_tie.py` | response to p, q rounding per argument form |
| `b4_local.py` | local-rounding regression on every intermediate |
| `b5_semi.py` | semi-exact s-rounding models |
| `b6_switch.py` | slopes on s1 / s2 per run (the switch rule) |
| `c1_model.py` | the ratio model and its variants (frozen) |

Outputs: `*.out` files of the same names. `c1_insample.out` and `c1_hold.out` hold the table in section 4.5.
`b6_insample.out` and `b6_hold.out` hold fact 3.
