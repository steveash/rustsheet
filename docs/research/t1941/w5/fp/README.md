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
Spread across subsamples and calibration sets is ±0.002, but AN2 and lat1 are close relatives. W2 measured on K2
that coefficient bits alone move the calibration by about ±0.03, so a structure call needs a gap well above 0.03.
W4's x87-top mask (0.07 below real) is excluded. The mildest mask (N top 4 / D top 3, 0.015 below) is NOT
excluded. **K1 is plain stored or has at most a small 80-bit head.** Mask lifts on K1 are coefficient errors hidden by unstored
ops, as on K3.

## K1 regional fingerprint (W4's request; `k1fp_region.py`, `k1fp_region.log`)
A (7,7) Chebyshev LS fit per region; real vs planted plain1 (W4's best K1 set, all stored); 3 subsamples of 150k.
| region | exact(plain1) | real − planted rms (3 seeds) | medians real / planted |
|---|---|---|---|
| [1.20,1.70) | 0.876 | −0.028, −0.006, −0.061 | 1.135/1.177, 0.633/0.638, 0.707/0.738 |
| [1.84,1.97) (the dip) | 0.412 | −0.005, −0.078, −0.050 | 0.831/0.832, 0.847/0.903, 0.830/0.842 |
| [1.00,1.20) | 0.874 | +0.017, −0.023, +0.039 | |
| [1.70,1.84) | 0.822 | +0.043, −0.007, +0.243 (seed 3 fit) | |
| [1.97,2.00) | 0.634 | +0.023, −0.007, fit failure | |
Narrow-region fits are less stable (rms varies by seed), so compare within a seed. In the dip, Excel's noise is NOT above
a stored chain's; it sits at or slightly below plain1's. So the model-free test finds no structural difference in
[1.84,1.97). The low exact rate there points at coefficients (plain1 is wrong where that region is sensitive),
not a different computation.
