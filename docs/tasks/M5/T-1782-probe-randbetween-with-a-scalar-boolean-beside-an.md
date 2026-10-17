---
id: T-1782
title: "Probe RANDBETWEEN with a scalar boolean beside an array bound, and which error wins per element"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles two RANDBETWEEN array-lift corners that T-1316 (#891) implemented without
probing them, and the engine matches it.

## Context
The steward's excel-fidelity re-review of #891 (2026-10-17) found these gaps (MINOR):
- `probes/randbetween-lift-operands` B7, `ROWS(RANDBETWEEN(SEQUENCE(3),TRUE))` = 3, cannot tell the
  engine's reading (a scalar TRUE beside an array is `#VALUE!` in every element) from TRUE read as 1
  (1, #NUM!, #NUM!). T-1316's Outcome says "a scalar bound still rejects booleans", and nothing
  probes that in the lifted case.
- When both bound elements at one position are errors (`RANDBETWEEN({"x"},{TRUE})`, or an error
  bottom against an error top), `lift_number_operands` (cast.rs) returns the bottom's error. No
  probe backs that order.
Use only scalar-returning wrappers (COUNT, ISERROR, ERROR.TYPE over INDEX), as the existing case
does, because RANDBETWEEN is volatile. Code: functions/math_and_trigonometry/random.rs. Record the
results in docs/excel-semantics/numbers.md beside the T-1316 entry.

## Acceptance criteria
- [ ] A probe (excel-graph) covers a scalar TRUE/FALSE beside an array bound in either position, and
      error-vs-error precedence per element (bottom vs top, text vs boolean).
- [ ] The engine matches every new cell; numbers.md records the rule with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other lifted functions' error order.
