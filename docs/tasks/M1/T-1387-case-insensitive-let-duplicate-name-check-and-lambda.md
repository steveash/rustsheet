---
id: T-1387
title: "Duplicate LET/LAMBDA names that differ only in case (follow-up of T-1378)"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Establish, then make the engine match, what Excel does with duplicate names that differ only in case: `LET(x,1,X,2,x)`, `LAMBDA(x,X,x+X)(1,2)` (and non-ASCII names such as `LET(é,1,É+1)`).

## Context
T-1378 made LET names and LAMBDA parameters case-insensitive (`same_variable`, functions/logical/let.rs; probes/let-lambda-name-case). Excel for the web refused (`FileCorruptTryRepair`) the spec2xlsx workbooks holding the three forms above, so they are unprobed. `seen_names` in let.rs (~138), lambda.rs and parser/lambda.rs still compare case-sensitively. Find out whether the refusal is Excel rejecting the formula (then the engine should refuse it too, see syntax.md) or a spec2xlsx encoding problem (the `é` form suggests the latter); desktop Excel may be needed (M14, decision 0039).

## Acceptance criteria
- [ ] A probe case with oracle values for the forms above; the engine matches it. Docs entry in references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
