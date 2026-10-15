---
id: T-1633
title: "Legacy import: @ on an operator over a range inside CHOOSE and on an array constant in a scalar argument"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (non-dynamic) formula gets Excel's implicit-intersection `@` placement for a range operand of an operator
inside CHOOSE and for an array constant in a scalar argument.

## Context
Found by T-1265: probes/legacy-one-cell-shape-intersection (excel-graph) FORMULATEXT cells H3, M3 and P3. Excel shows
`=CHOOSE(1,@E1:E1+1)` and `=CHOOSE(1,@E1:E2+1)` (the `@` on the range operand inside the arm, not on the call), and
`=ABS(@{5})` (an array constant in ABS's scalar argument). Ours gives `=@CHOOSE(1,E1:E1+1)`, `=@CHOOSE(1,E1:E2+1)`
and `=ABS({5})` (main gives `=CHOOSE(1,E1:E1+1)` for H3). docs/excel-semantics/references.md, "One-cell ranges and
1x1 arrays are not scalars". Code: `add_implicit_intersection` and `legacy_intersects` in
crates/rustsheet-engine/src/expressions/parser/static_analysis{.rs,/legacy.rs}.

## Acceptance criteria
- [ ] H3, M3 and P3 of probes/legacy-one-cell-shape-intersection pass; baseline updated with `make eval-baseline`.
- [ ] No other legacy probe regresses (`make eval-check`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The scalar-or-dynamic classification of these shapes (T-1265).

## Outcome
CHOOSE's value arms that are operations (`CHOOSE(1,E1:E2+1)`) now take the `@` on their range operand
instead of on the call, and a one-element array constant in a scalar argument (`ABS({5})`) takes an `@`
(`legacy.rs`: `legacy_value_operand`, `add_scalar_array_at`). Larger array constants stay unprobed.
Eval delta: probes known failures 5514 -> 5511 (H3, M3, P3), 0 regressions.
