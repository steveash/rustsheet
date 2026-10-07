---
id: T-1116
title: "Keep the written parentheses of every @ operand in FORMULATEXT"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT (and every formula print) keeps the parentheses written around the operand of `@`, as Excel does: `=@((A1 B1))`, `=@(A1:B1)` and `=@(A1)` print as written.

## Context
Probed in T-1111: probes/at-paren-operands E30, E31, E32 (excel-graph) print `=@((A1 B1))`, `=@(A1:B1)`,
`=@(A1)`; the engine prints `=@(A1 B1)`, `=@A1:B1`, `=@A1`. Values are already right (D30 `#NULL!`, D31 `#VALUE!`,
D32 1). T-1109 kept the parentheses only for an `@` operand of `:` or `@`; printing is in
crates/rustsheet-engine/src/expressions/parser/stringify.rs (`Node::ImplicitIntersection`), and the parser must
remember that parentheses were written (cf. how T-1109 did it). Mind the xlsx-import automatic `@`, which has no
parentheses and must keep printing bare (`@A1:A3 A2:B2` stays bare, probes/intersection-at-formulatext).

## Acceptance criteria
- [ ] probes/at-paren-operands passes; probes/intersection-at-formulatext and probes/at-operand-formulatext still pass.
- [ ] Printed text re-parses to the same formula (round-trip unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
