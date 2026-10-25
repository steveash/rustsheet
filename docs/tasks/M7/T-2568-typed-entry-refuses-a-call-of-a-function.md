---
id: T-2568
title: "Typed entry refuses a call of a function-spelled LET name with the function's wrong arity (=LET(Pv,LAMBDA(a,a),Pv(1)))"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Typed entry (API `Input::Formula`/`Parse`) refuses `=LET(Pv,LAMBDA(a,a),Pv(1))`, `=LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))` and `=LET(Pv,2,Pv(1))`, as Excel refuses them typed.

## Context
Found by T-2567: typed, a call of a LET name or LAMBDA parameter spelled like a function is the function (`=LET(Sum,LAMBDA(a,a*2),Sum(3))` is `SUM(3)`), and Excel for the web refuses the three formulas above (Graph PATCH 400 on the `entered` cell; the cases could not be filled and were not kept), presumably because PV takes at least three arguments, as `=PV(1)` is refused. The engine parses them as `PV(1)` and gives a value instead. Check first how the engine treats a typed `=PV(1)`: if a general typed arity refusal is missing, that is the task (reuse `Parser::typed_refusal`, decision 0103). docs/excel-semantics/references.md (T-2057 entry) has the evidence.

## Acceptance criteria
- [x] The three formulas fail typed with `EditError::InvalidInput` and are accepted stored (`Input::StoredFormula`); API unit test.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Typed entry (`Input::Formula`/`Parse`) now refuses any call of a function with fewer or more arguments than the function
takes, using the lifting table's counts (`Parser::note_argument_count`, reported by `Parser::typed_refusal`). The three
formulas refuse because typed their callee is PV (T-2567), and `=PV(1)` itself refuses as Excel does; stored text keeps
binding the variable (1, 3, `#NAME?`). Functions outside the lifting table are not checked (no false refusals). The
lifting table's minimums match eval/catalog/excel-functions.tsv for 349 of 350 functions; ACCRINTM's `par` is optional in the
engine and required in Microsoft's syntax line, so `ACCRINTM` with three arguments stays accepted (permissive). API test
`typed_calls_with_a_wrong_argument_count_are_refused`; syntax.md and references.md updated. Eval: no change (no `entered`
case has a wrong-count call). Off-cycle SORTBY counts stay accepted typed (`ArgKinds::takes_count`); a Python test pins the
lifting table's minimums to excel-functions.tsv.
