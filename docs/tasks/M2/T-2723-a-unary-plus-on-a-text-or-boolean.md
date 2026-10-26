---
id: T-2723
title: "A unary plus on a text or boolean literal is kept in formula text: FORMULATEXT shows +'a' and +TRUE, drops it on a number"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
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

Probed by the first T-2723 run (probes/t2723-unary-plus-formula-text, excel-graph 2026-10-25; syntax.md "A typed unary plus
stays in the formula text, except on a number literal"). The rule: the plus is kept everywhere except directly on a number
literal, where it folds into the number. A space breaks the fold (`+ 5` stays). Values are unaffected.
Why it was not done in that run: `parse_power` drops every plus, and it also loses the order of the signs
(`+-5`/`-+5`). Keeping a plus needs an AST form that every analysis must see through, as a parenthesis `Node::Group` is.
The options are an `OpUnary::Plus` node, which touches every `UnaryKind` match and is a risk for reference and array
analysis such as `is_plain_reference` and the legacy `@` markers, or a Group-like transparent wrapper. Write the design
down first (a short note in docs/specs/ or the task, since it changes the parser AST). Then implement it, with the eval
gate on every `@`/legacy-marker probe.

## Acceptance criteria
- [ ] probes/t2601-index-scalar-forms-t-n B14 and B15, and probes/t2723-unary-plus-formula-text's C cells, pass and leave
      eval/baseline/probes.txt.
- [ ] A design note for keeping the plus in the AST (docs/specs/ or this task) is reviewed before the parser changes.
- [x] syntax.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluation of a unary plus (the identity).
