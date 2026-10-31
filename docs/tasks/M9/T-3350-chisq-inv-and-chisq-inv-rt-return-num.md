---
id: T-3350
title: "CHISQ.INV and CHISQ.INV.RT return #NUM! at df of 2E7 and above"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
CHISQ.INV and CHISQ.INV.RT (and the legacy CHIINV) return Excel's quantile at df ≥ 2e7, mostly with p near 1, where the
engine returns `#NUM!` today.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 4). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_chisq_inv`, `fn_chisq_inv_rt` in crates/rustsheet-engine/src/functions/statistical/chisq.rs and the gamma
  quantile under statistical/incomplete_gamma/ (the huge-shape path, `huge_shape.rs`). Find which bound or search
  refuses the large shape.
- Excel's CHISQ.INV.RT is about 5e-9 off the math in places (decision 0188 R3): that is below G, so Excel's value is
  still the target there.
- Related: T-3294 (deep lower tail at shape 20 and above), T-3337 (df 1 right tail), T-3336 (its df 1 right-tail
  sweep), T-2860 (done: the df 2 closed forms). Neither covers df ≥ 2e7.

## Acceptance criteria
- [ ] CHISQ.INV and CHISQ.INV.RT at df from 2e7 to the documented limit (1e10) with p across (0,1), p near 1
      included, match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells, with the right type and error. The Gate A envelope below is only a floor on the fresh sweep;
      where Excel differs from the math by less than G = 1e-4, Excel's value is the target, never the math.
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
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [ ] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.
