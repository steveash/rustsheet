# Round 9: the [0.5, 4] ERFC kernel

- `gen910.txt` holds the generator: a relative minimax (9,10) of erfcx on [0.5, 4], with D0 = 1, from 90-digit
  mpmath Remez. Coefficients are highest degree first, in the format `N k c...` / `D k c...`.
- `mid_check.c` is the exact x87 checker: stored Horner, R = fl87(N/D), ERFC = fl87(E*R) with Excel's E column.
  - Build it with `gcc -O1 -mfpmath=387`.
  - Run it as `mid_check coef.txt pts.txt`, where `pts.txt` holds lines of `x_hex erfc_hex exp_hex case`.
  - Make `pts.txt` from all `kmyst-erfc-*` cases with 0.5 < x <= 4. That is 4,621,791 points at commit 27c7a260.
- Generator score: 31.3% exact, the smooth-model level. The structure evidence is in
  `docs/excel-semantics/statistics.md`.
- `cand2.txt` comes from the global-opt worker's phase-lattice solve. Each stored product's sawtooth phase is
  linear in the integer mantissa errors, so the errors are solved jointly as a closest-vector lattice problem.
  - It matches 3,961,381 / 4,621,791 points (85.71%): 99.4% on [0.5, 0.75), falling to 52% on [3.75, 4].
  - Excel's doubles are 1.5e-9 to 1.6e-8 relative from the generator (7e6 to 1.3e8 ulps), so the generator is a
    structural guide only, not an ulp-level start.
- `erfc_mid_coef.txt` holds the BIT-EXACT [0.5, 4] coefficients, highest degree first.
  - `mid_check.c` gives 5,694,938 / 5,694,943; all 5 misses are xlsx misparse inputs that match at the
    neighbouring double.
  - Found by the global-opt worker's phase lattice plus top-stage chain solves, and cross-checked by the D
    worker's peel.
