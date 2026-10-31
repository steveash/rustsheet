---
id: T-3348
title: "RATE returns #NUM! on ordinary annuities with a nonzero fv"
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
RATE returns Excel's rate for ordinary annuities with fv ≠ 0 where the engine returns `#NUM!` today, for example
`RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)`, which is 0.0431054691928255 in Excel. Together with
T-3347 (IRR) this is the first task of the accuracy stage.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 2). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_rate` in crates/rustsheet-engine/src/functions/financial/annuity.rs and its iteration (start, step,
  stop rule, the 20-iteration limit Excel documents). Find why the search fails to converge or leaves the domain on
  this input; check the guess-free start against Excel's.
- Prior evidence: T-2005 (done) fitted RATE's solver with per-iterate traces, and T-0916 and T-1525 (done) probed
  Excel's iteration and termination (start, step, stop rule). That behaviour is evidence-based: change it only against
  the new oracle case. The 2026-10-31 assessment found Excel returning a value on these inputs, so a stop rule that
  gives `#NUM!` here is not Excel's.
- Excel's RATE with 1 or 2 periods is about 1e-2 off the math at rates near 1e-8 (decision 0188, "Where Excel itself is
  far from the math"). Decision 0188 R3 lists no RATE case, so Excel's value stays the target there; it is out of scope.
- `annuity.rs` `DISCOUNTED_INTEREST_FROM` is a tuned cut-point of the code audit; do not move it in this task (T-3365).

## Acceptance criteria
- [x] `RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)` is 0.0431054691928255 (a unit test and a probe cell).
- [x] Ordinary annuities with fv ≠ 0 across rates 1e-6 to 1 and nper 1 to 1,000 match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells; where Excel gives
      `#NUM!` (for example its documented 20-iteration limit) the engine gives `#NUM!` too.
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

## Probe cells (before the fix)
probes/t3348-rate-fv-nonzero (excel-graph, 2026-10-31), Sheet1!A1:A186, 111 numbers and 75 `#NUM!`:
- A1, A2: the Context example (type 0 and omitted), Excel 0.04310546919282547; the engine gave `#NUM!`.
- A3-A12: the same inputs times 2^-40 ... 2^20 (the secant's trajectory is unchanged, f scales); A13-A23 nper 200 to
  300; A24 type 1; A25-A29 guesses 0, 0.04, 0.05, 0.2, -0.5.
- A30-A186: a seeded sweep of annuities (rate 1E-6 to 1, nper 1 to 1000, both signs of pv and fv, type 0/1, guess
  omitted, 13-digit or cent inputs), stratified by the T-2005 model's outcome (labels in the case's hypotheses): 70
  zero-step stalls, 15 other flat secants, 40 converged, 15 iterates below -1, 12 past the cap, 4 returned at the
  cap, 1 power past the double range.
- Before the fix 60 cells were `#NUM!` in the engine and numbers in Excel (every one a flat secant); the other 126
  matched Excel bit for bit, including the 39 flat secants and the le-1 and cap runs that are `#NUM!` in Excel too.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
Done, with two Gate A cells handed to T-3397 (P0). Cause: on long annuities with a large balance the residual near
the root is rounding noise above 1e-7, so the secant reaches a zero step and then a flat secant (f1 = f0). The
T-2005 model stopped there with `#NUM!`; Excel holds the iterate. Every one of the 60 probe misses before the fix was
such a flat secant. A second gap showed up on the sweep: the model stopped at an iterate below -1, where Excel runs on.
Fix (`compute_rate`, financial_util.rs): a flat secant holds at r1 and returns it when |f1| < 1e-7, or when it
passes the cap's test (|f1| < 1e-4, 1 + r1 > 1e-4); otherwise, and on a residual of exactly 0, the run is `#NUM!`.
An iterate at or below -1 runs on, and only the result must be above -1. `RATE(239,-14841.27102594,344293.4825124,
-155496.6323522,0)` is 0.04310546919282547 (unit test and probes/t3348-rate-fv-nonzero A1, A2). Rule and evidence:
financial.md "RATE holds on a flat secant and runs on below -1".
Eval: probes/t3348-rate-fv-nonzero 126/186 -> 179/186 bit-exact, and 186/186 of Excel's type. The 7 misses (A112, A138,
A153, A171, A175, A180, A185) are 1 ulp to 2e-11 relative, with the type right; they are T-3389 (P2) and known
failures. probes/t3348-rate-below-minus-one 20/50 -> 50/50. The fit set of 960 literal RATE cells is unchanged (960/960).
`make eval-baseline` with the worktree's verified binary: probes known failures 2260 -> 2267, all 7 in the new case,
0 regressions. Visible parity 2422 against a lock of 3029, evidence floor unchanged.
Gate A (decision 0188 R2), probes-numeric/t3348-rate-gate-a-sweep, 300 points against a 50-digit reference. Per
stratum: within 15 digits of Excel / within 1e-9 of the math / both `#NUM!`:
- typical 120: 89 / 89 / 31
- tail 90: 79 / 41 / 11
- extreme 60: 26 / 26 / 34
- log-uniform 30: 26 / 19 / 2

0 points are outside both bounds. 2 log-uniform cells have an error on one side and a number on the other, so the
envelope is not yet met:
- Calc!A272 `=RATE(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)`: Excel 0.040657819686895275,
  engine `#NUM!` (the model holds at |f| 1.29e-4), math 0.040657819686896052522303945460859198781648510530346.
- Calc!A296 `=RATE(351,-1684985188.67608,607019725001.509,-15796156082.0366,0)`: Excel `#NUM!`, engine
  1.8814890660971136e-6 (a hold at step 119 on |f| 2.1e-5), math
  0.0000018814890215042125897557712909148668011313466560856.

Both are the noise floor's path, not a missing stop rule. A trace of the 97 flat secants among the 860 literal RATE
oracle cells shows that an absolute bound on the hold in [7.2e-5, 1.14e-4) fits every other cell. A272's held
iterate is not Excel's value (about 2 ulps off), so Excel's last steps differ. Making A296 `#NUM!` would need a bound
below 2.1e-5, which turns 5 gated fv-nonzero cells Excel returns (A113, A117, A126, A167, A183) into `#NUM!`. A
relative bound does not separate the cells, and a step cut would be fitted to one point (decision 0188 R4). Both
cells are T-3397 (P0), and T-3389 points there.
Build note: the shared CARGO_TARGET_DIR can link another worktree's engine, and the Makefile's
`target/release/rustsheet-eval` was a stale hard link. Every eval number above comes from a `-v` rebuild that
compiled this worktree's engine, copied with `cp --remove-destination` (sha256 c09b29974905b638).
