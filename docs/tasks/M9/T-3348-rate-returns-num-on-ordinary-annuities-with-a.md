---
id: T-3348
title: "RATE returns #NUM! on ordinary annuities with a nonzero fv"
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
- [ ] `RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)` is 0.0431054691928255 (a unit test and a probe cell).
- [ ] Ordinary annuities with fv ≠ 0 across rates 1e-6 to 1 and nper 1 to 1,000 match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells; where Excel gives
      `#NUM!` (for example its documented 20-iteration limit) the engine gives `#NUM!` too.
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
