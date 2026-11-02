# T-3571 r5-bfrac: Excel's bfrac fraction arithmetic

Worker r5-bfrac, 2026-11-02 (T-3571; split from T-3468, T-3383, T-2512). Method: docs/research/excel-kernel-playbook.md.
Base: w4-fractions section 2 (bfrac's λ = fl(a - fl((a+b)x)), brcomp = Excel's density·x·y, a white residual of sd 3.8
units at (100, 120) that no TOMS storage mask explained). Scores are bit-exact / within 4 ulp / within half a unit of
the 15th significant digit (w15). Units are 2^-53, relative.

Python: any venv with numpy and mpmath (this run used one outside the repo; the repo's `bindings/python/.venv` with
`uv pip install mpmath numpy` works too), run from this directory. The C searches build with
`cc -O2 -mfma -ffp-contract=off -o <bin> <file>.c -lm` and read the input `search_in.py` / `search_in2.py` write.

## 1. Headline

1. **The residual is the fraction's own arithmetic, and brcomp is Excel's density.** On the new probes the implied
   fraction error (Excel's cdf / (Excel's pdf · fl(x·y)) against the exact fraction at Excel's stored inputs) grows
   with bfrac's term count: sd 1.8 at 5 terms, 2.4-3.1 at 9-12, 3.4-4.3 at 15-22 (`runs.out`). It is uncorrelated with
   the density's own error (|corr| <= 0.25, and mostly <= 0.1) even where that error is 336 units (2,500, 3,000).
   A prefactor noise would not grow with the term count; a density other than the worksheet's would carry its error.
2. **λ is fl(a - fl((a+b)x)) in both orientations.** For a > b TOMS (and R) take λ = (a+b)y - b; Excel does not:
   on (300, 60) and (700, 90) the residual sd falls from 10.9 and 18.3 (TOMS's λ) to 4.7 and 4.5 (`lamab.out`). This is
   the λ statistics.md's dispatch entry already names ("min(a,b) > 1 swaps when λ = a - (a+b)x < 0"); bfrac reuses it.
3. **Excel evaluates TOMS's fraction with Numerical Recipes' modified Lentz loop, on terms in closed form.** The
   frozen model (`model.py`, frozen at commit 0e334d864 before the hold-out was scored):
   ```
   lam = fl(a - fl((a+b)*x)),  y = fl(1 - x),  c = lam + 1,  b0 = c * (a/(a+1))
   term n:  u = a+n-1, v = a+b+n-1, s = a+2n-1
            alpha_n = v/s/s*u*n*(b-n)*x*x                          (left to right)
            beta_n  = n + fl(n*x*(b-n))/s + ((a+n)/(a+2n+1)) * (c + n*(y+1))
   r: f = 1e-30, C = f, D = 0; the partial (1, b0): D = 1/(b0 + D), C = b0 + 1/C, f = f*(C*D);
      then per term D = 1/(beta + alpha*D), C = beta + alpha/C, d = C*D, f = f*d, until |d - 1| <= 10*DBL_EPSILON
   cdf = fl(fl(P * fl(x*y)) * f),  P = BETA.DIST(x, a, b, FALSE)   (Excel's own density)
   ```
   r = 1/(b0 + K(alpha_n/beta_n)) is exactly TOMS bfrac's fraction (the same alpha_n, beta_n and c/c1 = b0 in exact
   arithmetic), so w4's λ sensitivity and tie response carry over; only the arithmetic differs. `model.py` is the
   canonical model; `lentz.py` is the exploration module the ranking scripts share (its loop is the same).
4. **Scores** (`score_final.out`, `score_fresh.out`, `score_extra.out`; M0 is TOMS bfrac as the engine ports it: R's
   order in doubles, eps 15·1e-15, TOMS's λ, on the same Excel density; CR is the correctly rounded I_x):

   | probe | cells | FROZEN | M0 (TOMS doubles) | CR |
   |---|---|---|---|---|
   | t3571-r5-bfrac-tune (in-sample) | 1,536 | 1,236/1,533/1,536 | 215/1,188/1,462 | 44/327/854 |
   | t3571-r5-bfrac-hold (filled with tune, scored once after the freeze) | 1,632 | **1,158/1,630/1,632** | 162/977/1,384 | 45/319/781 |
   | t3571-r5-bfrac-fresh (built and filled after the freeze) | 1,200 | **694/1,177/1,188** | 86/532/844 | 15/117/268 |
   | t2464-w2f-xruns-1 (100, 120) run (w4's best 185 exact) | 1,024 | 401/998/1,024 | | |

   - Hold-out rate 71.0% against tune 80.5%: inside the 10-point margin fixed before scoring (section 0), and far
     above CR's 45. Gate A envelope on the fresh sweep: 1,200 of 1,200 inside (within 15 digits of Excel or 1e-9 of
     50-digit math); none outside.
   - The exact rate falls with the term count (fresh sweep, `score_extra.out`): 168/170 under 10 terms, 435/690 at
     10-19, 69/232 at 20-29, 22/108 at 30+. Misses are ±1-3 ulp (`resid.py`: residual mean 0, sd 0.5 at 6 terms to 2.8
     at 23 with the earlier α form). One more rounding per term is not modelled (section 4).
5. **Region.** All three probes lie in a, b in [40, 5000] on the unswapped call (x below the mean, z from -8 to -0.2 sd); a0 < 40 with b0 >= 40, shapes above 5000, the swapped call and the
   band within 0.2 sd of the mean are not probed, so the model is claimed only on that region.
6. **Fits.** EPS = 10·DBL_EPSILON is `/// FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)`: a stop threshold chosen by scoring eight
   candidates on the tune probe; the scan brackets it in (2.1e-15, 2.5e-15) (`score_eps_tune.out`) and 10·DBL_EPSILON
   is the natural constant inside. Everything else is structural, ranked among candidates (loop family, start, term
   forms, association): hold-out and fresh sweep above, no tag. TINY = 1e-30 is Numerical Recipes' FPMIN (1e-300
   scores lower, `score_loop_tune.out`). No step targets sub-ulp differences: every choice moved whole cells.

