---
id: T-3147
title: "CHISQ.DIST at 50 and 100 degrees of freedom and CHISQ.DIST(40,10,FALSE): Excel's large-shape prefactor (9 unowned visibly-off cells of the m9 CHISQ.DIST probe)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The nine visibly-off cells listed below (the m9 CHISQ.DIST probe) reach visible parity: CHISQ.DIST at 50 and 100 degrees of
freedom (shapes 25 and 50, TOMS 654 `rcomp` branch of `incomplete_gamma::gamma_prefactor`) and
`CHISQ.DIST(40,10,FALSE)` (shape 5, the direct branch) agree with Excel to half a unit of the 15th significant digit.

## Context
Gap triage 2026-10-29 (`make eval` on main d57e9e9a2, excel-graph; `python3 scripts/exit_owners.py --all-off`): no open
task names these cells. T-2638 owns other CHISQ.DIST cells (the T-1049 probes, gen-m9) and T-2852 the GAMMA.DIST
density at shape 50 (its T-2753 ratio-sweep cells), but neither lists the cells below.

Cells (Excel vs ours; ulps against a 60-digit mpmath reference, `gammainc(k/2, 0, x/2)` regularized and
`exp((k/2-1) ln(x/2) - x/2 - lngamma(k/2))/2`):
- probes-numeric/m9-chisq-dist Calc!A50 `CHISQ.DIST(40,10,FALSE)`: 6.870512074795199e-6 vs 6.870512074795192e-6 (Excel +7 ulp, ours -1)
- probes-numeric/m9-chisq-dist Calc!A51 `CHISQ.DIST(5,50,TRUE)`: 5.198100275214529e-17 vs 5.198100275214567e-17 (Excel -52, ours +10)
- probes-numeric/m9-chisq-dist Calc!A52 `CHISQ.DIST(5,50,FALSE)`: 2.3501107958436413e-16 vs 2.3501107958436585e-16 (Excel -30, ours +5)
- probes-numeric/m9-chisq-dist Calc!A54 `CHISQ.DIST(25,50,FALSE)`: 0.0006359509531578977 vs 0.0006359509531578982 (Excel -2, ours +3)
- probes-numeric/m9-chisq-dist Calc!A58 `CHISQ.DIST(100,50,FALSE)`: 9.264464960126204e-6 vs 9.264464960126195e-6 (Excel +3, ours -2)
- probes-numeric/m9-chisq-dist Calc!A61 `CHISQ.DIST(10,100,TRUE)`: 2.1810592140784731e-32 vs 2.1810592140785043e-32 (Excel -57, ours +57)
- probes-numeric/m9-chisq-dist Calc!A62 `CHISQ.DIST(10,100,FALSE)`: 9.838365191393825e-32 vs 9.838365191393965e-32 (Excel -63, ours +65)
- probes-numeric/m9-chisq-dist Calc!A63 `CHISQ.DIST(50,100,TRUE)`: 6.953305247616096e-6 vs 6.95330524761611e-6 (Excel -3, ours +13)
- probes-numeric/m9-chisq-dist Calc!A64 `CHISQ.DIST(50,100,FALSE)`: 3.6021642652021993e-6 vs 3.602164265202206e-6 (Excel -5, ours +11)
Also off but within 15 digits (fix them if the same form does): A53 `CHISQ.DIST(25,50,TRUE)` (Excel -3, ours +2) and
A68 `CHISQ.DIST(200,100,FALSE)` (Excel +7, ours -5).

Both sides are inaccurate, in opposite directions, so this is decision 0083's second kind of work (Excel's own form,
stage `visible`), not a more accurate kernel. At x / a = 0.1 (A61, A62) the engine's `rcomp` takes
`rlog = (ratio - 1) - ln(ratio)` outside [a/2, 2a], and a times rlog's absolute error is the 60-ulp miss; Excel misses
by about as much the other way, so its exponent is rounded differently (candidates: `a ln x - x - ln Gamma(a)` with
Excel's GAMMALN, the x87 `exp`/`log` kernels of x87_exp.rs and x87_log10.rs, an 80-bit exponent, a log1p-based
rlog). The cumulative cells share the prefactor, so fit the density first and check A51, A61, A63 follow. A50 is the
direct branch (shape 5, `powf * exp / tgamma`), where Excel is 7 ulp off: test the same candidate forms there; the
shape split at 20 (`DIRECT_SHAPE_MAX`) is a guess, not Excel's.

Evidence already in eval/ for the fit: the m9 CHISQ.DIST probe (df 1 to 100, 70 cells), the probes-numeric m9
GAMMA.DIST probes at shapes 9, 50 and 500, and the T-2753 gamma density switch and ratio sweeps. If those cannot pin the form, file a research sweep (literal
CHISQ.DIST and GAMMA.DIST density cells at shapes 20 to 100 and x / a from 0.05 to 4) with `make new-task` and block on
it rather than fitting blind. docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)"
records these cells (T-3147 bullet). Coordinate with T-2852 (same `gamma_prefactor`, shape 50): whichever lands
first records its forms there; neither may regress the other's cells.

## Acceptance criteria
- [ ] The nine cells above (m9-chisq-dist A50, A51, A52, A54, A58, A61, A62, A63, A64) are within half a unit of
      Excel's 15th significant digit (`python3 scripts/exit_owners.py --all-off --function CHISQ.DIST` no longer
      lists them), with no new known-failure line in any eval/baseline file (`make eval-baseline`).
- [ ] The candidate forms tried, with their bit-exact and visible-parity counts on the evidence above, are recorded in
      docs/excel-semantics/statistics.md (decision 0056: two attempts, then time-box).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells of T-2638 (tails, tiny x), T-2852 (GAMMA.DIST density at x / beta 1E-15 to 1E-10) and the ulp residue of
  T-2367 and T-2695.
- The gamma quantiles (T-3016, T-3131, T-3146).

## Visible-parity stage
Decision 0083/0114: every cell this task owns is visibly off and a 60-digit reference shows Excel is not correctly
rounded there (2 to 63 ulp), so the stage is `visible`. The target is that each owned cell is within half a unit of the
15th significant digit of Excel's value; bit-level misses left then go to a `defer: phase-1b` follow-up.

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- probes-numeric/t3148-gamma-quantile-flat-band Calc!A356 `CHISQ.DIST(24.673905271877182,50,TRUE)`: Excel 0.000999999999999913, ours 0.0009999999999999105 (11 ulp), the shape-25 cdf this task fits. Its neighbours A340, A344, A348 and A350 are off by 1.6E-15 to 3.1E-15 relative but within 15 digits; check they follow.
