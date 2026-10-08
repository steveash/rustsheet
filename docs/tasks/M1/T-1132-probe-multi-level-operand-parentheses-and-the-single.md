---
id: T-1132
title: "Probe multi-level @ operand parentheses and the SINGLE export form"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT for `@` operands with more than the probed parenthesis levels, and for the xlsx stored form, is pinned by oracle cases.

## Context
T-1116 keeps every written level around an `@` operand (`ImplicitIntersection::written`), but only `@(A1)`, `@(A1:B1)` and `@((A1 B1))` are probed (probes/at-paren-operands). Unprobed: `=@((A1))`, `=@(((A1)))`, `=@((A1:B1))`, `=@((A1,B2))`, `=@(A1) B1` (group is only part of the operand), and whether the stored `_xlfn.SINGLE(...)` form keeps parentheses (export prints `_xlfn.SINGLE(A1 B1)` for `@((A1 B1))`). Follow the `excel-probe` skill; record in docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] A probe case with oracle values covers each unprobed form above; no cell pending.
- [ ] references.md records the rule; any engine mismatch is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
