---
id: T-1387
title: "Case-insensitive LET duplicate-name check and LAMBDA parameter lookup (follow-up of T-1378)"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Probe, then make the engine match, how case-insensitive names behave in `LET(x,1,X,2,x)` (duplicate binding), LAMBDA parameters (`LAMBDA(x,X+1)(2)`) and nested shadowing with different case.

## Context
T-1378 made LET names case-insensitive (`same_variable`, functions/logical/let.rs; `assign_variable_ids` takes `ci`, true for LET only, false for LAMBDA binding). `seen_names` in let.rs (~140), lambda.rs:206 and parser/lambda.rs:64 still compare case-sensitively; Excel's behavior is unprobed.

## Acceptance criteria
- [ ] A probe case with oracle values for the forms above; the engine matches it. Docs entry in references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
