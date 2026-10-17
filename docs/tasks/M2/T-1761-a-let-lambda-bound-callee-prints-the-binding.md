---
id: T-1761
title: "A LET/LAMBDA-bound callee prints the binding's spelling in FORMULATEXT"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A callee bound by an enclosing LET or LAMBDA prints with the binding's spelling in FORMULATEXT, whatever case the call used.

## Context
Excel: `=LET(Gg,LAMBDA(x,x*2),gG(3))` prints `...,Gg(3))` and `=LAMBDA(Hh,Hh(1))(LAMBDA(x,x))` keeps `Hh`; ours prints the lowercase name (`gg(3)`, `hh(1)`). Failing cells: Sheet1!B5, B7. Semantics: docs/excel-semantics/references.md "Unknown function call names" (T-1756); registry `CallNameSpellings` (T-1719).

## Acceptance criteria
- [ ] - [ ] Sheet1!B5 and B7 of probes/unknown-function-name-reach pass; B4 and B6 (already passing) keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other cells of probes/unknown-function-name-reach.
