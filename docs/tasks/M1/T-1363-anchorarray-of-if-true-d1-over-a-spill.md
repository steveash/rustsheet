---
id: T-1363
title: "ANCHORARRAY of IF(TRUE,D1) over a spill anchor gives TypeMismatch"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`=ANCHORARRAY(IF(TRUE,D1))` with D1 = SEQUENCE(3) spills 1, 2, 3 as Excel does.

## Context
probes/written-parens-reference-context E20:E22 (excel-graph, T-1339): Excel stores the formula as `=IF(TRUE,D1)#` and spills the three values; the engine gives TypeMismatch. The grouped form `ANCHORARRAY(@(IF(TRUE,D1)))` (G20:G22) already passes. docs/excel-semantics/references.md "Written parentheses are transparent in reference context". Related: anchorarray-at-operand probes, T-1321.

## Acceptance criteria
- [ ] E20:E22 of probes/written-parens-reference-context pass and leave the baseline; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
