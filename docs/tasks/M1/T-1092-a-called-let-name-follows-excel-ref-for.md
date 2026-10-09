---
id: T-1092
title: "A called LET name follows Excel: #REF! for a reference, odd results for values"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A LET name called like a function (`f(2)` inside `LET(f,...)`) gives Excel's value when the name is
not bound to a LAMBDA.

## Context
Found by T-0149 (docs/excel-semantics/references.md "Calling a parenthesized operand"). A scratch
oracle run (excel-graph, 2026-10-07, not committed) gave: `=LET(f,A1,f(2))` `#REF!`, `=LET(f,1,f(2))`
`#NAME?`, `=LET(f,"a",f(2))` 0, `=LET(f,A1+0,f(2))` `#NAME?`, `=LET(f,1/0,f(2))` `#DIV/0!`,
`=LET(f,{1,2},f(2))` `#NAME?`. The engine gives `#VALUE!` (`Model::call_lambda`, "Expected a LAMBDA").
The text result 0 is odd enough that the case builder's `_xlpm.` naming of a called LET name may play
a part: first check how Excel itself stores such a formula, then commit the probe as a case and
fill it. LET binds a single cell by value (`evaluate_node_in_context`), so the `#REF!` for a bound
reference needs the binding to keep its reference, as `call_callee` (functions/logical/lambda.rs)
expects.

## Acceptance criteria
- [ ] A committed probe case with the forms above (one workbook per form Excel may refuse) passes.
- [ ] references.md "Calling a parenthesized operand" records the rule, without the UNVERIFIED note.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
