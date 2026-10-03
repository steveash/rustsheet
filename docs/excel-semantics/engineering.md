# Engineering functions

## BESSELJ / BESSELY (T-0900)

Evidence: Excel-cached cells in `ironcalc-calc/ENGINEERING/BESSEL.xlsx`
(`make eval-show CASE=ironcalc-calc/ENGINEERING/BESSEL.xlsx`).

- For `|x| < 8` Excel is **not** fdlibm. Its values (accurate only to ~1e-8, e.g.
  `BESSELJ(0,0) = 1.00000000283141`) are bit-identical to the Numerical Recipes rational
  approximations `bessj0/bessj1/bessy0/bessy1` (with the truncated constant `0.636619772`),
  upward recurrence for higher orders, and Miller downward recurrence for `J_n(x)` when `n >= x`.
  `Y_n` recurrence is only bit-exact as `b * (2j * (1/x)) - a`.
  Implemented in `functions/engineering/transcendental/bessel_excel.rs`.
- For `|x| >= 8` (cells at x = 10 and 32) Excel differs from both fdlibm and the NR asymptotic
  branch (e.g. `BESSELJ(10,1)` = 0.04347224965 vs true 0.04347274617), so another algorithm is
  used there. UNRESOLVED; tracked in T-0908. fdlibm is kept for `|x| >= 8`.

Provenance: the coefficients are the published Numerical Recipes (2nd ed., §6.5) rational
approximations, reproduced only because Excel's output matches them bit-for-bit. Orders above
100_000 return `#NUM!` (Excel's real cap is UNVERIFIED) to bound per-cell work on untrusted input.
