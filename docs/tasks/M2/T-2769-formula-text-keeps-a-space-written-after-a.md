---
id: T-2769
title: "Formula text keeps a space written after a unary plus: FORMULATEXT of =+ 5 is =+ 5 (probes/t2723-unary-plus-formula-text C24, C25)"
milestone: M2
lane: engine
status: done
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
- [ ] probes/t2723-unary-plus-formula-text C24 and C25 pass and leave eval/baseline/probes.txt. Moved to T-2776
      (see Outcome): the probe showed the general case the Context said to split.
- [x] The whitespace rule probed in this task is recorded in syntax.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Evaluation (a unary plus is the identity).

## Outcome
Probed which whitespace Excel keeps (probes/t2769-formula-text-whitespace, 23 formulas, excel-graph `entered`,
2026-10-26): all of it, except trailing whitespace, whitespace before a comma and whitespace inside an array
constant (syntax.md "Formula text keeps typed whitespace, except at the end, before a comma and in an array
constant"). That is the general feature the Context named, so per the Context this task does not add a flag on
`UnaryPlus`: it filed T-2776 (opus, a spec first, then the implementation tasks), which now owns C24 and C25 and the
new probe's failing cells. No engine change; eval delta: the new probe's 18 cells with kept whitespace (of 47 checked) are new
known failures in eval/baseline/probes.txt, owned by T-2776; nothing else moves.
