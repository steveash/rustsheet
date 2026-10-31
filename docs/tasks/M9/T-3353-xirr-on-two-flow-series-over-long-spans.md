---
id: T-3353
title: "XIRR on two-flow series over long spans loses accuracy (1E-8 to 1E-7 off)"
milestone: M9
lane: functions-financial
status: todo
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
XIRR on two-flow series over long spans returns Excel's rate, which is close to the math, instead of a value 1e-8 to
1e-7 off.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 6). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_xirr` in crates/rustsheet-engine/src/functions/financial/mod.rs and its solver. A two-flow series has the
  closed form (−v1/v0)^(365/days) − 1; check whether the engine's stopping tolerance or start leaves the iterate early.
- Excel's XIRR is about 1e-8 off the math in places (solver tolerance; decision 0188 R3): below G, so Excel's value is
  the target where the two differ, and the closed form is a reference, not the answer.
- Related: T-1583 and T-2328 (done) fitted XIRR's XNPV stop test and bound on long spans, and T-3287 (todo) probes the
  negative-root stop over spans of several years. The long-span error may be Excel's own stop tolerance, which those
  tasks reproduce: change the stop only against the new oracle cells, and keep what they reproduce.

## Acceptance criteria
- [ ] Two-flow XIRR over spans of 1 to 100 years and rates from -0.9 to 10 matches Excel's value within 15
      significant digits, with the right type and error, on the oracle-filled cells. The Gate A envelope below is only a
      floor on the fresh sweep; Excel's value is the target where it differs from the math by less than G = 1e-4.
- [ ] The Outcome separates engine error from reproduced Excel stop behaviour (cell by cell for the cells that move),
      and says how this task and T-3287 split the cells.
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
