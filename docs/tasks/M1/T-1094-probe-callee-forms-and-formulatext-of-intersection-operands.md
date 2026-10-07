---
id: T-1094
title: "Probe callee forms and FORMULATEXT of @ intersection operands"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the callee forms and printing that T-0149 decided by inference or engine choice.

## Context
From the T-0149 reviews; docs/excel-semantics/references.md "Calling a parenthesized operand" and
"Intersection operands must be references". The engine treats `(1+1)(2)` and `(SUM(A1))(2)` as syntax
errors only by inference (Excel refuses `(1)(2)`), and accepts `(OFFSET(A1,0,0))(2)` and
`(INDIRECT("A1"))(2)` as calls of references (`#REF!`), also unprobed. It prints `=A1 @A1` as
`=A1 (@A1)` and `=@A1:A3 A2:B2` as `=@(A1:A3 A2:B2)` (engine choice). Use the `excel-probe` skill:
one workbook per form Excel may refuse (a refused workbook gives `FileCorruptTryRepair` and has no
values); FORMULATEXT of the two `@` formulas in one case.

## Acceptance criteria
- [ ] Probe cases with oracle sidecars for `(1+1)(2)`, `(SUM(A1))(2)`, `(OFFSET(A1,0,0))(2)`,
  `(INDIRECT("A1"))(2)` and FORMULATEXT of `=A1 @A1` and `=@A1:A3 A2:B2` (refused forms recorded, not committed).
- [ ] references.md updated with the results; any engine mismatch filed as an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
