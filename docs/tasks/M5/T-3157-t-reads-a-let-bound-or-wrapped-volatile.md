---
id: T-3157
title: "T reads a LET-bound or @-wrapped volatile pick once"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3142]
---

## Goal
`T(LET(i,RANDBETWEEN(1,2),CHOOSE(i,D1,REPT("x",256))))` and `T(@CHOOSE(RANDBETWEEN(1,2),D1,REPT("x",256)))`
(with 256 units in D1) read their value and run their cell-text test on one draw, as T-3142 made a root IF, CHOOSE, IFS or
SWITCH do.

## Context
- T-3142's Excel-fidelity review: `t_argument` (`functions/text/t_function.rs`) picks only IF, CHOOSE, IFS and
  SWITCH at the argument's root. A LET body and an `ImplicitIntersection` still go down the `Written` path, where
  `written_reads_cell_text` → `is_reference` evaluates the argument again. The same happens when such a LET is the
  arm a root picker picked (`picked_arm_is_reference`).
- Same shape of test as `text_functions/fn_t_volatile_pick.rs`: the result is never `#VALUE!` for a D1 pick.

## Acceptance criteria
- [x] Both shapes above never give `#VALUE!` when D1 is picked, over 200 recalculations; the T-3107 probe shapes
      are unchanged (`make eval-check`: 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions that test `is_reference` after reading a value.

## Outcome
- T's kernel (`t_function.rs`): a LET in T's argument, at its root or picked by a root IF, CHOOSE, IFS or SWITCH, is
  read inside the LET's scope (`t_let_argument`, through `let_scope`), so its body's value and its cell-text test
  share the LET's bindings and a volatile value is drawn once. The `@` of a picker is read once as `@` reads its
  operand (`t_at_picker_argument`); its cell-text test is whether that operand was a reference. A LET body as
  written keeps the reference-context test `is_reference` gave a LET, so `T(LET(x,D1,+x))` keeps its (unprobed) "".
- Tests: `text_functions/fn_t_volatile_pick.rs`. The volatile test fails on the old code within 200
  recalculations; the LET and `@` shape test pins the readings the old code gave.
- Eval delta: none (`make check`: 0 regressions, 0 fixes; probes known failures 3534 → 3534).
