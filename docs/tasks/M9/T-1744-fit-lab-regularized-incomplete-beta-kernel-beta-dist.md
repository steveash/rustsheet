---
id: T-1744
title: "Fit lab: regularized incomplete beta kernel (BETA.DIST, F.DIST, T.DIST, BINOM.DIST) against the oracle sweeps"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A fit-lab fit for the regularized incomplete beta function I_x(a, b) that names the algorithm Excel
uses, or the best candidate, with every candidate's counts recorded. This is rank 3 in the numeric kernel
map: 1,376 near-miss cells, 1,375 of them in this kernel only.

## Context
Owner request (2026-10-16). Decision 0056 says numeric work is kernel-first and every fit is a
fit-lab comparison of named candidates against an oracle sweep (docs/architecture/fit-lab.md,
`tools/src/rustsheet_tools/fitlab/`). The fit lab already found PV/FV/NPER, XIRR, IRR and Excel's `^`
this way (T-1524, T-1525, T-1533; implemented by T-0901 and T-0916). It has no fit yet for this kernel.
- docs/research/numeric-kernel-map.md, rank 3: the engine calls `statrs` `beta_reg` from
  `statistical/beta.rs`, `t_dist.rs`, `fisher.rs` and `binom.rs`. 1,084 cells are `close`, not ulp
  misses, so Excel uses a different algorithm, not only different rounding.
- Sweeps (probes-numeric, excel-graph):
  - `m9-beta-dist-a*` (a = 0.0001 … 100);
  - `m9-f-dist`, `m9-f-dist-rt`, `m9-t-dist-cdf`, `m9-t-dist-rt`, `m9-t-dist-2t`;
  - `m9-legacy-betadist`, `m9-legacy-fdist`, `m9-legacy-binomdist`, `t1048-tdist`, `t1049-fdist`,
    `t1049-fdistrt`.
- Candidates to try, each in doubles and with Excel's operators (`exact.py` roundings):
  - Lentz and modified-Lentz continued fractions (Numerical Recipes `betacf`) with their tolerances
    and the x > (a+1)/(a+b+2) symmetry switch;
  - TOMS 708 (Didonato and Morris `bratio`), Cephes `incbet`, and AS 63 (Majumder and Bhattacharjee);
  - power series for small x, and the direct sum for integer a, b (binomial and t with integer df);
  - the log-gamma prefactor by `lgamma` against Excel's GAMMALN (fit lab `gammaln-int`).
- Owner tasks that implement the winner: T-0970 (BETA.DIST/F.DIST/F.TEST) and T-0969 (T.DIST/T.TEST)
  now depend on this fit. T-0927 (BINOM.DIST family) gains from it for the cumulative forms, but its
  direct-pmf work does not wait on it. The inverse beta fit, T-1745, depends on this kernel.

## Acceptance criteria
- [x] `fitlab/kernels/beta.py` adds fits over the sweeps above, one per function family or one shared
  fit on I_x(a, b) with the arguments mapped. Each shipped fit appears in `rustsheet-fitlab --list`
  and in fit-lab.md's table.
- [x] Every candidate's bit-exact, ≤ 4 ulp and ≤ 1e-9 counts are recorded in the Outcome, the rejected
  ones included, with the engine's row.
- [x] docs/excel-semantics/statistics.md records the result:
  the fitted algorithm with evidence cells, or the best candidate and why it fails.
- [x] Time-boxed per decision 0056: two attempts without a gain over the engine end in
  `status: blocked` with the candidates listed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0970 and T-0969 implement it).

## Outcome
Done 2026-10-17.
- **Fits.** `uv run rustsheet-fitlab beta-dist` (also `f-dist` and `t-dist`), in `fitlab/kernels/beta.py`.
- **Reference.** A 70-digit decimal continued fraction with a Stirling ln Gamma.
- **Engine rows.** These are from the same command with the release `rustsheet-eval`. The engine
  also scores the cells with text or bool arguments, which the candidates do not take, so its
  ≤1e-9 column counts more cells.
- **Closed forms.** Excel computes I_x(a,1) = exp(a ln x) on 27 of 27 cells (VERIFIED; pow gets
  14). I_x(1,b) = 1 - exp(b ln(1-x)) matches 22 of 24 cells, as does 1 - pow(1-x,b) (PARTIAL).
- **General algorithm (UNVERIFIED).** The best combination is Cephes' fractions without the power
  series, a mean switch, and an exp-log prefactor with Gamma correctly rounded. Two families were
  tried, NR and Cephes. Both beat the engine, so the task is done, not blocked (decision 0056).
  The rest reopens only on a new hypothesis.

Exact / within 4 ulp / within 1e-9:

| candidate | beta-dist (283; 279 numeric) | f-dist (33; 30) | t-dist (150) |
|---|---|---|---|
| best combination | 158 / 220 / 279 | 15 / 28 / 30 | 95 / 129 / 148 |
| Cephes incbet + closed forms | 133 / 214 / 279 | 13 / 26 / 30 | 103 / 132 / 148 |
| correctly rounded | 130 / 209 / 279 | 17 / 30 / 30 | 112 / 142 / 148 |
| Cephes, exp(a ln x), Gamma correctly rounded | 121 / 210 / 279 | 12 / 26 / 30 | 104 / 133 / 148 |
| Cephes, exp(a ln x), tgamma | 103 / 209 / 279 | 9 / 27 / 30 | 104 / 132 / 148 |
| NR betai (eps 3e-16, 1e-15; log or log1p) | 98 / 153–156 / 279 | 3 / 17–18 / 30 | 89 / 120 / 148 |
| NR betai eps 1e-14 | 97 / 149–152 / 279 | 4 / 16 / 30 | 86 / 119 / 148 |
| Cephes, pow, Gamma correctly rounded | 94 / 193 / 279 | 15 / 27 / 30 | 105 / 130 / 148 |
| Cephes as published (pow, tgamma) | 89 / 190 / 279 | 13 / 28 / 30 | 106 / 129 / 148 |
| engine (statrs `beta_reg`) | 87 / 127 / 282 | 6 / 15 / 32 | 78 / 105 / 150 |

**Where the remaining misses are.** Shapes of 50 and 100 at x = 0.5 and 0.9. There Excel is
more accurate than exp(a ln x)*exp(b ln(1-x)), but at x = 0.1 it is not.

**For the implementers.** T-0970 can implement the b = 1 closed form now (exact on every cell)
and the a = 1 form (22 of 24),
and the best combination is a candidate on top. For T-0969, T.DIST through I_x scores best with Cephes as published (106/150); it needs the
complement mapping for large n (see statistics.md, "Incomplete beta: Excel's closed forms and
the best candidate").
