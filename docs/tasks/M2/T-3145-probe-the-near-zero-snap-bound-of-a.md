---
id: T-3145
title: "Probe the near-zero snap bound of a SUM with three or more addends: running total or left argument"
milestone: M2
lane: research
status: done
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
- [x] An excel-graph probe (milestone M2) with those cells; numbers.md states the rule for 3+ addends.
- [x] If the engine differs, `SumParts::snapped` is changed to match, with unit tests either side of the bound.
- [x] `make check` green; reviewers approve.

## Out of scope
- The opposite-sign condition (separate probe if wanted).

## Outcome
Done 2026-10-30. probes/t3145-snap-multi-addend (384 cells, excel-graph): with three or more addends the snap
bound is 8 ULP of the running total before the last addend. Four blocks whose first argument or last addend
predict another cut-off (k ≤ 3 or k ≤ 15) all snap exactly k ≤ 7 (running total 1), in SUM over arguments, a range
and an array, SUBTOTAL(9), the `+` chain, and AVERAGE, which snaps the sum before dividing. The engine's
`SumParts::snapped` already used the running total: 384/384 pass, no code change. numbers.md records the rule.
Eval delta: +384 newly covered probe cells, all passing; no regressions.
