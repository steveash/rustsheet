# Round 9: the [0.5, 4] ERFC kernel

- `gen910.txt` holds the generator: a relative minimax (9,10) of erfcx on [0.5, 4], with D0 = 1, from 90-digit
  mpmath Remez. Coefficients are highest degree first, in the format `N k c...` / `D k c...`.
- `mid_check.c` is the exact x87 checker: stored Horner, R = fl87(N/D), ERFC = fl87(E*R) with Excel's E column.
  - Build it with `gcc -O1 -mfpmath=387`.
  - Run it as `mid_check coef.txt pts.txt`, where `pts.txt` holds lines of `x_hex erfc_hex exp_hex case`.
  - Make `pts.txt` from all `kmyst-erfc-*` cases with 0.5 < x <= 4. That is 4,621,791 points at commit 27c7a260.
- Generator score: 31.3% exact, the smooth-model level. The structure evidence is in
  `docs/excel-semantics/statistics.md`.
