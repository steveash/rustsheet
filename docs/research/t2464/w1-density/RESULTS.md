# T-2464 w1-density: Excel's beta density BETA.DIST(x,a,b,FALSE)

Worker w1-density, 2026-10-23. Method: solve the density first, because its composition is the cdf's prefactor.
Only the pdf branch was worked on, so there are no direct or reflected cdf counts here. Every count is
bit-exact / within 4 ulp / within 15 significant digits ("w15").

## Data
- `old`: the 2,035 pdf cells of `../beta_exact.tsv`. The exact-rounded value scores 337 / 1,618 / 1,812 on them.
- `probe1`: **new oracle data**, 16,000 cells (excel-graph, 2026-10-23). It has ten shape pairs. Each pair has
  1,000 irregular x (x = 0.00137 + 0.000997713k), plus 300 consecutive doubles from 0.0123456789 and 300 from
  0.7123456789. The pairs are (0.5,7), (0.5,1), (1,7), (0.25,3.3), (2.5,3.5), (12,20), (0.5,20), (3,0.5), (1.5,1)
  and (1,20).
  - Files: `probe/make_probe1.py` builds `probe/probe1.case.json`. `cd tools && uv run rustsheet-oracle recalc -b
    excel-graph <case>` printed `probe/probe1.out.json`. One fill took 16 s.
  - The only write outside this directory was the oracle's own cache (`eval/.cache/`, git-ignored). The case is
    not in `eval/suites`; the coordinator may want to commit it there.
  - The exact-rounded value scores 4,537 / 13,100 / 14,688 on probe1.
- Positive controls:
  - `ctl_elem.py`: the mpmath EXP V6 and LN models (`lib.exp_v6`, `lib.ln_x87`) match the engine harness on
    5,000 / 5,000 EXP and 4,000 / 4,000 LN inputs. Because they are mpmath models, they also accept extended
    (64-bit) inputs.
  - `ctl_cscan.py`: a lnB shift of +3 units planted in synthetic data is recovered exactly (800/800 at c = -3,
    0/800 at c = 0).
  - `pair1.py`: the B-free pair test scores rms 0.83 on its own synthetic truth.
- mog (`/home/user/fundamental-research-labs/mog`): BETA.DIST is statrs `Beta` (`compute-functions/.../beta.rs`,
  `support.rs`). It gives no hints about Excel's kernel.

## Best model M1 (`model_final.py`)
For min(a,b) <= 1:
1. **Logs: the TOMS 708 `brcomp` switch at x = 0.375.**
   - lnx = LN(x), Excel's stored LN.
   - ly = fl(log1p(-x)) for x <= 0.375, and LN(fl(1-x)) above.
2. **Stored products.** tx = fl(fl(a-1)·lnx) and ty = fl(fl(b-1)·ly).
3. **lnB is TOMS 708 `betaln`, with Excel's GAMMALN as `gamln` and Excel's LN as `log`.**
   - min < 1 and max <= 8: fl(GAMMALN(min) + fl(GAMMALN(max) - GAMMALN(a+b))). The switch is at 8 inclusive,
     where TOMS 708 uses `< 8`: at b = 8, 8/10 against 2/10. It takes the smaller shape first: for a > b, 34/60
     against 20/60 in BETA.DIST's argument order.
   - max > 8: gamln(min) + algdiv(min, max), in double.
   - a = 1, 2 < b < 8: TOMS's L60 reduction, with the product z held in an x87 register, then LN. This equals
     -LN(b).
4. **E = tx + (ty - lnB) in extended precision, never stored.** b = 1 is the exception: there E = fl(tx - lnB).
5. **Density = Excel's EXP (model V6) of E.**

Both shapes > 1, and a >= 8, are not modelled. M1 returns the exact density there.

| subset | n | M1 exact | w4 | w15 | exact-rounded exact |
|---|---|---|---|---|---|
| old, all | 2,035 | **685** | 1,838 | 1,926 | 337 |
| old, a<1, b<8 (B form known) | 368 | 252 (even 126/186, odd 126/182) | 368 | 368 | 37 |
| old, a<1, b=8 | 10 | 8 | 10 | 10 | 0 |
| old, a<1, b>8 (algdiv) | 562 | 207 | 557 | 560 | 139 |
| old, 1<a<8, b<1 | 60 | 34 | 60 | 60 | 9 |
| old, a=1 (all b) | 27 | 20 | 27 | 27 | 10 |
| old, b=1, a<8 | 19 | 14 | 18 | 19 | 9 |
| old, unmodelled (both shapes > 1, or a >= 8) | about 950 | exact-rounded fallback | | | |
| probe1, all | 16,000 | **10,301** | 14,827 | 15,502 | 4,537 |
| probe1 (0.5,1) | 1,600 | **1,591** (even 794, odd 797) | 1,600 | 1,600 | 1,163 |
| probe1 (1.5,1) | 1,600 | **1,593** (even 796, odd 797) | 1,600 | 1,600 | 1,247 |
| probe1 (1,7) | 1,600 | 1,261. The x in [0.375, 0.5) zone is 124/125. | 1,600 | 1,600 | 332 |
| probe1 (3,0.5) | 1,600 | 1,286 | 1,600 | 1,600 | 749 |
| probe1 (0.25,3.3) | 1,600 | 1,105 | 1,600 | 1,600 | 389 |
| probe1 (1,20) | 1,600 | 1,146 | 1,552 | 1,581 | 97 |
| probe1 (0.5,7) | 1,600 | 1,060 | 1,599 | 1,600 | 101 |
| probe1 (0.5,20) | 1,600 | 1,015 | 1,554 | 1,584 | 215 |
| probe1 (2.5,3.5), unmodelled | 1,600 | 133 | 1,448 | 1,545 | 133 |
| probe1 (12,20), unmodelled | 1,600 | 111 | 674 | 1,192 | 111 |

