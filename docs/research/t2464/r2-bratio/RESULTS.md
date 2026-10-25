# T-2464 r2-bratio: one end-to-end TOMS 708 `bratio` model of Excel's I_x(a, b)

Worker r2-bratio, round 2, 2026-10-23. Method: a complete port of TOMS 708 `bratio` and every subroutine it calls,
with each round-1 Excel finding behind a switch, scored per bratio branch on every cdf data set. The deliverable is
`bratio_excel.py`, written to be followed line by line by an engine port.

Scores are bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15). "Hold-out" is the
md5-odd half (`score_lib._hold`). CR is the correctly rounded value. Run everything with
`/home/user/rustsheet/bindings/python/.venv/bin/python` from this directory.

## 1. Headline

- **Integrated model (BEST): 22,731 / 35,884 / 36,944 of 40,014 cdf cells.** CR scores 11,812 / 33,858 / 36,618.
  On the hold-out half BEST is bit-exact on 11,393 of 20,064 cells, against 5,920 for CR.
  - This is the first single model that beats CR on all three measures at once.
  - It is still short of visible parity on 3,070 cells. CR misses 3,396.
- **Positive control:** the port in R configuration reproduces R's compiled `toms708.c` bit for bit on
  **36,431 of 36,433** distinct points (`ctl_r.out`). The port's density reproduces w1's M1 exactly: 685 and 10,301
  (`ctl_density.out`).
- **New: Excel uses TOMS 1992's `x0 >= 0.3`, not R's 0.29 (PR#13786).** On new probe cells in [0.29, 0.3) where
  the two give different doubles, Excel equals the 0.3 dispatch on 284 of 432 and the 0.29 dispatch on 85. Both
  halves agree: 137 vs 41 and 147 vs 44 (`x29_test.out`). So Excel ported the original code, not R's.
- **New: w4's probe w4p1 was built at the wrong shape.** The formula parser cuts literals to 15 significant digits,
  so `BETA.DIST(.., 0.5999999999999996, ..)` was evaluated at 0.6.
  - That explains w4's "worksheet density is not bup's density" at (0.6, 7.3): inside the kernel b0' is
    11.6 - 11 = 0.5999999999999996.
  - Even at 0.6, the (7.3, 0.6) block is unexplained, see section 4.
- **New: `brcmp1` = Excel density times x times y (w4) holds out of sample.** On F, T, BINOM and NEGBINOM L140 cells
  (ix, which w4 never used), it gets 1,011 / 1,542 / 1,585 of 1,620. TOMS's own `brcmp1` gets 639 and CR gets 705.
- **New: w3's bpser takes an extended EXP argument (z64).** z64 beats z53 out of sample: beta_exact, ix and w4p1
  bpser cells, 405 / 3,196 / 1,625 against 364 / 2,774 / 3. Only w2's runs, which w3 tuned on, prefer z53
  (12,009 against 10,941).

## 2. The module and its switches

`bratio_excel.py`: `kernel(a, b, x, y, cfg)` returns (w, w1, branch). The `Cfg` fields are listed in the module
docstring. BEST is `Cfg()`:

```
elem=excel  gamln=excel  x29=0.3  closed=True  bpser=w3  w3_z=z64  w3_lg=toms  brcmp1=pdf  brcomp=pdf  bup_arith=w4
bgrat=orig  bup_break=True  algdiv=toms  pdf_lnb=w1
```

- **Elementary functions:** Excel's EXP (V6, w1's mpmath model; it also accepts an extended input), Excel's LN, and
  Excel's GAMMALN.
  - GAMMALN is the oracle value at the 5,094 kernel arguments of the new probe, else the engine model.
  - `ctl_density.out`: the EXP and LN models equal the harness on 3,000 / 3,000 inputs each.
