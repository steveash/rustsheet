# T-1778 round 2: the (4,5) rational erf kernel

- `best_coef.txt`: the best global coefficient doubles for x < 0.5 (P quartic, Q quintic in z = fl(x*x)). Under
  `sc.c` mode 0300 (x87 Horner with every operation stored to double, co = fl(x*N), then an 80-bit divide rounded
  to double), they reproduce 770,405 of 771,176 oracle points (99.90%). Every miss is 1 ulp, with Excel's N or W
  one ulp lower near a rounding midpoint.
- `predicted_coeffs.txt`: coefficients slaved to A, W0, p1 and p2 through a constrained (4,5) minimax. They
  match the fit, but no source construction (minimax, Padé, Chebyshev interpolation) reproduces q2.
- `sc.c`: the exact scorer, which reads the run files written by `../extract_runs.py`.
