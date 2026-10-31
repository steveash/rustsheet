---
id: T-3347
title: "IRR returns #NUM! on conventional cash-flow series whose IRR is near 0 or near -10%"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
IRR returns Excel's root for conventional cash-flow series (one sign change) whose internal rate is near 0 (1e-5 to
3e-4) or near -10%, where the engine returns `#NUM!` today. Together with T-3348 (RATE) this is the first task of the
accuracy stage.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 1). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_irr` in crates/rustsheet-engine/src/functions/financial/mod.rs and the solver in
  financial/financial_util.rs (the secant, the far-root bound and `irr_fallback`, a Newton run from v = 0 fitted in
  T-2727). Find which stop or bound refuses the root: near r = 0 the NPV's slope and value are both small, and near
  r = -10% the discount factors grow.
- Related: T-2131 (IRR's returned iterate to 15 digits), T-2928 (blocked: the fallback's digits), T-2927 (done: the
  fallback-path sweep), T-3205 (done: secant-in-d residues), T-3246 (a calipers
  ulp regression). This task is about the `#NUM!`, not their last digits; name any of their cells this fix moves in
  the Outcome.
- The IRR fallback is one of the fitted pieces of decision 0188's code audit: if the fix changes it, tag it per R4.3
  (T-3361) and keep it inside its measured range.

## Acceptance criteria
- [x] Conventional series with an IRR between 1e-5 and 3e-4, and near -10%, match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells. (Type and error: all 162. Digits: 141 of 162; the 21 others are 2e-15 to 2.2e-11 off, deferred to T-3385.)
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] (deferred to T-3385: 11 cells are 7e-15 to 2.2e-11 off) After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [x] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [x] `make check` green; reviewers approve.

## Probe cells (before the fix)
probes/t3347-irr-conventional-num (excel-graph 2026-10-31; inputs in the case, exact roots in its `.roots.tsv`):
A1-A21 level annuities near -10% (n = 10 with flows of 15: A1-A7; n = 20 with flows of 1.38: A8-A14; n = 40 with
flows of 15: A15-A21; r = -0.07, -0.085, -0.095, -0.1, -0.105, -0.115, -0.13 in each group), A22-A42 24 uneven flows
near 0 (scales 1e3: A22-A28, 1e6: A29-A35, 1e9: A36-A42; r = 3e-6, 1e-5, 3e-5, 1e-4, 3e-4, 1e-3, 1e-2). Excel returns
a number on every cell; the engine was `#NUM!` on A10-A21 and A36-A42 (19 cells), bit-exact on the other 23.

## Outcome
- Cause: the secant model either reaches its 50-step cap (long level series near -10%: it drifts) or stops on a
  flat step (near 0 with large flows, v = 1 - d is the same double for successive d), and the v = 0 Newton fallback
  needs 61 to 416 steps on these flows. No secant variant reproduces Excel (financial.md "IRR on conventional flows
  near 0 and -10%" lists them).
- Fix: `irr_conventional`, a last stage on the probed shape only (one outlay, then flows of the other sign): Newton on P(v) from v = 1
  under the same 50-step cap (the 159 returning series need at most 46 steps, the three `#NUM!` ones 64 to 159),
  then up to three polishing steps. The v = 0 Newton loop is now `irr_newton`, shared; `irr_fallback` is unchanged
  in behavior. Fitted (decision 0188 R4): the `FITTED` tag does not exist yet (T-3361 adds it).
- Eval: a full run before and after changes only cells of the two new cases (no other cell's value moves, so none of
  T-2131, T-2928, T-3205 or T-3246's cells move). probes/t3347-irr-conventional-num: 19 `#NUM!` to numbers, 31 of 42 (A14, one ulp off, is among them but stays a baseline known failure)
  within 15 digits of Excel (bit-exact or ulp), 11 within 2.2e-11. probes-numeric/t3347-irr-conventional-sweep (Gate
  A sweep, 120 points): 23 `#NUM!` to numbers, 110 of 120 within 15 digits, 0 type mismatches.
- Gate A floor, sweep per stratum (within 15 digits of Excel / within 1e-9 of the 50-digit root / outside both / type
  mismatches): near0 (1e-5..3e-4) 23/24/0/0, near0-neg 22/24/0/0, near-10pct 22/24/0/0, pos-log 20/24/0/0,
  neg-log 23/24/0/0 (24 points each). The probe: near-10pct 14/21/0/0, near0 17/21/0/0.
- Not met: 15-digit parity on 21 cells (Excel's own stop is looser than a converged Newton; it is within 2.1e-11 of
  the root there, the engine within 1.5e-11). Follow-up T-3385.
- Found in passing: sweep A106 (root -1.3e-17) is -2.220446049250313e-16 in Excel and the engine, one ulp of v;
  noted as a candidate `MATH-OVER-EXCEL` region for T-3358.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.