Hold-out:
- Every choice in M1 is discrete (a form, a branch, an order). Even and odd halves agree throughout.
- The algdiv lnB was predicted, not tuned. On the even k of probe1, `cscan2.py` tuned a lnB offset of c = 12.0
  units for (0.5,20) and 3.8 for (1,20). The TOMS `betaln` value differs from the naive GAMMALN form by 12.00 and
  4.00 units (`tomscmp.py`).
- On the odd k (hold-out), the tuned offset gives 509/800 for (0.5,20) and 561/800 for (1,20). With no offset
  the naive GAMMALN form gets 0/800 and 3/800.

Region map (`model_final_*.out`, per zone):
- **Fits to within a few cells (99.5%): b = 1.** E = fl(fl((a-1)·LN x) - lnB), then EXP.
- **Fits on one zone (99%): a = 1, x in [0.375, 0.5).**
- **Everywhere else with min <= 1, about 63-80% exact and 97-100% w4.** This covers a<1 at any b, 1<a<8 with
  b<1, and a=1 with x outside [0.375, 0.5).
- **Both shapes > 1, and a >= 8: a different composition.** M1 scores at the exact-rounded level or below there.

## Key findings (each with its evidence)
1. **The final exponent is held in extended precision**, both on the old data and on probe1.
   - Old data, a<1, b<=8: EXP of the extended E gets 263/386, against 229/386 when E is stored (`enum3.py`).
   - probe1 (1,7), x >= 0.5 (E < -2): a stored E matches 0 of 800 cells and the extended E matches 680
     (`c2o.py`, `c2r.py`). At (1,20) it is 0 against 606.
2. **b = 1 takes a stored E**, the opposite of finding 1, and fits 99.5% on two sheets (`c1b.py`).
   - (0.5,1): stored 1,591/1,600, extended 1,326/1,600.
   - Its 9 misses sit where fl(tx - lnB) rounds inside E in [1, 2). There the extended value fits 8 of the 9.
   - So b = 1 is probably its own closed-form path, like the cdf's b = 1 closed form (T-1744).
3. **tx uses Excel's stored LN(x)** (`c0run.py`).
   - On the consecutive-double run at x ~ 0.0123, the needed exponent shift follows (a-1)·err(LN(x)), with its
     ±2-unit, period-6 sawtooth.
   - An extended ln x scores lower (old data, `enum3.py`: 192 against 263).
4. **ln(1-x) switches at x = 0.375**, as in TOMS 708 `brcomp` (`c2f.py`, `c2m.py`).
   - At a = 1, LN(fl(1-x)) is bit-exact on every 0.025-wide x bin from 0.375 to 0.5 (124/125). Below 0.375 it
     drops to 3-15/25 per bin; an accurate log1p gets 14-22/25 there.
   - Below 0.375, Excel's log1p(-x) is not the correctly rounded value. For x in [0.15, 0.375], 33 of 226 cells
     need ln(1-x) one ulp off the correctly rounded log1p away from rounding midpoints (`c2t.py`, `c2v.py`).
   - Ten log1p implementations score at most the correctly rounded value's 170/226 (`c2u.py`). They are fdlibm,
     Cephes, Kahan's two forms, TOMS `alnrel` in double and in x87, the atanh series, FYL2XP1 and log(1+t).
5. **lnB is TOMS 708 `betaln` built from Excel's GAMMALN and LN** (`tomscmp.py`, `tomsscore.py`).
   - It predicts the needed offsets at b = 20 (12 and 4 units); the naive form gets 0/1600 and 6/1600 there.
   - For a = 1, b = 7, Excel takes -LN(7). The double-precision TOMS L60 product gives 15/1600; L60 with an
     extended product gives -LN(7).
   - The max = 8 boundary goes to the GAMMALN form.
   - The algdiv internals (double against x87 extended, the log variants) are not resolved: 193-215 of 572 cells
     either way (`algdiv_chk2.py`). The rounding of individual b values carries ±2 units of unexplained noise
     (`algdiv_chk.py`).
6. **The density is not a cdf prefactor divided by x or x·y** (`cdfpref.py`).
   - b = 1: a·EXP(a·LN x)/x gets 931/1600, against 1,591.
   - In general, EXP(a lnx + b lny - lnB)/(x·y) gets 300/1600 at (0.5,7), against 1,060.
