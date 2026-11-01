---
id: T-3638
title: "CHISQ.DIST.RT and POISSON's upper tail at a subnormal result: kept or flushed (t3269-fresh-sweep A92)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel's rule for a subnormal right tail Q(a, x) (CHISQ.DIST.RT, CHIDIST, and POISSON's cdf, which is Q(k + 1, mean)) is
probed with `(x)=0` companions (decision 0042) and the engine follows it, so probes-numeric/t3269-fresh-sweep Calc!A92
scores on evidence instead of on Graph's 0.

## Context
- Gap triage 2026-11-01 (no open task named it): probes-numeric/t3269-fresh-sweep.case.json Calc!A92
  `CHISQ.DIST.RT(1529.95,21)` = Q(10.5, 764.975). Excel (excel-graph) reads 0.0; ours is 1.3222416595277E-311, a
  subnormal (`make eval-show CASE=probes-numeric/t3269-fresh-sweep`). Graph reports a kept subnormal as 0.0 too
  (numbers.md "Subnormal results become 0", decision 0042), and the case has no companion cell, so the cell cannot tell
  a flush from a kept subnormal today.
- What is known: Excel flushes the GAMMA.DIST and CHISQ.DIST lower cdf and the CHISQ.DIST density to 0 when subnormal
  (probes/t2706-subnormal-density-paths B35, probes/t2737-subnormal-cdf-shapes B13:B17), and the engine does it in
  `chisq.rs` `fn_chisq_dist` (`flush_subnormal`). The right tail is unprobed: no subnormal-companion case calls
  CHISQ.DIST.RT, CHIDIST or POISSON's cdf. `fn_chisq_dist_rt` and `poisson.rs` (`incomplete_gamma::poisson_upper`, the
  far-tail form of T-3269 in `incomplete_gamma/excel_tail.rs` and `accurate_tail` past x = 600) do not flush.
- "The flush follows each function's internal path, not the size of the result" (numbers.md, T-2693, T-2706): probe each
  path, not one point.

## Acceptance criteria
- [ ] An excel-probe case `probes/t3638-subnormal-upper-tail` (excel-graph, column A the call, column B its `(x)=0`
      companion), filled with no pending cells, with true values between 4.9E-324 and 2.2E-308 at: CHISQ.DIST.RT and
      CHIDIST at df 21 (x = 1529.95, as A92), df 2 (the exp closed form), df 1 (the erfc form), df 7 and 39 (an odd and
      a large shape), x both inside (30 < x/2 <= 600) and past the far-tail band; POISSON(k,mean,TRUE) at the same
      shapes (k = a - 1); and one normal-result neighbour per path (companion FALSE expected) to place the cut.
- [ ] numbers.md "Subnormal results become 0" (distribution paths paragraph) records which paths keep and which flush,
      citing the B cells, and statistics.md "Excel's far upper tail (T-3269)" links it.
- [ ] The engine follows the evidence per path (`arithmetic::flush_subnormal` where Excel's companion is TRUE, no change
      where it is FALSE), with a unit test in `test/statistical/test_subnormal_results.rs` per path.
- [ ] t3269-fresh-sweep Calc!A92 passes: if Excel flushes, by the engine's 0; if Excel keeps it, by adding the
      `(CHISQ.DIST.RT(1529.95,21))=0` companion to that case (filled, decision 0042 scoring). probes-numeric is
      ungated, so no eval/baseline line moves; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits of the normal far-tail cells of the same sweep (T-3640, T-3461).

## Visible-parity stage
Decisions 0083 and 0114 (`semantics`): done when A92 scores on companion evidence and every probed subnormal upper-tail
path matches Excel's keep-or-flush.
