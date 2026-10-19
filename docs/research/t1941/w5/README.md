# W5: the common-factor hypothesis for the GAMMALN near-1 kernel (T-1941)

Target: A = fl(N(x)/D(x)) on [0.7, 1.5), res = fl((x-1)*A), raw-x Horner, every op fl53(fl64(.)).
Hypothesis under test: Excel's coefficients are round(N*·F, D*·F), where N*/D* is a reproducible
best-fit rational and F is a low-order common factor.

**Status: final, negative. Nothing beats c5_f99t** (47.25% on all 980,655 points in [0.7, 1.5); far 31.8%, near-1
74.6% on the `metrics.py` samples).

## Summary
1. **A blind scan over F cannot work: the exact-score basin is about 1e-14 wide** (positive control, `pc1.py`).
   - Synthetic data come from round(2^-13·P8·(1+a0·x), 2^-13·Q8·(1+a0·x)), with P8/Q8 the (8,8) minimax.
   - The true a0 scores 100%. An a0·(1+1e-15) offset scores 97-99.8%, and 1e-14 scores 71-79%.
   - 1e-13 already scores 36-38%, the random-smooth level, and so does every offset up to 1e-4.
   - So F must be known to about 1e-14 relative before any exact-match scan or integer polish can see it. A
     scan over a in [-0.02, 0.02] would need about 1e12 points.
2. **The coordinator's doublet test is negative** (`ascan.py`). The parents were the (8,8) relative minimax of
   lgamma(x)/(x-1) on [0.7, 1.5], and the (8,8) minimax of c5_f99t's own function.
   - Each was multiplied by (1+a·x) and rounded, over 6,003 values of a in [-0.03, 0.03], plus ±1/353.42.
   - The best scores were far 0.30/0.33 and near 0.41/0.37, against c5's 0.30/0.56 on the same sample. That is
     random-smooth level, as point 1 predicts.
3. **No hidden common-factor offset near c5_f99t** (`nullscan.py`).
   - The test moves c5 along its four sloppiest function-preserving directions: SVD of the relative coefficient
     sensitivities on [0.7, 1.5], singular values 4.7e-22 to 2.7e-16.
   - Steps were 2,001 per direction, at ±1e-10 and ±1e-8 relative.
   - c5 is a sharp local maximum on every direction (`sloppy.py`): near-1 0.75 drops to 0.70 at 1e-10 and to 0.48
     at 1e-8. The function changes by less than 1e-27.
4. **Corollary: the near-1 data pin c5's sloppy coordinates to about 1e-10 relative.**
   - A function-level fit cannot do this; the data do it through rounding.
   - Excel's rational is therefore known well enough to read off invariants that any common factor leaves alone.
     See the fingerprint section below.
5. **Structural facts that remove variants of the hypothesis:**
   - *Common factor means a common root.* round(N*F, D*F) gives an N and a D that share F's roots, to rounding.
     - c5's roots: N: -0.0361, -0.169, -0.439, -0.937, 2.000, -2.06, -4.90, -16.1, -353.4.
       D: -0.0209, -0.1165, -0.320, -0.723, -1.476, -3.108, -7.45, -27.0, +397637.
     - No pair is shared. The only "spare" pair is a lone N zero at -353 against D9 ≈ 0. That is a (9,8)
       shape, not a doublet.
     - Near the interval, c5 has 5 negative zeros and 6 negative poles, the same pattern as the (8,8) minimax.
       The (9,9) minimax has 6 zeros.
   - *F in t = x-1 is the same family as F in raw x.* D0 is exactly 2^-13 (W1). F must then have F(0) a power of
     two, so F_t(x)/F_t(0) is a raw-x factor with constant term 1.
   - *Truncating N*·F back to degree 9 is excluded.* Dropping the x^10 term a·N9·x^10 changes N by about
     a·2.3e-6·x^10, relative 1e-8 for a = 0.005. Only |a| < 1e-11 survives, and that is invisible.
   - *Normalisation.* D0 = 2^-13 exactly, so the parent has D0 = 1 (or a power of two), and N0 is free:
     N0/D0 = -5.4058… is not a power of two. An "N0 = 1" normalisation is excluded.
