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
The parser now binds every LET name and LAMBDA parameter (`bound_names`, was `reference_shaped_names`) and skips the function lookup for a bound callee, so `LET(Pv,LAMBDA(a,a),Pv(1))` is 1 and `LET(Pv,LAMBDA(a,Pv),Pv(1))` `#NAME?`. New probe probes/let-name-function-spelled-call (excel-graph: other function names, any case, a LAMBDA parameter, arity errors, a called non-LAMBDA, scope end) passes fully; docs/excel-semantics/references.md updated. Eval: probes known failures 1903 -> 1900 (3 fixed), plus 16 new cells passing; no regressions.
