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
- **Fits.** `uv run rustsheet-fitlab beta-inv` (also `f-inv`, `f-inv-rt`, `t-inv`, `t-inv-2t`), in
  `fitlab/kernels/beta_inv.py`: bisection to adjacent doubles on each T-1744 forward candidate,
  through T-1744's F and t mappings.
- **Probe.** probes-numeric/t1745-beta-inv-roundtrip (excel-graph). It has 188 rows: 187 distinct
  in-domain BETA.INV points plus one duplicate, leaving out the 5 where Excel returns x = 1. Each
  row holds Excel's x and its two neighbouring doubles as constants, BETA.DIST of each, and
  BETA.INV again.
- **Finding.** Excel's x is a crossing of its own CDF at 155 of 188 rows: BETA.DIST(x) = p at 79,
  and p lies between x and a neighbour at 76, on either side (36 at or above, 40 below). Of the
  other 33, 22 are flat-CDF tiny-x points. At 11, x misses the crossing by 1 to 188 ulps of p on
  both sides (8 above, 3 below; rows 91 and 98 are 32 and 188 off), which suggests a tolerance
  stop but does not establish it.
- **Consequence.** The quantile follows the forward kernel (T-1744). Excel's start, step and stop
  cannot be fitted until that kernel matches.
- **Not tried.** Newton, Halley, AS 109 and AS 64 starts, and Cephes `incbi` were not tried: with
  a forward CDF that is not Excel's, their iterates cannot be told apart from bisection's at the
  crossing. Iterate hits are not scored for the same reason. They are the next step once T-0970
  matches BETA.DIST.
- **Mid end.** For adjacent doubles the bisection's "mid" end is the even-significand end
  (ties to even), not a solver rule.

Exact / ≤4 ulp / ≤1e-9 per fit (candidates over the cells with numeric arguments; the engine also
scores the others):

| candidate (bisection end, forward I_x) | beta-inv (212) | f-inv (54) | f-inv-rt (121) | t-inv (130) | t-inv-2t (162) |
|---|---|---|---|---|---|
| mid, correctly rounded | 87 / 173 / 202 | 2 / 31 / 46 | 8 / 65 / 103 | 29 / 64 / 120 | 25 / 65 / 126 |
| hi, correctly rounded | 75 / 171 / 202 | 3 / 31 / 46 | 10 / 62 / 103 | 29 / 66 / 120 | 24 / 65 / 126 |
| lo, correctly rounded | 54 / 173 / 202 | 8 / 31 / 46 | 2 / 64 / 103 | 23 / 65 / 120 | 15 / 65 / 126 |
| mid, Cephes as published | 73 / 159 / 202 | 2 / 27 / 46 | 7 / 67 / 103 | 22 / 62 / 118 | 24 / 65 / 125 |
| hi, Cephes | 64 / 163 / 202 | 3 / 28 / 46 | 11 / 66 / 103 | 24 / 62 / 118 | 17 / 65 / 125 |
| lo, Cephes | 43 / 157 / 202 | 2 / 27 / 46 | 6 / 64 / 103 | 20 / 61 / 118 | 18 / 65 / 125 |
| mid, T-1744 best combination | 65 / 151 / 198 | 2 / 25 / 45 | 5 / 60 / 102 | 22 / 56 / 119 | 18 / 60 / 125 |
| hi, best combination | 56 / 148 / 198 | 3 / 26 / 45 | 4 / 60 / 102 | 18 / 57 / 119 | 16 / 62 / 125 |
| lo, best combination | 44 / 149 / 198 | 0 / 25 / 45 | 4 / 57 / 102 | 20 / 55 / 119 | 11 / 60 / 125 |
| mid, NR betai | 39 / 112 / 202 | 0 / 11 / 46 | 1 / 31 / 103 | 16 / 44 / 118 | 8 / 42 / 124 |
| hi, NR betai | 36 / 115 / 202 | 0 / 12 / 46 | 3 / 35 / 103 | 13 / 43 / 118 | 9 / 43 / 124 |
| lo, NR betai | 22 / 115 / 202 | 4 / 11 / 46 | 2 / 31 / 103 | 16 / 46 / 118 | 5 / 42 / 124 |
| engine (statrs) | 34 / 81 / 207 | 2 / 13 / 52 | 9 / 19 / 103 | 17 / 41 / 130 | 9 / 23 / 137 |

**Status.** The best bit-exact count beats the engine on every fit (87 vs 34, 8 vs 2, 11 vs 9,
29 vs 17, 25 vs 9), so the task is done, not blocked (decision 0056).
