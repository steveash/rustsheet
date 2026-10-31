---
id: T-3351
title: "BETA.INV returns #NUM! at p near 1-1E-8 with a small b, where the answer is 1"
milestone: M9
lane: functions-stat
status: blocked
priority: P0
size: S
tier: opus
depends_on: [T-3430, T-3431]
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
- Related: T-1871 (statrs saturation and panics), T-2511 (crossing of the ported CDF), T-2555 (BETA.INV's early
  Newton stop at a huge shape), T-2633 (BETA.INV chained on BETA.DIST). Name any of their cells this
  moves in the Outcome.

## Acceptance criteria
- [ ] BETA.INV at p from 1 − 1e-6 to 1 − 1e-15 with b from 1e-3 to 1 returns Excel's value (1.0 where Excel gives it).
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Probe cells (before the fix)
probes/t3351-beta-inv-p-near-1 (254 cells, excel-graph, 2026-10-31): `BETA.INV(1-1E-k, a, b)` for a in 0.5, 2, 10,
b in 1E-3, 3E-3, 0.01, 0.03, 0.1, 0.3, 1 and k in 6, 7, 8, 9, 10, 12, 14, 15, 16 (A1-A189). BETAINV at a = 2 (A190-A195),
the bounds form (A196), the underflow boundary at b = 0.03 for a = 2 and 0.5 (A197-A214, with the lower-tail mirror
`BETA.INV(q, 0.03, a)` in B197-B214) for a = 10 (C197-C204), and p at or below 1/2 (D197-D210). Before the fix every cell where Excel gives 1 was
`#NUM!` in the engine: A28-A31, A37-A54, A91-A94, A100-A117, A126, A154-A157, A163-A180, A189, A194, A195,
A197-A199, A206-A211 and C197-C199 (86 cells), and D197-D202, D204, D205 and D210, where Excel gives
0.9999999999999999 (9 cells).

## Blocked because
Blocked because: the fix in PR #2573 cannot merge until this task is done, and it cannot be done until T-3430 and
T-3431 land. All four reviewers approved PR #2573 on 78d90067c and CI is green there, but `make mark-ready` refuses
it (V5: the branch's task must be `status: done`). The architecture review (round 1) rejected `done` while the Gate A
criterion and the 15-digit match are unmet, and no decision narrows them. Owner's call: (a) rule that this partial
fix may land, by splitting the met part into its own task or relaxing V5 for it, or (b) keep the PR as a draft until
T-3431 (P0) and T-3430 (P1) land, then merge `origin/main` in, re-score the probe and sweep, tick the criteria and
finish here.

## Status: partial (PR #2573)
PR #2573 lands the fix for every point where the flipped tail's y is a normal double (81 of the 86 probe cells that
were `#NUM!`, 37 of the sweep's 40 points where Excel is 1), with no regression. The task stays open (`status: todo`)
because three criteria are not met: the first (Excel's 1 where Excel's search still answers below the subnormal cut),
the 15-digit match on every probe cell, and the clean Gate A envelope. No decision narrows them. The remaining cells
are owned by T-3430 (where Excel's AS 109 search fails: probe A199, A209-A211, C199, column B; sweep A88, A89, A122,
A175, A188) and T-3431 (sweep A200, the upper-tail kernel). Close this task after both land: re-score the sweep and
the probe, tick the criteria and set `status: done`.

## Outcome (partial, PR #2573)
- `beta.rs` `upper_quantile_near_one`: when BETA.INV's upper-tail quantile rounds to 1 (p > 1/2), the flipped tail's
  y (the lower quantile of Beta(beta, alpha) at 1 - p, on Excel's BETA.DIST kernel through `beta_lower_crossing`) gives
  1 - y for a normal y, and `#NUM!` for a y of 0 or a subnormal one. The cut sits at the IEEE normal boundary as a
  provisional rule. It was not chosen by scoring candidates (no fit, decision 0188 R4), and T-3430 owns Excel's real
  cut. At p of 1/2 or less a quantile that rounds to 1 is now 1 - 2^-53, as Excel gives (probe D197-D210, added after
  the excel-fidelity review; it was `#NUM!` before this task). Unit test `beta_inv_quantile_rounding_to_one`;
  statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)".
- Probe: 81 of the 86 cells listed above now match Excel. Of the 9 D cells, 8 match and D205 is one ulp off (within 15
  digits). The 5 left (A199, A209-A211, C199) are where Excel still
  answers 1 at a subnormal y (T-3430). All the probe's other A-column cells are within 15 digits of Excel (A55, A57,
  A124, A181, A187 are last-digit residue away from 1, unchanged; Phase 1b). Column B, the lower-tail mirror, is outside
  this task's p-near-1 domain: B199-B203 and B209-B213 are Excel's subnormal cut (T-3430), and B197, B198 and B206-B208
  are about 1E-13 off Excel, with Excel itself 1.5E-14 to 7.2E-14 off the math (listed in T-3430). Baselined as 27
  new-coverage known failures (D205 and D209 are last-digit residue, within 15 digits).
- Gate A envelope on the fresh sweep probes-numeric/t3351-beta-inv-sweep (200 points, seed 3351; the hold-out is the
  whole sweep, since nothing was fitted). Scored against 50-digit mpmath with a scratch script; SPEC-0029's generator
  is T-3357. Per stratum, as within 15 digits of Excel (both errors counted) / within 1E-9 of the math / outside both /
  error against number:
  typical (p 0.01 to 0.99, shapes 0.1 to 30) 53 / 80 / 0 / 0 of 80;
  tail (p = 1 - q, q 3E-16 to 1E-6, alpha 0.1 to 10, beta 1E-3 to 1) 57 / 37 / 0 / 3 of 60;
  extreme (p near 0 or 1, shapes 1E-3 to 1E4) 31 / 30 / 0 / 1 of 40;
  log-uniform (shapes 1E-3 to 1E5) 13 / 13 / 1 / 1 of 20. A186's mpmath value did not finish (shapes of 2E4), but it
  is bit-exact with Excel.
  Excel gives exactly 1 on 40 sweep points. All 40 were `#NUM!` before this change; 37 now match.
  **The envelope is not clean, so this criterion is not fully met.** None of the 6 points outside it comes from
  this change's code path, and none got worse:
  - A88, A89, A122 (tail): Excel is 1 where the true y is far below the subnormals, and ours stays `#NUM!` as before.
    T-3430 owns where Excel's search fails.
  - A175, A188 (lower tail, p < 1/2, an existing path): one side is an error and the other a number. Also T-3430.
  - A200 (`BETA.INV(1-1.1E-14,0.005,171)`): ours is 2.9E-5 where Excel and the math give 0.1294. The upper-tail
    incomplete beta returns exactly 0 or 1 at these shapes, so the existing refinement moves statrs' correct start
    off the root. The quantile does not round to 1 there, so this task's path never runs. Filed as T-3431 (P1).
- Eval delta: probes +1 case (254 cells, 27 new-coverage known failures, 0 regressions); probes-numeric +1 case
  (gate-free). Every other suite is unchanged. No gen-m9 or ironcalc cell moved.
- Follow-ups: T-3430 (Excel's AS 109 search failure at tiny shapes: the subnormal cut, the sweep's A88, A89, A122,
  A175, A188, and the lower-tail mirror cells) and T-3431 (the upper-tail incomplete beta at a tiny alpha with a large
  beta).
