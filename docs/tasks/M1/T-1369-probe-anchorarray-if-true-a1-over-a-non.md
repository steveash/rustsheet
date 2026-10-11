---
id: T-1369
title: "probe ANCHORARRAY(IF(TRUE,A1)) over a non-spill cell and ANCHORARRAY(@IF(TRUE,D1))"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for `=ANCHORARRAY(IF(TRUE,A1))` with a plain-value A1, and for `=ANCHORARRAY(IF(TRUE,D1:D3))`, is recorded as oracle evidence and the engine matches it.

## Context
T-1363 made a plain IF that picks a reference the operand of `#`/ANCHORARRAY (`spill_anchor_operand`). Only the one-cell IF over a real spill anchor is evidenced (probes/written-parens-reference-context E20:E22); these variants are unprobed. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with oracle values for both variants (and `@IF(TRUE,D1)#` over a spill anchor) lands; engine mismatches are fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
