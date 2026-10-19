# W1 tooling for the GAMMALN near-1 kernel (T-1941)

- `simA.c` / `csim.py`: the exact x87 simulator of A = fl(N(x)/D(x)), raw-x Horner, every op fl53(fl64).
- `load2.py`: splits the `extract_runs.py` rows into runs of consecutive doubles.
- `exsearch.py`: the dataset and scoring helpers.
- The exact coordinate search `c1search5.py` (stage moves in each stage's own ulp at x = 1, D x N pair moves,
  same-chain pairs, coefficient moves, p0 moves) was sent as text and lives in W1's session report. It is
  summarised in `../README.md`.

How c5_f99t was made:
1. An mpmath (9,9) least-squares fit, with q0 = 2^-13 fixed, to Excel A = model A + per-run mean(Excel - model),
   plus Taylor a0..a3 constraints on the lower stages at x = 1.
2. Then `c1search5 in.pkl out.pkl 2400 4096 3`.
