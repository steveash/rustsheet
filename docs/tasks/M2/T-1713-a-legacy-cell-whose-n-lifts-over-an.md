---
id: T-1713
title: "A legacy cell whose N lifts over an array is #VALUE!"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (non-array) cell `=N(IF(1,{2;3}))` gives `#VALUE!`, as Excel does, instead of spilling 2, 3.

## Context
probes/n-array-lift S!D4:D5 (excel-graph, T-1616). T-1616 made N lift over an array (dynamic `=N(IF(1,{2;3}))` spills 2, 3, matching Excel), but a legacy cell holding the same formula is `#VALUE!` in Excel, not its top-left element, while legacy `SUM(N({1,2}))` is 3 and `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` 5. Likely shape: in the legacy rewrite (`add_implicit_intersection`, expressions/parser/static_analysis/legacy.rs) a top-level N over an array argument is `#VALUE!`; probe first whether other scalar-argument functions (ABS, LEN) behave the same in a legacy cell. docs/excel-semantics/information.md "N over an array lifts element-wise".

## Acceptance criteria
- [ ] probes/n-array-lift S!D4:D5 leave eval/baseline/probes.txt.
- [ ] information.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Progress (2026-10-16, in flight)
- Probe probes/legacy-array-result (excel-graph) answers "do other scalar-argument functions behave
  the same": no. ABS, LEN, ISNUMBER, negation and a top-level IF get an `@` in a legacy cell (top-left
  value), and so do N of an array constant and of TRANSPOSE. Only N and T over `IF(...)` get no `@`
  and are `#VALUE!` (A10, G10, I10). Recorded in references.md "Legacy formulas put `@` on
  array-valued calls and array constants". The general `@` is filed as T-1717 (engine, opus).
- Next: decide whether T-1713 lands before or with T-1717. A narrow fix needs the legacy evaluation
  of N/T to see an array argument that is not `@`-ed and give `#VALUE!`; check whether CHOOSE,
  INDEX or OFFSET arguments behave like IF (probe) before choosing the rule.
