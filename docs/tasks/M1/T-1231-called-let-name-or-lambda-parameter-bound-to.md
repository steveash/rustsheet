---
id: T-1231
title: "Called LET name or LAMBDA parameter bound to a union is #REF!, not #VALUE! (follow-up of T-1224)"
milestone: M1
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Calling a LET name or LAMBDA parameter bound to a union gives `#REF!` as Excel does.

## Context
probes/let-name-call-forms H1, H2, H13 (excel-graph 2026-10-09, T-1224): `=LET(f,(A1,A2),f(2))`, `=LAMBDA(f,f(2))((A1,A2))` and `=LET(f,(A1,A2),f())` are `#REF!`; the engine gives `#VALUE!` (`Binding::Areas` -> `several_areas`, functions/logical/lambda.rs). See docs/excel-semantics/references.md "Calling a parenthesized operand".

## Acceptance criteria
- [ ] the three cells pass and leave eval/baseline
- [ ] `make check` green; reviewers approve.

## Out of scope
-
