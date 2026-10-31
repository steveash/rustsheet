---
id: T-3354
title: "F.DIST at a tiny x with a huge d2 is off 5E-4"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
F.DIST at x ≈ 1e-12 with d2 ≈ 7.8e6 returns Excel's value instead of one 5e-4 off.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 7). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_f_dist` in crates/rustsheet-engine/src/functions/statistical/fisher.rs, f_density.rs (`excel_f_density`,
  `STIRLING_ERROR` past df 56, an extrapolated fit of decision 0188's code audit) and the incomplete beta for the cdf.
  Find whether the density, the cdf or both are off; at a huge d2 the F law approaches a scaled chi-squared, which
  gives a reference.
- Related: T-2647 (last digits where density·x is subnormal), T-2642 (last ulp at a huge df), T-3364 (the extrapolated
  fits fall back to accurate math). This task is the 5e-4 error, not their last digits.

## Acceptance criteria
- [ ] F.DIST (density and cdf) and F.DIST.RT at x from 1e-15 to 1e-6 with d2 from 1e5 to 1e10 are within 15 digits of
      Excel or within the Gate A envelope.
- [ ] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [ ] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number.
- [ ] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.
