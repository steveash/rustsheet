---
id: T-2567
title: "A LET name spelled like a function wins in call position (Pv(1))"
milestone: M7
lane: engine
status: in-progress
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
- [ ] Probe cells match the oracle and the engine agrees; unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
