---
id: T-1321
title: "ANCHORARRAY(@IF(TRUE,D1)) spills the anchor's array (engine #VALUE!; T-1285)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
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

## Outcome
`eval_ref_or_index` (reference_operators.rs) keeps the reference of an `@` whose operand is an IF picking one cell, as it already did for `@INDEX`, so `ANCHORARRAY(@IF(TRUE,D1))` spills D1#. Unit test in test_spill_functions.rs. Eval delta: probes known failures 930 -> 927 (fixed 3: at-signed-operand H71:H73), 0 regressions.
