---
id: T-1745
title: "Fit lab: inverse incomplete beta kernel (BETA.INV, F.INV, T.INV) against the oracle sweeps"
milestone: M9
lane: research
status: todo
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
- [ ] `fitlab/kernels/beta_inv.py` adds fits over the sweeps above, shipped in `--list` and fit-lab.md.
- [ ] Every candidate's counts (bit-exact, ≤ 4 ulp, ≤ 1e-9, iterate hits) are recorded in the Outcome,
  with the engine's row.
- [ ] docs/excel-semantics/statistics.md records the fitted iteration or the best candidate.
- [ ] Time-boxed per decision 0056; `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0974).