## 0. Probes, split, housekeeping
- **Probes** (excel-graph, filled 2026-11-02, built by `mkprobe.py`, cases in `eval/suites/probes-numeric/cases/`):
  - `t3571-r5-bfrac-tune` (1,536 points, 3,072 cells): shapes (40, 45), (100, 120), (1000, 1200), (300, 60),
    (45, 2000), (60.5, 75.25); runs of 96 consecutive doubles from x0 = mean + z·sd at z = -7, -4, -1 (a run with any
    basym point is dropped: (1000, 1200) z = -1, (45, 2000) z = -7). Row k: A = cdf, B = pdf.
  - `t3571-r5-bfrac-hold` (1,632 points): (42.5, 50), (150, 170.5), (700, 90), (55, 400), (2500, 3000), (80, 81), the
    same design; built and filled with tune, scored once after the freeze.
  - `t3571-r5-bfrac-fresh` (1,200 points): log-uniform (a, b) in [40, 5000], one x per shape at z uniform in
    [-8, -0.2] (`mkprobe.py --fresh`, seed 35719), built and filled after the freeze commit.
  - Every point is a bfrac point of TOMS's dispatch (`mkprobe.dispatch`). The z levels set bfrac's term count (5 to
    42), which section 1 item 1 uses.
- **Hold-out plan** (committed in 65189a46d, before any hold-out scoring): split by shape as above; margin: hold-out
  exact rate within 10 points of tune and above CR's hold-out count; a fresh sweep after the freeze with its Gate A
  envelope count.
- **Exact references**: `exact_r.py` (bfrac's fraction at 40 digits on the stored inputs), `runs.py` (per-run
  decomposition, exact density), `score_final.cr` (I_x at exact λ and y). Caches (`exact_r_*.tsv`, `runs_*.tsv`) are
  git-ignored.
- Old data: w2's (100, 120) run (t2464-w2f-xruns-1 rows 14337-16384), read by `data.py`.

## 2. How the model was found (all on tune or the old (100, 120) run)
1. **TOMS's recurrence does not share Excel's noise** (`decomp.out`, `loops.out`). On the (100, 120) run Excel's
   implied fraction error has sd 3.76 and mean +3.00; TOMS bfrac in doubles has its own noise (sd 3.2) but correlates
   +0.04 with Excel's. Small changes to the loop decorrelate completely (TOMS vs no-rescale +0.06, vs Lentz +0.18), so
   only the exact sequence can show a signal. The +3.0 mean is not a truncation length (`trunc.out`: after N terms the
   bias alternates +134.6, -14.4, +1.47, -0.14).
2. **TOMS-loop searches** reach corr 0.22 at best: 96 FMA-contraction masks × 3 eps (`bfrac_fma.c`, `score_fma.out`:
   best 156/1,024, the uncontracted one), 13,824 op-order variants (`search.c`, `search_xruns.out`: best 193/1,024;
   a shared 1/bnp1 in the rescale is the one choice that lifts the correlation, 0.02 → 0.12 mean), and 2,592 loop ×
   term-order × assembly variants (`search2.c`, `search2_xruns.out`).
3. **Lentz loops.** Lentz on TOMS's terms correlated more (0.13). On the new short runs, `search3.c`
   (`search3_tune.out`) put Numerical Recipes' start (f = tiny, then the partial (1, b0), output f = r itself) at
   corr 0.64-0.89 and 83 of 96 exact on (100, 120) z = -7, against <= 0.2 for every TOMS loop. `search4.c` checked
   the other starts (tiny 1e-300, DBL_MIN, f = 1/b0): none better.
