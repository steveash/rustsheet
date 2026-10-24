---
id: T-2558
title: "A LAMBDA body naming its own LET name is #NAME?, not #ERROR!"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=LET(Pv,LAMBDA(a,Pv),Pv(1))` (and the same with no defined name: a LAMBDA body naming its own LET name) is `#NAME?`, as in Excel, instead of the engine's parse-error value `#ERROR!`.

(Done for names that are not function names; the `Pv` spelling moved to T-2567.)

## Context
Found by the Excel-fidelity review of T-2057. Excel gives `#NAME?` for `=LET(Pv,LAMBDA(a,Pv),Pv(1))` with a workbook name `Pv` (probes/let-name-shadows-defined-name B2, excel-graph; docs/excel-semantics/references.md "A LET name or LAMBDA parameter shadows a defined name"); `LET(pv,pv+1,pv)` is `#NAME?` too and already matches. Probe the no-defined-name twin `LET(y,LAMBDA(a,y),y(1))` first (`excel-probe`), record it in docs/excel-semantics/references.md, then fix.

## Acceptance criteria
- [x] Probe cell with oracle value; the engine matches it; unit test.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes/let-lambda-body-own-name (excel-graph): all five forms `#NAME?` except the control. `let_scope` no longer patches a name's id into its own value expression, so the LAMBDA body does not read the LET-bound lambda (it gave `#CALC!`). Unit test added.
The task's own example `LET(Pv,LAMBDA(a,Pv),Pv(1))` (probes/let-name-shadows-defined-name B2) stays `#ERROR!`: `Pv` is also the function PV, so `Pv(1)` is read as a call of it. Filed as T-2567.
