---
id: T-1285
title: "Probe a signed operand of @: =@-1, =@+1, =@-B1:B3, =@-B1 B1:C1, =@-1%, =@-2^2 and their FORMULATEXT (follow-up of T-1247)"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The value, precedence and FORMULATEXT of a signed `@` operand are probed in Excel and the engine matches them.

## Context
T-1247 made `@` take a signed operand (parse_implicit calls parse_power), so plain `=@-1` now parses; only `ANCHORARRAY(@-1)` is evidenced (H101). Inferred, unverified: `=@-1` is -1, `=@-B1:B3` off-row is the top-left of the negated array (Excel may give #VALUE!), `@-B1 B1:C1` is `@(-(B1 B1:C1))`, `@-1%` is `@((-1)%)`, `@-2^2` is `(@-2)^2` and prints `=(@-2)^2`. Also probe `ANCHORARRAY(@IF(TRUE,D1))` (engine: #VALUE!), `@OFFSET`, `@INDIRECT`. docs/excel-semantics/references.md (T-1240, T-1247).

## Acceptance criteria
- [ ] A probe case with oracle values for the forms above; deviations fixed or baselined with tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