4. **Terms in closed form.** `termswap.out`: replacing TOMS's b0 = c/c1 by its correctly rounded value alone lifts
   (40, 45) z = -7 from 15 to 38 of 96. `score_hyb_tune.out`: b0 = c·(a/(a+1)) with (a+n)/(a+2n+1) in β gives 735 of
   1,536 (TOMS's c/c1 and (t+1)/(c1+2t): 421); `score_b0_tune.out` keeps c·(a/(a+1)) over six other b0 forms
   (923-992 against 1,236 with the final terms).
5. **The stop rule** (`fixedn.out`, `score_eps_tune.out`): with N forced, the best N per run is the first term whose
   |C·D - 1| is at most about 2e-15; runs at 2.4e-15 and 2.9e-15 go on. eps 1e-15 scores 1,047, 10·DBL_EPSILON 1,236,
   3e-15 1,165 (final terms).
6. **α and β forms.** `alpha_enum.py` scored 11,592 left-associated orders of the factors (u, v, w, x),
   (u, v, n·(b-n), x, x), (u, v, n·x·(b-n), x), (u, v, n, b-n, x, x) with the two divisions by s anywhere
   (`alpha_enum4_tune.out`: v/s/s·u·n·(b-n)·x·x 1,236; next 1,084), and 1,080 TOMS-style p·(p+c0)·e·e·w·x orders
   (`alpha_enum3_tune.out`, best 840). `beta_enum.py` scored 980 β forms (`beta_enum2_tune.out`): w = n·x·(b-n) (R's
   order) beats n·(b-n)·x (1,236 against 1,200); the X and E variants tie or lose. A correctly rounded β instead of
   the modelled one drops 926 to 502 (`termswap2.py`), so β's roundings are Excel's.

## 3. Negatives (counts on the scorer they name)
- TOMS bfrac's forward recurrence in doubles, every storage, FMA and order variant above: best on the
  (100, 120) run is 193/1,024 (corr <= 0.22); Steed's algorithm 194-198, backward recurrence 160 (`loops.out`).
- Lentz from b0 (f = b0, D = 0) instead of NR's tiny start: corr <= 0.22; FMA in the D or C update, D kept
  un-inverted (C/D), C = β + α·(1/C): all lower (`search3.c`, `score_loop_tune.out`).
- x87-extended terms, FMA-contracted β (c + n·yp1 or e·X fused): 363-425 against 421 for the plain TOMS terms in the
  Lentz loop (`score_lentz.py` opt variants); a correctly rounded α or β: lower than the modelled forms.
- Assembly fl(fl(P·fl(xy))/F), fl(fl(fl(P·x)·y)·r), fl(P·fl(fl(xy)·r)): lower (`search2.c` AS1-AS3).
- A brcomp other than the worksheet density: refuted by section 1 item 1 (no correlation with the density error).

## 4. Open (T-3813; the engine port is T-3812)
- **One rounding per term.** Misses grow with the term count (section 1 item 4). `nudge.py` (`nudge.out`, on the
  earlier α form) finds a one-ulp nudge of a single β_n or α_n that fixes 409 of 468 misses, β about three times as
  often as α, spread over n. The next step is the same enumeration on the fresh sweep's 20+ term points with β's
  sub-expressions as products in other orders (e.g. (a+n)·X/(a+2n+1) with X's sum order), and checking whether α's
  v/s/s is really v/(s·s) rounded differently at large s.
- The swapped call (x above the mean) is not probed: whether its λ is |fl(a - fl((a+b)x))| is open (T-3813).
- The (100, 120) cells stay the weakest of the tune shapes at z = -1 (28/96); a = 100 makes a/(a+1), 1/a and n/a
  inexact together, so a term using a/(a+1) or (a+n)/(a+2n+1) elsewhere is the first candidate.

## Scripts
| script | purpose | output |
|---|---|---|
| `data.py` | old (100, 120) run from t2464-w2f-xruns-1 | |
| `exact_r.py`, `runs.py` | exact fraction and density; per-run decomposition | `runs.out` |
| `mkprobe.py` | tune, hold and fresh probes | |
| `decomp.py`, `loops.py`, `trunc.py`, `sens.py` | TOMS-loop noise, loop families, truncation, input sensitivities | `*.out` |
| `bfrac_fma.c` + `score_fma.py`; `search.c`, `search2.c`, `search3.c`, `search4.c` + `search_in*.py` | C searches | `score_fma.out`, `search*_*.out` |
| `lamab.py` | λ for a > b | `lamab.out` |
| `lentz.py` | Lentz loop and term forms (exploration) | |
| `score_lentz.py`, `score_hyb.py`, `score_expl.py`, `score_loop.py`, `score_eps.py`, `score_b0.py`, `alpha_enum.py`, `beta_enum.py` | ranked structural choices | `*_tune.out` |
| `termswap*.py`, `fixedn.py`, `nudge.py`, `resid.py` | diagnostics | `termswap.out`, `fixedn.out`, `nudge.out` |
| `model.py` | the frozen model | |
| `score_final.py`, `score_fresh.py`, `score_extra.py` | scores | `score_final.out`, `score_fresh.out`, `score_extra.out` |
