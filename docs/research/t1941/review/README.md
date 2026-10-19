# T-1941 reviewer scripts (Fable 5.1, 2026-10-19)

Companion to `../HANDOFF.md` section 9. Run from this directory after dumping the rows from the repo root:

```
python3 ../g_rows.py g_rows.txt        # (x, GAMMA(x)) rows, 1 <= x < 2 (clean =GAMMA(A<row>) cells)
python3 ln_rows.py ln_rows.txt         # (x, GAMMALN.PRECISE(x), LN(x)) rows, formula-filtered
```
(`ln_rows.py` and `../g_rows.py` glob `eval/suites/...`, so run them from the repo root and move the dumps here.)
Needs numpy (x87 longdouble) and mpmath.

| Script | What it showed |
|---|---|
| `k1lib.py`, `k23lib.py` | exact x87 simulators: K1 (any (n,n) raw-y N/D, three rounding schemes, stage flips); K2 grid_a; K3 c5_f99t |
| `k1_scheme_bins.py CAND` | AN2: stored 78.2%, product-64 49.8%, all-64 38.0%; bin means of (Excel - sim) up to +0.28 ulp at 1.45, -0.16 at 1.88 |
| `k1_missrate_blame.py CAND` | per-bin miss rate tracks the mean residual except 1.80-1.97; single-stage blame is uninformative (last stages "fix" anything) |
| `k1_dispersion_crossings.py CAND` | std of (Excel - sim) 0.37 near 1, 1.6 at 1.85-1.95; AN2's binade crossings |
| `k1_weakspot_hist.py CAND` | 1.78-1.97 errors are +-1..+-3 ulps, lag-1 autocorrelation about -0.1 |
| `k1_flipscan.py CAND` | no single +-1..+-4 ulp flip removes the weak-spot dispersion |
| `k1_two_scale_lsq.py CAND K` | linearised function correction of AN2: real solution needs 1e5-1e6 ulp moves (null directions) |
| `k1_ridge_sweep.py CAND` | every integer move that flattens the residual drops the exact rate (78% to 45-67%) |
| `k1_degree_modelfree.py 100` | (7,7) fits Excel's measured K1 function at chi2/dof 1.42; (6,6) 5.8e5; (8,8) 3.8; the rounded (7,7) LSQ set scores 28.3% exact |
| `k23_resid_bins.py` | bin means/std of (Excel - sim) for c5 (K3) and grid_a (K2) |
| `k23_bias_curve.py K2|K3` | model-free Excel-minus-lgamma curves: `bias_K2.txt`, `bias_K3.txt` (staircases) |
| `k23_degree_modelfree.py K2|K3` | rational fits to the raw staircase curves fail for every degree (chi2/dof >= 237) |
| `k2_plateau_sim.py` | grid_a's intermediate binade crossings on (2,3] and the simulated chain's own staircase (same step positions) |
| `k3_crossings.py` | c5's intermediate binade crossings on [0.7,1.5) |
| `k3_continuity_15.py` | (Excel - c5) and model-free curves across 1.5 |
| `gl_15_2_glue.py` | GAMMALN(x)+LN(x) is a 53-bit double on 7.4% of (1.5,2] points: not a stored-value subtraction |
| `gl_15_2_variants.py` | composition variants through grid_a (blind test, all about 3%) |
| `norm_survey.py` | max coefficient, D0 and endpoint values of the three candidates |
| `erf_decimal_digits.py` | bit-exact ERF/ERFC coefficients need 16-17 decimal digits: no short-literal prior |

Simulated grid_a staircase (sim minus grid_a's exact rational, ulps of output, 0.005 bins; `k2_plateau_sim.py`):
+0.3 on (2, 2.277), -0.2..-0.5 on (2.277, 2.34), -1.1 on (2.35, 2.40), -0.4 on (2.40, 2.46), +0.4 on (2.46, 2.60),
+1.4 on (2.606, 2.655), +0.3..+0.5 on (2.66, 2.72), +0.9 on (2.73, 2.78), +0.8..+1.0 on (2.78, 3]. Compare the
Excel curve in `bias_K2.txt`: same step positions, different heights.
