---
id: T-2776
title: "Spec: keep typed whitespace in formula text (FORMULATEXT, export) as Excel does"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in `docs/specs/` for keeping the whitespace Excel keeps in formula text, so FORMULATEXT (and the formula
text we export) print a typed formula as Excel stores it: `=1 + 2`, `=SUM( 1 )`, `=+ 5`, a line feed after an
operator. The spec ends with an ordered list of S/M implementation tasks, filed with `make new-task`.

## Context
Split from T-2769, which probed the rule (probes/t2769-formula-text-whitespace, excel-graph `entered`, 2026-10-26;
syntax.md "Formula text keeps typed whitespace, except at the end, before a comma and in an array constant").
Excel keeps spaces and line feeds before and after binary operators, after `=`, after a unary sign (`=- 5`,
`=-+ 5`), before `%`, after `(` and before `)` of a call or group, and after a comma; it drops trailing
whitespace, whitespace before a comma, and whitespace inside an array constant; a folded plus keeps the
spaces before it (`=  +5` prints `=  5`). Today the tree (`Node`) keeps no whitespace and the printer
(expressions/parser/stringify.rs) writes canonical text, so the 18 probe cells with kept whitespace fail (C1:C5, C7, C8, C10:C15, C17:C20, C22), plus
probes/t2723-unary-plus-formula-text C24, C25 (owned here until the spec's tasks take them).
SPEC-0007 lists whitespace preservation as a non-goal (its Goals / non-goals); the new spec picks that up.
Design questions for the spec: where the whitespace lives (`Node` must not grow, `NODE_SIZE`, SPEC-0007; formulas
are shared R1C1 trees, so whitespace that differs per cell must not break sharing), how it survives reference
moves, copy/fill and structural edits, whether stored xlsx `<f>` text keeps it on load and export (unprobed:
probe it), and what Excel does with whitespace in other places (before `(` of a call, around `:` and `!`,
inside a structured reference, extra whitespace next to an intersection, where one space is the
operator and the rest is cosmetic, `=B1:B2  B2`, and a folded plus after an operator, `=1+  +5`); unknowns become probe cases first (excel-probe skill).

## Acceptance criteria
- [ ] `docs/specs/SPEC-NNNN-formula-text-whitespace.md` (from TEMPLATE.md) covers the probed rule, the open
      questions above (probed, or listed as probe tasks), the design and its alternatives.
- [ ] Implementation tasks filed with `make new-task`, each S/M, and the failing cells of
      probes/t2769-formula-text-whitespace and probes/t2723-unary-plus-formula-text C24, C25 owned by one of them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation itself.

## Outcome
SPEC-0023 (docs/specs/SPEC-0023-formula-text-whitespace.md) designs typed whitespace as a transparent
`Node::Spaced` wrapper attached at the primary level, peeled with `Group`, holding Excel's run shape (line-feed
count + space count). New probes (excel-graph, 2026-10-26) answered the open questions: probes/t2776-formula-
text-whitespace-shapes (typed and stored), t2776-ws-after-at, -after-bang, -after-colon, -before-colon. Findings
in syntax.md: a run keeps only its line feeds and the spaces after the last one; its space count wraps at 256;
spaces around `:` and after `!` drop; a stored formula's leading run drops; `=SUM (1)` is refused (observed). Ours fails to parse
`=S! B1` (`#ERROR!`). Eval: probes +5 cases, 27 new-coverage known failures (all owned below), 0 regressions;
`make eval-baseline` (`update-baseline`) wrote the visible-parity count 4443 -> 4448 for those new cells.
Tasks filed: T-2781 (step 1, opus M), T-2782, T-2783, T-2784 (steps 2-4, sonnet S), T-2785 (`=S! B1`, sonnet S),
T-2786 (structured-reference and saved-text probes, research sonnet S).
