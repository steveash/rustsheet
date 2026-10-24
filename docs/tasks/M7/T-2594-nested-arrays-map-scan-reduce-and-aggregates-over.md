---
id: T-2594
title: "Nested arrays: MAP, BYROW, BYCOL, REDUCE, the aggregates and reference cells over a GROUPBY/PIVOTBY result (SPEC-0018)"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2592, T-2593, T-2602]
---

## Goal
MAP, BYROW, BYCOL, REDUCE and the aggregates read a nested GROUPBY/PIVOTBY cell as Excel does, and a one-cell reference body makes a reference cell (SPEC-0018 rules 4, 5 and 7).

## Context
docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md rules 4, 5 and 7 and Design "Consumers with their own rule".
- MAP, BYROW, BYCOL and REDUCE hand the LAMBDA each nested cell whole. A step that returns an array is `#CALC!` (A104).
- SUM, COUNT and COUNTA flatten nested cells. AVERAGE is `#N/A`, ARRAYTOTEXT `#CALC!`, and TEXTJOIN and CONCAT `#VALUE!`.
- A one-cell reference body makes a `Reference` cell: it displays `#VALUE!`, and INDEX reads it as the referenced value (A21).

Owned:
- probes/t2311-pivotby-value-columns-error P!D12. Excel's error comes from its `IFERROR(x&"",..)` step, which stays an array, so
  it is `#CALC!` (rule 5). Ours is a value.
- probes/t2592-nested-groupby-cells A1:A15, A21 and A100:A109, whichever of them T-2593 leaves failing.

If the aggregates and the `Reference` kind together grow the task past size S, split it (planner) first.

## Acceptance criteria
- [ ] P!D12 and T-2592's remaining cells pass and leave eval/baseline, with no new known-failure lines.
- [ ] Every flatten and every MAP/BYROW/BYCOL/REDUCE hand-over charges SPEC-0018's walk budget: `SUM(VSTACK(r,r,..))` and a REDUCE
      over a fan-out of a ~2^20-cell nested cell are `#ERROR!` in bounded time (unit tests).
- [ ] arrays.md records each rule with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SCAN over a GROUPBY result (T-2398).
- An operator over the whole result (`&""` keeps the shape, A13; what each cell holds is unprobed).
