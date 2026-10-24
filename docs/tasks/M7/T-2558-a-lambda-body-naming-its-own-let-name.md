---
id: T-2558
title: "A LAMBDA body naming its own LET name is #NAME?, not #ERROR!"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=LET(Pv,LAMBDA(a,Pv),Pv(1))` (and the same with no defined name: a LAMBDA body naming its own LET name) is `#NAME?`, as in Excel, instead of the engine's parse-error value `#ERROR!`.

## Context
Found by the Excel-fidelity review of T-2057. Excel gives `#NAME?` for `=LET(Pv,LAMBDA(a,Pv),Pv(1))` with a workbook name `Pv` (probes/let-name-shadows-defined-name B2, excel-graph; docs/excel-semantics/references.md "A LET name or LAMBDA parameter shadows a defined name"); `LET(pv,pv+1,pv)` is `#NAME?` too and already matches. Probe the no-defined-name twin `LET(y,LAMBDA(a,y),y(1))` first (`excel-probe`), record it in docs/excel-semantics/references.md, then fix.

## Acceptance criteria
- [ ] Probe cell with oracle value; the engine matches it; unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