6. **Excel's function is not a high-precision minimax of lgamma** (`fcheck.py`, `deltaE.py`).
   - It sits about +1.5 ulp of A above lgamma(x)/(x-1) for x < 1.08 and about -1.5 ulp below for x > 1.09.
     c5's exact function shows +1.9e-16 at 0.9 and -2.3e-16 at 1.2.
   - Random 0.5-ulp coefficient rounding gives only 4e-17 rms (max 1.4e-16).
   - The step at 1.08 cannot come from any coefficient perturbation either: `lptest.py` needs δ ~ 1e7 ulps at a
     tolerance of 1e-16. So per-bin means of Excel minus model still carry systematic stage-rounding bias, and
     this function-level test is inconclusive. Recorded as a lead, not a result.

## Generator fingerprint: x=0 invariants
Any common factor cancels in N/D everywhere in the complex plane. Excel's rational therefore has the same Taylor
coefficients at x = 0 as its parent N*/D*. Rounding the coefficients moves them by only about 4e-16
(`control.py`). These coefficients are pure extrapolation, 0.7 below the interval, so they fingerprint the
generator: degree, interval, weight and target.

- **Excel's values, from c5_f99t** (`taylor0.py`): R0 = -5.40585248391, R1 = 134.942282379,
  R2 = -5937.32019206, R3 = 281011.5087.
  - W1's (9,10) sets agree to 1.5e-10 (C1) and 1e-10 (the first candidate).
  - By point 4, these are pinned to about 1e-10 relative.
  - Caveat: near 1, compensated single-stage moves of a few ulps are invisible (`control2.py`). The pin is at the
    1e-10 level, not the bit level.
- **Validation:** an (8,8) relative minimax on [0.5, 1.5] gives -5.63918 / 171.3366 / -9493.1. Cody's published
  ALGAMA near-1 set (`cody.py`) gives -5.63923 / 171.3447 / -9494.0, a match to about 1e-5 (Cody's form and
  weighting differ slightly). So the invariants do identify generator settings.
- **Sweep** (`sweep.py`, `sweep_*.txt`): degrees (7,7), (8,8), (8,9) and (9,8), relative weight, a in
  {0.5, 0.6, 0.65, 0.7, 0.75} and b in {1.25, 1.5, 1.75}. The values on [0.7, 1.5] are:

  | parent | R0 | R1 |
  |---|---|---|
  | (6,6) | -4.930 | 82.1 |
  | (7,7) | -5.214 | 110.5 |
  | (8,8) | -5.457 | 142.2 |
  | (8,9) | -5.567 | 159.2 |
  | (9,8) | -5.569 | 159.6 |
  | (9,9) | -5.679 | 178.7 |

  Cody's set gives -5.639. None matches Excel's -5.406.
- **1-D solves** (`solve2.py`, `runfix*.sh`, `fix_*.txt`): hold a, solve b so that R0 matches, then read R1-R3.

  | family (target, degree, weight) | a | b matching R0 | dR1 | dR2 | dR3 |
  |---|---|---|---|---|---|
  | A (8,8) rel | 0.60 | 1.966060 | -8.4e-4 | -2.6e-3 | -4.8e-3 |
  | A (8,8) rel | 0.65 | 1.811954 | -6.7e-4 | -2.1e-3 | -3.8e-3 |
  | A (8,8) rel | 0.70 | 1.670448 | -5.4e-4 | -1.7e-3 | -3.1e-3 |
  | A (8,8) rel | 0.75 | 1.548006 | -4.4e-4 | -1.4e-3 | -2.5e-3 |
  | A (8,8) rel | 0.80 | 1.442510 | -3.6e-4 | -1.1e-3 | -2.1e-3 |
  | A (8,8) abs in A | 0.70 | 1.709362 | -5.5e-4 | -1.7e-3 | -3.2e-3 |
  | A (8,8) abs in lgamma | 0.70 | 1.685321 | -5.6e-4 | -1.8e-3 | -3.2e-3 |
  | g (8,8) rel | 0.70 | 2.073733 | -2.1e-4 | -6.5e-4 | -1.2e-3 |
  | g (8,8) rel | 0.75 | 1.898748 | -8.9e-5 | -2.8e-4 | -5.1e-4 |

  - g = lgamma/((x-1)(x-2)), with N = (x-2)·P. The coordinator has since withdrawn the exact-root-at-2 claim:
    every good approximant has that zero. The g rows serve as another weighting.
  - Several families cannot reach Excel's R0 with a in {0.5, 0.7} and b up to 1.95-2.4:
    (9,8), (8,9) and (9,9) relative in A, A (8,8) relative at a = 0.5, (7,7), and g (8,9) on 1-D.
