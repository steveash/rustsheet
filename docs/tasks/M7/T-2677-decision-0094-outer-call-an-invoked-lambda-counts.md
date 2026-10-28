---
id: T-2677
title: "Decision 0094 outer call: an invoked LAMBDA(..)(..) counts as a LAMBDA outer call"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A generated cell whose formula is one invoked LAMBDA, `LAMBDA(x,x+1)(3)`, counts as a LAMBDA outer-call cell in the
"Fifteen-digit parity checkpoint", so LAMBDA can meet decision 0094's 200-generated-cell bar.

## Context
- Gap triage 2026-10-25: LAMBDA has 6,317 Excel cells and 0 generated outer-call cells (`report.json`
  `v15_checkpoint`), so it is uncovered however many cells the generator writes.
- Cause: `outer_call` in crates/rustsheet-eval/src/formula_calls.rs returns a name only when the call spans the whole
  formula, so `LAMBDA(x,x+1)(3)` (text after LAMBDA's closing parenthesis) is `None`. The generator writes LAMBDA
  invoked: catalog `invoke: true`, tools/src/rustsheet_tools/gen/generator.py appends `(<args>)`. A bare
  uninvoked LAMBDA in a cell is `#CALC!`, so invoking it is the only way to score its value.
- The checkpoint counts the outer call through `outer_call` (decision 0094). Only invocation lists should be accepted:
  one or more `(...)` groups directly after the call, with nothing else (`LAMBDA(x,x)(1)(2)` is still LAMBDA,
  `LAMBDA(x,x)(1)+1` is not).

## Acceptance criteria
- [ ] `outer_call("LAMBDA(x,x+1)(3)")` is `Some("LAMBDA")`, and so is `_xlfn.LAMBDA(_xlpm.x,_xlpm.x)(1)`;
      `LAMBDA(x,x)(1)+1`, `-LAMBDA(x,x)(1)` and `LAMBDA(x,x)(1` stay `None`. Unit tests in formula_calls.rs.
- [ ] The existing `outer_call` tests still pass (`ABS(1)+SQRT(4)` is `None`).
- [ ] After `make eval`, LAMBDA's checkpoint row counts the invoked-LAMBDA cells already in generated suites. The
      Outcome states the new count. No other function's generated count drops.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Generating more LAMBDA cells (T-2675).
- Counting LAMBDA inside LET, MAP or other helpers as an outer call.

## Outcome
`outer_call` (crates/rustsheet-eval/src/formula_calls.rs) accepts one or more `(...)` invocation groups after a
LAMBDA call, only for LAMBDA (`ABS(1)(2)` stays `None`). Unit test `an_invoked_lambda_is_a_lambda_outer_call`.
After `make eval`, LAMBDA's checkpoint row has 89 generated outer-call cells (was 0); the change only accepts more
LAMBDA formulas, so no other function's count can drop. Still short of the 200-cell bar: generating more is T-2675.
