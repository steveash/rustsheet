---
id: T-1981
title: "Probe legacy @ on array constants under unary, parenthesized, concat, comparison and range-mixed operators"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-probed legacy `@` placement for array constants under unary signs, parentheses, `&`, comparisons and range-mixed operands (`ABS(-{2;3})`, `N(({2;3}))`, `LEN({"ab";"c"}&"x")`, `N({2;3}+A1:A2)`), plus a one-element constant outside a value position (`SUM(ABS({5}+0))`).

## Context
T-1911 put `@` on every array constant reached by `for_each_value_operand` in legacy scalar arguments (static_analysis/legacy.rs `add_array_at`), but only `{2;3}+0` was probed (probes/legacy-array-at-shapes). The reviewer flagged the rest as extrapolated.

## Acceptance criteria
- [x] A probe case covers each shape with Excel values and FORMULATEXT; the implementation and references.md are corrected where Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/legacy-array-at-operators (10 shapes, legacy cell + dynamic twin + FORMULATEXT) with excel-graph values. The engine already matched all 37 cells, so no implementation change; references.md updated with the evidence. Eval delta: +37 passing probe cells.
