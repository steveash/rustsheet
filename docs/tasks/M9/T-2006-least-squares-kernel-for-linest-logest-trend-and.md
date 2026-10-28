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
defer: phase-1b
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

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: LINEST and GROWTH last digits (the shared kernel). Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/285d87cea1aaa73c Calc!A183 `=LINEST(Data!E11:E12)`: Excel -1e-15, ours -9.999999999999997e-16
Cells:
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A183
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A178
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A341

Also from T-1741 (visibly off, the solve's noise digits): probes/linest-modes K106:P110 (collinear `LINEST(B2:B9,
A2:E9,TRUE,TRUE)`: Excel 1.4e-16-sized coefficients and se, F 3.1e34 at K109 where ours is exactly 0 and `#NUM!`),
K190:L194 and K197:L201 (y in x: F 1.36e32 at K193, K200, ours `#NUM!`), and K134:L138 (two points: Excel exactly 0.5,
-1.5 and ss_resid 0; ours 0.49999999999999867 and 3.6e-30). Excel's exact or collinear fits carry its solve's residual
noise, so F is a number there; ours is exact, so F is `#NUM!`.
The singular test is an absolute pivot under 1e-12 (`solve_system`, `invert_matrix`) while `independent_columns`
drops on a relative 1e-10, so a badly scaled x (values near 1e-7) can fail the solve with nothing to drop and give
`#NUM!` where Excel would fit: the new kernel should test singularity relative to scale.

Also from T-1742 (visibly off, the solve): probes/logest-modes K85 `LOGEST(D2:D9,A2:C9,TRUE,TRUE)` (Excel
0.982454941667086, ours 0.9824549416670796, past the 15th digit), the exact-fit stats K169:L173 and K197:L201
(`LOGEST({1,2,4,8},{1,2,3,4},TRUE,TRUE)`, `LOGEST({0.5,0.25,0.125},{1,2,3},TRUE,TRUE)`: Excel's se and ss_resid are
exactly 0 and F `#NUM!`; ours carry 2E-16 and 1E-31 of noise and F 1E+31), and gen-m9 589f42e67f4b4b8d Calc!A193
`LOGEST(Data!A10:B11)` (Excel 2.220601291150046, ours 2.2206012911500514). Unlike LINEST's exact fits, LOGEST's ln(y)
is exactly linear here only to rounding, so a kernel that gets these to exactly 0 is a strong lead.

Also from T-1743 (visibly off, the solve): probes/growth-modes K145:K152 (`GROWTH(D2:D9,A2:C9,A2:C9)`, e.g. K147 Excel
9.034855848946608, ours 9.034855848947016), K157 and K169 (a multi-column x with `{1,2,3}` as new_x), and K3, K15, K27,
K123, K219 (Excel 9.215274223288255, ours 9.21527422328826); gen-m9 8fec7b207f4dd753 Calc!A75 `GROWTH(Data!D10:E10)`
(Excel 0.19999999999999996, ours 0.2000000000000001, a last-digit line).

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

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