7. **Remaining gap.** With min <= 1, the plateau of about 66-80% is per-cell noise below 1 unit, not a constant.
   - A constant exponent offset helps nowhere: the peak is at c = 0 (`cscan.py`, `cscan2.py`).
   - Neither EXP internals (`expvar.py`, 10 variants) nor any storage pattern or term order of the exponent sum
     (`sum4.py`, 3-5 terms, every d/e pattern) closes it.
   - At x <= 0.375 it is partly finding 4's log1p. At x >= 0.5 at a = 1 (75-87%), the cause is unknown.

## Negatives (all on the same scorer)
- `enum1.py` (old data): no product, quotient or single-EXP form beats exact-rounded's 337 overall.
  - Forms: power terms (EXP of a stored or extended (a-1)·LN, pow_cr, x^a/x) × B (GAMMALN groupings, GAMMA
    ratios, exact) × four combine orders.
  - The best is the single EXP with the exact lnB: 334.
  - GAMMA-ratio B gets at most 235.
- `enum2.py`: with 1 - x held exactly, the best is 464 / 1,658 / 1,836. It still uses the exact B.
- `enum4.py`: x87 base-2 pow forms (FYL2X / FYL2XP1 exponents, then F2XM1) reach at most 131/386 in a<1, b<=8,
  against 263 for the base-e extended form.
- `enum5.py`: brcomp-style a·lnx + b·lny then /x/y, and (a-1) written as a·LN - LN, add nothing (at most 264/386).
- Pair test (`pair1.py`, `pair2.py`): the y-term fits only with 1-x exact, not with fl(1-x) as one global form.
  - On density-b, exact (1-x) gets 186/320 and fl(1-x) 22/320.
  - This is finding 4's switch, seen before the switch was known.
- `c2l.py`, `expvar.py`: EXP variants make no difference. These are correctly rounded exp, an exact t, F2XM1 on
  negative f, floor in place of nint, a stored t, u, w or reciprocal, and log2(e) as a double. Each is equal or
  worse, by up to 2x.
- `c2i.py`, `c2j.py`: at a = 1, density = fl(b·P) with P a stored power fails. No candidate P lies in the
  admissible set.

## Most promising next hypotheses
1. **Excel's log1p(-x) for x <= 0.375.**
   - It is faithful but not correctly rounded: about 15% of cells are one ulp off.
   - Probe ln(1-x) directly through BETA.DIST(x,1,b,FALSE) on dense runs at a = 1 with large b, where ty
     amplifies ly's error.
   - Then peel it as a rational approximation, the way the ERF and GAMMALN kernels were peeled. TOMS `alnrel` with
     Excel's own coefficients is the first candidate structure.
2. **The x >= 0.5 residue at a = 1** (75-87%) and its counterpart at a < 1.
   - Test whether, for y <= 0.375, ln x = alnrel(-y) and ln y = LN(y) both enter, as in brcomp, with an Excel
     alnrel. M1 uses LN(x) everywhere.
   - Also test a stored partial sum that only shows for |E| > 2.
3. **Both shapes > 1, and a >= 8.**
   - TOMS `brcomp`'s a0 >= 8 Stirling branch (rlog1, bcorr) and its 1 <= a0 < 8 branch, with betaln's a > 2
     reduction (L40/L60).
   - probe1 (2.5,3.5) and (12,20) are ready for this: M1's composition gets 21% at the best offset and 3%.
4. **algdiv internals** (±2-unit noise per b at b in (8, 9.5]): x87 extended evaluation with Excel's LN, and an
   Excel alnrel.
5. **The cdf.** Use M1's ingredients (brcomp logs, TOMS betaln with Excel's GAMMALN and LN, extended exponent) as
   the prefactor of TOMS `bratio`'s bpser/bup/bfrac. The density shows that Excel's beta code is a TOMS 708 port.

## Scripts
| script | what it does |
|---|---|
| `lib.py` | Shared helpers: data loaders, cached Excel functions, mpmath EXP V6 / LN / 2^x with extended input, scorer. |
| `forms.py`, `gen1.py`, `gen2.py`, `sum4.py` | Composition enumerators, per x zone. |
| `enum1.py` to `enum5.py` | Enumerators on the old data. |
| `pair1.py`, `pair2.py` | B-free pair tests. |
| `interval1.py`, `interval2.py`, `ivlib.py` | Interval inversion of the exponent. |
| `c0run*.py`, `c1*.py`, `c2*.py` | Sheet studies. |
| `cscan.py`, `cscan2.py` | Offset scans, with hold-out. |
| `toms.py`, `toms_betaln.py`, `tomscmp.py`, `tomsscore.py`, `algdiv_chk*.py` | TOMS 708 pieces and checks. |
| `log1p_impls.py` | Ten log1p implementations. |
| `ctl_elem.py`, `ctl_cscan.py` | Positive controls. |
| `model_final.py` | M1 and the score tables. Run it with the venv python. |
