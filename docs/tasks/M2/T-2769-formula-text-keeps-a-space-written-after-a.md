---
id: T-2769
title: "Formula text keeps a space written after a unary plus: FORMULATEXT of =+ 5 is =+ 5 (probes/t2723-unary-plus-formula-text C24, C25)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT and stored formula text keep a space written between a unary plus and its operand, as Excel does:
`=+ 5` prints `=+ 5` and `=+ "a"` prints `=+ "a"`.

## Context
Split from T-2723. T-2723 keeps a unary plus in the tree (`Node::UnaryPlus`, expressions/parser/mod.rs `parse_power`), so
`=+"a"` prints back as written. The parser already sees the space, because it stops the plus folding into a number
(`=+ 5` stays a `UnaryPlus`). But the tree does not keep it, so the printer writes `=+5` and `=+"a"`.
- probes/t2723-unary-plus-formula-text C24 (`=+ 5`) and C25 (`=+ "a"`) are known failures in eval/baseline/probes.txt
  (excel-graph 2026-10-25; syntax.md "A typed unary plus stays in the formula text, except on a number literal").
- Open question: which other whitespace Excel keeps in formula text (`=1 + 2`, `=SUM( 1 )`). Probe that first
  (excel-probe skill). If Excel keeps all typed whitespace, this is a general feature: a spec, a larger task, and
  a split. If only the plus case is in scope, a flag on `UnaryPlus` is enough. `Node` must not grow
  (`NODE_SIZE`, SPEC-0007).

## Acceptance criteria
- [ ] probes/t2723-unary-plus-formula-text C24 and C25 pass and leave eval/baseline/probes.txt.
- [ ] The whitespace rule probed in this task is recorded in syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluation (a unary plus is the identity).
