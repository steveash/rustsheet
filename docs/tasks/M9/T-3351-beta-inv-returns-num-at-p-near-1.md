---
id: T-3351
title: "BETA.INV returns #NUM! at p near 1-1E-8 with a small b, where the answer is 1"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.INV (and BETAINV) return Excel's value at p ≈ 1 − 1e-8 with a small b, where the engine returns `#NUM!` today and
the answer is 1.0.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 4). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_beta_inv` in crates/rustsheet-engine/src/functions/statistical/beta.rs and inverse_beta.rs (the statrs
  port: Newton steps, step adjustments and their caps). The quantile rounds to 1 in doubles; the search must return it
  instead of failing.
- Cells already failing (gap triage 2026-10-31, `make exit-owners MILESTONE=M9`, eval on a64f963e; Excel 1, ours
  `#NUM!`): probes-numeric/m9-beta-inv Calc!A7 `BETA.INV(0.9999999999,0.5,0.5)`, A42 (a = 1), A77 (a = 2.5), A112
  (a = 10) and A147 (a = 100), all at b = 0.5. statistics.md "Incomplete beta sweep (T-1044)" already records
  `BETA.INV(0.9999999999,0.5,0.5)` as exactly 1. The new probe below still covers the rest of the region.
- Related: T-1871 (statrs saturation and panics), T-2511 (crossing of the ported CDF), T-2555 (BETA.INV's early
  Newton stop at a huge shape), T-2633 (BETA.INV chained on BETA.DIST). Name any of their cells this
  moves in the Outcome.

## Acceptance criteria
- [ ] BETA.INV at p from 1 − 1e-6 to 1 − 1e-15 with b from 1e-3 to 1 returns Excel's value (1.0 where Excel gives it).
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
