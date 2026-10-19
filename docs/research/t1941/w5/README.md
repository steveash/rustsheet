# W5: the common-factor hypothesis for the GAMMALN near-1 kernel (T-1941)

Target: A = fl(N(x)/D(x)) on [0.7, 1.5), res = fl((x-1)*A), raw-x Horner, every op fl53(fl64(.)).
Hypothesis under test: Excel's coefficients are round(N*·F, D*·F), where N*/D* is a reproducible
best-fit rational and F a low-order common factor.

Status: work in progress. Nothing beats c5_f99t yet.

## Tooling
- `sc.c` / `sc.py`: an exact x87 scorer (OpenMP), with the same arithmetic as W1's `simA.c`, and the data loader
  (`ROWS` = `extract_runs.py` output). With 980,655 unique points in [0.7, 1.5), c5_f99t scores 47.25% (31.1%
  for |x-1| > 0.05). W1's 51% used 512-per-run sampling.
- `metrics.py`: far exact rate (|x-1| > 0.02, 120k sample), near-1 exact rate (|x-1| < 1e-3, 60k sample) and
  per-run bias. c5_f99t gives far 0.318 and near 0.746.
- `remez2.py`: a robust rational Remez in mpmath, with Newton on (p, q, E) and real golden-section extrema.
  `remez.py` is the first, less robust version.
- `taylor0.py`: Taylor coefficients of N/D at x = 0. `stages.py`: Horner stage sums at x = 1.
