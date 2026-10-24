---
id: T-2567
title: "A LET name spelled like a function wins in call position (Pv(1))"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A LET name (or LAMBDA parameter) spelled like a built-in function, called by name, is the variable: `LET(Pv,LAMBDA(a,a),Pv(1))` is 1 and `LET(Pv,LAMBDA(a,Pv),Pv(1))` is `#NAME?`, as in Excel, instead of `#ERROR!`.

## Context
Oracle: probes/let-lambda-body-own-name A6:A7 (excel-graph); also probes/let-name-shadows-defined-name B2. `parse_named_call` (parser/operands.rs) looks the name up as a function (`lookup_stored`) before any LET-bound name, and the parser keeps no list of every LET name (`reference_shaped_names` holds only reference-shaped ones). Probe other function names and a LAMBDA parameter first; `PV(1)` as a plain call is refused by Excel.

## Acceptance criteria
- [x] Probe cells match the oracle and the engine agrees; unit test.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel's rule turned out to depend on the form: stored, a call of a bound name is written `_xlpm.Pv(1)` and calls the variable (`LET(Pv,LAMBDA(a,a),Pv(1))` is 1); typed, the function always wins (`=LET(Sum,LAMBDA(a,a*2),Sum(3))` is `SUM(3)`, 3) and `=LET(Pv,LAMBDA(a,a),Pv(1))` is refused. The parser now records every LET name and LAMBDA parameter (`bound_names`, was `reference_shaped_names`; `bind_name`, was `bind_reference_shaped_name`) and, in unprefixed stored text only (A1 grammar, not typed: `Input::StoredFormula`, the harness's case formulas), skips the function lookup for a bound callee (`calls_bound_name`). Stored R1C1 text and typed input keep the function, so `LET(sum,SUM(1,2),sum*2)` is still 6. New probes: probes/let-name-function-spelled-call (stored, 16 cells) and probes/let-function-name-typed-* (10 typed cases with FORMULATEXT); docs/excel-semantics/references.md updated. Eval: probes known failures 1903 -> 1900 (3 fixed), plus 11 new cases passing; no regressions. Follow-up: T-2568 (typed refusal of the three refused formulas).
