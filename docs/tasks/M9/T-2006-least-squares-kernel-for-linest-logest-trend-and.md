---
id: T-2006
title: "Least-squares kernel for LINEST, LOGEST, TREND and GROWTH: fit Excel's evaluation order (about 300 last-digit cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The least-squares solve behind LINEST, LOGEST, TREND and GROWTH gives Excel's bits: one fitted kernel replaces the
normal-equations solve, so the four functions' last-digit failures go together. For example, `TREND(A2:A6)` should give
0.18400000000000014, as Excel does, not the exact 0.184.

## Context
Gap triage 2026-10-19. The four functions share `linear_regression` (functions/statistical/linest.rs). It forms D'D
and D'y and solves them with Gauss-Jordan (`solve_system`). Excel's values carry rounding error that this does not
reproduce, even on exactly representable fits. statistics.md "LINEST, LOGEST, GROWTH: probe findings":
`LOGEST({1,2,4,8},{1,2,3,4})` is 1.9999999999999998 and 0.5000000000000001, `GROWTH(5,5,5)` is 4.999999999999999.
Microsoft's LINEST help describes a QR decomposition since Excel 2003. Treat that as a lead, not evidence: the fit
decides which decomposition it is, if any (Householder or Gram-Schmidt, column order, pivoting, centring).

Numeric cells (both values numbers, verdict ulp, close or mismatch), 303 in total:
- probes/linest-modes 81 (38 ulp, 32 close, 11 mismatch), e.g. collinear `LINEST(B2:B9,A2:E9,TRUE,TRUE)`.
- probes/logest-modes 84 (20 ulp, 63 close, 1 mismatch).
- probes/growth-modes 57 (28 ulp, 29 close).
- probes/regression-trend 15 ulp, e.g. O2 `INDEX(TREND(A1:A6,B1:B6),1,1)` expected 2.571428571428571, ours
  2.5714285714285694, and O3 expected 7.2, ours 7.200000000000003.
- ironcalc-calc/STATISTICAL/TREND_GROWTH.xlsx 26 and LINEST_LOGEST.xlsx 24 (desktop cache, Mac 16.x), e.g.
  TREND_GROWTH Sheet1!F2 `TREND(A2:A6)` expected 0.18400000000000014, ours 0.184, and LINEST_LOGEST Sheet1!E3
  `LINEST(A2:A6)` intercept expected -0.031000000000000028, ours -0.03100000000000025.
Each function's own task keeps its shape, flag, error and stats-layout cells: T-1741 (LINEST), T-1742 (LOGEST),
T-1743 (GROWTH), T-1726 (TREND). The numerics were in those tasks too, so four sessions would have fitted the same
kernel. Decision 0056 says to fit the kernel first: one lab fit over all of them. This needs numeric fitting against
several hundred cells, hence opus.

## Acceptance criteria
- [ ] A fit-lab kernel (tools/src/rustsheet_tools/fitlab/kernels/, a new `regression` fit) scores named candidates
      on the cells above. Candidates: normal equations (today's), Householder QR, modified and classical Gram-Schmidt,
      centred data with and without a separate intercept, and LOGEST/GROWTH taking ln(y) before or inside the solve.
      Each candidate's bit-exact and within-4-ulp counts go into statistics.md, rejected ones included.
- [ ] The best candidate is implemented once in linest.rs and used by all four functions. No baseline gains a line.
      The F lines for the ulp/close cells of linest-modes, logest-modes, growth-modes, regression-trend, TREND_GROWTH
      and LINEST_LOGEST shrink, and the PR lists how many leave each case. If two attempts do not raise the bit-exact
      count, the task is set `blocked` with "Blocked because: time-boxed (decision 0056)".
- [ ] Stats rows (se, r2, F, ss) use the same decomposition, and the PR states which of them now match.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Result shapes, `#N/A` fill, flags, omitted arguments and error choice (T-1741, T-1742, T-1743, T-1726).
- FORECAST/FORECAST.LINEAR, SLOPE, INTERCEPT, STEYX and the CORREL family. They are closed forms in forecast.rs and
  correl.rs, not this solve (T-1725, T-1723, T-1707).
