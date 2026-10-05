---
id: T-0149
title: "Intersection with @, union and (A1)(A2) operands"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine matches Excel for three intersection forms it gets wrong today: an `@` operand, a
union operand, and `(A1)(A2)` with no space.

## Context
Evidence (oracle excel-graph, 2026-10-05, T-0138); docs/excel-semantics/references.md
"Intersection operands must be references":
- `=A1 @A1` = 1 (engine `#ERROR!`): probes/intersection-operands H10.
- `=@A1:A3 A2:B2` = 2 in row 11 (engine `#VALUE!`): H11. `@A1:A3` alone would be `#VALUE!` in row 11, so
  the space binds tighter than `@`: it is `@(A1:A3 A2:B2)`.
- `=(A1)(A2)` = `#REF!` (engine `#ERROR!`): H13. Also fix the hand expectation of
  eval/suites/smoke/cases/trailing-input.case.json B6 (`#ERROR!`) to `#REF!` in the same PR.
- `=A1:A3 (A2:B2,A2)` = `#VALUE!` (engine `#ERROR!`): probes/intersection-function-operands H5. The union
  has two areas (A2 twice), and a union in a value context is `#VALUE!` (references.md, T-0134).
Why `(A1)(A2)` is `#REF!` is not known; find a rule that fits, and probe more forms
(`(A1)(A1)`, `(A1:A3)(A2:B2)`) with the `excel-probe` skill before generalising.

## Acceptance criteria
- [ ] The four cells above pass, with no new F lines in any suite (`make eval-baseline`).
- [ ] Every new rule found is recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Separators and non-reference operands (T-0148).
