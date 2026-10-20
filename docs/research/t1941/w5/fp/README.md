# T-1941 W5 fp/: coefficient-free structure fingerprint (W2's method, re-implemented as a cross-check)

Method (`fp.py`):
- For each point, the target quotient A* is the mean of the doubles A that reproduce Excel's y through the outer
  ops.
- The function is fitted by weighted linear LS of N − A*·D in a Chebyshev basis on the data interval (D's T0
  coefficient fixed; iterative refinement in long double, rcond 1e-30, best iterate kept).
- The residual rms in ulp(A) is the rounding noise of the op structure. It is calibrated by simulating candidate
  structures with a close real coefficient set and refitting the same way.
- Lessons from building it:
  - The default lstsq cutoff kills needed directions: A·T_k is nearly expressible by the T_k, so use rcond ≈ 0.
  - Monomial fits are too ill-conditioned.
  - Calibrating with the fitted rational normalised to D(0) is unstable (D(0) is undetermined along null
    directions), so the calibration uses a real candidate set (ss_fc, AN2, lat1) instead.

## K3 (`k3fp.py`, rows6, 200k points; reproduces W2)
Real rms 1.025 / median 0.669. Simulated with ss_fc: all stored 1.028 / 0.669; stored below 3 on both 0.855;
N<2 with D 80-bit 0.596; all 80-bit 0.335. **K3 is plain stored.**

## K1 (`k1fp.py`, `k1fp_rep.py`; G = fl(N/D) on [1,2), (7,7), D0 = −1/16; g_rows from g_rows.py, 3.59M unique points)
Three 200k subsamples; calibration with AN2 and with lat1:
- real 1.042–1.045 (median 0.700);
- all stored 1.043–1.044;
- W4's x87-top mask (N top 5, D top 4 in 80-bit = stored below 2 / below 3): 0.970–0.974;
- N top 4 / D top 3 80-bit: 1.028–1.030;
- N top 6 / D top 5 80-bit: 0.883; all 80-bit: 0.289.
Spread across subsamples and calibration sets is ±0.002. **K1 is plain stored.** W4's mask is excluded (~35
spreads); even the mildest mask tested is ~7 spreads off. Mask lifts on K1 are coefficient errors hidden by unstored
ops, as on K3.
