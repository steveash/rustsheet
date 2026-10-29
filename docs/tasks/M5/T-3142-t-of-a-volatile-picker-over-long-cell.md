---
id: T-3142
title: "T of a volatile picker over long cell text re-evaluates the pick"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`T(CHOOSE(RANDBETWEEN(1,2),D1,REPT("x",256)))` agrees with Excel (computed branch `#VALUE!`, cell branch "") on the branch the first evaluation took.

## Context
T-3107 `reads_cell_text` (t_function.rs) calls `is_reference`, which evaluates a picker again; a volatile pick can differ from the first pass. Fix by recording whether the first evaluation came through a reference, not evaluating twice. Probe first if Excel evidence is missing.

## Acceptance criteria
- [x] The pick used for the value is the pick used for the reference test; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- T's kernel (`t_function.rs`, `t_argument`) replaces each IF, CHOOSE, IFS and SWITCH at its argument's root by
  the argument it picks (`pick_step`, condition or index read once), reads that arm as the picker would
  (`picked_value`, new beside `pick_step`) and tests the same arm for cell text with `picked_arm_is_reference`
  (new beside `is_reference`, sharing its `pick_reference` step), so a volatile index is drawn once. A written
  argument keeps its test unchanged.
- Tests: `text_functions/fn_t_volatile_pick.rs`. The volatile test fails on the old code (it saw `#VALUE!` for a
  D1 pick within 200 recalculations); the probe-backed IF/CHOOSE shapes pass on both.
- Eval delta: none expected (no eval case uses a volatile picker under T).
