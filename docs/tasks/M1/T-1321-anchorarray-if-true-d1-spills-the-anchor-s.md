---
id: T-1321
title: "ANCHORARRAY(@IF(TRUE,D1)) spills the anchor's array (engine #VALUE!; T-1285)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`ANCHORARRAY(@IF(TRUE,D1))` spills D1's array like Excel.

## Context
Excel spills 1,2,3 (D1 = SEQUENCE(3)) and prints `=@IF(TRUE,D1)#`; the engine gives #VALUE!. Failing cells probes/at-signed-operand.case.json Sheet1!H71:H73 (T-1285).

## Acceptance criteria
- [x] `probes/at-signed-operand.case.json` H71:H73 pass and leave the baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
What I tried: in `eval_ref_or_index` (reference_operators.rs) I merged the `@INDEX` arm with a new `@IF` arm (an `@` whose operand is an IF that picks one cell keeps the reference), so `ANCHORARRAY(@IF(TRUE,D1))` spills D1#. Unit tests for the spilling case and for `@IF(FALSE,D1,5)` (#VALUE!), probes/at-signed-operand H71:H73 pass, `make check` is green, and the baseline lost exactly H71:H73 (930 -> 927).

Review findings (security and excel-fidelity reviewers REQUEST_CHANGES; architecture approved; quality's duplicate-arm finding is fixed):
- Security (MAJOR): when the IF does not pick a single cell the arm falls back to `evaluate_node_with_reference(node)`, which evaluates the IF subtree a second time. Nested `ANCHORARRAY(@IF(TRUE,ANCHORARRAY(@IF(TRUE,...))))` costs about 2^depth (n=20: ~0.5 s), a one-cell CPU DoS against directive 5. The existing `@INDEX` arm has the same shape. Fix: evaluate once and reuse the result for the fallback, and add a regression test with about 40 nested levels that finishes quickly. Fix `@INDEX` the same way or file a follow-up.
- Excel fidelity (MAJOR): `eval_ref_or_index` has two other callers, the VLOOKUP/HLOOKUP index argument (`lookup_index_arg`, functions/lookup_and_reference/index_arg.rs) and LOOKUP's result_vector (`lookup_result`, lookup.rs), so `@IF(...)` there now becomes a reference while plain `IF(...)` stays a value, with no oracle evidence. Fix: limit the new behaviour to the spill-anchor caller (`SpillRangeOperator`, model.rs ~765) or a separate helper, or probe `VLOOKUP(...,@IF(TRUE,B1))` with B1 text "2" and `LOOKUP(99,A1:A3,@IF(TRUE,F1))` first.
- Minor: `ANCHORARRAY(@IF(TRUE,D1:D3))` (the IF picks a multi-cell range) is unprobed; file a probe follow-up. (My unit test for it was wrong: in H1 it intersects to D1 and gives 1, so I removed it.)

Next: move the IF handling out of `eval_ref_or_index` into the spill-anchor evaluation, evaluate the operand once, add the nested regression test, re-run `make check` and the reviewers.
