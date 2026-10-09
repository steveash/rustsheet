---
id: T-1178
title: "Probe whether Excel keeps parentheses nested inside an @ operand"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for whether Excel keeps parentheses nested inside an `@` operand, with the engine matching it.

## Context
T-1167 found the engine drops groups nested inside an `@` operand (`@(A1+(B1))` prints `=@(A1+B1)`, `@((A1) (B1))` prints `=@(A1 B1)`). No probe covers them; see the UNVERIFIED note in docs/excel-semantics/references.md and the test `at_operand_adds_no_level_for_a_nested_inner_group`.

## Acceptance criteria
- [ ] Probe `=@(A1+(B1))`, `=@((A1) (B1))` and `=@(((A1)) B1)` with FORMULATEXT (excel-probe skill); the engine and the T-1167 test match, and the references.md entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
