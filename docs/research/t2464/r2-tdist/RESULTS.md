# T-2464 r2-tdist: the Student t family through Excel's incomplete-beta kernel

Worker r2-tdist, round 2, 2026-10-23. Method: map every T.DIST.RT / T.DIST.2T / TDIST / T.DIST(TRUE) cell onto TOMS 708
`bratio` (r2-bratio's port, imported, not copied), tabulate the branch each cell takes and how far Excel is from the
model there, then model the branches one at a time. The T density (T.DIST FALSE, 19,776 cells) was attacked too, with
negative results only.

Scores are bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15), "of n". The hold-out half
is md5(case, cell) odd (r2-bratio's rule), never used for a choice. CR is the correctly rounded exact value at the
double t. Python: `/home/user/rustsheet/bindings/python/.venv/bin/python`, run from this directory.

## 1. Headline

1. **bgrat at b = 1/2 is solved, to the bit on 99.8% of cells, out of sample too.** Excel's value is
   ```
   I = RN( F(a, lnx) * Q_xl / Q(1/2, -nu lnx) )        nu = a - 1/4
   lnx = LN(x)                      if the complement argument y > 0.01   (the double 0.01 itself takes the series)
       = Excel log1p(-y)            if y <= 0.01      (r2-log1p's atanh series, x87 stored)
   Q_xl = ERFC.PRECISE(fl(sqrt(fl(-nu lnx))))   Excel's own ERFC (engine-exact model, T-1778), grat1's a = 1/2 path
   F(a, lnx) = the exact I at ln x = lnx;  Q(1/2, .) = exact erfc(sqrt(.))
   ```
   i.e. everything in bgrat is exact (extended precision) except three stored doubles: lnx, z = fl(-nu lnx) and
   Excel's ERFC of fl(sqrt(z)). An implementable form, TOMS bgrat in x87 extended with those three stored pieces
   (`xlk.bgrat_half_ld`), is nearly as good.

   | data | n | ideal (`bg=ideal`) | x87-extended TOMS (`bg=ld`) | r2-bratio BEST | CR |
   |---|---|---|---|---|---|
   | T tail cells in cells.tsv, branch bgrat | 1,564 | **1,560/1,562/1,563** (hold-out 780/783) | 1,545/1,562/1,563 | 35/323/649 | 48/350/676 |
   | new probe p1 part C (out of sample: n 31..2000, t 1..8) | 4,915 | **4,906/4,910/4,912** | 4,895/4,910/4,912 | 158/1,242/1,716 | 236/1,335/1,699 |
   | non-T cells with min(a,b) = 1/2 in bgrat (BETA.DIST, F, x29 probe; r2-bratio data, never used here) | 387 | **365/369/369** | | 277/331/356 | 232/339/362 |

2. **The T wrapper is one kernel call, K(n/2, 1/2, y, z)**, with y = fl(n/fl(n + fl(t t))) and z = fl(fl(t t)/fl(n + t t))
   as the complement (not fl(1 - y)), except at df 2, where the complement is fl(1 - y) (w5's df-2 closed forms).
   L = P(T > |t|) = I/2; T.DIST.2T and TDIST(.,.,2) = I; the upper side (T.DIST(t > 0), T.DIST.RT(t < 0)) is fl(1 - L).
   The "switch at t = 1" of statistics.md is bratio's own dispatch (x0 = z < 0.1 and (z n/2)^(1/2) <= 0.7 takes
   bpser), and the "rounded y beyond y = 0.99" rule is bgrat's lnx switch at z = 0.01 (finding 1). bfrac and basym
   can never fire for T (min(a, b) = 1/2).

3. **Whole T tail family** (FINAL = `args=t2,alnrel=xl,erfc=xl,bg_lnx=cut0.01,bg=ideal`):

   | cells | n | FINAL | r2-bratio BEST, same call | CR |
   |---|---|---|---|---|
   | cells.tsv T tails, L side (t != 0, Excel != 0) | 5,152 | **3,383/5,126/5,145** (hold-out 1,666/2,537) | 1,824/3,879/4,224 | 798/2,947/3,866 (403/2,537) |
   | cells.tsv upper side (fl(1 - L)) | 153 | **141/153/153** | | 133/153/153 |
   | probe p1 part C (bgrat region) | 5,000 | **4,931/4,995/4,997** (hold-out 2,517/2,559) | 183/1,327/1,801 | 238/1,350/1,732 |
   | probe p1 part D (n 3..30, t 0.05..12, all branches) | 4,000 | **2,720/3,993/3,994** (hold-out 1,366/2,025) | 2,675/3,970/3,982 | 1,319/3,039/3,578 |

   Visibly off (w15) on the cells.tsv L side: 7 of 5,152 (CR: 1,286). They are listed in section 4.

4. **The T density is not solved.** No composition tried beats the correctly rounded density in exact matches
   (CR 4,258/19,149/19,547 of 19,776; best other 4,132). Section 5 has the negatives and what they rule out.

## 2. Branch map of the T tails (`tscore.py`, output `tscore_final.out`)

Per branch as fired by FINAL. Columns: FINAL | `bg=ld` | r2-bratio BEST with K(n/2,1/2,y,z) | BEST with fl(1-y) | CR.

| branch | where (T) | n | FINAL (hold-out) | bg=ld | BEST yz | BEST y1y | CR |
|---|---|---|---|---|---|---|---|
| bgrat/sw | n > 30, z >= 0.1 or (z n/2)^.5 > 0.7, z < 0.3 | 1,564 | **1,560/1,562/1,563** (780/783) | 1,545/1,562/1,563 | 35/323/649 | 96/800/1,237 | 48/350/676 |
| bup+bgrat/sw | n <= 30, same | 1,344 | 356/1,334/1,343 (169/648) | 353/1,335/1,343 | 347/1,330/1,339 | 199/1,250/1,324 | 110/773/1,154 |
| w_bpser (no swap) | t^2 > n (y < 0.5) | 850 | 351/838/847 (162/409) | same | same | same | 216/681/752 |
| w_bpser/sw | centre, t <~ 1 | 557 | 410/555/555 (209/285) | same | same | 42/211/289 | 291/537/549 |
| w1_bpser/sw | z >= 0.3 | 320 | 189/320/320 (89/155) | same | same | same | 22/153/223 |
| closed:a0=1 | df 2, y >= 0.1 | 479 | 479/479/479 | same | same | same | 98/419/477 |
| closed:b0=1/sw | df 2 centre | 38 | **38/38/38** (fl(1-y)) | same | 13/34/35 (z) | 38/38/38 | 13/34/35 |

Which complement (argument variants with r2-bratio BEST, `tails.py` → `tails.out`, `brk.py`): the centre bpser
(w_bpser/sw) needs z (yz 410 vs fl(1-y) 42 of 557); bup (n <= 30) needs z as its x0 factor (`bup_lab.py`: pdf*y*z
494 vs pdf*y*fl(1-y) 224 of 1,344); df 2 needs fl(1-y) (38 vs 13). In bgrat, fl(1-y) looked better (96 vs 35) only
because bgrat reads LN(y) above z = 0.01; with that switch z and fl(1-y) give the same values.

## 3. The bgrat finding in detail

Scripts: `hx.py` (the identity, lnx variants, cut scan) → `hx.out`; `hx_ctl.py` (controls) → `hx_ctl.out`;
`xlk.py` (`bgrat_half_ideal`, `bgrat_half_ld`, the `XCfg` switches); `p1.py B` → `p1_B.out` (the cut);
`oos_bgrat.py` → `oos_bgrat.out`; `const_bg.py` → `const_bg.out`.

- **Controls on the 1,564 T bgrat cells** (`hx_ctl.out`), each one ingredient removed:

  | variant | exact/w4/w15 |
  |---|---|
  | identity (HX) | **1,560/1,562/1,563**; tune half 780/781, hold-out 780/783 |
  | correctly rounded erfc in place of Excel's ERFC | 395/1,421/1,507 |
  | z not rounded before sqrt | 1,370/1,432/1,509 |
  | no ERFC ratio, F(lnx) only | 291/1,260/1,392 |
  | exact ln of the rounded y | 232/1,220/1,386 |

  So Excel's ERFC (not a TOMS erfc1, not a correctly rounded one) and the stored z are both required.
- **lnx variants** (`hx.out`, same identity): cut 0.01 1,560; always LN(y) 1,418; LN(fl(1 - z)) above the cut 1,189;
  Excel log1p of fl(1 - y) 1,189; correctly rounded log1p(-z) 333. The cut scanned at 0.005 / 0.0095 / 0.0105 / 0.02
  scores 1,499 / 1,550 / 1,553 / 1,392.
- **The cut, pinned by probe p1 part B** (`p1_B.out`, 400 cells per n straddling z = 0.01): for n = 100, 200, 300,
  1000 and 3000, every cell with z <= 0.01 (the double 0.01 included) fits the log1p form and every cell above
  (first observed z = 0.010000003960000378) fits LN(y), with 1 cell of 2,000 fitting neither. Whether Excel tests z or
  y = 0.99 is not separable here (both are exact at the cut). n = 40 and 64 at z = 0.01 are in bpser, not bgrat.
- **What it means.** TOMS bgrat takes lnx = log(x) when y > 0.375, else alnrel(-y); Excel's threshold is 0.01. The
  prefactor (r, algdiv, b ln nu) and the asymptotic sum leave no trace: the constant per df is exact to well under an
  ulp (`const_bg.out`: per-df medians against TOMS-double, GAMMALN-difference and exact constants; the GAMMALN
  difference is off by 10^2 to 10^11 units and is rejected). Only the function-call results (LN or log1p, ERFC) and
  the double z handed to SQRT/ERFC are rounded. bpser does not behave this way (section 5).
- **Port.** `xlk.bgrat_half_ld` is TOMS bgrat at b = 1/2 in numpy longdouble (x87 extended) with the three stored
  pieces: 1,545/1,564 and 4,895/4,915 bit-exact (w15 1,563 and 4,912). The engine has `extended_*` helpers. The
  remaining 1% are final-rounding near-ties of the longdouble exp/log; a double-double evaluation should close them.
- For b != 1/2 (BETA.DIST/F/x29 bgrat cells with other b) grat1 needs Q(b, z): the same identity with Excel's
  incomplete-gamma kernel is the obvious next test (not done).

## 4. What is left visibly off in the T tails (FINAL, cells.tsv L side, 7 of 5,152)

| cell | branch | Excel | FINAL | CR |
|---|---|---|---|---|
| T.DIST.RT(1E-10, 3), TDIST(1E-10,3,1) (t1048-tdist B27, A27) | w_bpser/sw | 0.5 | 0.5 - 1.8E-11 | same as FINAL |
| T.DIST.2T(100,2), (1E6,2); T.DIST.RT(1E6,2) (m9-t-dist-2t A22/A24, rt A27) | w_bpser at (1, 1/2), df 2 below y = 0.1 | | -5 ulp | -1 ulp |
| T.DIST.RT(5.4513, 500) (tail-grid B1484) | bgrat | | -21 ulp | +53 ulp |
| T.DIST.RT(2.5, 30) (beta-link C140) | bup+bgrat | | +3 ulp (value 0.00905...) | +4 ulp |

The first two: when n + t^2 rounds to n, y = 1 and Excel returns 0.5 (statistics.md "Tiny t"); the model reads z
(nonzero) instead. A `y == 1 → L = 0.5` guard fixes them; it is not modelled here. The df-2 deep tail is w3's bpser at
a0 = 1, b0 = 1/2, where w3's series is 5 ulp low.

## 5. Negatives (all scored; scripts named)

### T tails
- Complement choices for one call (`tails.py` → `tails.out`, `brk.py`, `tscore_final.out`): fl(1-y) everywhere
  (`y1y`) 1,394/3,936/4,534 of 5,152; z for y > 0.99 only (`cut99`), z for t <= 1 or y > 0.99 (`hyb`, 1,900/4,888/5,035
  with Excel log1p/ERFC), the BETA.DIST(z) call in the centre (`split`) — all below the single K(n/2,1/2,y,z) call.
- TOMS bgrat with Excel pieces only (Excel log1p and ERFC substituted, structure kept in doubles): 261/1,409/1,474
  (with hyb) and 308/1,553/1,558 (with the 0.01 cut) of 1,564, against 1,560 for the identity. A per-df offset of
  -1.3 to +3.4 units stays (`pern.py`, `const_bg.py`).
- Upper side (`compose.py` → `compose.out`, 153 cells): fl(1 - L) 141, 0.5 + J/2 (J the kernel's complement) 132,
  (0.5 - L) + 0.5 126, CR 133.
- bup + bgrat (n <= 30; `hx_bup.py`, `bup_lab.py`, `p1.py A` → `p1_A.out`): with the solved bgrat part, the bup part
  limits. bup = P y z / a W (W exact, one final rounding): with w1's M1 density as P 494/1,344, with Excel's own
  BETA.DIST(y, n/2, 1/2, FALSE) from probe p1 part A 598/1,344 (hold-out 273/648); w4's stored-W bup 356; exact bup at
  the rounded y 200; P*fl(y*z) stored 570. All are within w15 on 1,343-1,344, so this branch is already at visible
  parity; the bit-level bup arithmetic is open. (M1's density matches Excel's at these (y, n/2, 1/2) on 759/1,344.)
- An "ideal" bpser in the bgrat style (`bpser_ideal.py` → `bpser_ideal.out`; exact I with only Excel's LN(y) and
  EXP of the extended exponent): 63/309/369 of 446 cells.tsv w_bpser cells and 100/623/767 of 875 probe cells, against
  w3's stored-op series 263/446/446 and 423/867/867. bpser really is a stored-double series (w3); bgrat is not.

### T density (T.DIST(t, n, FALSE), 19,776 cells; CR 4,258/19,149/19,547)
- `pdf_lab.py` → `pdf_lab.out`: RN(K_exact * EXP_xl(E)) with E = -fl(h LN(u)) (`lnu` 3,168), extended (`lnu_x`
  3,400), -fl(h log1p_xl(t^2/n)) (`l1p` 4,003/19,179/19,553), -fl(h LN(1 + t^2/n)) (`ln1p` 3,685; extended 4,132),
  LN(u/n) (`ratio` 3,760), h LN(y) (`lny` 3,420, extended 3,788), w1's beta-density composition with lnB(n/2, 1/2)
  (`m1` 3,233). At df 2, `lnu` (= EXP(-1.5 LN(2 + t^2)), K = 1) is 1,050/1,050, as statistics.md says.
- `pdf_brc.py` → `pdf_brc.out`: the density as TOMS brcomp(n/2, 1/2, y, z)/|t| = y^(n/2) z^(1/2)/(B |t|), ideal
  with Excel's logs 4,311/19,139/19,491; with exact logs of the double y, z 4,280. Not it.
- `pdf_reg.py` → `pdf_reg.out` (t0969-t-df-ratio exact-input runs, n = 3, 4, 5, 7; t^2 and n + t^2 exact): Excel's
  error has sd 1.0 unit and a per-df mean of +3.3, -0.9, +2.2, +0.6; it does not correlate with the rounding of
  Excel's LN(u), LN(1 + t^2/n), the product h LN(u), q = t^2/n or Excel's EXP (|corr| <= 0.28, residual sd 0.95-1.0).
  So no Excel LN of u or 1 + q enters, and the exponent is not stored after an Excel LN.
- `pdf_ereg.py` → `pdf_ereg.out` (all cells, by |E| band): only Excel's log1p series form reaches slope 0.56,
  corr 0.58 (|E| in [3, 6)); the forms through LN(1 + q) or LN(u) are excluded at small q (they predict 10^3-10^5
  unit errors Excel does not have).
- `pdf_const.py` → `pdf_const.out`: the per-df offset is not EXP(GAMMALN(h) - GAMMALN(n/2))/SQRT(n PI) (in double or
  extended), GAMMA(h)/GAMMA(n/2)/SQRT(n PI) (two orders) or 1/(sqrt(n) EXP(lnB_M1)); none reproduces +3.1 (n = 3),
  -1.2 (n = 4), +2.3 (n = 5).

## 6. Which branch to port first

1. **bgrat at b = 1/2 (`bgrat_half_ld`)**, with the T wrapper K(n/2, 1/2, y, z) (df 2: fl(1 - y)), L = I/2,
   upper = fl(1 - L), 2T = I. It takes the T bgrat region from 48 (CR) / 35 (r2-bratio BEST) to 1,545-1,560 bit-exact
   of 1,564 and to 4,895-4,906 of 4,915 out of sample, and the T tails' visibly-off cells from 1,286 (CR) to 7.
   It also fixes BETA.DIST/F bgrat cells at b = 1/2 (365 of 387 against 277). Prerequisites already exist: Excel's
   LN, ERFC, r2-log1p's log1p, extended arithmetic.
2. Then the y == 1 guard (0.5), and the T composition itself; the bpser branches are w3's (r2-bratio BEST).

## 7. Next hypotheses

1. **bgrat for general b** (BETA.DIST x29 bgrat cells with b != 1/2: 600; F: 26): the same identity with Q(b, z) from
   Excel's incomplete-gamma kernel (GAMMA.DIST's upper tail) in place of ERFC. If Excel's generic-shape incomplete gamma
   is unsolved, the bgrat cells become a readout of it.
2. **bup arithmetic**: with Excel's density known (probe p1 part A gives it at 1,344 points), test an x87-extended bup
   (ratio products and the sum unrounded) with only P stored, then the w4 variants, on the bup+bgrat T cells.
3. **T density**: the evidence (no LN/EXP rounding signature, sd 1 unit, per-df offsets) points at an extended-precision
   computation with a few stored values, like bgrat. Probe the density at exact-input runs with inner columns
   (EXP(-h LN(1+q)), GAMMALN(h), GAMMALN(n/2), POWER(1+q,-h)) and test RN(K_d * P_ext) with K_d one stored double
   built by each Excel route; the stored constant is the per-df offset.
4. **Why bgrat is extended**: check whether Excel's L140 bup+bgrat (BETA.DIST, a, b > 1; r2-bratio's open
   sd-47 block at (30.7, 1.7, 0.81)) takes the same identity with Q(b, z) for b = 0.7 (b0' of 1.7). That block is the
   largest remaining BETA.DIST bgrat gap.

## 8. Probe added

`eval/suites/probes-numeric/cases/t2464-r2-tdist-p1.case.json` (12,524 cells; built by `make_probe.py --write`, meta
`t2464-r2-tdist-p1.meta.json`; filled from Excel (excel-graph) on 2026-10-23 with
`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric --case probes-numeric/t2464-r2-tdist-p1.case.json`,
which wrote `t2464-r2-tdist-p1.oracle.json` next to it). Parts: A 1,344 `BETA.DIST(y, n/2, 0.5, FALSE)` at the
bup+bgrat T cells' y; B 2,800 `T.DIST.RT` with z straddling 0.01 (n 40..3000); C 5,000 `T.DIST.RT`, n 31..2000,
t 1..8; D 4,000 `T.DIST.RT`, n 3..30, t 0.05..12 (seeded). Caveat: w2-forensics' `num()` writes some t >= 10 as
16-digit literals, which Excel cuts to 15 digits; `p1.py` reads those cells at the 15-digit value (`_seen`).

## 9. Scripts

| file | what |
|---|---|
| `tlib.py` | T cells from ../cells.tsv, worksheet y/z, exact tails and density (mpmath, cached), scorer (r2-bratio's) |
| `englib.py` | Excel functions not in the harness (ERFC.PRECISE, ERF.PRECISE) through the engine's python bindings |
| `xlk.py` | `XCfg`/`XKernel`: r2-bratio's kernel with Excel log1p (alnrel), Excel ERFC (grat1), bgrat's lnx switch, the ideal and x87-extended bgrat at b = 1/2 |
| `tails.py` → `tails.out` | argument variants of the T call (r2-bratio BEST kernel) |
| `tscore.py` → `tscore_final.out` | per-branch scores for any (args, XCfg) spec; `brk.py`, `errstat.py`, `pern.py` break-downs |
| `compose.py` → `compose.out` | upper-side composition |
| `hx.py` → `hx.out`, `hx_ctl.py` → `hx_ctl.out` | the bgrat identity, lnx variants, cut scan, controls |
| `const_bg.py` → `const_bg.out` | per-df constant of bgrat against candidate constants |
| `oos_bgrat.py` → `oos_bgrat.out` | out-of-sample bgrat at b = 1/2 on r2-bratio's BETA.DIST/F/x29 cells |
| `hx_bup.py` → `hx_bup.out`, `bup_lab.py` → `bup_lab.out` | bup + bgrat with the solved bgrat part |
| `bpser_ideal.py` → `bpser_ideal.out` | ideal-bpser negative |
| `make_probe.py`, `p1.py` → `p1_A.out`, `p1_B.out`, `p1_CD.out` | probe p1 and its analysis |
| `pdf_lab.py`, `pdf_brc.py`, `pdf_reg.py`, `pdf_ereg.py`, `pdf_const.py` (`.out`) | T density negatives |

Caches (`*.pkl`) are git-ignored and rebuild (ERFC values come from the bindings on demand; xlk.run re-evaluates
until no lazy lookup is missing).
