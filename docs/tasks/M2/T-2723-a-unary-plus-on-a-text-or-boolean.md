---
id: T-2723
title: "A unary plus on a text or boolean literal is kept in formula text: FORMULATEXT shows +'a' and +TRUE, drops it on a number"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Formula text (FORMULATEXT, stored text) keeps a unary plus where Excel keeps it: on a text or boolean literal it stays
(`+"a"`, `+TRUE`), and on a number literal it is dropped (`+5` prints as `5`).

## Context
Found by T-2601 (probes/t2601-index-scalar-forms-t-n, excel-graph 2026-10-25). Excel's FORMULATEXT of `=T(INDEX(+"a",0,0))` is
`=T(INDEX(+"a",0,0))` (B14), and of `=N(INDEX(+TRUE,0))` it is `=N(INDEX(+TRUE,0))` (B15). Of `=N(INDEX(+5,0,0))` it is
`=N(INDEX(5,0,0))` (B5). Ours drops every unary plus at parse time (references.md: `=@+1` prints `=@1`). Unprobed: a unary
plus on a reference, a function call or a parenthesised expression. Probe those first (excel-probe skill). Code: the
parser's unary handling in crates/rustsheet-engine/src/expressions/parser/ and the printer. Evaluation is unaffected:
a unary plus is the identity.

## Acceptance criteria
- [ ] probes/t2601-index-scalar-forms-t-n B14 and B15 pass and leave eval/baseline/probes.txt; new probes pass.
- [ ] syntax.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluation of a unary plus (the identity).
