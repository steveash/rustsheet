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
