---
id: T-1246
title: "ANCHORARRAY of an @ operand prints bare unless parentheses were written (follow-up of T-1240)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of a stored `ANCHORARRAY` of an `@` operand prints `@operand#` for every operand kind, and keeps parentheses only where they were written.

## Context
probes/anchorarray-at-operand-kinds (excel-graph, T-1240): `ANCHORARRAY(@A1:B2)` is `=@A1:B2#`, `ANCHORARRAY(@"a")` `=@"a"#`, `ANCHORARRAY(@SUM(A1))` `=@SUM(A1)#`, `ANCHORARRAY(@INDEX(D1:D2,1))` `=@INDEX(D1:D2,1)#`, `ANCHORARRAY(@LAMBDA(x,x))` `=@LAMBDA(x,x)#`, but `ANCHORARRAY((@1))` is `=(@1)#`. The engine prints bare only for `@<number>` and parenthesizes every other operand (`spill_operand`, parser/reference_operators.rs); it must follow the written parentheses instead. docs/excel-semantics/references.md (T-1240).

## Acceptance criteria
- [ ] I41, I51, I61, I71, I81, I91 of the case leave eval/baseline (the I101/H81 cells are T-1247)
- [ ] `make check` green; reviewers approve.

## Out of scope
-