- **x87 stored ops:** `sx()`, numpy longdouble.
- **The whole TOMS chain is ported:** `bratio` dispatch, `fpser`, `apser`, `bpser`, `bup`, `bfrac`, `bgrat`,
  `grat1` (plus R's `grat_r`), `basym`, `brcomp`/`brcmp1`, `betaln`, `gsumln`, `algdiv`, `bcorr`, `gamln`, `gamln1`,
  `gam1`, `alnrel`, `rlog1`, `rexpm1`, `esum`, `exparg`, `erf`, `erfc1`.
  - It follows the 1992 Fortran semantics, with `bgrat="r"` for R's variants: log_r bgrat, its grat_r z test,
    algdiv's t = 1/(b·b), and bfrac's w and beta association.
- **Excel pieces:**
  - `density()` is w1's M1, with M1's lnB in `lnb_m1`.
  - `pdf_xy` is w4's `brcmp1`/`brcomp`.
  - `bpser_w3` and `lnb_w3` are w3's series and its lnB.
  - `_bup_term` is w4's W arithmetic.
  - `closed()` is w5's closed forms.
- **Stand-ins:**
  - log1p(-x) in the density is correctly rounded.
  - The density for min(a, b) > 1 is correctly rounded.
  - apser's psi is mpmath (apser needs a0 < 1e-15).
  - w3's lnB for max > 8 is the accurate value.

## 3. Data (`score_lib.py`)

| set | cells | source |
|---|---|---|
| be | 993 | `../beta_exact.tsv` cdf cells via `w4-fractions/data.tsv` (50-digit exact) |
| w2f | 21,066 | w2 probes xruns-1/-2, absweep (data.tsv) and unit-shape (`cells_w2f.tsv`, exact computed here) |
| w4p1 | 5,118 | w4 probe column C, BETA.DIST(x, a, b0', TRUE). b0' is taken at the 15-digit literal value Excel saw |
| ix | 9,757 | `w5-triangulation/ix_evidence.tsv`, confidence direct/derived/closed (F, T, BINOM, NEGBINOM, BETA.DIST(ref)) |
| x29 | 3,080 | new probe t2464-r2-bratio-x29 |

BETA.DIST rows use y = fl(1-x). ix rows use w5's mapped x and y and the tail named by `side`.

## 4. Results per bratio branch (`report_best.out`; the full ladder is in `report_ladder.out`)

| branch (as fired in BEST) | n | R_LIKE | TOMS_XL | **BEST** | CR | BEST hold-out ex | CR hold-out ex |
|---|---|---|---|---|---|---|---|
| closed:b0=1 | 1,528 | 395/1508/1516 | 440/1508/1516 | **1528/1528/1528** | 478/1506/1515 | 770/770 | 222/770 |
| closed:a0=1 | 815 | 220/710/742 | 220/705/743 | **815/815/815** | 300/740/749 | 402/402 | 158/402 |
| w_bpser (min ≤ 1) | 11,845 | 1794/9920/10179 | 1799/9912/10177 | **6767/10777/10808** | 2580/9696/10319 | 3415/5965 | 1319/5965 |
| w1_bpser | 1,845 | 1016/1653/1756 | 1016/1653/1756 | **1666/1841/1842** | 788/1727/1781 | 824/909 | 400/909 |
| bpser(a,b>1) | 1,523 | 682/1113/1270 | 687/1108/1277 | **978/1340/1439** | 533/1027/1218 | 491/781 | 268/781 |
| bup+bgrat (min ≤ 1) | 3,592 | 1338/3009/3215 | 1659/3146/3350 | **2096/3469/3570** | 1500/3441/3546 | 1027/1756 | 745/1756 |
| bgrat (min ≤ 1, b0 > 15) | 2,451 | 1029/1801/2161 | 989/1761/2156 | 989/1761/2156 | **902/2073/2241** | 503/1225 | 452/1225 |
| L140 bup+bpser | 11,742 | 3467/9699/11298 | 3462/9699/11298 | **7505/11673/11713** | 3956/10379/11573 | 3756/5853 | 1956/5853 |
| L140 bup+bgrat | 3,301 | 432/2031/2259 | 444/2034/2261 | 212/2151/2327 | **628/2725/2914** | 121/1718 | 326/1718 |
| L140 bup+bup+bgrat | 55 | 16/51/54 | 16/51/54 | 27/51/54 | 19/53/55 | 13/24 | 8/24 |
| bfrac | 1,315 | 123/396/592 | 119/413/589 | 146/476/690 | 128/489/705 | 70/660 | 66/660 |
| basym | 2 | 2/2/2 | 2/2/2 | 2/2/2 | 0/2/2 | 1/1 | 0/1 |
| **all** | **40,014** | 10514/31893/35044 | 10853/31992/35179 | **22731/35884/36944** | 11812/33858/36618 | **11393/20064** | 5920/20064 |

- R_LIKE is plain TOMS with glibc, which is R.
- TOMS_XL is plain TOMS (0.3) with Excel's EXP, LN and GAMMALN substituted, nothing else.

Per data set (README branch; direct means x <= a/(a+b)):

| set | n | BEST | CR |
|---|---|---|---|
| beta_exact, direct cdf | 601 | **276/514/564** | 96/368/471 |
| beta_exact, reflected cdf | 392 | **366/390/391** | 314/391/391 |
| ix (F, T, BINOM, NEGBINOM) | 9,757 | **4989/8649/9302** | 3192/8156/8810 |
| w2f, direct | 15,651 | **6830/13741**/14097 | 2937/12859/**14389** |
| w2f, reflected | 5,415 | **4653/5415/5415** | 2586/5315/5315 |
| w4p1 | 5,118 | **2760/4095**/4095 | 679/3689/**4162** |
| x29 | 3,080 | **2857/3080/3080** | 2008/3080/3080 |

Attribution ladder, all 40,014 cells (`report_ladder.out`; each step adds one switch):

| step | exact/w4/w15 | hold-out exact |
|---|---|---|
| R_LIKE | 10,514/31,893/35,044 | 5,254 |
| TOMS_XL (Excel EXP/LN/GAMMALN, 0.3) | 10,853/31,992/35,179 | 5,415 |
| + w5 closed forms | 12,536/32,122/35,263 | 6,263 |
| + w3 bpser (z53, accurate lnΓ(v<1)) | 17,130/33,008/35,875 | 8,592 |
| + z64 | 18,046/33,411/36,137 | 9,041 |
| + w3_lg=toms (lnΓ(v<1) = gamln1(v) - LN v, extended) | 18,513/33,414/36,140 | 9,267 |
| + brcmp1 = Excel pdf·x·y (w4) | 22,585/35,816/36,843 | 11,307 |
| + w4 W arithmetic in bup | 22,704/35,821/36,843 | 11,377 |
| + brcomp = Excel pdf·x·y in bfrac | **22,731/35,884/36,944** | **11,393** (= BEST) |
| BEST with x29 = 0.29 instead | 22,533/35,882/36,939 | 11,298 |

Branches not at visible parity in BEST (`report_best.out`; cells failing w15):

| branch | n | BEST fails w15 | CR fails w15 | where |
|---|---|---|---|---|
| w_bpser | 11,845 | 1,037 | 1,526 | 1,023 are w4p1 (7.3, 0.6), see below; 14 elsewhere |
| L140 bup+bgrat | 3,301 | 974 | 387 | w2f (30.7, 1.7, x 0.81) a-sweep 356, b-sweep 428, run 181 |
| bfrac | 1,315 | 625 | 610 | 586 in the (100, 120) run (w4: the pdf for a, b > 2 is unsolved); 6 t1745 (100, 100) |
| bgrat | 2,451 | 295 | 210 | T.DIST.RT at (n/2, 1/2), n/2 = 25..500: 249 |
| bpser(a,b>1) | 1,523 | 84 | 305 | BINOM/F, 3 cells per shape, spread |
| L140 bup+bpser | 11,742 | 29 | 169 | |
| bup+bgrat | 3,592 | 22 | 46 | |
| w1_bpser | 1,845 | 3 | 64 | |
| L140 bup+bup+bgrat | 55 | 1 | 0 | |
| closed forms, basym | 2,345 | 0 | 79 | |

## 5. Findings, with evidence

1. **The port is right.** `ctl_r.py` compares R_LIKE against w2's compiled R `bratio_sse`. The result is
   36,431/36,433 bit-identical in both w and w1. The two misses are 1-ulp bgrat w1 tails at (100, 0.5) and
   (500, 0.5).
   - Reaching that took three corrections of the first draft, each a real TOMS/R difference: TOMS's own
     erf/erfc1, R's M_SQRT_PI constant (sqrt(pi) rounds differently), and R's bfrac association.
   - R_LIKE against CR has a |ulp| histogram of 10,822 at 0, 11,227 at 1, 6,552 at 2 and 2,588 at 9 or more. This
     is TOMS's eps = 1e-15 accuracy, so "near correctly rounded" holds to a few ulps, as TOMS promises.
2. **x0 >= 0.3, not 0.29.** See section 1 and `x29_test.py`. The probe was 3,080 cells at x = 0.2900..0.3009 with
   a in {0.25, 0.5, 0.75, 0.9} and b in {2.5, ..., 50}. With 0.3, the [0.29, 0.3) cells take bup+bgrat (b0 <= 15)
   or bgrat.
3. **The closed forms (w5) are exact across every set:** 2,343 / 2,343 (`closed:*`), on the hold-out too.
4. **bpser (w3) generalises, with z64 and lnΓ(v<1) = gamln1(v) - LN(v).**
   - w_bpser goes from 1,799 to 6,767 exact (CR 2,580); hold-out 3,415 against 1,319.
   - w1_bpser: 1,666 / 1,845 (CR 788). bpser(a,b>1): 978 / 1,523 (CR 533).
   - Per set on the bpser cells (base z53 and accurate lnΓ, then z64, then z64 with toms lnΓ):

     | set | z53, accurate lnΓ | z64 | z64, toms lnΓ |
     |---|---|---|---|
     | w2f | 12,009 | 10,941 | 11,312 |
     | be | 364 | 405 | 406 |
     | ix | 2,774 | 3,196 | 3,192 |
     | w4p1 | 3 | 1,625 | 1,736 |

   - w3_order (min-first) changes nothing. w3_lg=gl is worse.
   - w3_big=toms (betaln with Excel pieces for max > 8, in place of the accurate proxy) is slightly worse:
     ix 3,094 against 3,196.
5. **L140 (x0 <= 0.7) is solved to visible parity.** It scores 7,505 / 11,673 / 11,713 of 11,742; CR gets 3,956 and
   fails w15 on 169. The step comes from `brcmp1` = pdf·x·y (w4), and it holds on ix (finding in section 1).
   - w4's W arithmetic adds +119 exact.
   - `bup_break` (TOMS's relative stop against summing every term) makes no difference: 1,011 = 1,011 on ix L140.
6. **GAMMALN at the kernel's arguments** (new probe, 5,094 cells, `check_gl.out`): the engine model misses 512.
   - 428 of the misses are at v < 1. The others are the 1-ulp sweep arguments near 1.5, 2.5 and 3, and 6.750000000000004 (w3's
     `amiss`).
   - Replacing the model by the measured values changes the cdf score by -4 / +12 / +8 (`ablate_gl_*.out`). It
     moves the M1 density from 685 to 690 of 2,035.
   - So GAMMALN model error is not what limits the kernel.
7. **(7.3, 0.6) in w4p1 is still unexplained** (1,023 cells).
   - At b = 0.6, the density M1 is a constant +26.7 units (sd 0.9) above Excel's worksheet density. w3's bpser
     is +27.5 above Excel's cdf.
   - So Excel's lnB(0.6, 7.3) is about 24 units (2.7e-15) above the exact value, for both the pdf and the cdf.
   - These forms do not give it (offsets from the exact value, in units):
     - the GL form: -2.3;
     - LN of GAMMA ratios, in 3 orders: +9.7;
     - GL(a) + algdiv: -4.9;
     - TOMS's brcomp a0 < 1 < b0 < 8 product form: -1.4.
   - Excel's GAMMALN at 0.6, 7.3 and 7.9 equals the engine model (probe).
   - (2.5, 0.5) in the same probe fits: z64 854 / 1,024 on S1, med 0.
8. **The bgrat family is the main open area.** Excel is not plain TOMS bgrat with Excel's EXP and LN: TOMS_XL bgrat
   989 / 2,156 (w15) against CR 902 / 2,241.
   - L140 bup+bgrat at (30.7, 1.7, x 0.81) has per-block offsets of -9 to +2 units in BEST.
   - The a-sweep there has an Excel-only noise of sd 47 units. CR and every model show the same sd 47 against
     Excel, so Excel has a stored, cancelling intermediate that no TOMS piece has.

## 6. Negatives (all on the same scorer; scripts named)
- `algdiv=gl` (ln Γ(b)/Γ(a+b) = fl(GL(b) - GL(a+b)) everywhere): w2f L140 bup+bgrat 57/325/532 against
  110/1936/2105. In bgrat only (`gl_bgrat`): 283/2125/2227, better in exact but with sd up to 8 where BEST has 2.7
  (absweep:7). Not structural. (`exp.py w2f "L140 bup+bgrat" BEST algdiv=gl algdiv=gl_bgrat`.)
- `pdf_lnb=gl` (GL-form lnB in the prefactor density for max > 8): +245 units at (40, 1.03). Rejected.
- `brcmp1=toms` in the L140 bgrat path: better on two blocks, worse on two (exact 357 against 110, w15 2,066 against
  2,105). No single choice fits.
- The z53 bpser out of sample (section 5.4).
- x29 = 0.29 (section 5.2).
- w3_order=min, w3_lg=gl, w3_big=toms (section 5.4).
- lnB(0.6, 7.3) forms (section 5.7).

## 7. Next hypotheses (most promising first)
1. **bgrat with Excel pieces** (L140 bup+bgrat 974 cells, bgrat 295).
   - First find the stored, cancelling intermediate behind the sd 47 a-sweep noise at (30.7, 1.7, 0.81). Candidates
     are a stored lnΓ difference for a ≈ 31 (ulp 128 units), z = fl(-nu·lnx), and r's three EXP factors.
   - Regress Excel's error in absweep:3 on the rounding of each candidate (w2's method), then fix the structure.
   - bgrat's T.DIST.RT cells (n/2, 1/2) are the clean test set: 249 cells fail w15.
2. **lnB at (0.6, 7.3).** Probe BETA.DIST(x, 0.6, b, FALSE) and the cdf for b = 6.5..8 in 0.1 steps, plus b0' =
   0.6 ± k ulp, to see whether the +24-unit offset is shape-local (a b = 7.3 / a + b = 7.9 artefact) or a law.
3. **The a, b > 2 density** (bfrac at (100, 120): 586 cells; Loader-like, w2/w4/w6). It is brcomp in every
   min ≥ 8 path.
4. **Excel's log1p** (another round-2 worker's probes t2464-r2-log1p-*): plug it into `density()` (x <= 0.375), into
   `alnrel` and into bgrat's lnx.

## 8. Probes added
- `eval/suites/probes-numeric/cases/t2464-r2-bratio-gammaln.case.json`: 5,094 cells, `=GAMMALN(exact literal)` at
  every kernel argument. Built by `make_probe_gl.py --write` (args in `t2464-r2-bratio-gammaln.args.json`).
- `eval/suites/probes-numeric/cases/t2464-r2-bratio-x29.case.json`: 3,080 cells. Built by `make_probe_x29.py --write`
  (meta in `t2464-r2-bratio-x29.meta.json`).
- Both were filled from Excel (excel-graph) on 2026-10-23.
  - `rustsheet-oracle recalc -b excel-graph <case>` wrote the `t2464-r2-bratio-*.out.json` files in this directory.
  - `fill --suite probes-numeric --case probes-numeric/<name>.case.json` wrote the two `.oracle.json` sidecars next
    to the cases. The case id needs the `.case.json` suffix; without it fill writes nothing.
  - The sidecars equal the recalc outputs on 5,094/5,094 and 3,080/3,080 cells.

## 9. Scripts
| file | what |
|---|---|
| `bratio_excel.py` | the model: TOMS 708 port, Excel pieces, `Cfg`, `kernel`, `run` |
| `score_lib.py` | data sets, scorer, tables |
| `ctl_r.py` → `ctl_r.out` | positive control against R's binary, and against CR |
| `ctl_density.py` → `ctl_density.out` | EXP/LN model check; density reproduces w1's M1 |
| `score.py` → `score_ladder.out`, `results.pkl` | attribution ladder, per branch and per set |
| `report.py` → `report_best.out`, `report_ladder.out` | compact tables, visible-parity misses |
| `exp.py` | ad-hoc variant scoring per block (`exp.py <set> <branch> BEST key=value ...`) |
| `peek.py` | per-block error medians from results.pkl |
| `make_probe_gl.py`, `check_gl.py` → `check_gl.out` | GAMMALN probe and the model check |
| `make_probe_x29.py`, `x29_test.py` → `x29_test.out` | the 0.29/0.3 probe and test |

Caches (`*.pkl`) are git-ignored and rebuilt on demand. GAMMALN values come from the harness. w1's `lib.py` is
imported read-only, for the EXP and LN models and the density data loaders.
