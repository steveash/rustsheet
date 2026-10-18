---
id: T-1506
title: "ISREF(@INDEX(range,n)) is TRUE in Excel; engine gives FALSE"
milestone: M1
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ISREF(@INDEX(B1:B3,2))` is TRUE, as in Excel.

## Context
Excel (excel-graph, 2026-10-13) gives TRUE for `=ISREF(@INDEX(B1:B3,2))`; the engine gives FALSE. `is_reference` in functions/information/mod.rs has no `ImplicitIntersection` arm for INDEX. Evidence: probes/let-bind-implicit-index H10, docs/excel-semantics/references.md. Found by T-1503 (the LET/LAMBDA binding already keeps the reference, probes/let-bind-implicit-index). Add the ISREF cell to that probe case (or a new one), probe `@XLOOKUP`/`@TAKE` forms too.

## Acceptance criteria
- [x] A probe case covers ISREF of `@INDEX`; the engine matches it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`is_reference` now treats `@INDEX(...)` as a reference when INDEX picks cells, so `ISREF(@INDEX(B1:B3,2))` is TRUE (probes/let-bind-implicit-index H10, already in the probe case). Eval delta: probes known failures 5143 -> 5142 (1 fixed, 0 regressions). `@XLOOKUP`/`@TAKE` forms were not probed; no evidence yet.
