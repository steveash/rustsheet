---
id: T-1744
title: "Fit lab: regularized incomplete beta kernel (BETA.DIST, F.DIST, T.DIST, BINOM.DIST) against the oracle sweeps"
milestone: M9
lane: research
status: todo
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
- [ ] `fitlab/kernels/beta.py` adds fits over the sweeps above, one per function family or one shared
  fit on I_x(a, b) with the arguments mapped. Each shipped fit appears in `rustsheet-fitlab --list`
  and in fit-lab.md's table.
- [ ] Every candidate's bit-exact, ≤ 4 ulp and ≤ 1e-9 counts are recorded in the Outcome, the rejected
  ones included, with the engine's row.
- [ ] docs/excel-semantics/statistics.md records the result:
  the fitted algorithm with evidence cells, or the best candidate and why it fails.
- [ ] Time-boxed per decision 0056: two attempts without a gain over the engine end in
  `status: blocked` with the candidates listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0970 and T-0969 implement it).
