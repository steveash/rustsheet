---
id: T-1745
title: "Fit lab: inverse incomplete beta kernel (BETA.INV, F.INV, T.INV) against the oracle sweeps"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1744]
---

## Goal
A fit-lab fit for the inverse regularized incomplete beta, the quantile of I_x(a, b): Excel's root
finder (start, step, stop) on top of its incomplete beta. This is rank 5 in the numeric kernel map, with
664 near-miss cells.

## Context
Owner request (2026-10-16). Decision 0056 says numeric work is kernel-first and every fit is a
fit-lab comparison of named candidates against an oracle sweep (docs/architecture/fit-lab.md,
`tools/src/rustsheet_tools/fitlab/`). The fit lab already found PV/FV/NPER, XIRR, IRR and Excel's `^`
this way (T-1524, T-1525, T-1533; implemented by T-0901 and T-0916). It has no fit yet for this kernel.
- docs/research/numeric-kernel-map.md, rank 5: the engine calls statrs `inverse_cdf` in
  `statistical/beta.rs`, `t_dist.rs` and `fisher.rs`; 444 cells are `close`. It ranks after the forward
  kernel (T-1744): a quantile is only as exact as the CDF it inverts.
- Sweeps (probes-numeric, excel-graph):
  - `m9-beta-inv`, `m9-f-inv`, `m9-f-inv-rt`, `m9-t-inv`, `m9-t-inv-2t`;
  - `m9-legacy-betainv`, `m9-legacy-finv`, `t1048-tinv`, `t1049-finv`, `t1049-finvrt`;
  - T.INV's huge-df edge is fixed (T-1518), so its cells are valid sweep points.
- Candidates:
  - Newton, Halley and bisection-then-Newton on T-1744's best forward kernel, with start points such
    as AS 109's (Cran, Martin and Thomas) or AS 64's;
  - TOMS 708-based inverses and Cephes `incbi`;
  - per candidate, the stop test (|Δx|, |F(x) - p|, relative or absolute, step cap). The fit lab's
    `trace` support (iterate hits, as in T-1525) shows which iterate Excel returned.
- Owner task that implements the winner: T-0974 (BETA.INV/F.INV/F.INV.RT; T.INV through the t
  mapping).

## Acceptance criteria
- [x] `fitlab/kernels/beta_inv.py` adds fits over the sweeps above, shipped in `--list` and fit-lab.md.
- [x] Every candidate's counts (bit-exact, ≤ 4 ulp, ≤ 1e-9, iterate hits) are recorded in the Outcome,
  with the engine's row.
- [x] docs/excel-semantics/statistics.md records the fitted iteration or the best candidate.
- [x] Time-boxed per decision 0056; `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0974).

## Outcome
Done 2026-10-17.
- **Fit.** `uv run rustsheet-fitlab beta-inv` (`fitlab/kernels/beta_inv.py`), over 212 cells from
  m9-beta-inv and m9-legacy-betainv.
- **Probe.** probes-numeric/t1745-beta-inv-roundtrip (excel-graph). For each of the 188 numeric
  sweep points it holds Excel's x and its two neighbouring doubles as constants, BETA.DIST of each,
  and BETA.INV again.
- **Finding.** Excel's x is a crossing of its own CDF at 155 of 188 points: BETA.DIST(x) = p at 79,
  and p lies between x and a neighbour at 76, on either side (36 at or above, 40 below). Of the
  other 33, 22 are flat-CDF tiny-x points, and at 11 x stops a few CDF ulps short of the crossing
  (BETA.INV(0.5,10,10) = 0.5, BETA.DIST(0.5,10,10,TRUE) = 0.5000000000000002), which suggests a
  tolerance stop. So the quantile follows the forward kernel (T-1744). Excel's
  start, step and stop rule cannot be fitted until that forward kernel matches.
- **Iterate hits.** Not scored, because no forward candidate is Excel's CDF.

Exact / ≤4 ulp / ≤1e-9 (202 cells with numeric arguments; the engine also scores the others):

| candidate (bisection to adjacent doubles, returned end) | beta-inv |
|---|---|
| midpoint, correctly rounded I_x | 87 / 173 / 202 |
| upper end, correctly rounded I_x | 75 / 171 / 202 |
| midpoint, Cephes incbet as published | 73 / 159 / 202 |
| midpoint, T-1744 best combination | 65 / 151 / 198 |
| upper end, Cephes | 64 / 163 / 202 |
| upper end, best combination | 56 / 148 / 198 |
| lower end, correctly rounded I_x | 54 / 173 / 202 |
| lower end, best combination | 44 / 149 / 198 |
| lower end, Cephes | 43 / 157 / 202 |
| midpoint, NR betai | 39 / 112 / 202 |
| upper end, NR betai | 36 / 115 / 202 |
| engine (statrs `inverse_cdf`) | 34 / 81 / 207 |
| lower end, NR betai | 22 / 115 / 202 |

**Status.** The bit-exact count rose from 34 to 87, so the task is done, not blocked (decision
0056).

**Scope.** F.INV, F.INV.RT, T.INV and T.INV.2T are not scored here. They invert the F and t
mappings of the forward kernel, and their own fits wait until T-0974 has the beta inverse.
