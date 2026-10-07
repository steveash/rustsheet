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

Sweep (T-1004, `probes-numeric/m9-bessel`, excel-graph; 364 cells: x in {8..100} x orders 0-10 for both functions):
- For x in [8,100] Excel's values are off the true values at about 1e-9 to 1e-8 relative (e.g.
  `BESSELJ(30,1)` = -0.11875106091123117, `BESSELY(8,0)` = 0.22352148924968093). The engine's
  fdlibm fallback matches only 164 of 350 compared cells within the harness tolerance.
- A fractional order truncates: `BESSELJ(8,9.99)` equals `BESSELJ(8,9)`, `BESSELY(30,1.99)` equals `BESSELY(30,1)`.
- Large orders: `BESSELJ(x,n)` is `0` (not an error) for n = 1000, 10000, 100000 and 1000000 (x = 10, 50), and
  `BESSELY(x,n)` is `#NUM!` for those n. The engine matches this up to n = 100000 but gives `#NUM!` for
  `BESSELJ(10,1000000)` (see "BESSEL order cap"). The cap itself between 101 and 1000 for J is not visible
  since J underflows to 0; Y is `#NUM!` from n = 1000 (the point where it overflows is not probed).
Status: VERIFIED for these cells; fitting the algorithm is T-0908.

Provenance: the coefficients are the published Numerical Recipes (2nd ed., §6.5) rational
approximations, reproduced only because Excel's output matches them bit-for-bit.

## BESSEL order cap
Rule: RustSheet returns `#NUM!` for BESSELJ and BESSELY orders above 100,000 to bound per-cell work on
untrusted input (CLAUDE.md directive 5). Excel gives `BESSELJ(10,1000000)` = `0` (probes-numeric/m9-bessel, T-1004), so the engine differs above 100,000 for J.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/engineering/transcendental/bessel_excel.rs (`MAX_ORDER`)