- **The mismatch is nearly one-dimensional.** In every family dR2 ≈ 3.15·dR1 and dR3 ≈ 5.75·dR1, so R0 and R1
  carry almost all the information. That is why the 2-D Newton is ill-conditioned.
- **2-D solves** (`solve3.py`, `s3_*.txt`): solve (a, b) so that R0 and R1 both match, then test R2 and R3:
  - **g (8,9) relative.** Both match exactly at a = 0.6766924, b = 2.8721896 (E = 4.2e-19). The residuals are
    dR2 = +5.7e-6 and dR3 = +1.7e-5, about 1e4-1e5 times the pin. Excluded, and the interval is unnatural anyway.
  - **A (8,8) relative.** Stalls at a = 0.8258, b = 1.3928 with dR0 = 6e-5, dR1 = 8e-6. dR2 = -3.7e-4 and
    dR3 = -9.1e-4 there; no solution in reach.
  - **g (8,8) relative.** Stalls at a = 0.7634, b = 1.8569 with dR0 = 1.1e-5, dR1 = 1.7e-6. dR2 = -6.9e-5 and
    dR3 = -1.7e-4 there. Not fully converged, but R2 would need a sensitivity about 6 times dR0 just to cancel.
  - **A (8,8) absolute.** Diverges.
- **Precision checks.** remez2's invariants are converged to 16 digits: dps 40, 50 and 60 agree (`conv.py`).
  The older `remez.py` values in `sweep_*.txt` are good to about 1e-7.
- **Verdict:** no relative or absolute minimax of lgamma/(x-1), or of lgamma/((x-1)(x-2)), at degrees (7,7)-(9,9)
  on any interval tried matches Excel's x=0 fingerprint. Excel's parent, if any, is not one of these
  generators, even allowing any common factor F. This agrees with the playbook's ERFC finding: Excel's
  coefficients are off any reproducible Remez point.

## Conclusions for T-1941
- The common-factor hypothesis is unfalsifiable as stated: any (9,9) can be written as some (8,8)·F to 1e-17 on
  the interval. It is testable only when the parent is reproducible, and every reproducible parent tried fails
  the x=0 fingerprint by 1e-5 or more.
- Even with a correct parent, F would need about 1e-14 relative precision before exact scoring sees it (point 1).
  No coarse objective bridges that gap: spectral overlap is blind below 1e-4 to 1e-5.
  - The only bridge is the playbook's phase-lattice global solve on far runs.
  - Or the near-1 stage sums S_j(1): they are linear in F's parameters, but pinned only to about 1e-10.
- Useful by-products for the next worker:
  - The x=0 invariants R0-R3 as a cheap generator fingerprint, pinned to about 1e-10 by near-1 data.
  - c5 is a sharp local optimum along every function-preserving direction.
  - The doublet and truncation variants are excluded.
  - D0 = 2^-13 forces the D0-normalised form.

## Tooling
- `sc.c` / `sc.py`: an exact x87 scorer (OpenMP), with the same arithmetic as W1's `simA.c`, and the data loader
  (`ROWS` = `extract_runs.py` output). With 980,655 unique points in [0.7, 1.5), c5_f99t scores 47.25% (31.1%
  for |x-1| > 0.05). W1's 51% used 512-per-run sampling.
- `metrics.py`: far exact rate (|x-1| > 0.02, 120k sample), near-1 exact rate (|x-1| < 1e-3, 60k sample) and
  per-run bias.
- `remez2.py`: a robust rational Remez in mpmath, with Newton on (p, q, E) and real golden-section extrema.
  `remez.py` is the first version, used for `parents.py` and `sweep.py`.
- `taylor0.py`: Taylor coefficients of N/D at x = 0. `stages.py`: Horner stage sums at x = 1.
- `solve2.py` / `solve3.py`: 1-D and 2-D fingerprint solves; `conv.py`: Remez convergence check.
- `spec.py`: a phase-free spectral overlap (c5 gives 0.857). It is blind even to 1e-5 coefficient changes,
  because within a 2,048-ulp run a function change is just a per-run offset. No objective sits between this
  coarse level and the 1e-14 exact basin.
