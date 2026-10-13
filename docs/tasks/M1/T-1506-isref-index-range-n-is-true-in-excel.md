---
id: T-1506
title: "ISREF(@INDEX(range,n)) is TRUE in Excel; engine gives FALSE"
milestone: M1
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ISREF(@INDEX(B1:B3,2))` is TRUE, as in Excel.

## Context
Excel (excel-graph, 2026-10-13) gives TRUE for `=ISREF(@INDEX(B1:B3,2))`; the engine gives FALSE. `is_reference` in functions/information/mod.rs has no `ImplicitIntersection` arm for INDEX. Found by T-1503 (the LET/LAMBDA binding already keeps the reference, probes/let-bind-implicit-index). Add the ISREF cell to that probe case (or a new one), probe `@XLOOKUP`/`@TAKE` forms too.

## Acceptance criteria
- [ ] A probe case covers ISREF of `@INDEX`; the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
