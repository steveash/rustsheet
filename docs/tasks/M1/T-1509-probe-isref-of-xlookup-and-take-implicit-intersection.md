---
id: T-1509
title: "Probe ISREF of @XLOOKUP and @TAKE (implicit intersection keeps the reference?)"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ISREF(@XLOOKUP(...))` and `ISREF(@TAKE(...))` match Excel, backed by an oracle probe.

## Context
T-1506 made `ISREF(@INDEX(range,n))` TRUE (`is_reference` in functions/information/mod.rs). Whether `@` keeps the reference for XLOOKUP and TAKE, which are references without `@`, is unprobed (docs/excel-semantics/references.md, T-1506 entry). Extend probes/let-bind-implicit-index or add a case, get oracle values with the excel-probe skill, then match them.

## Acceptance criteria
- [ ] A probe case covers ISREF of `@XLOOKUP` and `@TAKE` (and LET-bound forms); the engine matches it.
- [ ] The probe also covers `@INDEX` shapes that pick several cells or fail (`ISREF(@INDEX(B1:C3,0,1))`, `ISREF(@INDEX(B1:B3,9))`); the engine matches Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
