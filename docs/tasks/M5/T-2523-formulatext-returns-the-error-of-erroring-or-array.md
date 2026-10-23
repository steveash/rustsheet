---
id: T-2523
title: "FORMULATEXT returns the error of erroring or array-shaped reference calls"
milestone: M5
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's FORMULATEXT matches Excel on every cell of probes/formulatext-erroring-references.

## Context
Probed in T-2301; Excel values and the rules are in docs/excel-semantics/references.md ("FORMULATEXT of an erroring or array-shaped reference-returning call"). The engine gives `#ERROR!` for erroring OFFSET, INDIRECT, CHOOSE and IF arguments (Excel returns the argument's own error; INDIRECT of a missing sheet is `#N/A`), and does not spill `#VALUE!` for array-shaped INDEX, IF and CHOOSE results. See `fn_formulatext`.

## Acceptance criteria
- [ ] Every cell of probes/formulatext-erroring-references passes; its entries leave eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
