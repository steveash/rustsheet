---
id: T-1092
title: "A called LET name follows Excel: #REF! for a reference, odd results for values"
milestone: M1
lane: engine
status: done
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
- [x] A committed probe case with the forms above (one workbook per form Excel may refuse) passes.
- [x] references.md "Calling a parenthesized operand" records the rule, without the UNVERIFIED note.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (excel-graph, 2026-10-09): a called LET name or LAMBDA parameter bound to a reference is
`#REF!` (INDEX's reference included), a number or boolean is `#NAME?`, text is the number 0, an
error is itself and an array is called per element. Excel stores the call as `_xlpm.f(2)` and reads
it back as `f(2)`, so the builder's naming is Excel's own. `Model::call_variable` implements the
value rule; LET/LAMBDA bindings are now read in reference context (`bind_argument`), so INDEX keeps
its reference. `=LET(f,"",f(2))` makes Excel for the web refuse the workbook (not a case).
Eval delta: +10 probe cases (let-name-call*, lambda-param-call), 38 cells, all pass; no regressions.
