---
id: T-2055
title: "A LET name shadows a defined name of the same name in any case"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
With a workbook defined name `Mk`, `=LET(mk,LAMBDA(a,a+1),mk(1))` reads the LET name (2), as LET and LAMBDA
names shadow outer names elsewhere, instead of `#VALUE!`.

## Context
Found by the Excel-fidelity review of T-0439 (on main too): a LET name that matches a defined name
ignoring case resolves to the defined name. Probe first (`excel-probe`): a defined name `Mk` (a LAMBDA
and a plain value) with `LET(mk,..,mk(1))`, `LET(Mk,..,Mk+1)` and a LAMBDA parameter of the same name;
record the rule in docs/excel-semantics/references.md, then fix name resolution in the parser/evaluator.

## Acceptance criteria
- [ ] Probe case with oracle values; the engine matches it; unit tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names spelled like cells.
