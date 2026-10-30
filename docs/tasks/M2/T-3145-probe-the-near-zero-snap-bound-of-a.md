---
id: T-3145
title: "Probe the near-zero snap bound of a SUM with three or more addends: running total or left argument"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for which value sets the near-zero snap bound of a SUM with three or more addends, and the engine matching it.

## Context
- T-3100 (docs/excel-semantics/numbers.md "Near-zero snapping", probes/t3100-snap-operand-order) found the snap
  bound is 8 ULP of the left addend, but every SUM it probed has two addends. `SumParts::snapped`
  (functions/math_and_trigonometry/sum.rs) takes the running total before the last addend as the left addend; with
  three or more addends that may differ from the first argument, and earlier multi-addend probes (0.1, 0.2, -0.3) fit
  either reading.
- Probe forms where the running total and the first argument have different exponents, e.g.
  `SUM(0.5,0.5,-(1-k*2^-52))` against `SUM(1-k*2^-52,0,-1)`-style orders, k = 3 to 8, from cells and as an array,
  plus AVERAGE and SUBTOTAL(9) over the same values.

## Acceptance criteria
- [ ] An excel-graph probe (milestone M2) with those cells; numbers.md states the rule for 3+ addends.
- [ ] If the engine differs, `SumParts::snapped` is changed to match, with unit tests either side of the bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The opposite-sign condition (separate probe if wanted).
