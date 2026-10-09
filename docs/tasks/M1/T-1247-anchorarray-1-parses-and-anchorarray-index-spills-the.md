---
id: T-1247
title: "ANCHORARRAY(@-1) parses, and ANCHORARRAY(@INDEX(..)) spills the reference (follow-up of T-1240)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ANCHORARRAY(@-1)` is the `#VALUE!` Excel gives, and `ANCHORARRAY(@INDEX(D1:D2,1))` spills the reference it selects.

## Context
probes/anchorarray-at-operand-kinds (excel-graph, T-1240): H101 `=ANCHORARRAY(@-1)` is `#VALUE!` printed `=@-1#`, the engine gives `#ERROR!` (a parse failure; its text prints as the input). H81 `=ANCHORARRAY(@INDEX(D1:D2,1))` spills 1,2,3 (it is `D1#`, INDEX gives a reference), the engine gives `#VALUE!`. Other `@`-of-value forms (`@"a"`, `@SUM(A1)`, `@LAMBDA(x,x)`) are `#VALUE!` and match. docs/excel-semantics/references.md (T-1240).

## Acceptance criteria
- [ ] H81:H83 (the spill 1, 2, 3) and H101 of the case leave eval/baseline
- [ ] `make check` green; reviewers approve.

## Out of scope
-
