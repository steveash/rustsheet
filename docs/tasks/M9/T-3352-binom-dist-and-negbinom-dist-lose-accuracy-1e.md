---
id: T-3352
title: "BINOM.DIST and NEGBINOM.DIST lose accuracy (1E-9 to 1E-8) at a very large n or number of failures"
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
BINOM.DIST at n ≥ 6e5 and NEGBINOM.DIST with a large number of failures are within about 1e-13 of Excel, as Excel is of
the math, instead of 1e-9 to 1e-8 off today.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 5). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: crates/rustsheet-engine/src/functions/statistical/binom.rs (`fn_binom_dist`, `fn_negbinom_dist`) and
  discrete.rs (T-0927's exact walk, capped, with statrs past it). At a large n the pmf needs a saddle-point (Loader)
  form or the incomplete beta with an accurate prefactor, not a sum or a log-gamma difference that cancels.
- Related: T-2146 (BINOM last digits, ironcalc-calc), T-3366 (BINOM.DIST generalizes poorly to fresh points). This task
  is the gross loss at a large n; it does not chase last digits.

## Acceptance criteria
- [ ] BINOM.DIST at n from 6e5 to 1e9 and NEGBINOM.DIST with failures from 1e5 to 1e9, pmf and cdf, are within
      15 digits of Excel or within the Gate A envelope.
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
